//! One tab request (skein/docs/design/playground.md, 4).
use skein_http::{Header, Method, client as http};
use skein_io as io;
use skein_lib::{
    Env, Queue, Token,
    stream::{Down, Read, Up},
};

#[derive(Debug, PartialEq, Eq)]
enum State {
    Connecting,
    Active,
    Closing,
}

pub(super) struct Answer {
    pub status: u16,
    pub body: Box<[u8]>,
    pub location: Option<Box<[u8]>>,
    pub failed: bool,
}
#[derive(Debug)]
pub(super) struct Connection {
    state: State,
    socket: Option<Token>,
    http: http::Client,
    events: Queue<http::Event>,
    calls: Queue<http::Request>,
    below: Queue<Down>,
    status: u16,
    body: Vec<u8>,
    location: Option<Box<[u8]>>,
    bound: u32,
}
impl Connection {
    pub(super) fn new(limits: &http::Limits, authority: Box<[u8]>, target: Box<[u8]>, bound: u32) -> Self {
        let mut calls = Queue::uncapped();
        calls.push(http::Request::Call(http::Call {
            method: Method::Get,
            target,
            headers: Box::new([Header { name: b"Host".as_slice().into(), value: authority }]),
            body: http::Body::Length(0),
            close: true,
        }));
        calls.push(http::Request::Upload(Down::Finish));
        Self {
            state: State::Connecting,
            socket: None,
            http: http::Client::new(limits),
            events: Queue::uncapped(),
            calls,
            below: Queue::uncapped(),
            status: 0,
            body: Vec::new(),
            location: None,
            bound,
        }
    }
    pub(super) fn connecting(&mut self, socket: Token, requests: &mut Queue<io::Request>) {
        self.socket = Some(socket);
        if self.state == State::Closing {
            requests.push(io::Request::Abort { entity: socket });
        }
    }
    pub(super) fn connected(&mut self) {
        if self.state == State::Connecting {
            self.state = State::Active;
        }
    }
    pub(super) fn close(&mut self, requests: &mut Queue<io::Request>) {
        if self.state != State::Closing {
            self.state = State::Closing;
            self.calls.clear();
            self.events.clear();
            self.below.clear();
            if let Some(entity) = self.socket {
                requests.push(io::Request::Abort { entity });
            }
        }
    }
    pub(super) fn up(&mut self, up: Up, env: &Env<http::Limits>) {
        if self.state == State::Active {
            http::up(&mut self.http, env, up, &mut self.events, &mut self.below);
        }
    }
    fn read(&mut self, limits: &http::Limits) {
        let remaining = self.bound.saturating_sub(u32::try_from(self.body.len()).expect("bounded body"));
        self.calls.push(http::Request::Body(Down::Demand {
            read: Read::Arrived { max: remaining.max(1).min(limits.read) },
            room: 0,
        }));
    }
    pub(super) fn finish(&mut self, failed: bool, requests: &mut Queue<io::Request>) -> Answer {
        self.close(requests);
        Answer {
            status: self.status,
            body: core::mem::take(&mut self.body).into_boxed_slice(),
            location: self.location.take(),
            failed,
        }
    }
    pub(super) fn progress(&mut self, env: &Env<http::Limits>, requests: &mut Queue<io::Request>) -> Option<Answer> {
        if self.state != State::Active {
            return None;
        }
        for _ in 0..self.events.len() {
            let Some(event) = self.events.pop() else { break };
            match event {
                http::Event::Response(response) => {
                    self.status = response.status;
                    self.location = response.headers.iter().find(|h| h.is(b"location")).map(|h| h.value.clone());
                    self.read(&env.limits);
                }
                http::Event::Body(Up::Bytes(bytes)) => {
                    let room = skein_lib::length::usize_of(self.bound).saturating_sub(self.body.len());
                    self.body.extend_from_slice(bytes.get(..room.min(bytes.len())).expect("prefix within body piece"));
                    if bytes.len() > room {
                        return Some(self.finish(true, requests));
                    }
                    self.read(&env.limits);
                }
                http::Event::Done(_) => return Some(self.finish(false, requests)),
                http::Event::Failed(_) | http::Event::Body(Up::Failed(_)) => return Some(self.finish(true, requests)),
                http::Event::Upload(Up::Room | Up::Failed(_) | Up::Bytes(_) | Up::End)
                | http::Event::Body(Up::Room | Up::End)
                | http::Event::Closed => {}
            }
        }
        if let Some(call) = self.calls.pop() {
            http::down(&mut self.http, env, call, &mut self.events, &mut self.below);
        }
        for _ in 0..self.below.len() {
            if let Some(down) = self.below.pop() {
                requests.push(io::Request::Stream { stream: self.socket.expect("active socket"), down });
            }
        }
        None
    }
    pub(super) fn work_pending(&self) -> bool {
        self.state == State::Active && (!self.events.is_empty() || !self.calls.is_empty() || !self.below.is_empty())
    }
}
