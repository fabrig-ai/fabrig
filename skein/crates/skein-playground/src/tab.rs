//! A sign-in's separate HTTP tab (skein/docs/design/playground.md, 4).
mod address;
mod connection;

use connection::Connection;
use skein_http::client as http;
use skein_io::{self as io, kernel};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_shell::Host;
use alloc::collections::BTreeMap;

/// The last answer, including a visit that failed before receiving an HTTP status (zero).
#[derive(Debug, PartialEq, Eq)]
pub struct Shown {
    pub address: Box<[u8]>,
    pub status: u16,
    pub body: Box<[u8]>,
    pub failed: bool,
}

/// A GET on its own connection, with no cookie jar or opener's origin.
/// Closing and replacing a visit abort its connection; old completions cannot change the new visit.
#[derive(Debug)]
pub struct Tab {
    io: io::Io,
    limits: io::Limits,
    http: http::Limits,
    bound: u32,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    connections: BTreeMap<Token, Connection>,
    serial: u64,
    current: Option<Token>,
    pending: Option<Box<[u8]>>,
    shown: Option<Shown>,
    redirects: u8,
}
impl Tab {
    #[must_use]
    pub fn new(io: &io::Limits, http: &http::Limits, body: u32) -> Self {
        assert!(http::largest_read(http) <= io.largest_read(), "HTTP reads fit io");
        assert!(http::largest_room(http) <= io.largest_room(), "HTTP writes fit io");
        let operations = io::operations(io).expect("bounded io");
        Self {
            io: io::Io::new(io), limits: *io, http: *http, bound: body,
            events: Queue::uncapped(), requests: Queue::uncapped(),
            submissions: Queue::with_capacity(operations), completions: Queue::with_capacity(operations),
            connections: BTreeMap::new(), serial: 0, current: None, pending: None, shown: None, redirects: 0,
        }
    }

    /// Point between world steps. A newer point supersedes even an answer already on its way.
    pub fn point(&mut self, address: &[u8]) {
        self.close();
        self.pending = Some(address.into());
    }
    pub fn close(&mut self) {
        if let Some(owner) = self.current.take()
            && let Some(connection) = self.connections.get_mut(&owner)
        {
            connection.close(&mut self.requests);
        }
        self.pending = None;
        self.shown = None;
        self.redirects = 0;
    }
    #[must_use]
    pub fn shown(&self) -> Option<&Shown> {
        self.shown.as_ref()
    }
    fn open(&mut self, address: Box<[u8]>) {
        let parsed = address::parse(&address);
        self.shown = Some(Shown { address, status: 0, body: Box::new([]), failed: false });
        let Some(parsed) = parsed else {
            self.shown.as_mut().expect("visit").failed = true;
            return;
        };
        // Replacement waits for its predecessor's abort to reclaim the socket.
        if self.connections.len() >= usize::try_from(self.limits.sockets).expect("socket count") {
            self.pending = Some(self.shown.take().expect("visit").address);
            return;
        }
        self.serial = self.serial.checked_add(1).expect("tab tickets fit u64");
        let owner = Token::new(self.serial);
        self.connections.insert(owner, Connection::new(&self.http, parsed.authority, parsed.target, self.bound));
        self.current = Some(owner);
        self.requests.push(io::Request::Connect { owner, addr: io::Addr::Ip(parsed.socket) });
    }
    fn failed(&mut self, owner: Token) {
        if self.current == Some(owner) {
            let answer = self.connections.get_mut(&owner).expect("current connection").finish(true, &mut self.requests);
            let shown = self.shown.as_mut().expect("current visit");
            shown.status = answer.status;
            shown.body = answer.body;
            shown.failed = true;
            self.current = None;
        }
        if let Some(connection) = self.connections.get_mut(&owner) {
            connection.close(&mut self.requests);
        }
    }
    fn event(&mut self, event: io::Event, env: &Env<http::Limits>) {
        let Some(owner) = event.owner() else { return };
        let Some(connection) = self.connections.get_mut(&owner) else { return };
        match event {
            io::Event::Connecting { socket, .. } => connection.connecting(socket, &mut self.requests),
            io::Event::Connected { .. } => connection.connected(),
            io::Event::Stream { up, .. } => connection.up(up, env),
            io::Event::Failed { .. } => self.failed(owner),
            io::Event::Closed { .. } => {
                self.failed(owner);
                self.connections.remove(&owner);
            }
            io::Event::Listening { .. } | io::Event::Accepted { .. } | io::Event::Output { .. }
            | io::Event::Spawned { .. } | io::Event::Exited { .. } | io::Event::Usage { .. }
            | io::Event::Resized | io::Event::Window { .. } | io::Event::Shutdown { .. } => {}
        }
    }
}
impl Host for Tab {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits };
        let http_env = Env { now, wall, limits: self.http };
        io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        for _ in 0..self.events.len() {
            if let Some(event) = self.events.pop() { self.event(event, &http_env); }
        }
        if let Some(owner) = self.current {
            let connection = self.connections.get_mut(&owner).expect("current connection");
            if let Some(answer) = connection.progress(&http_env, &mut self.requests) {
                self.current = None;
                let shown = self.shown.as_mut().expect("current visit");
                shown.status = answer.status;
                shown.body = answer.body;
                shown.failed = answer.failed;
                if !shown.failed && [301, 302, 303, 307, 308].contains(&shown.status) {
                    if self.redirects == 8 {
                        shown.failed = true;
                    } else if let Some(location) = answer.location {
                        if let Some(to) = address::resolve(&shown.address, &location) {
                            self.redirects = self.redirects.checked_add(1).expect("eight redirects");
                            self.pending = Some(to);
                        } else {
                            shown.failed = true;
                        }
                    }
                }
            }
        }
        if let Some(address) = self.pending.take() { self.open(address); }
        io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> { &mut self.completions }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> { &mut self.submissions }
    fn work_pending(&self, _: Time) -> bool {
        self.io.is_ready() || !self.events.is_empty() || !self.requests.is_empty() || !self.completions.is_empty()
            || self.pending.is_some() || self.connections.values().any(Connection::work_pending)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> { self.io.next_deadline() }
    fn next_policy_deadline(&self) -> Option<Time> { None }
    fn is_empty(&self) -> bool {
        self.io.is_empty() && self.events.is_empty() && self.requests.is_empty() && self.submissions.is_empty()
            && self.completions.is_empty() && self.connections.is_empty() && self.pending.is_none()
    }
    fn operations(&self) -> u32 { io::operations(&self.limits).expect("bounded io") }
}
