//! The stream and its close rule (skein/docs/design/net/link.md, 3.1–3.3).
use core::fmt;
use skein_io as io;
use skein_lib::stream::{Down, Up};
use skein_lib::{Duration, Env, Queue, Time, Token};
use skein_tls::client as tls;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub tls: tls::Limits,
    pub close_timeout: Duration,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Start {
    Handshaking,
    Ready,
    Closing,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Ready,
    Stream(Up),
    Failed,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Moved {
    Nothing,
    Within,
    Sent,
    Above(Event),
}

#[derive(Clone, Copy, Debug)]
enum Ending {
    Open,
    Flushing { until: Time },
    Graceful,
    Aborted,
    Closed,
}

pub struct Link {
    socket: Option<Token>,
    tls: Option<tls::Client>,
    events: Queue<tls::Event>,
    cipher: Queue<Down>,
    ending: Ending,
}
#[expect(clippy::missing_fields_in_debug, reason = "show TLS only as encrypted")]
impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Link");
        debug.field("socket", &self.socket).field("encrypted", &self.encrypted());
        debug.field("events", &self.events.len()).field("cipher", &self.cipher.len());
        debug.field("ending", &self.ending).finish()
    }
}

impl Link {
    #[must_use]
    pub fn new(tls: Option<tls::Client>) -> Link {
        Link { socket: None, tls, events: Queue::uncapped(), cipher: Queue::uncapped(), ending: Ending::Open }
    }
    #[must_use]
    pub fn encrypted(&self) -> bool {
        self.tls.is_some()
    }
    pub fn connecting(&mut self, socket: Token, below: &mut Queue<io::Request>) {
        match self.ending {
            Ending::Closed => return,
            Ending::Open | Ending::Flushing { .. } | Ending::Graceful | Ending::Aborted => {}
        }
        self.socket = Some(socket);
        self.ask(below);
    }
    pub fn connected(&mut self, env: &Env<Limits>) -> Start {
        match self.ending {
            Ending::Open => match &mut self.tls {
                Some(client) => {
                    tls::down(
                        client,
                        &env.with(env.limits.tls),
                        tls::Request::Handshake,
                        &mut self.events,
                        &mut self.cipher,
                    );
                    Start::Handshaking
                }
                None => Start::Ready,
            },
            Ending::Flushing { .. } | Ending::Graceful | Ending::Aborted | Ending::Closed => Start::Closing,
        }
    }
    pub fn up(&mut self, env: &Env<Limits>, event: Up) {
        match self.ending {
            Ending::Open | Ending::Flushing { .. } => match &mut self.tls {
                Some(client) => tls::up(client, &env.with(env.limits.tls), event, &mut self.events, &mut self.cipher),
                None => self.events.push(tls::Event::Stream(event)),
            },
            Ending::Graceful | Ending::Aborted | Ending::Closed => {}
        }
    }
    pub fn down(&mut self, env: &Env<Limits>, down: Down) {
        match self.ending {
            Ending::Open => match &mut self.tls {
                Some(client) => tls::down(
                    client,
                    &env.with(env.limits.tls),
                    tls::Request::Stream(down),
                    &mut self.events,
                    &mut self.cipher,
                ),
                None => self.cipher.push(down),
            },
            Ending::Flushing { .. } | Ending::Graceful | Ending::Aborted | Ending::Closed => {}
        }
    }
    pub fn pump(&mut self, below: &mut Queue<io::Request>) -> Moved {
        if let Some(down) = self.cipher.pop() {
            let moved = match &down {
                Down::Send(bytes) if !bytes.is_empty() => Moved::Sent,
                Down::Send(_) | Down::Demand { .. } | Down::Finish => Moved::Within,
            };
            below.push(io::Request::Stream { stream: self.socket.expect("a stream after Connecting"), down });
            return moved;
        }
        let Some(event) = self.events.pop() else { return Moved::Nothing };
        match (self.ending, event) {
            (Ending::Open, tls::Event::Ready(_)) => Moved::Above(Event::Ready),
            (Ending::Open, tls::Event::Stream(up)) => Moved::Above(Event::Stream(up)),
            (Ending::Open, tls::Event::Failed(_)) => Moved::Above(Event::Failed),
            (Ending::Open, tls::Event::Closed) => unreachable!("TLS Closed answers the link's Close"),
            (Ending::Flushing { .. }, tls::Event::Closed) => {
                self.end(Ending::Graceful, below);
                Moved::Within
            }
            (Ending::Flushing { .. }, tls::Event::Ready(_) | tls::Event::Stream(_) | tls::Event::Failed(_))
            | (
                Ending::Graceful | Ending::Aborted | Ending::Closed,
                tls::Event::Ready(_) | tls::Event::Stream(_) | tls::Event::Failed(_) | tls::Event::Closed,
            ) => Moved::Within,
        }
    }
    pub fn close(&mut self, env: &Env<Limits>, abort: bool, below: &mut Queue<io::Request>) {
        match self.ending {
            Ending::Open | Ending::Flushing { .. } | Ending::Graceful if abort => {
                self.end(Ending::Aborted, below);
            }
            Ending::Open => match &mut self.tls {
                Some(client) => {
                    tls::down(
                        client,
                        &env.with(env.limits.tls),
                        tls::Request::Close,
                        &mut self.events,
                        &mut self.cipher,
                    );
                    self.ending = Ending::Flushing { until: env.now.saturating_add(env.limits.close_timeout) };
                }
                None => self.end(Ending::Graceful, below),
            },
            Ending::Flushing { .. } | Ending::Graceful | Ending::Aborted | Ending::Closed => {}
        }
    }
    pub fn closed(&mut self) {
        self.ending = Ending::Closed;
        self.events.clear();
        self.cipher.clear();
        self.socket = None;
    }
    #[must_use]
    pub fn has_work(&self) -> bool {
        !self.cipher.is_empty() || !self.events.is_empty()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        match self.ending {
            Ending::Flushing { until } => Some(until),
            Ending::Open | Ending::Graceful | Ending::Aborted | Ending::Closed => None,
        }
    }
    /// `next_deadline` has come: `fire` at `now` aborts.
    #[must_use]
    pub fn due(&self, now: Time) -> bool {
        match self.ending {
            Ending::Flushing { until } => now >= until,
            Ending::Open | Ending::Graceful | Ending::Aborted | Ending::Closed => false,
        }
    }
    pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) {
        if self.due(now) {
            self.end(Ending::Aborted, below);
        }
    }
    /// Ends the stream: nothing queued leaves, and the socket hears what `next` says, or will once it is known.
    fn end(&mut self, next: Ending, below: &mut Queue<io::Request>) {
        self.events.clear();
        self.cipher.clear();
        self.ending = next;
        self.ask(below);
    }
    fn ask(&self, below: &mut Queue<io::Request>) {
        let Some(entity) = self.socket else { return };
        match self.ending {
            Ending::Graceful => below.push(io::Request::Close { entity }),
            Ending::Aborted => below.push(io::Request::Abort { entity }),
            Ending::Open | Ending::Flushing { .. } | Ending::Closed => {}
        }
    }
}
