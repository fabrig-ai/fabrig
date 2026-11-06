//! Native HTTP connections on the host's io, with volatile records.
use jig_client as client;
use jig_client_shell_native::Transport;
use skein_io::{self as io, Addr};
use skein_lib::{Queue, Time, Token};
use std::collections::BTreeSet;
const OWNER: u64 = 1 << 60;
pub(crate) struct Memory {
    transport: Transport,
    pub(crate) events: Queue<client::Event>,
    requests: Queue<io::Request>,
    connected: BTreeSet<Token>,
    refused: bool,
    connected_once: bool,
}
impl Memory {
    pub(crate) fn new(engine: Addr, limits: &jig_client_shell_native::Limits) -> Self {
        let sockets = limits.exchanges.checked_add(1).expect("connections fit");
        Self {
            transport: Transport::new(engine, limits, sockets, OWNER),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            connected: BTreeSet::new(),
            refused: false,
            connected_once: false,
        }
    }
    pub(crate) fn down(&mut self, now: Time, request: client::Request) {
        if matches!(request, client::Request::Save { .. } | client::Request::Erase { .. }) {
            self.events.push(client::Event::Stored);
        }
        self.transport.down(now, request);
    }
    pub(crate) fn owns(&self, owner: Token) -> bool {
        self.transport.owns(owner)
    }
    pub(crate) fn event(&mut self, now: Time, event: io::Event) {
        match &event {
            io::Event::Connected { owner } => {
                self.connected.insert(*owner);
                self.connected_once = true;
            }
            io::Event::Failed { owner, error: io::Error::Refused } if !self.connected.contains(owner) => {
                self.refused = true;
            }
            io::Event::Closed { owner } => {
                self.connected.remove(owner);
            }
            io::Event::Connecting { .. }
            | io::Event::Stream { .. }
            | io::Event::Failed { .. }
            | io::Event::Output { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => {}
        }
        self.transport.event(now, event);
    }
    /// A connection taken at any time during this call (client/tools.md, 6.3).
    pub(crate) fn connected(&self) -> bool {
        self.connected_once
    }
    /// A connection the kernel refused, before any was made (client/tools.md, 6.3).
    pub(crate) fn refused(&self) -> bool {
        self.refused
    }
    pub(crate) fn close(&mut self, now: Time) {
        self.transport.close(now);
        while self.events.pop().is_some() {}
    }
    pub(crate) fn progress(&mut self, now: Time) {
        self.transport.iterate(now, &mut self.events, &mut self.requests);
    }
    pub(crate) fn request(&mut self) -> Option<io::Request> {
        self.requests.pop()
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        self.transport.next_deadline()
    }
    pub(crate) fn pending(&self, now: Time) -> bool {
        !self.events.is_empty() || !self.requests.is_empty() || self.transport.work_pending(now)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.transport.is_empty() && self.events.is_empty() && self.requests.is_empty()
    }
}
