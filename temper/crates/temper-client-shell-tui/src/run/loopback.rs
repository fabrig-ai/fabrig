//! The run's listener, driven only by io events (client/tui/README.md, 4).
use crate::{
    host, limits,
    loopback::{Answer, Returned},
};
use skein_http::{
    Method,
    server::{self, Server},
};
use skein_io as io;
use skein_lib::{Env, Queue, Time, Token, Wall, stream};

// Beside OPENER, front's FETCH and BOX, and lending's LOGIN owner.
const LISTENER: Token = Token::new(u64::MAX - 4);
const CONNECTION: u8 = 7;

fn http_limits() -> server::Limits {
    server::Limits {
        head: limits::RETURN_HEAD,
        // Even an empty field takes a name byte, a colon and a line ending.
        headers: limits::RETURN_HEAD / 3,
        body: u64::MAX,
        read: 1,
        response: limits::RETURN_HEAD,
        send: 1,
    }
}

enum Listening {
    Idle,
    Starting,
    Cancelling,
    Open(Token),
    Closing,
}

pub(super) struct Loopback {
    listener: Listening,
    wanted: bool,
    connections: Vec<Connection>,
    next_owner: u64,
    pub requests: Queue<io::Request>,
    pub reports: Queue<(Option<Token>, host::Report)>,
}

impl Loopback {
    pub fn new() -> Self {
        Self {
            listener: Listening::Idle,
            wanted: false,
            connections: Vec::new(),
            next_owner: 0,
            requests: Queue::uncapped(),
            reports: Queue::uncapped(),
        }
    }

    pub fn listen(&mut self, has_opener: bool) -> Option<host::Report> {
        if !has_opener {
            return Some(host::Report::Listening { port: None });
        }
        self.wanted = true;
        self.start();
        None
    }

    fn start(&mut self) {
        if self.wanted && matches!(self.listener, Listening::Idle) {
            self.listener = Listening::Starting;
            self.requests.push(io::Request::Listen { owner: LISTENER, addr: io::Addr::Ip(([127, 0, 0, 1], 0).into()) });
        }
    }

    // A head already reported still gets the answer from that turn's duties.
    pub fn unlisten(&mut self, now: Time, wall: Wall) {
        self.stop_listener();
        for connection in &mut self.connections {
            if connection.wait.is_some() {
                connection.close(now, wall, false, &mut self.requests);
            }
        }
    }

    pub fn close(&mut self, now: Time, wall: Wall) {
        self.stop_listener();
        self.reports.clear();
        for connection in &mut self.connections {
            connection.close(now, wall, false, &mut self.requests);
        }
    }

    fn stop_listener(&mut self) {
        self.wanted = false;
        match self.listener {
            Listening::Open(entity) => self.close_listener(entity),
            Listening::Starting => self.listener = Listening::Cancelling,
            Listening::Idle | Listening::Cancelling | Listening::Closing => {}
        }
    }

    fn close_listener(&mut self, entity: Token) {
        self.listener = Listening::Closing;
        self.requests.push(io::Request::Abort { entity });
    }

    pub fn owns(&self, event: &io::Event) -> bool {
        event.owner().is_some_and(|owner| owner == LISTENER || self.connections.iter().any(|c| c.owner == owner))
    }

    pub fn event(&mut self, now: Time, wall: Wall, event: io::Event) {
        if event.owner() == Some(LISTENER) {
            self.listener_event(now, wall, &event);
            return;
        }
        let owner = event.owner().expect("a connection's event has its owner");
        if let io::Event::Closed { .. } = event {
            self.connections.retain(|c| c.owner != owner);
            return;
        }
        let connection = self.connections.iter_mut().find(|c| c.owner == owner).expect("owned event");
        match event {
            io::Event::Stream { up, .. } => {
                if connection.expired(now) {
                    connection.close(now, wall, false, &mut self.requests);
                } else if !connection.closing {
                    server::up(
                        &mut connection.server,
                        &env(now, wall),
                        up,
                        &mut connection.above,
                        &mut connection.below,
                    );
                    connection.progress(now, wall, &mut self.requests, &mut self.reports);
                }
            }
            io::Event::Failed { .. } => connection.close(now, wall, false, &mut self.requests),
            io::Event::Closed { .. } => unreachable!("closed connection retired above"),
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => unreachable!("loopback connection's io event"),
        }
    }

    fn listener_event(&mut self, now: Time, wall: Wall, event: &io::Event) {
        match event {
            io::Event::Listening { listener, addr, .. } => match self.listener {
                Listening::Cancelling => self.close_listener(*listener),
                Listening::Starting => {
                    self.listener = Listening::Open(*listener);
                    let port = addr.socket().expect("loopback listens on IP").port();
                    self.reports.push((None, host::Report::Listening { port: Some(port) }));
                }
                Listening::Idle | Listening::Open(_) | Listening::Closing => unreachable!("only one listen in flight"),
            },
            io::Event::Accepted { socket, .. } => {
                if !matches!(self.listener, Listening::Open(_)) || self.connections.len() >= limits::RETURNS {
                    self.requests.push(io::Request::Reject { socket: *socket });
                    return;
                }
                let owner = Token::tagged(CONNECTION, self.next_owner).expect("connection owner namespace");
                self.next_owner = self.next_owner.checked_add(1).expect("next connection owner");
                let mut connection = Connection::new(*socket, owner, now);
                self.requests.push(io::Request::Bind { socket: *socket, owner });
                connection.down(now, wall, server::Request::Next);
                connection.flush(&mut self.requests);
                self.connections.push(connection);
            }
            io::Event::Failed { .. } => match self.listener {
                Listening::Starting | Listening::Open(_) => {
                    self.reports.push((None, host::Report::Listening { port: None }));
                    self.stop_listener();
                }
                // Failure of a cancelled listen or accept does not settle a new listen.
                Listening::Idle | Listening::Cancelling | Listening::Closing => {}
            },
            io::Event::Closed { .. } => {
                self.listener = Listening::Idle;
                self.start();
            }
            io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => unreachable!("listener's io event"),
        }
    }

    pub fn answer(&mut self, now: Time, wall: Wall, owner: Token, answer: Option<Answer>) {
        if let Some(connection) = self.connections.iter_mut().find(|c| c.owner == owner && !c.closing) {
            connection.down(now, wall, server::Request::Respond(answer.unwrap_or(Answer::NotFound).response()));
            connection.progress(now, wall, &mut self.requests, &mut self.reports);
        }
    }

    pub fn fire(&mut self, now: Time, wall: Wall) {
        for connection in &mut self.connections {
            if connection.expired(now) {
                connection.close(now, wall, false, &mut self.requests);
            }
        }
    }

    pub fn deadline(&self) -> Option<Time> {
        self.connections.iter().filter_map(|c| c.wait).min()
    }
}

fn env(now: Time, wall: Wall) -> Env<server::Limits> {
    Env { now, wall, limits: http_limits() }
}

struct Connection {
    socket: Token,
    owner: Token,
    server: Server,
    above: Queue<server::Event>,
    below: Queue<stream::Down>,
    wait: Option<Time>,
    closing: bool,
}

impl Connection {
    fn new(socket: Token, owner: Token, now: Time) -> Self {
        Self {
            socket,
            owner,
            server: Server::new(&http_limits()),
            above: Queue::uncapped(),
            below: Queue::uncapped(),
            wait: Some(now.saturating_add(limits::RETURN_WAIT)),
            closing: false,
        }
    }

    fn expired(&self, now: Time) -> bool {
        self.wait.is_some_and(|at| now >= at)
    }

    fn down(&mut self, now: Time, wall: Wall, request: server::Request) {
        server::down(&mut self.server, &env(now, wall), request, &mut self.above, &mut self.below);
    }

    fn flush(&mut self, requests: &mut Queue<io::Request>) {
        while let Some(down) = self.below.pop() {
            requests.push(io::Request::Stream { stream: self.socket, down });
        }
    }

    fn close(&mut self, now: Time, wall: Wall, drain: bool, requests: &mut Queue<io::Request>) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.wait = None;
        self.down(now, wall, server::Request::Close);
        self.flush(requests);
        requests.push(if drain {
            io::Request::Close { entity: self.socket }
        } else {
            io::Request::Abort { entity: self.socket }
        });
    }

    fn progress(
        &mut self,
        now: Time,
        wall: Wall,
        requests: &mut Queue<io::Request>,
        reports: &mut Queue<(Option<Token>, host::Report)>,
    ) {
        while let Some(event) = self.above.pop() {
            match event {
                server::Event::Call(call) => {
                    self.wait = None;
                    let host = call.header(b"host").map(Into::into);
                    reports.push((
                        Some(self.owner),
                        host::Report::Returned(Returned { get: call.method == Method::Get, target: call.target, host }),
                    ));
                }
                server::Event::Done(_) | server::Event::Failed(_) => self.close(now, wall, true, requests),
                server::Event::Ended => self.close(now, wall, false, requests),
                server::Event::Refused(_) => {
                    self.down(now, wall, server::Request::Respond(Answer::NotFound.response()));
                }
                server::Event::Body(_) | server::Event::Closed => {}
                server::Event::Reply(_) => unreachable!("loopback answers have no body"),
            }
        }
        self.flush(requests);
    }
}

#[cfg(test)]
mod tests;
