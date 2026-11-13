//! Accepted streams and the workers' component (jig/docs/design/protocol/channel/README.md, 9.2).
//! The owner's pull and listener lifecycle (jig/docs/design/protocol/service.md, 11.3).
use crate::{Component, Event, Limits, Loss, Request, Worker};
use alloc::boxed::Box;
use jig_domain_host as hub;
use jig_domain_link::Grant;
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_worker_wire::{Agreed, Part, Spoken};
use skein_channel::{Lower, LowerEvent};
use skein_io as io;
use skein_lib::{List, Map, Queue, Time, Token, stream};

/// The owner's namespaces for this part's io (skein/docs/design/runtime/lib.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tokens {
    pub listeners: u8,
    pub connections: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Closing {
    Drain,
    Abort,
}

/// What the channel hands its root, in the component's order; the owner routes each before the next.
#[derive(Debug)]
pub enum Output {
    Opened { worker: Box<[u8]>, wires: List<Agreed> },
    Refused { worker: Option<Box<[u8]>>, reason: u16, text: Box<[u8]> },
    Hello { channel: Token, worker: u32, hello: jig_domain_link::Hello },
    Up { channel: Token, up: hub::LinkUp, parts: Box<[Part]> },
    Lost { channel: Token },
    Unsent { worker: Box<[u8]>, channel: Token, unsent: crate::Unsent },
    Ended { worker: Option<Box<[u8]>>, channel: Option<Token>, why: Loss },
}

/// What became of an event `up` was given.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Seen {
    Taken,
    /// An event this part has no use for.
    Unknown,
    /// A listener closed: `ending` says whether the part was already closing.
    ListenerClosed {
        ending: bool,
    },
}

#[expect(clippy::partial_pub_fields, reason = "the owner reads io, the bound address and the component")]
pub struct Workers {
    /// Bound address awaiting the owner's report.
    pub listening: Option<core::net::SocketAddr>,
    pub component: Component,
    pub io: Queue<io::Request>,
    out: Queue<Request>,
    listeners: List<(Token, io::Addr, Option<Token>)>,
    sockets: Map<Token, Token>,
    tokens: Tokens,
    next: u64,
    opened: bool,
    ending: Option<Closing>,
    component_closed: bool,
}
impl core::fmt::Debug for Workers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Workers")
            .field("listening", &self.listening)
            .field("opened", &self.opened)
            .field("ending", &self.ending)
            .field("io", &self.io.len())
            .field("out", &self.out.len())
            .field("sockets", &self.sockets.len())
            .field("listeners", &self.listeners.len())
            .finish_non_exhaustive()
    }
}
/// A problem at `engine.hosts`.
#[must_use]
pub fn invalid(what: What) -> Problems {
    let mut found = Problems::empty();
    found.push(Path::root().member(b"engine").member(b"hosts"), what);
    found
}
impl Workers {
    pub fn new(
        config: &config::Engine,
        files: &config::Files,
        tokens: Tokens,
        wires: List<Spoken>,
        limits: &Limits,
    ) -> Result<Self, Problems> {
        let mut workers = List::with_capacity(limits.names);
        for worker in &config.hosts.workers {
            let digest = match config::digest(files.contents(&worker.digest).ok_or(invalid(What::Unreadable))?) {
                Ok(digest) => digest,
                Err(what) => return Err(invalid(what)),
            };
            workers.push(Worker { name: worker.name.clone(), digest }).or(Err(invalid(What::Bound)))?;
        }
        let component = Component::new(limits, workers.into_boxed(), wires).or(Err(invalid(What::Bound)))?;
        let count = u32::try_from(config.hosts.listener.len()).or(Err(invalid(What::Bound)))?;
        if count == 0 || count > config.limits.io.sockets {
            return Err(invalid(What::Bound));
        }
        let mut listeners = List::with_capacity(count);
        let mut io = Queue::uncapped();
        for (index, address) in config.hosts.listener.iter().enumerate() {
            let config::Address::Addr(address) = address else {
                return Err(invalid(What::Form));
            };
            let owner = Token::tagged(tokens.listeners, u64::try_from(index).expect("listener count"))
                .expect("service token fits its namespace");
            io.push(io::Request::Listen { owner, addr: *address });
            listeners.push((owner, *address, None)).expect("listener count");
        }
        Ok(Self {
            listening: None,
            component,
            out: Queue::uncapped(),
            io,
            listeners,
            sockets: Map::with_capacity(limits.workers.checked_add(limits.opening).expect("connections")),
            next: 1,
            opened: false,
            ending: None,
            component_closed: false,
            tokens,
        })
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.component.next_deadline()
    }
    #[must_use]
    pub fn opened(&self) -> bool {
        self.opened
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        let deadline = self.next_deadline();
        !self.io.is_empty() || !self.out.is_empty() || (deadline.is_some() && deadline <= Some(now))
    }
    #[must_use]
    pub fn ending(&self) -> Option<Closing> {
        self.ending
    }
    #[must_use]
    pub fn listener_address(&self, owner: Option<Token>) -> Option<io::Addr> {
        for (name, address, _) in &self.listeners {
            if Some(*name) == owner {
                return Some(*address);
            }
        }
        None
    }
    pub fn down(&mut self, now: Time, event: Event) {
        self.component.down(now, event, &mut self.out);
    }
    pub fn up(&mut self, now: Time, event: io::Event) -> Seen {
        let Some(owner) = event.owner() else {
            return Seen::Unknown;
        };
        match event {
            io::Event::Listening { listener, addr, .. } => {
                self.listening = addr.socket();
                for index in 0..self.listeners.len() {
                    let (name, _, slot) = self.listeners.get_mut(index).expect("listener index");
                    if *name == owner {
                        *slot = Some(listener);
                    }
                }
                self.opened = listeners_open(&self.listeners);
                self.late_listener(listener);
            }
            io::Event::Accepted { socket, .. } => {
                let connection =
                    Token::tagged(self.tokens.connections, self.next).expect("service token fits its namespace");
                self.next = self.next.checked_add(1).expect("connection owner sequence");
                if self.sockets.insert(connection, socket).is_err() {
                    self.io.push(io::Request::Reject { socket });
                    self.out.push(Request::Ended { connection, worker: None, channel: None, why: Loss::NoRoom });
                } else {
                    self.down(now, Event::Accepted { connection });
                }
            }
            io::Event::Stream { up, .. } => {
                self.down(now, Event::Lower { connection: owner, event: LowerEvent::Read(up) });
            }
            io::Event::Output { up, .. } => {
                self.down(now, Event::Lower { connection: owner, event: LowerEvent::Write(up) });
            }
            io::Event::Closed { .. } => {
                if self.listener_closed(owner) {
                    return Seen::ListenerClosed { ending: self.ending.is_some() };
                }
                let _socket = self.sockets.remove(&owner);
                self.down(now, Event::Gone { connection: owner });
            }
            io::Event::Failed { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => return Seen::Unknown,
        }
        Seen::Taken
    }
    /// The component's requests waiting for `output`.
    #[must_use]
    pub fn pending(&self) -> u32 {
        self.out.len()
    }
    /// Takes one request, doing itself what needs no owner.
    pub fn output(&mut self) -> Option<Output> {
        match self.out.pop()? {
            Request::Take { connection } => self.io.push(io::Request::Bind {
                socket: *self.sockets.get(&connection).expect("accepted socket"),
                owner: connection,
            }),
            Request::TurnAway { connection } => {
                if let Some(socket) = self.sockets.remove(&connection) {
                    self.io.push(io::Request::Reject { socket });
                }
            }
            Request::Close { connection } => {
                if let Some(socket) = self.sockets.get(&connection) {
                    self.io.push(io::Request::Close { entity: *socket });
                }
            }
            Request::Abort { connection } => {
                if let Some(socket) = self.sockets.get(&connection) {
                    self.io.push(io::Request::Abort { entity: *socket });
                }
            }
            Request::Lower { connection, lower } => {
                if let Some(socket) = self.sockets.get(&connection) {
                    self.io.push(match lower {
                        Lower::Read(down) => io::Request::Stream { stream: *socket, down },
                        Lower::Write(down) => io::Request::Output { stream: *socket, down },
                        Lower::FinishWrite => io::Request::Stream { stream: *socket, down: stream::Down::Finish },
                    });
                }
            }
            Request::Opened { worker, wires } => return Some(Output::Opened { worker: self.component.workers.get(skein_lib::usize_of(worker)).expect("configured worker").name.clone(), wires }),
            Request::Refused { worker, reason, text } => {
                let worker = match worker {
                    Some(index) => Some(self.component.workers.get(skein_lib::usize_of(index)).expect("configured worker").name.clone()),
                    None => None,
                };
                return Some(Output::Refused { worker, reason, text });
            }
            Request::Hello { channel, worker, hello } => return Some(Output::Hello { channel, worker, hello }),
            Request::Up { channel, up, parts } => return Some(Output::Up { channel, up, parts }),
            Request::Lost { channel } => return Some(Output::Lost { channel }),
            Request::Unsent { worker, channel, unsent } => {
                let worker = self
                    .component
                    .workers
                    .get(skein_lib::length::usize_of(worker))
                    .expect("authenticated worker")
                    .name
                    .clone();
                return Some(Output::Unsent { worker, channel, unsent });
            }
            Request::Ended { worker, channel, why, .. } => {
                let worker = match worker {
                    Some(worker) => Some(
                        self.component
                            .workers
                            .get(skein_lib::length::usize_of(worker))
                            .expect("authenticated worker")
                            .name
                            .clone(),
                    ),
                    None => None,
                };
                return Some(Output::Ended { worker, channel, why });
            }
            Request::Closed => self.component_closed = true,
        }
        None
    }
    pub fn iterate(&mut self, now: Time) {
        self.component.fire(now, &mut self.out);
    }
    pub fn abort(&mut self, now: Time) {
        if self.ending != Some(Closing::Abort) {
            self.ending = Some(Closing::Abort);
            for (_, _, listener) in &self.listeners {
                if let Some(listener) = listener {
                    self.io.push(io::Request::Abort { entity: *listener });
                }
            }
            self.down(now, Event::Abort);
        }
    }
    fn listener_closed(&mut self, owner: Token) -> bool {
        for index in 0..self.listeners.len() {
            let (name, _, listener) = self.listeners.get_mut(index).expect("listener index");
            if *name == owner {
                *listener = None;
                return true;
            }
        }
        false
    }
    fn late_listener(&mut self, listener: Token) {
        match self.ending {
            Some(Closing::Abort) => self.io.push(io::Request::Abort { entity: listener }),
            Some(Closing::Drain) => self.io.push(io::Request::Close { entity: listener }),
            None => {}
        }
    }
    /// Close listeners and drain the channel component.
    pub fn close(&mut self, now: Time) {
        self.ending = Some(Closing::Drain);
        for (_, _, listener) in &self.listeners {
            if let Some(listener) = listener {
                self.io.push(io::Request::Close { entity: *listener });
            }
        }
        self.down(now, Event::Close);
    }
    /// Every listener, accepted socket and queued request has closed.
    #[must_use]
    pub fn closed(&self) -> bool {
        if !self.component_closed || !self.sockets.is_empty() || !self.io.is_empty() {
            return false;
        }
        for (_, _, listener) in &self.listeners {
            if listener.is_some() {
                return false;
            }
        }
        true
    }
}
fn listeners_open(listeners: &List<(Token, io::Addr, Option<Token>)>) -> bool {
    for (_, _, entity) in listeners {
        if entity.is_none() {
            return false;
        }
    }
    true
}

/// The grants a down carries, whose values the owner holds, in order.
#[must_use]
pub fn grants(down: &hub::LinkDown) -> &[Grant] {
    match down {
        hub::LinkDown::AssignJob { job } => &job.grants,
        hub::LinkDown::Assign { assignment } => &assignment.assignment.grants,
        hub::LinkDown::ReadList { grant, .. } | hub::LinkDown::Grant { grant, .. } => ::core::slice::from_ref(grant),
        hub::LinkDown::Sources { .. }
        | hub::LinkDown::Check { .. }
        | hub::LinkDown::SignOut { .. }
        | hub::LinkDown::Message { .. }
        | hub::LinkDown::Relayed { .. }
        | hub::LinkDown::TurnKept { .. }
        | hub::LinkDown::Acknowledge { .. }
        | hub::LinkDown::Cancel { .. } => &[],
    }
}
