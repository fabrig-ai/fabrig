//! Readiness follows the peer's claim, with or without a kernel Listen.
use super::{FIRST, Peers};
use crate::harness::Lent;
use skein_fake_peers::Listening;
use skein_io::{
    self as io,
    kernel::{Complete, Fd, Op, Submit},
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host, Wait};
use skein_world::{Deadline, Expired, ring::Ring};
use std::{
    net::{SocketAddr, TcpListener},
    os::fd::{AsRawFd, IntoRawFd},
};

struct Peer {
    io: io::Io,
    env: Env<io::Limits>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    listener: Option<Token>,
    address: Option<SocketAddr>,
    adopted: bool,
    stopping: bool,
}
impl Peer {
    fn new(listener: Option<TcpListener>) -> Self {
        let limits = io::Limits {
            sockets: 1,
            intake: 4096,
            output: 4096,
            sends: 1,
            retry: Duration::from_millis(1),
            ..io::Limits::TEST
        };
        let mut io = io::Io::new(&limits);
        let adopted = listener.is_some();
        let address = listener.map_or_else(
            || ([127, 0, 0, 1], 0).into(),
            |listener| {
                let address = listener.local_addr().expect("held address");
                io.adopt_listener(Fd::new(listener.as_raw_fd()), address).expect("held listener");
                let _ = listener.into_raw_fd();
                address
            },
        );
        let mut requests = Queue::with_capacity(64);
        requests.push(io::Request::Listen { owner: Token::new(1), addr: io::Addr::Ip(address) });
        Self {
            io,
            env: Env { limits, now: Time::ZERO, wall: Wall::EPOCH },
            completions: Queue::with_capacity(64),
            submissions: Queue::with_capacity(64),
            events: Queue::with_capacity(64),
            requests,
            listener: None,
            address: None,
            adopted,
            stopping: false,
        }
    }
}
impl Listening for Peer {
    fn address(&self) -> SocketAddr {
        self.address.expect("the peer processed listening before its address is read")
    }
    fn listening_address(&self) -> Option<SocketAddr> {
        // The ordinary path exercises compatibility with peers that only
        // expose their address after the harness delivers a Listen completion.
        self.adopted.then_some(self.address).flatten()
    }
    fn shutdown(&mut self) {
        self.stopping = true;
    }
}
impl Host for Peer {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        io::stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        while let Some(event) = self.events.pop() {
            match event {
                io::Event::Listening { listener, addr, .. } => {
                    self.listener = Some(listener);
                    self.address = addr.socket();
                }
                io::Event::Closed { .. } => {}
                other @ (io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Stream { .. }
                | io::Event::Output { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. }
                | io::Event::Failed { .. }) => panic!("unexpected listener event: {other:?}"),
            }
        }
        if self.stopping
            && let Some(listener) = self.listener.take()
        {
            self.requests.push(io::Request::Close { entity: listener });
        }
        io::stage::down(&mut self.io, &self.env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        self.io.is_ready()
            || !self.completions.is_empty()
            || !self.requests.is_empty()
            || self.stopping && self.listener.is_some()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.io.is_empty() && self.requests.is_empty() && self.completions.is_empty() && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("listener operations")
    }
}

fn ready_and_drained(mut peer: Peer) -> (SocketAddr, usize) {
    let mut lent = [Lent { name: b"listener", peer: &mut peer }];
    let mut peers = Peers::new(&mut lent);
    let mut limits = vec![0; FIRST];
    limits.extend_from_slice(&peers.limits);
    let mut ring = Ring::new(limits.iter().sum(), limits).expect("listener kernel");
    let clock = Clock::new();
    let end = Deadline::arm("the lent peer's readiness and shutdown", Duration::from_secs(2));
    let mut starved = None;
    let mut listens = 0;
    let mut address = None;
    loop {
        while let Some(complete) = ring.reap().pop() {
            let (host, complete) = ring.complete(complete);
            listens += usize::from(matches!(complete.kind, Op::Listen { .. }));
            assert!(peers.completion(host, &complete).is_none(), "startup succeeds");
            ring.returned(host, complete);
        }
        let stamp = clock.now();
        if starved.is_none() && end.passed() {
            match end.expired() {
                Expired::Failed(figures) => panic!("listener readiness and shutdown settle: {figures}"),
                Expired::Starved(verdict) => starved = Some(verdict),
            }
        }
        peers.drive(&mut ring, stamp.now, stamp.wall, address.is_some() || starved.is_some());
        if starved.is_none() && address.is_none() && peers.ready() {
            let [(name, bound)] = peers.addresses().try_into().expect("one lent listener");
            assert_eq!(name.as_ref(), b"listener");
            let bound: SocketAddr = bound.parse().expect("numeric listener address");
            assert_eq!(
                TcpListener::bind(bound).expect_err("listener remains held").kind(),
                std::io::ErrorKind::AddrInUse
            );
            address = Some(bound);
        }
        if (address.is_some() || starved.is_some()) && peers.empty() && ring.is_empty() {
            break;
        }
        ring.enter(Wait::No);
    }
    drop(peers);
    drop(ring);
    drop(peer);
    if let Some(starved) = starved {
        starved.pass();
    }
    let address = address.expect("listener became ready");
    assert!(TcpListener::bind(address).is_ok(), "shutdown releases the owned listener");
    (address, listens)
}

#[test]
fn a_lent_peer_becomes_ready_after_its_kernel_listen_is_processed() {
    let (address, listens) = ready_and_drained(Peer::new(None));
    assert_ne!(address.port(), 0);
    assert_eq!(listens, 1);
}

#[test]
fn a_lent_peer_becomes_ready_after_claiming_its_adopted_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve listener");
    let reserved = listener.local_addr().expect("reserved address");
    let (address, listens) = ready_and_drained(Peer::new(Some(listener)));
    assert_eq!(address, reserved);
    assert_eq!(listens, 0, "adoption does not submit a kernel Listen");
}

#[test]
fn a_lent_peer_is_named_before_control_and_only_after_the_name_changes() {
    let output = std::cell::RefCell::new(Vec::new());
    let mut last = None;
    for (name, expected) in [
        (b"neighbor".as_slice(), b"lent peer neighbor\n".as_slice()),
        (b"neighbor".as_slice(), b"lent peer neighbor\n".as_slice()),
        (b"other".as_slice(), b"lent peer neighbor\nlent peer other\n".as_slice()),
    ] {
        let result = super::call_with_report(
            &mut last,
            name,
            || {
                assert_eq!(output.borrow().as_slice(), expected, "the name precedes control");
                Err::<(), _>("peer's value")
            },
            |line| {
                use std::io::Write;
                writeln!(output.borrow_mut(), "{line}").expect("write the test's report");
            },
        );
        assert_eq!(result, Err("peer's value"), "the peer's return value is unchanged");
    }
}

#[test]
fn a_lent_peers_startup_error_names_the_peer() {
    let mut peer = Peer::new(None);
    let mut lent = [Lent { name: b"neighbor", peer: &mut peer }];
    let mut peers = Peers::new(&mut lent);
    let complete = Complete {
        op: Token::new(7),
        kind: Op::Socket { family: skein_io::kernel::Family::Ipv4 },
        result: Err(skein_io::kernel::Error::Refused),
    };
    assert_eq!(peers.completion(FIRST, &complete), Some("lent peer neighbor startup: Refused".into()));
}
