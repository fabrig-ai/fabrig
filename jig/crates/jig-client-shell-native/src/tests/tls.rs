//! HTTPS on simulated io, with the TLS kit's certificates and wall time.
use super::{EXCHANGE, TOKEN, file_io_limits, limits};
use crate::{Engine, Settings, Shell, Tls};
use jig_client as client;
use skein_io::{
    self as io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{
    Duration, Env, Queue, Secret, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use skein_shell::Host;
use skein_tls::client::{Certificate, Error};
use skein_tls_kit::{pki, server::Server};
use std::collections::{BTreeMap, VecDeque};
use std::net::SocketAddr;

const ROOM: u32 = 4096;
const DEADLINE: Duration = Duration::from_secs(2);
fn address(port: u16) -> SocketAddr {
    (std::net::Ipv4Addr::LOCALHOST, port).into()
}
fn io_limits() -> io::Limits {
    io::Limits { intake: 32_768, output: 8192, receive: ROOM, sends: 4, ..file_io_limits() }
}
#[derive(Clone, Copy)]
enum Story {
    Stream,
    Refused,
    Heal,
    Reset,
}
struct Peer {
    tls: Server,
    output: VecDeque<Box<[u8]>>,
    answered: bool,
}
struct Process {
    shell: Option<Shell>,
    io: io::Io,
    env: Env<io::Limits>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    peers: BTreeMap<Token, Peer>,
    listener: Option<Token>,
    chain: pki::Chain,
    reset_next: bool,
    started: bool,
    closing: bool,
    received: Vec<Vec<u8>>,
    answers: Vec<client::Event>,
    refused: Vec<Option<Error>>,
    attempts: Vec<io::Addr>,
}
impl Process {
    fn new(root: Option<Fd>, name: &str, chain: pki::Chain, addresses: Box<[SocketAddr]>) -> Self {
        let shell = root.map(|root| {
            let mut shell = Shell::new(Settings {
                engine: Engine::Tls(Tls {
                    name: skein_tls::Name::new(name).expect("DNS name"),
                    host: b"skein.test:8443".as_slice().into(),
                    addresses,
                    config: pki::client(&[b"http/1.1"]),
                }),
                directory: b"private".as_slice().into(),
                limits: limits(),
                io: io_limits(),
            });
            shell.adopt(root, 1000, 7);
            shell
        });
        Self {
            shell,
            io: io::Io::new(&io_limits()),
            env: Env { now: Time::ZERO, wall: pki::VALID, limits: io_limits() },
            complete: Queue::uncapped(),
            submit: Queue::uncapped(),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            peers: BTreeMap::new(),
            listener: None,
            chain,
            reset_next: false,
            started: false,
            closing: false,
            received: vec![],
            answers: vec![],
            refused: vec![],
            attempts: vec![],
        }
    }
    fn exchange(&mut self, number: u64) {
        self.shell.as_mut().expect("client").down(client::Request::Exchange {
            exchange: Token::new(number),
            message: client::Message::Request(b"ask".as_slice().into()),
            credential: Some(Secret::new(Box::new(TOKEN))),
        });
    }
    fn demand(&mut self, socket: Token) {
        let peer = self.peers.get(&socket).expect("peer");
        self.requests.push(io::Request::Stream {
            stream: socket,
            down: Down::Demand {
                read: Read::Arrived { max: ROOM },
                room: peer.output.front().map_or(0, |bytes| u32::try_from(bytes.len()).expect("chunk")),
            },
        });
    }
    fn peer_event(&mut self, event: io::Event) {
        match event {
            io::Event::Listening { listener, .. } => self.listener = Some(listener),
            io::Event::Accepted { socket, .. } => {
                self.requests.push(io::Request::Bind { socket, owner: socket });
                if self.reset_next {
                    self.reset_next = false;
                    self.requests.push(io::Request::Abort { entity: socket });
                } else {
                    self.peers.insert(
                        socket,
                        Peer {
                            tls: Server::new(
                                pki::Server {
                                    chain: self.chain,
                                    alpn: vec![b"http/1.1".to_vec()],
                                    ..pki::Server::plain()
                                }
                                .config(),
                            ),
                            output: VecDeque::new(),
                            answered: false,
                        },
                    );
                    self.demand(socket);
                }
            }
            io::Event::Stream { owner, up: Up::Bytes(bytes) } => {
                let peer = self.peers.get_mut(&owner).expect("peer");
                peer.tls.receive(&bytes);
                let received = &peer.tls.received;
                if !peer.answered
                    && (received.ends_with(b"\r\n\r\nask")
                        || received.starts_with(b"GET ") && received.ends_with(b"\r\n\r\n"))
                {
                    peer.answered = true;
                    self.received.push(received.clone());
                    peer.tls.write(if received.starts_with(b"GET ") {
                        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\ndata: Zmlyc3Q=\n\ndata: c2Vjb25k\n\n"
                    } else {
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok"
                    });
                }
                for chunk in peer.tls.transmit().chunks(usize::try_from(ROOM).expect("chunk")) {
                    peer.output.push_back(chunk.into());
                }
                self.demand(owner);
            }
            io::Event::Stream { owner, up: Up::Room } => {
                let bytes = self.peers.get_mut(&owner).expect("peer").output.pop_front().expect("TLS output");
                self.requests.push(io::Request::Stream { stream: owner, down: Down::Send(bytes) });
                self.demand(owner);
            }
            io::Event::Stream { owner, up: Up::End | Up::Failed(_) } => {
                self.requests.push(io::Request::Abort { entity: owner });
            }
            io::Event::Closed { owner } => {
                self.peers.remove(&owner);
                if self.listener == Some(owner) {
                    self.listener = None;
                }
            }
            other @ (io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Failed { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. }) => panic!("peer event: {other:?}"),
        }
    }
}
impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if let Some(shell) = &mut self.shell {
            shell.iterate(now, wall);
            for submit in shell.submissions().iter() {
                if let io::kernel::Op::Connect { addr, .. } = submit.kind {
                    self.attempts.push(io::Addr::Ip(addr));
                }
            }
            let loaded = shell.loaded().is_some();
            while let Some(event) = shell.events().pop() {
                if matches!(event, client::Event::Reply { .. } | client::Event::Lost { .. }) {
                    self.refused.push(shell.refused());
                }
                self.answers.push(event);
            }
            if !self.started && loaded {
                self.started = true;
                self.exchange(EXCHANGE);
            }
            return;
        }
        self.env.now = now;
        self.env.wall = wall;
        if !self.started {
            self.started = true;
            self.requests.push(io::Request::Listen { owner: Token::new(0), addr: io::Addr::Ip(address(8443)) });
        }
        io::stage::up(&mut self.io, &self.env, &mut self.complete, &mut self.events, &mut self.submit, now);
        while let Some(event) = self.events.pop() {
            self.peer_event(event);
        }
        if self.closing {
            self.requests.clear();
            for entity in self.peers.keys().copied().chain(self.listener) {
                self.requests.push(io::Request::Abort { entity });
            }
        }
        io::stage::down(&mut self.io, &self.env, &mut self.requests, &mut self.submit);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.shell.as_mut().map_or(&mut self.complete, Shell::completions)
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.shell.as_mut().map_or(&mut self.submit, Shell::submissions)
    }
    fn work_pending(&self, now: Time) -> bool {
        self.shell.as_ref().map_or_else(
            || self.io.is_ready() || !self.complete.is_empty() || !self.requests.is_empty(),
            |shell| shell.work_pending(now),
        )
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.shell.as_ref().map_or_else(|| self.io.next_deadline(), Shell::next_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.shell.as_ref().map_or_else(|| self.closing && self.io.is_empty(), Shell::is_empty)
    }
    fn operations(&self) -> u32 {
        self.shell.as_ref().map_or_else(|| io::operations(&io_limits()).expect("io"), Shell::operations)
    }
}
struct Machine(skein_fake_machine::Machine);
impl skein_world::Machine for Machine {
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        skein_fake_machine::step(&mut self.0, call, answers);
    }
}
struct Judge {
    story: Story,
    stage: usize,
    done: bool,
}
impl skein_world::Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let (server, client) = procs.split_at_mut(1);
        let server = &mut server[0];
        let client = &mut client[0];
        if self.done {
            return;
        }
        match self.story {
            Story::Stream | Story::Reset => {
                if self.stage == 0 && !client.refused.is_empty() {
                    client.shell.as_mut().expect("client").down(client::Request::OpenStream {
                        credential: Some(Secret::new(Box::new(TOKEN))),
                        replaces: None,
                        version: jig_client_protocol_wire::VERSION,
                    });
                    self.stage = 1;
                }
                self.done = client
                    .answers
                    .iter()
                    .filter(|event| matches!(event, client::Event::Streamed { message: client::Streamed::Record(_) }))
                    .count()
                    == 2
                    || client
                        .answers
                        .iter()
                        .any(|event| matches!(event, client::Event::Lost { .. } | client::Event::StreamGone));
            }
            Story::Refused => self.done = !client.refused.is_empty(),
            Story::Heal => {
                if client.refused.len() > self.stage {
                    self.stage += 1;
                    match self.stage {
                        1 => server.chain = pki::Chain::Big,
                        2 => {
                            server.chain = pki::Chain::Leaf;
                            server.reset_next = true;
                        }
                        3 => {}
                        4 => self.done = true,
                        _ => panic!("four attempts"),
                    }
                    if !self.done {
                        client.exchange(EXCHANGE + u64::try_from(self.stage).expect("attempt"));
                    }
                }
            }
        }
        if self.done {
            server.closing = true;
            client.shell.as_mut().expect("client").close();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Process]) -> Result<(), String> {
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.done).then_some(Time::ZERO.saturating_add(DEADLINE))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.done && now >= Time::ZERO.saturating_add(DEADLINE)).then(|| "TLS shell did not finish".into())
    }
    fn passed(&self) -> bool {
        self.done
    }
}
fn run(
    name: &str,
    chain: pki::Chain,
    addresses: &[SocketAddr],
    story: Story,
) -> skein_world::Outcome<Process, Machine> {
    let mut machine = skein_fake_machine::Machine::for_user(1000);
    let root = machine.lay(&[]);
    let config = skein_sim::Config { wall: pki::VALID, ..skein_sim::Config::calm() };
    let mut world =
        skein_world::World::new(7, config, Judge { story, stage: 0, done: false }, skein_world::Memory::Unchecked)
            .with_machine(Machine(machine));
    world.spawn(|| {
        let mut server = Process::new(None, name, chain, addresses.into());
        server.reset_next = matches!(story, Story::Reset);
        server
    });
    world
        .spawn_root(skein_sim::Handle::new(root.raw()), |root| Process::new(Some(root), name, chain, addresses.into()));
    world.run()
}
#[test]
fn a_trusted_certificate_carries_the_exchange_and_stream() {
    let outcome = run("skein.test", pki::Chain::Leaf, &[address(8443)], Story::Stream);
    assert_eq!(outcome.procs[1].refused, [None]);
    assert!(outcome.procs[1].answers.iter().any(|event| matches!(event, client::Event::Reply { exchange, message } if exchange.raw() == EXCHANGE && message.as_ref() == b"ok")));
    let records: Vec<_> = outcome.procs[1]
        .answers
        .iter()
        .filter_map(|event| {
            if let client::Event::Streamed { message: client::Streamed::Record(bytes) } = event {
                Some(bytes.as_ref())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(records, [b"first".as_slice(), b"second".as_slice()]);
    assert_eq!(outcome.procs[0].received.len(), 2);
    for request in &outcome.procs[0].received {
        let request = std::str::from_utf8(request).expect("HTTP");
        assert!(request.contains("Host: skein.test:8443\r\n"), "{request}");
        assert!(
            request.contains("Authorization: Bearer BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw\r\n"),
            "{request}"
        );
    }
    assert!(outcome.procs.iter().all(Host::is_empty));
}
#[test]
fn a_certificate_without_a_trusted_issuer_loses_the_exchange() {
    let outcome = run("skein.test", pki::Chain::Untrusted, &[address(8443)], Story::Refused);
    assert_eq!(outcome.procs[1].refused, [Some(Error::Certificate(Certificate::Issuer))]);
    assert!(matches!(outcome.procs[1].answers.as_slice(), [client::Event::Lost { .. }]));
}
#[test]
fn a_certificate_for_another_name_loses_the_exchange() {
    let outcome = run("another.test", pki::Chain::Leaf, &[address(8443)], Story::Refused);
    assert_eq!(outcome.procs[1].refused, [Some(Error::Certificate(Certificate::Name))]);
    assert!(matches!(outcome.procs[1].answers.as_slice(), [client::Event::Lost { .. }]));
}
#[test]
fn the_newest_tls_refusal_survives_a_reset_until_a_handshake_succeeds() {
    let outcome = run("skein.test", pki::Chain::Untrusted, &[address(8443)], Story::Heal);
    assert_eq!(
        outcome.procs[1].refused,
        [
            Some(Error::Certificate(Certificate::Issuer)),
            Some(Error::Certificate(Certificate::Name)),
            Some(Error::Certificate(Certificate::Name)),
            None,
        ]
    );
    assert_eq!(outcome.procs[1].answers.iter().filter(|event| matches!(event, client::Event::Lost { .. })).count(), 3);
    assert!(
        outcome.procs[1]
            .answers
            .iter()
            .any(|event| matches!(event, client::Event::Reply { message, .. } if message.as_ref() == b"ok"))
    );
}
#[test]
fn a_refused_first_address_gives_way_to_the_second() {
    let outcome = run("skein.test", pki::Chain::Leaf, &[address(8442), address(8443)], Story::Stream);
    assert!(
        outcome.procs[1]
            .answers
            .iter()
            .any(|event| matches!(event, client::Event::Reply { message, .. } if message.as_ref() == b"ok"))
    );
    assert_eq!(outcome.procs[0].received.len(), 2);
    assert_eq!(outcome.procs[1].attempts, [8442, 8443, 8442, 8443].map(|port| io::Addr::Ip(address(port))));
}
#[test]
fn a_reset_during_handshake_gives_way_to_the_next_address() {
    let outcome = run("skein.test", pki::Chain::Leaf, &[address(8443), address(8443)], Story::Reset);
    assert_eq!(outcome.procs[1].attempts, [io::Addr::Ip(address(8443)); 3]);
    assert_eq!(outcome.procs[1].refused, [None]);
    assert!(
        outcome.procs[1]
            .answers
            .iter()
            .any(|event| matches!(event, client::Event::Reply { message, .. } if message.as_ref() == b"ok"))
    );
}
#[test]
fn an_exchange_is_lost_when_every_address_refuses() {
    let outcome = run("skein.test", pki::Chain::Leaf, &[address(8441), address(8442)], Story::Refused);
    assert!(matches!(outcome.procs[1].answers.as_slice(), [client::Event::Lost { .. }]));
    assert_eq!(outcome.procs[1].refused, [None]);
    assert_eq!(outcome.procs[1].attempts, [io::Addr::Ip(address(8441)), io::Addr::Ip(address(8442))]);
    assert_eq!(outcome.procs[0].received, Vec::<Vec<u8>>::new());
}
