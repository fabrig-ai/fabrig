//! The site's server Host (skein/docs/design/http.md, 5; playground.md, 8).
use super::{Body, Config, Route};
use skein_http::{Header, server as http};
use skein_io::{self as io, kernel};
use skein_lib::{
    Env, Queue, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use skein_shell::Host;
use std::collections::{BTreeMap, VecDeque};
const LISTENER: Token = Token::new(u64::MAX);
fn limits() -> http::Limits {
    http::Limits { head: 2048, headers: 16, body: 4096, read: 256, response: 2048, send: 256 }
}
#[derive(PartialEq, Eq)]
enum State {
    Request,
    Reply,
    Closing,
}

struct Connection {
    socket: Token,
    machine: http::Server,
    events: Queue<http::Event>,
    calls: Queue<http::Request>,
    below: Queue<Down>,
    call: Option<http::Call>,
    received: Vec<u8>,
    route: Option<Route>,
    pieces: VecDeque<(Time, Box<[u8]>)>,
    finish: bool,
    ready: bool,
    state: State,
}
impl Connection {
    fn new(socket: Token) -> Self {
        let mut calls = Queue::uncapped();
        calls.push(http::Request::Next);
        Self {
            socket,
            machine: http::Server::new(&limits()),
            events: Queue::uncapped(),
            calls,
            below: Queue::uncapped(),
            call: None,
            received: Vec::new(),
            route: None,
            pieces: VecDeque::new(),
            finish: false,
            ready: false,
            state: State::Request,
        }
    }
    fn read(&mut self) {
        self.calls.push(http::Request::Body(Down::Demand { read: Read::Arrived { max: limits().read }, room: 0 }));
    }
    fn respond(&mut self, now: Time) {
        let route = self.route.take().expect("route for request");
        let call = self.call.as_ref().expect("request head");
        let (body, pieces, finish) = match route.body {
            Body::Fixed(bytes) => {
                (http::Body::Length(u64::try_from(bytes.len()).expect("body fits")), vec![(now, bytes)], true)
            }
            Body::Echo => (
                http::Body::Length(u64::try_from(self.received.len()).expect("body fits")),
                vec![(now, self.received.clone().into_boxed_slice())],
                true,
            ),
            Body::Headers => {
                let mut bytes = Vec::new();
                for Header { name, value } in &call.headers {
                    bytes.extend_from_slice(name);
                    bytes.push(b':');
                    bytes.extend_from_slice(value);
                    bytes.push(b'\n');
                }
                (
                    http::Body::Length(u64::try_from(bytes.len()).expect("body fits")),
                    vec![(now, bytes.into_boxed_slice())],
                    true,
                )
            }
            Body::Pieces { pieces, finish } => (
                http::Body::Chunked,
                pieces.into_iter().map(|(delay, bytes)| (now.saturating_add(delay), bytes)).collect(),
                finish,
            ),
        };
        self.pieces = pieces.into();
        self.finish = finish;
        self.state = State::Reply;
        self.calls.push(http::Request::Respond(http::Response {
            status: route.status,
            headers: route.headers.into_boxed_slice(),
            body,
            close: true,
        }));
    }
    fn close(&mut self, requests: &mut Queue<io::Request>) {
        if self.state != State::Closing {
            self.state = State::Closing;
            self.calls.clear();
            self.events.clear();
            for _ in 0..self.below.len() {
                if let Some(down) = self.below.pop() {
                    requests.push(io::Request::Stream { stream: self.socket, down });
                }
            }
            requests.push(io::Request::Close { entity: self.socket });
        }
    }
    fn progress(&mut self, now: Time, wall: Wall, requests: &mut Queue<io::Request>) {
        let env = Env { now, wall, limits: limits() };
        for _ in 0..self.events.len() {
            let Some(event) = self.events.pop() else { break };
            match event {
                http::Event::Call(_) => unreachable!("host routes the call"),
                http::Event::Body(Up::Bytes(bytes)) => {
                    self.received.extend_from_slice(&bytes);
                    self.read();
                }
                http::Event::Body(Up::End) => self.respond(now),
                http::Event::Reply(Up::Room) => {
                    let (_, bytes) = self.pieces.pop_front().expect("piece demanded");
                    self.calls.push(http::Request::Reply(Down::Send(bytes)));
                    self.ready = false;
                }
                http::Event::Done(_)
                | http::Event::Ended
                | http::Event::Failed(_)
                | http::Event::Body(Up::Failed(_))
                | http::Event::Reply(Up::Failed(_)) => self.close(requests),
                http::Event::Refused(reason) => panic!("scripted response refused: {reason:?}"),
                http::Event::Body(Up::Room) | http::Event::Reply(Up::Bytes(_) | Up::End) | http::Event::Closed => {}
            }
        }
        if self.state == State::Closing {
            return;
        }
        if (self.state == State::Reply) && !self.ready {
            if let Some((at, bytes)) = self.pieces.front().filter(|(at, _)| *at <= now) {
                let _ = at;
                if bytes.is_empty() {
                    self.pieces.pop_front();
                } else {
                    // Every scripted piece fits the server's per-send bound.
                    self.calls.push(http::Request::Reply(Down::Demand {
                        read: Read::Nothing,
                        room: u32::try_from(bytes.len()).expect("piece fits"),
                    }));
                    self.ready = true;
                }
            } else if self.pieces.is_empty() && self.finish {
                self.calls.push(http::Request::Reply(Down::Finish));
                self.finish = false;
            }
        }
        for _ in 0..self.calls.len() {
            if let Some(call) = self.calls.pop() {
                http::down(&mut self.machine, &env, call, &mut self.events, &mut self.below);
            }
        }
        for _ in 0..self.below.len() {
            if let Some(down) = self.below.pop() {
                requests.push(io::Request::Stream { stream: self.socket, down });
            }
        }
    }
    fn due(&self) -> Option<Time> {
        if self.state == State::Closing || self.ready { None } else { self.pieces.front().map(|(at, _)| *at) }
    }
    fn work(&self, now: Time) -> bool {
        self.state != State::Closing
            && (!self.events.is_empty()
                || !self.calls.is_empty()
                || !self.below.is_empty()
                || self.due().is_some_and(|at| at <= now)
                || ((self.state == State::Reply) && self.pieces.is_empty() && self.finish))
    }
}

/// The independent peer's counts are outside observations, available to the referee.
pub struct Server {
    io: io::Io,
    limits: io::Limits,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    listener: Option<Token>,
    connections: BTreeMap<Token, Connection>,
    serial: u64,
    routes: Vec<Route>,
    pub(super) requests_seen: usize,
    pub(super) closed: usize,
    pub(super) stop: bool,
}
impl Server {
    pub(super) fn new(config: &Config, routes: Vec<Route>) -> Self {
        let mut requests = Queue::uncapped();
        requests.push(io::Request::Listen { owner: LISTENER, addr: io::Addr::Ip(config.listener) });
        let io_limits =
            io::Limits { sockets: config.io.sockets.checked_add(1).expect("listener beside clients"), ..config.io };
        let operations = io::operations(&io_limits).expect("bounded io");
        Self {
            io: io::Io::new(&io_limits),
            limits: io_limits,
            events: Queue::uncapped(),
            requests,
            submissions: Queue::with_capacity(operations),
            completions: Queue::with_capacity(operations),
            listener: None,
            connections: BTreeMap::new(),
            serial: 0,
            routes,
            requests_seen: 0,
            closed: 0,
            stop: false,
        }
    }
    pub(super) fn listening(&self) -> bool {
        self.listener.is_some()
    }
}
impl Host for Server {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits };
        let http_env = Env { now, wall, limits: limits() };
        io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        for _ in 0..self.events.len() {
            let Some(event) = self.events.pop() else { break };
            match event {
                io::Event::Listening { listener, .. } => self.listener = Some(listener),
                io::Event::Accepted { socket, .. } => {
                    self.serial = self.serial.checked_add(1).expect("finite connections");
                    let owner = Token::new(self.serial);
                    self.requests.push(io::Request::Bind { socket, owner });
                    self.connections.insert(owner, Connection::new(socket));
                }
                io::Event::Stream { owner, up: Up::End | Up::Failed(_) } => {
                    self.closed = self.closed.checked_add(1).expect("finite peer endings");
                    if let Some(connection) = self.connections.get_mut(&owner) {
                        connection.close(&mut self.requests);
                    }
                }
                io::Event::Stream { owner, up: up @ (Up::Bytes(_) | Up::Room) } => {
                    if let Some(connection) =
                        self.connections.get_mut(&owner).filter(|connection| connection.state != State::Closing)
                    {
                        http::up(&mut connection.machine, &http_env, up, &mut connection.events, &mut connection.below);
                    }
                }
                io::Event::Closed { owner } if owner != LISTENER => {
                    self.connections.remove(&owner);
                }
                io::Event::Failed { error, .. } => panic!("site io failed: {error:?}"),
                io::Event::Closed { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Output { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. } => {}
            }
        }
        for connection in self.connections.values_mut() {
            let mut events = Queue::uncapped();
            for _ in 0..connection.events.len() {
                if let Some(event) = connection.events.pop() {
                    if let http::Event::Call(call) = event {
                        self.requests_seen = self.requests_seen.checked_add(1).expect("finite requests");
                        connection.route = Some(
                            self.routes.iter().find(|route| route.path == call.target).expect("scripted route").clone(),
                        );
                        connection.call = Some(call);
                        connection.read();
                    } else {
                        events.push(event);
                    }
                }
            }
            connection.events = events;
            connection.progress(now, wall, &mut self.requests);
        }
        if self.stop
            && let Some(entity) = self.listener.take()
        {
            self.requests.push(io::Request::Close { entity });
        }
        io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || !self.completions.is_empty()
            || (self.stop && self.listener.is_some())
            || self.connections.values().any(|connection| connection.work(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.connections.values().filter_map(Connection::due).min()
    }
    fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
            && self.connections.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.limits).expect("bounded io")
    }
}
