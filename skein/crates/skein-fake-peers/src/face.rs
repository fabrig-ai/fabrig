//! Shared io passes and declared queue/connection bounds; no peer semantics.

use std::net::SocketAddr;

use skein_io::{self as io, kernel};
use skein_lib::{Env, Queue, Time, Token, Wall};

pub(crate) const LISTENER: Token = Token::new(u64::MAX);

/// Fixed connection, plaintext/TLS buffer and observation caps supplied by a world.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub io: io::Limits,
    pub connections: u32,
    /// Each internal queue's record count; at least 64.
    pub queue: u32,
    pub plaintext: u32,
    pub ciphertext: u32,
    pub observations: u32,
    /// Total owned payload bytes held by observations.
    pub observation_bytes: u32,
}

/// Constructor refusal returned to the world before any effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The world supplied unusable limits, configuration or a non-loopback address,
    /// or requested TLS on the portable target (testing.md, 10).
    Limits,
}

/// One listening host's io passes and queues (skein/docs/design/testing.md, 5).
/// The host routes events, retains its listener and address, and closes admission.
pub struct Face {
    pub io: io::Io,
    pub limits: Limits,
    pub events: Queue<io::Event>,
    pub requests: Queue<io::Request>,
    pub submissions: Queue<kernel::Submit>,
    pub completions: Queue<kernel::Complete>,
    pub listener: Option<Token>,
    pub address: Option<SocketAddr>,
    pub stopping: bool,
}

impl Face {
    pub fn new(address: SocketAddr, limits: Limits) -> Result<Self, Error> {
        if !address.ip().is_loopback() || !valid(&limits) {
            return Err(Error::Limits);
        }
        let mut requests = Queue::uncapped();
        requests.push(io::Request::Listen { owner: LISTENER, addr: io::Addr::Ip(address) });
        Ok(Self {
            io: io::Io::new(&limits.io),
            limits,
            events: Queue::uncapped(),
            requests,
            submissions: Queue::with_capacity(limits.queue),
            completions: Queue::with_capacity(limits.queue),
            listener: None,
            address: None,
            stopping: false,
        })
    }

    pub fn up(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits.io };
        io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
    }

    pub fn down(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits.io };
        io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }

    /// The scenario ends admission after its clients settle, or names a peer hang-up.
    pub fn shutdown(&mut self) {
        self.stopping = true;
    }

    pub fn close_listener(&mut self) {
        if self.stopping
            && let Some(listener) = self.listener.take()
        {
            self.requests.push(io::Request::Close { entity: listener });
        }
    }

    #[must_use]
    pub fn work_pending(&self) -> bool {
        self.io.is_ready()
            || !self.completions.is_empty()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || (self.stopping && self.listener.is_some())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
    }
}

/// Whether these limits are representable: every edge bound they size is finite
/// and every capacity they derive fits (programming-model.md, 6.3).
/// The listener and connections fit io, and transport buffers fit its streams.
pub(crate) fn valid(limits: &Limits) -> bool {
    limits.io.is_usable()
        && limits.queue >= 64
        && limits.connections > 0
        && limits.connections.checked_add(1).is_some_and(|sockets| limits.io.sockets >= sockets)
        && io::operations(&limits.io).is_some_and(|operations| operations.checked_mul(2).is_some())
        && limits.plaintext > 0
        && limits.ciphertext >= 18_432
        && limits.io.intake >= 18_432
        && limits.io.output >= limits.ciphertext
}

#[cfg(test)]
mod tests {
    use super::{Error, Face, Limits};
    use skein_io::{self as io, kernel};
    use skein_lib::{Time, Token, Wall};

    fn limits() -> Limits {
        Limits {
            io: io::Limits { intake: 18_432, output: 18_432, ..io::Limits::TEST },
            connections: 1,
            queue: 64,
            plaintext: 1024,
            ciphertext: 18_432,
            observations: 1,
            observation_bytes: 1024,
        }
    }

    #[test]
    fn a_public_face_drives_a_listen_refusal_and_drains_on_shutdown() {
        let mut face = Face::new("127.0.0.1:0".parse().expect("loopback"), limits()).expect("usable limits");
        assert!(face.work_pending());
        face.down(Time::ZERO, Wall::EPOCH);
        let submit = face.submissions.pop().expect("socket submission");
        face.completions.push(kernel::Complete {
            op: submit.op,
            kind: submit.kind,
            result: Err(kernel::Error::TooManyOpenFiles),
        });
        face.up(Time::ZERO, Wall::EPOCH);
        assert!(face.events.pop().expect("refusal").owner().is_some());
        assert!(face.events.pop().expect("closed").is_closed());
        face.shutdown();
        face.close_listener();
        face.down(Time::ZERO, Wall::EPOCH);
        assert!(face.is_empty());
        assert!(!face.work_pending());
    }

    #[test]
    fn limits_and_loopback_are_checked_before_starting() {
        assert!(matches!(Face::new("192.0.2.1:0".parse().expect("address"), limits()), Err(Error::Limits)));
        let mut invalid = limits();
        invalid.queue = 63;
        assert!(matches!(Face::new("127.0.0.1:0".parse().expect("address"), invalid), Err(Error::Limits)));
        let mut face = Face::new("127.0.0.1:0".parse().expect("address"), limits()).expect("usable limits");
        face.requests.clear();
        face.listener = Some(Token::new(3));
        face.close_listener();
        assert!(face.requests.is_empty());
        face.shutdown();
        face.close_listener();
        assert_eq!(face.requests.pop(), Some(io::Request::Close { entity: Token::new(3) }));
        face.close_listener();
        assert!(face.requests.is_empty());
    }
}
