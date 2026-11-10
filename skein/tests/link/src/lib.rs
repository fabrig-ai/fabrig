//! A hand-driven socket and a real rustls peer (skein/docs/design/net/link.md, 4).
use skein_io::Request;
use skein_lib::stream::Down;
use skein_lib::{Duration, Env, Queue, Time, Token};
use skein_link::{Event, Limits, Link, Moved, Start};
use skein_tls::client;
use skein_tls_kit::{drive::Wire, pki, server::Server};

pub const SOCKET: Token = Token::new(1);
// Small plaintext demands, with rustls's maximum handshake (including its header) held.
pub const TLS: client::Limits = client::Limits { read: 1024, send: 1024, records: 65_539 };
pub const CLOSE_TIMEOUT: Duration = Duration::from_secs(3);
pub const NOW: Time = Time::from_nanos(1_000_000_000);
pub const DEADLINE: Time = Time::from_nanos(4_000_000_000);
// A handshake reads headers and bodies separately; 256 steps leaves room for every flight.
const MAX_STEPS: usize = 256;

#[must_use]
pub fn env() -> Env<Limits> {
    Env { now: NOW, wall: pki::VALID, limits: Limits { tls: TLS, close_timeout: CLOSE_TIMEOUT } }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trace {
    Demand,
    Send,
    Empty,
    Finish,
    Close,
    Abort,
}

#[derive(Debug)]
pub struct World {
    pub link: Link,
    pub env: Env<Limits>,
    pub wire: Wire,
    pub below: Queue<Request>,
    pub trace: Vec<Trace>,
    pub above: Vec<Event>,
    pub room: bool,
    socket_closing: bool,
}

impl World {
    #[must_use]
    pub fn fresh() -> World {
        let config = pki::client(&[]);
        let client = client::Client::new(&config, pki::name(), &TLS);
        World {
            link: Link::new(Some(client)),
            env: env(),
            wire: Wire::new(Server::new(pki::Server::plain().config())),
            below: Queue::uncapped(),
            trace: Vec::new(),
            above: Vec::new(),
            room: true,
            socket_closing: false,
        }
    }
    #[must_use]
    pub fn ready() -> World {
        let mut world = Self::fresh();
        world.link.connecting(SOCKET, &mut world.below);
        assert_eq!(world.link.connected(&world.env), Start::Handshaking);
        world.settle();
        assert_eq!(world.above, [Event::Ready], "the TLS handshake finishes");
        world.trace.clear();
        world.above.clear();
        world
    }
    pub fn close(&mut self, abort: bool) {
        self.link.close(&self.env, abort, &mut self.below);
    }
    /// Every move uses at most one io request; requests after Close are dropped as io drops them.
    pub fn settle(&mut self) {
        for _ in 0..MAX_STEPS {
            let before = self.below.len();
            let moved = self.link.pump(&mut self.below);
            assert!(self.below.len() <= before + 1, "one io request per move");
            let progress = moved != Moved::Nothing;
            match moved {
                Moved::Above(event) => self.above.push(event),
                Moved::Nothing | Moved::Within | Moved::Sent => {}
            }
            self.deliver();
            if !progress
                && !self.socket_closing
                && (self.room || self.wire.demand.is_some_and(|(_, room)| room == 0))
                && let Some(up) = self.wire.answer()
            {
                self.link.up(&self.env, up);
                continue;
            }
            if !progress {
                return;
            }
        }
        panic!("the link settles within its flights");
    }
    fn deliver(&mut self) {
        while let Some(request) = self.below.pop() {
            match request {
                Request::Stream { stream, down } => {
                    assert_eq!(stream, SOCKET);
                    self.trace.push(match &down {
                        Down::Demand { .. } => Trace::Demand,
                        Down::Send(bytes) if bytes.is_empty() => Trace::Empty,
                        Down::Send(_) => Trace::Send,
                        Down::Finish => Trace::Finish,
                    });
                    if !self.socket_closing {
                        self.wire.take(down);
                    }
                }
                Request::Close { entity } => {
                    assert_eq!(entity, SOCKET);
                    self.trace.push(Trace::Close);
                    self.socket_closing = true;
                }
                Request::Abort { entity } => {
                    assert_eq!(entity, SOCKET);
                    self.trace.push(Trace::Abort);
                    self.socket_closing = true;
                }
                Request::Listen { .. }
                | Request::Connect { .. }
                | Request::Bind { .. }
                | Request::Reject { .. }
                | Request::Output { .. }
                | Request::Spawn { .. }
                | Request::SpawnAlone { .. }
                | Request::Signal { .. }
                | Request::Usage { .. }
                | Request::Window { .. } => panic!("a link only streams or closes"),
            }
        }
    }
}
