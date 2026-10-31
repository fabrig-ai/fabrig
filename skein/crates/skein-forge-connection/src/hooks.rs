//! Loopback webhook listener and its bounded connections (forge-connection.md, 5, 7 and 8).
use crate::{Event, Limits};
use skein_forge::api;
use skein_io::Addr;
use skein_lib::{Env, Id, Map, Queue, Slab, Time, Token};
pub(crate) mod connection;
use connection::{Closing, Hooked};
/// The listener; each forge's webhook secret is in its access.
#[derive(Clone, Debug)]
pub struct Webhooks {
    pub listener: Addr,
}
/// A delivery hint, naming its webhook or log-read connection for refusal.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hinted {
    pub connection: Token,
    pub hint: api::Hint,
}
/// The listener's end, including escalation after a graceful close.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ending {
    Live,
    Closing,
    Aborting,
}
impl Ending {
    fn live(self) -> bool {
        match self {
            Self::Live => true,
            Self::Closing | Self::Aborting => false,
        }
    }
}
/// An accept held outside the full webhook slab, or settling after an end request.
#[derive(Debug)]
enum Accepted {
    Waiting { socket: Token, at: Time },
    Closing { socket: Token, owner: Token },
    Aborting { owner: Token },
}
const OWNER: Token = Token::new(1_u64 << 59_u32);
#[derive(Debug)]
pub(crate) struct Hooks {
    listener: Option<Token>,
    waiting: Option<Accepted>,
    connections: Map<Token, Id<Hooked>>,
    slab: Slab<Hooked>,
    next: u64,
    live: bool,
    ending: Ending,
    close_sent: bool,
    wake: Option<Time>,
}
impl Hooks {
    pub fn new(l: &Limits) -> Self {
        Self {
            listener: None,
            waiting: None,
            connections: Map::with_capacity(l.hooks),
            slab: Slab::with_capacity(l.hooks),
            next: OWNER.raw().saturating_add(1),
            live: false,
            ending: Ending::Live,
            close_sent: false,
            wake: None,
        }
    }
    pub fn begin(&mut self, settings: &Webhooks, io: &mut Queue<skein_io::Request>) {
        self.live = true;
        io.push(skein_io::Request::Listen { owner: OWNER, addr: settings.listener });
    }
    pub fn owns(&self, owner: Token) -> bool {
        let accepted = match self.waiting {
            Some(Accepted::Closing { owner: accepted, .. } | Accepted::Aborting { owner: accepted }) => {
                owner == accepted
            }
            Some(Accepted::Waiting { .. }) | None => false,
        };
        owner == OWNER || self.connections.contains_key(&owner) || accepted
    }
    pub fn up(&mut self, env: &Env<Limits>, event: skein_io::Event, up: &mut Queue<Event>) {
        match event {
            skein_io::Event::Listening { listener, addr, .. } => {
                self.listener = Some(listener);
                if self.ending.live() {
                    up.push(Event::Listening { addr });
                }
            }
            skein_io::Event::Accepted { socket, .. } => {
                assert!(self.waiting.is_none(), "one unanswered accept per listener");
                self.waiting = Some(Accepted::Waiting { socket, at: env.now });
            }
            skein_io::Event::Stream { owner, up } => {
                if let Some(id) = self.connections.get(&owner) {
                    self.slab.get_mut(*id).expect("indexed hook").stream(env, up);
                }
            }
            skein_io::Event::Failed { owner, .. } => {
                if owner == OWNER {
                    if self.ending.live() {
                        up.push(Event::ListenerLost);
                    }
                    self.close();
                } else if let Some(id) = self.connections.get(&owner) {
                    let c = self.slab.get_mut(*id).expect("indexed hook");
                    match c.closing {
                        Closing::Open | Closing::Pending => c.closing = Closing::Pending,
                        Closing::Graceful | Closing::Aborted => {}
                    }
                }
            }
            skein_io::Event::Closed { owner } => {
                if owner == OWNER {
                    self.listener = None;
                    self.live = false;
                } else if let Some(id) = self.connections.remove(&owner) {
                    self.slab.retire(id);
                } else {
                    match self.waiting {
                        Some(Accepted::Closing { owner: accepted, .. } | Accepted::Aborting { owner: accepted })
                            if owner == accepted =>
                        {
                            self.waiting = None;
                        }
                        Some(Accepted::Closing { .. } | Accepted::Aborting { .. } | Accepted::Waiting { .. })
                        | None => {}
                    }
                }
            }
            skein_io::Event::Connecting { .. }
            | skein_io::Event::Connected { .. }
            | skein_io::Event::Output { .. }
            | skein_io::Event::Spawned { .. }
            | skein_io::Event::Exited { .. }
            | skein_io::Event::Usage { .. }
            | skein_io::Event::Resized
            | skein_io::Event::Window { .. }
            | skein_io::Event::Shutdown { .. } => {}
        }
    }
    fn next_owner(&mut self) -> Token {
        let owner = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("hook token space");
        assert!(self.next < 1_u64 << 60_u32, "component owner prefix");
        owner
    }
    pub fn refused(&mut self, hinted: Hinted) {
        if let Some(id) = self.connections.get(&hinted.connection) {
            self.slab.get_mut(*id).expect("indexed hook").refused(hinted.hint);
        }
    }
    pub fn offer(&mut self) {
        for (_, id) in &self.connections {
            self.slab.get_mut(*id).expect("indexed hook").offer();
        }
    }
    pub fn close(&mut self) {
        if self.ending.live() {
            self.ending = Ending::Closing;
        }
    }
    pub fn abort(&mut self) {
        if self.ending != Ending::Aborting {
            self.ending = Ending::Aborting;
            self.close_sent = false;
        }
    }
    pub fn closed(&self) -> bool {
        !self.live && self.connections.is_empty() && self.waiting.is_none()
    }
    pub fn deadline(&self) -> Option<Time> {
        let mut earliest = self.wake;
        for (_, id) in &self.connections {
            let c = self.slab.get(*id).expect("indexed hook");
            let deadline = match c.closing {
                Closing::Open | Closing::Pending => Some(c.deadline),
                Closing::Graceful | Closing::Aborted => None,
            };
            if let Some(deadline) = deadline
                && (earliest.is_none() || Some(deadline) < earliest)
            {
                earliest = Some(deadline);
            }
        }
        earliest
    }
    pub fn drive(
        &mut self,
        env: &Env<Limits>,
        settings: &crate::settings::Settings,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        self.wake = None;
        if !self.ending.live()
            && let Some(listener) = self.listener
            && !self.close_sent
        {
            io.push(match self.ending {
                Ending::Closing => skein_io::Request::Close { entity: listener },
                Ending::Aborting => skein_io::Request::Abort { entity: listener },
                Ending::Live => unreachable!("ending listener"),
            });
            self.close_sent = true;
        }
        if let Some(accepted) = self.waiting.take() {
            match accepted {
                Accepted::Waiting { socket, at } => match self.ending {
                    Ending::Aborting => {
                        let owner = self.next_owner();
                        io.push(skein_io::Request::Bind { socket, owner });
                        io.push(skein_io::Request::Abort { entity: socket });
                        self.waiting = Some(Accepted::Aborting { owner });
                    }
                    Ending::Closing => {
                        let owner = self.next_owner();
                        io.push(skein_io::Request::Bind { socket, owner });
                        io.push(skein_io::Request::Close { entity: socket });
                        self.waiting = Some(Accepted::Closing { socket, owner });
                    }
                    Ending::Live => {
                        if self.connections.len() < self.connections.capacity() {
                            let owner = self.next_owner();
                            self.slab.reclaim();
                            let mut c = Hooked::new(socket, env);
                            c.deadline = at.saturating_add(env.limits.hook_deadline);
                            let id = self.slab.insert(c).expect("connection room");
                            let _previous = self.connections.insert(owner, id).expect("connection index room");
                            io.push(skein_io::Request::Bind { socket, owner });
                        } else {
                            self.waiting = Some(Accepted::Waiting { socket, at });
                        }
                    }
                },
                Accepted::Closing { socket, owner } => match self.ending {
                    Ending::Aborting => {
                        io.push(skein_io::Request::Abort { entity: socket });
                        self.waiting = Some(Accepted::Aborting { owner });
                    }
                    Ending::Closing => self.waiting = Some(Accepted::Closing { socket, owner }),
                    Ending::Live => unreachable!("a closing accept cannot become live"),
                },
                Accepted::Aborting { owner } => self.waiting = Some(Accepted::Aborting { owner }),
            }
        }
        for (owner, id) in &self.connections {
            let c = self.slab.get_mut(*id).expect("indexed hook");
            let abort = match c.closing {
                Closing::Open | Closing::Pending => !self.ending.live() || c.deadline <= env.now,
                Closing::Graceful => self.ending == Ending::Aborting,
                Closing::Aborted => false,
            };
            if abort {
                c.closing = Closing::Aborted;
                io.push(skein_io::Request::Abort { entity: c.socket });
            }
            if c.closing == Closing::Open || c.closing == Closing::Pending {
                c.pump(*owner, env, settings, up, io);
                if c.pending() {
                    self.wake = Some(env.now);
                }
            }
        }
    }
}
