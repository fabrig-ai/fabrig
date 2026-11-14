//! One bounded read over simulated io and the independent list peer.
use std::net::{Ipv4Addr, SocketAddr};

use skein_fake_llm_domain as fake;
use skein_fake_llm_protocol::{documents, provider};
use skein_fake_peers::{Transport, llm};
use skein_io::{self as io, kernel};
use skein_lib::stream::Up;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_llm::{Credential, Provider, Scheme};
use skein_llm_connection::list::{Exchange, ExchangeLimits, ListAnswer};
use skein_llm_connection::{Deadlines, Endpoint};
use skein_world::{Delegate, Host, Memory, Outcome, Referee, World};

const OWNER: Token = Token::new(7);
const ROOM: u32 = 64; // One io pass, as in the sibling simulated world.
const BODY: &[u8] = br#"{"data":[{"id":"listed-model"}]}"#;
const FIXTURE_GRANT: &[u8] = b"list-exchange-fixture";
const WHOLE: Duration = Duration::from_secs(2);
const HORIZON: Time = Time::from_nanos(10_000_000_000);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    Refuse,
    Quiet,
    Truncate,
}

#[derive(Clone, Copy)]
struct Setup {
    route: Provider,
    scheme: Option<Scheme>,
    transport: Transport,
    body: &'static [u8],
    bound: u32,
    status: Option<u16>,
    fault: Fault,
}

impl Setup {
    fn plain() -> Self {
        Self {
            route: Provider::OpenAi,
            scheme: Some(Scheme::Key),
            transport: Transport::Plaintext,
            body: BODY,
            bound: u32::try_from(BODY.len()).expect("fixture fits"),
            status: None,
            fault: Fault::None,
        }
    }
    fn grant(self) -> Option<Credential> {
        self.scheme.map(|scheme| Credential { scheme, value: FIXTURE_GRANT.into() })
    }
    fn endpoint(self, address: SocketAddr) -> Endpoint {
        Endpoint {
            address,
            transport: match self.transport {
                Transport::Plaintext => skein_llm_connection::Transport::Plaintext,
                Transport::Tls => skein_llm_connection::Transport::Tls {
                    server_name: skein_tls_kit::pki::name(),
                    trust: skein_tls_kit::pki::client(&[]),
                },
            },
            llm: match self.route {
                Provider::OpenAi => skein_llm::Endpoint::openai(),
                Provider::Anthropic => skein_llm::Endpoint::anthropic(),
            },
            limits: skein_llm_kit::limits_for(self.route),
            credential: skein_llm::client::CredentialLimits { value: 2048 },
        }
    }
    fn limits(self) -> ExchangeLimits {
        ExchangeLimits {
            list: skein_llm::list::Limits { body: self.bound, ..skein_llm::list::Limits::DEFAULT },
            wall: skein_tls_kit::pki::VALID,
            ..ExchangeLimits::DEFAULT
        }
    }
}

struct Client {
    setup: Setup,
    io: io::Io,
    exchange: Option<Exchange>,
    address: Option<SocketAddr>,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    answer: Option<ListAnswer>,
    settled: bool,
    incoming: Vec<u8>,
}

impl Client {
    fn new(setup: Setup) -> Self {
        Self {
            setup,
            io: io::Io::new(&io::Limits::TEST),
            exchange: None,
            address: None,
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            submissions: Queue::with_capacity(ROOM),
            completions: Queue::with_capacity(ROOM),
            answer: None,
            settled: false,
            incoming: Vec::new(),
        }
    }
}

impl Host for Client {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.exchange.is_none()
            && let Some(address) = self.address
        {
            let mut exchange = Exchange::new(
                OWNER,
                &self.setup.endpoint(address),
                self.setup.route,
                self.setup.grant(),
                Deadlines { whole: Some(WHOLE), ..Deadlines::none() },
                &self.setup.limits(),
            );
            exchange.start(now, &mut self.requests);
            self.exchange = Some(exchange);
        }
        let env = Env { now, wall, limits: io::Limits::TEST };
        io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        for _event in 0..ROOM {
            let Some(event) = self.events.pop() else { break };
            match &event {
                io::Event::Closed { .. } => self.settled = true,
                io::Event::Stream { up: Up::Bytes(bytes), .. } if self.setup.transport == Transport::Plaintext => {
                    self.incoming.extend_from_slice(bytes);
                }
                _ => {}
            }
            self.exchange.as_mut().expect("io follows start").up(event, now, &mut self.requests);
        }
        if let Some(exchange) = &mut self.exchange {
            if exchange.next_deadline().is_some_and(|due| due <= now) {
                exchange.fire(now, &mut self.requests);
            }
            if let Some(answer) = exchange.answer() {
                assert!(self.settled, "an answer follows the socket's Closed");
                assert!(self.answer.replace(answer).is_none(), "only one answer");
                assert!(exchange.answer().is_none(), "the answer is consumed once");
                assert!(exchange.next_deadline().is_none(), "nothing remains armed after settlement");
            }
        }
        io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || !self.completions.is_empty()
            || (self.exchange.is_none() && self.address.is_some())
            || self.exchange.as_ref().is_some_and(|exchange| exchange.next_deadline().is_some_and(|due| due <= now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.exchange.as_ref().and_then(Exchange::next_deadline)
    }
    fn is_empty(&self) -> bool {
        self.answer.is_some()
            && self.io.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&io::Limits::TEST).expect("bounded io")
    }
}

struct HostedPeer {
    peer: llm::Peer,
    quiet: bool,
    held: Queue<kernel::Complete>,
}
impl Host for HostedPeer {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if !self.quiet || self.peer.address().is_none() {
            while let Some(complete) = self.held.pop() {
                self.peer.completions().push(complete);
            }
            self.peer.iterate(now, wall);
        } else {
            while let Some(complete) = self.peer.completions().pop() {
                self.held.push(complete);
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        self.peer.completions()
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        self.peer.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        (!self.quiet || self.peer.address().is_none()) && self.peer.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.peer.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.peer.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.peer.is_empty()
    }
    fn operations(&self) -> u32 {
        self.peer.operations()
    }
}
enum Process {
    Client(Box<Client>),
    Peer(Box<HostedPeer>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Client(client) => client.as_ref(),
            Self::Peer(peer) => peer.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Client(client) => client.as_mut(),
            Self::Peer(peer) => peer.as_mut(),
        }
    }
}

struct Judge {
    fault: Fault,
    done: bool,
    activated: bool,
    teardown: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, processes: &mut [Process]) {
        let address = processes.iter().find_map(|process| match process {
            Process::Peer(peer) => peer.peer.address(),
            Process::Client(_) => None,
        });
        let truncate = processes.iter().any(|process| match process {
            Process::Client(client) => client
                .incoming
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .is_some_and(|end| client.incoming.len() > end + 4),
            Process::Peer(_) => false,
        });
        for process in processes {
            match process {
                Process::Client(client) => client.address = address,
                Process::Peer(peer) => {
                    if self.done || (self.fault == Fault::Truncate && truncate) {
                        self.teardown = self.done;
                        peer.quiet = false;
                        peer.peer.shutdown();
                        self.fault = Fault::None;
                    }
                }
            }
        }
    }
    fn watch(&mut self, _now: Time, processes: &[Process]) -> Result<(), String> {
        self.activated = processes.iter().any(|process| match process {
            Process::Client(client) => client.exchange.is_some(),
            Process::Peer(_) => false,
        });
        self.done = processes.iter().any(|process| match process {
            Process::Client(client) => client.answer.is_some(),
            Process::Peer(_) => false,
        });
        Ok(())
    }
    fn passed(&self) -> bool {
        self.done
    }
    fn next_deadline(&self) -> Option<Time> {
        if !self.activated || (self.done && !self.teardown) {
            Some(Time::ZERO)
        } else {
            (!self.done).then_some(HORIZON)
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= HORIZON && !self.done).then(|| "list did not settle".into())
    }
}

fn run(setup: Setup) -> Outcome<Process> {
    let mut config = skein_sim::Config::calm();
    config.wall = skein_tls_kit::pki::VALID;
    config.buffer = 32; // Cut both the head and body across several simulated receives.
    if setup.fault == Fault::Refuse {
        config.faults.refuse = 1000;
    }
    let mut world = World::new(
        23,
        config,
        Judge { fault: setup.fault, done: false, activated: false, teardown: false },
        Memory::Checked,
    );
    world.spawn(|| {
        let domain_limits = skein_llm_kit::fake::config();
        let domain = fake::Domain::try_scripted(&domain_limits, 23, Box::new([])).expect("empty scripts");
        let protocol_limits = skein_llm_kit::fake::limits(&skein_llm_kit::limits_for(setup.route));
        let peer = llm::Peer::new(
            (Ipv4Addr::LOCALHOST, 0).into(),
            setup.transport,
            skein_fake_peers::Limits {
                io: io::Limits::TEST,
                connections: 1,
                queue: ROOM,
                plaintext: 32768,
                ciphertext: 32768,
                observations: 8,
                observation_bytes: 32768,
            },
            provider::Config {
                list: Some(setup.body.into()),
                list_status: setup.status,
                without_credential: setup.scheme.is_none(),
                usage_fields: documents::UsageFields::ALL,
                echo: skein_llm::openai::Echo::NONE,
                provider: match setup.route {
                    Provider::OpenAi => documents::Provider::OpenAi,
                    Provider::Anthropic => documents::Provider::Anthropic,
                },
                path: setup.endpoint((Ipv4Addr::LOCALHOST, 0).into()).llm.target,
                headers: Box::new([]),
            },
            setup.grant().unwrap_or_else(|| Credential::key(FIXTURE_GRANT.into())),
            protocol_limits,
            domain,
            domain_limits,
        )
        .expect("independent list peer");
        Process::Peer(Box::new(HostedPeer { peer, quiet: setup.fault == Fault::Quiet, held: Queue::uncapped() }))
    });
    world.spawn(|| Process::Client(Box::new(Client::new(setup))));
    world.run()
}

fn client(outcome: &Outcome<Process>) -> &Client {
    outcome
        .procs
        .iter()
        .find_map(|process| match process {
            Process::Client(client) => Some(client.as_ref()),
            Process::Peer(_) => None,
        })
        .expect("client process")
}

fn listed() -> ListAnswer {
    ListAnswer::Listed(Box::new([skein_llm::list::Listed {
        name: b"listed-model".as_slice().into(),
        shown: None,
        window: None,
        output: None,
        efforts: Box::new([]),
    }]))
}

#[test]
fn lists_use_each_routes_credential_and_settle_once() {
    for (route, scheme) in [
        (Provider::OpenAi, Some(Scheme::Bearer)),
        (Provider::OpenAi, Some(Scheme::Key)),
        (Provider::OpenAi, None),
        (Provider::Anthropic, Some(Scheme::Key)),
        (Provider::Anthropic, None),
    ] {
        for transport in [Transport::Plaintext, Transport::Tls] {
            let setup = Setup { route, scheme, transport, ..Setup::plain() };
            let outcome = run(setup);
            assert_eq!(client(&outcome).answer, Some(listed()));
            let peer = outcome
                .procs
                .iter()
                .find_map(|process| match process {
                    Process::Peer(peer) => Some(peer),
                    Process::Client(_) => None,
                })
                .expect("peer process");
            let lists: Vec<_> = peer
                .peer
                .observations()
                .iter()
                .filter_map(|event| match event {
                    llm::Observation::List { route, scheme, index, .. } => Some((*route, *scheme, *index)),
                    _ => None,
                })
                .collect();
            assert_eq!(lists, vec![(route, scheme, scheme.map(|_| 0))]);
            let heads: Vec<_> = peer
                .peer
                .observations()
                .iter()
                .filter_map(|event| match event {
                    llm::Observation::Head { headers, .. } => Some(headers),
                    _ => None,
                })
                .collect();
            assert_eq!(heads.len(), 1);
            let headers = heads[0];
            assert!(headers.iter().any(|field| field.is(b"connection") && field.value.eq_ignore_ascii_case(b"close")));
            if route == Provider::Anthropic {
                assert!(
                    headers.iter().any(|field| field.is(b"anthropic-version") && field.value.as_ref() == b"2023-06-01")
                );
            }
            if transport == Transport::Plaintext {
                let replay = run(setup);
                assert_eq!(outcome.trace, replay.trace, "plaintext list replays");
            }
        }
    }
}

#[test]
fn refusals_keep_only_the_status() {
    for status in [401, 404, 500, 302] {
        let outcome = run(Setup { status: Some(status), ..Setup::plain() });
        assert_eq!(client(&outcome).answer, Some(ListAnswer::Refused { status }));
        let mut packet = format!("HTTP/1.1 {status} Refused\r\nContent-Length: 4096\r\n\r\n").into_bytes();
        let head_len = packet.len();
        packet.extend_from_slice(LONG_BODY);
        let mut bytes = Bytes::new(Setup::plain(), Some(packet), Deadlines::none());
        bytes.until_close();
        assert_eq!(bytes.received, head_len, "a refusal body is never read");
        assert_eq!(bytes.settle(), ListAnswer::Refused { status });
    }
}

#[test]
fn unreadable_lists_stop_at_the_body_bound() {
    for (body, bound) in [(b"not json".as_slice(), 8), (BODY, u32::try_from(BODY.len() - 1).expect("fixture fits"))] {
        let outcome = run(Setup { body, bound, ..Setup::plain() });
        let client = client(&outcome);
        assert_eq!(client.answer, Some(ListAnswer::Unreadable));
        if body == BODY {
            assert!(client.incoming.ends_with(b"\r\n\r\n"), "a known oversized length reads no body");
        }
    }
}

// Long enough that the referee's shutdown lands while the response body is in flight.
const LONG_BODY: &[u8] = &[b'x'; 4096];
#[test]
fn failed_connects_and_lost_bodies_are_unreachable() {
    for fault in [Fault::Refuse, Fault::Truncate] {
        let outcome = run(Setup { fault, body: LONG_BODY, bound: 4096, ..Setup::plain() });
        assert_eq!(client(&outcome).answer, Some(ListAnswer::Unreachable));
    }
}

#[test]
fn a_peer_that_never_answers_meets_the_virtual_whole_deadline() {
    let outcome = run(Setup { fault: Fault::Quiet, ..Setup::plain() });
    assert_eq!(client(&outcome).answer, Some(ListAnswer::Deadline));
    assert!(outcome.end >= Time::ZERO.saturating_add(WHOLE));
}

// Byte-level companions observe exact heads, read demand bounds and delayed settlement.
const SOCKET: Token = Token::new(100);
const TURNS: u32 = 40_000; // The sibling protocol world's bounded driver.
struct Bytes {
    exchange: Exchange,
    wire: skein_world::stream::Wire,
    requests: Queue<io::Request>,
    tls: Option<skein_tls_kit::server::Server>,
    ciphertext: usize,
    response: Option<Vec<u8>>,
    received: usize,
    closing: bool,
    aborted: bool,
}
impl Bytes {
    fn new(setup: Setup, response: Option<Vec<u8>>, deadlines: Deadlines) -> Self {
        let endpoint = setup.endpoint((Ipv4Addr::LOCALHOST, 443).into());
        let mut exchange = Exchange::new(OWNER, &endpoint, setup.route, setup.grant(), deadlines, &setup.limits());
        let mut requests = Queue::uncapped();
        exchange.start(Time::ZERO, &mut requests);
        Self {
            exchange,
            wire: skein_world::stream::Wire::new(23),
            requests,
            tls: (setup.transport == Transport::Tls)
                .then(|| skein_tls_kit::server::Server::new(skein_tls_kit::pki::Server::plain().config())),
            ciphertext: 0,
            response,
            received: 0,
            closing: false,
            aborted: false,
        }
    }
    fn head(&self) -> &[u8] {
        self.tls.as_ref().map_or(self.wire.received.as_slice(), |server| server.received.as_slice())
    }
    fn tick(&mut self) {
        if let Some(request) = self.requests.pop() {
            match request {
                io::Request::Connect { owner, .. } => {
                    self.exchange.up(io::Event::Connecting { owner, socket: SOCKET }, Time::ZERO, &mut self.requests);
                    self.exchange.up(io::Event::Connected { owner }, Time::ZERO, &mut self.requests);
                }
                io::Request::Stream { down, .. } => self.wire.take(down),
                io::Request::Close { .. } => self.closing = true,
                io::Request::Abort { .. } => {
                    self.closing = true;
                    self.aborted = true;
                }
                _ => panic!("a list owns only one socket"),
            }
        }
        if self.closing {
            return;
        }
        if let Some(server) = &mut self.tls {
            server.receive(&self.wire.received[self.ciphertext..]);
            self.ciphertext = self.wire.received.len();
            self.wire.write(&server.transmit());
        }
        if self.head().ends_with(b"\r\n\r\n")
            && let Some(response) = self.response.take()
        {
            if let Some(server) = &mut self.tls {
                server.write(&response);
                self.wire.write(&server.transmit());
            } else {
                self.wire.write(&response);
            }
        }
        if let Some(up) = self.wire.answer() {
            if let Up::Bytes(bytes) = &up {
                self.received += bytes.len();
            }
            self.exchange.up(io::Event::Stream { owner: OWNER, up }, Time::ZERO, &mut self.requests);
        }
        self.exchange.fire(Time::ZERO, &mut self.requests);
    }
    fn until_close(&mut self) {
        for _turn in 0..TURNS {
            self.tick();
            if self.closing {
                return;
            }
        }
        panic!("the exchange closes its socket");
    }
    fn settle(&mut self) -> ListAnswer {
        assert!(self.exchange.answer().is_none(), "no answer before Closed");
        self.exchange.up(io::Event::Closed { owner: OWNER }, Time::ZERO, &mut self.requests);
        let answer = self.exchange.answer().expect("one settled answer");
        assert!(self.exchange.answer().is_none());
        assert!(self.exchange.next_deadline().is_none());
        answer
    }
}

fn response(body: &[u8], fields: &str) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 200 OK\r\n{fields}\r\n").into_bytes();
    response.extend_from_slice(body);
    response
}

#[test]
fn list_heads_use_the_target_and_the_calls_credential_fields() {
    for (route, scheme, target, credential) in [
        (Provider::OpenAi, Some(Scheme::Bearer), "/v1/models", "authorization: bearer list-exchange-fixture\r\n"),
        (Provider::OpenAi, Some(Scheme::Key), "/v1/models", "authorization: bearer list-exchange-fixture\r\n"),
        (Provider::OpenAi, None, "/v1/models", ""),
        (Provider::Anthropic, Some(Scheme::Key), "/v1/models?limit=1000", "x-api-key: list-exchange-fixture\r\n"),
        (Provider::Anthropic, None, "/v1/models?limit=1000", ""),
    ] {
        for transport in [Transport::Plaintext, Transport::Tls] {
            let setup = Setup { route, scheme, transport, ..Setup::plain() };
            let mut bytes = Bytes::new(
                setup,
                Some(response(BODY, &format!("Content-Length: {}\r\n", BODY.len()))),
                Deadlines::none(),
            );
            bytes.until_close();
            let head = String::from_utf8_lossy(bytes.head()).to_ascii_lowercase();
            assert!(head.starts_with(&format!("get {target} http/1.1\r\n")), "route target");
            assert!(head.contains("\r\nconnection: close\r\n"));
            assert!(head.contains(credential), "the route's credential field");
            assert_eq!(head.contains("authorization:"), scheme.is_some() && route == Provider::OpenAi);
            assert_eq!(head.contains("x-api-key:"), scheme.is_some() && route == Provider::Anthropic);
            assert_eq!(head.contains("anthropic-version: 2023-06-01\r\n"), route == Provider::Anthropic);
            if let Some(server) = &bytes.tls {
                assert!(server.closed, "TLS close_notify precedes socket Close");
            }
            assert_eq!(bytes.settle(), listed());
        }
    }
}

#[test]
fn unknown_lengths_read_at_most_one_byte_past_the_bound() {
    for (framing, suffix) in [("Transfer-Encoding: chunked\r\n", b"\r\n0\r\n\r\n".as_slice()), ("", b"".as_slice())] {
        for (body, expected, read_body) in [
            (BODY.to_vec(), listed(), BODY.len()),
            ([BODY, b"x"].concat(), ListAnswer::Unreadable, BODY.len() + 1),
            ([BODY, LONG_BODY].concat(), ListAnswer::Unreadable, BODY.len() + 1),
        ] {
            let prefix = if framing.is_empty() { Vec::new() } else { format!("{:x}\r\n", body.len()).into_bytes() };
            let encoded = [prefix.as_slice(), &body, suffix].concat();
            let packet = response(&encoded, framing);
            let head_len = packet.windows(4).position(|part| part == b"\r\n\r\n").expect("head") + 4;
            let mut bytes = Bytes::new(Setup::plain(), Some(packet), Deadlines::none());
            bytes.wire.eof = framing.is_empty();
            bytes.until_close();
            assert_eq!(
                bytes.received,
                head_len + prefix.len() + read_body + if expected == listed() { suffix.len() } else { 0 },
                "reads only the bounded body and its framing"
            );
            assert_eq!(bytes.settle(), expected);
        }
    }
}

#[test]
fn gzip_is_unreadable_without_reading_its_body() {
    let packet = response(LONG_BODY, "Content-Encoding: gzip\r\nContent-Length: 4096\r\n");
    let head_len = packet.len() - LONG_BODY.len();
    let mut bytes = Bytes::new(Setup { bound: 4096, ..Setup::plain() }, Some(packet), Deadlines::none());
    bytes.until_close();
    assert_eq!(bytes.received, head_len);
    assert_eq!(bytes.settle(), ListAnswer::Unreadable);
}

#[derive(Clone, Copy)]
enum Point {
    New,
    Connecting,
    Handshaking,
    Reading,
    Closing,
}

fn at_point(point: Point, deadlines: Deadlines) -> Bytes {
    let setup = Setup {
        transport: match point {
            Point::Handshaking => Transport::Tls,
            Point::New | Point::Connecting | Point::Reading | Point::Closing => Transport::Plaintext,
        },
        ..Setup::plain()
    };
    let mut bytes = Bytes::new(setup, None, deadlines);
    match point {
        Point::New => {
            let endpoint = setup.endpoint((Ipv4Addr::LOCALHOST, 443).into());
            bytes.exchange = Exchange::new(OWNER, &endpoint, setup.route, setup.grant(), deadlines, &setup.limits());
            bytes.requests.clear();
        }
        Point::Connecting => {}
        Point::Handshaking | Point::Reading => bytes.tick(),
        Point::Closing => {
            bytes.response = Some(response(BODY, &format!("Content-Length: {}\r\n", BODY.len())));
            bytes.until_close();
        }
    }
    bytes
}

#[test]
fn cancellation_answers_deadline_after_settlement_in_each_phase() {
    for point in [Point::Connecting, Point::Handshaking, Point::Reading, Point::Closing] {
        let mut bytes = at_point(point, Deadlines::none());
        bytes.exchange.cancel(&mut bytes.requests);
        bytes.until_close();
        assert_eq!(bytes.settle(), ListAnswer::Deadline);
    }
    let mut bytes = at_point(Point::New, Deadlines::none());
    bytes.exchange.cancel(&mut bytes.requests);
    assert_eq!(bytes.exchange.answer(), Some(ListAnswer::Deadline));
    assert!(bytes.exchange.answer().is_none());
    assert!(bytes.requests.is_empty());
}

#[test]
fn connect_handshake_and_whole_bound_their_phases() {
    const PHASE: Duration = Duration::from_secs(1);
    for (point, deadlines, due) in [
        (Point::Connecting, Deadlines { connect: Some(PHASE), whole: Some(WHOLE), ..Deadlines::none() }, PHASE),
        (Point::Handshaking, Deadlines { handshake: Some(PHASE), whole: Some(WHOLE), ..Deadlines::none() }, PHASE),
        (
            Point::Reading,
            Deadlines { head: Some(PHASE), idle: Some(PHASE), whole: Some(WHOLE), ..Deadlines::none() },
            WHOLE,
        ),
    ] {
        let mut bytes = at_point(point, deadlines);
        let until = Time::ZERO.saturating_add(due);
        // Drain internal TLS/HTTP work without giving the peer any answer.
        for _turn in 0..TURNS {
            if bytes.exchange.next_deadline() != Some(Time::ZERO) {
                break;
            }
            bytes.exchange.fire(Time::ZERO, &mut bytes.requests);
        }
        assert_eq!(bytes.exchange.next_deadline(), Some(until));
        bytes.exchange.fire(until, &mut bytes.requests);
        bytes.until_close();
        assert!(bytes.aborted, "a deadline aborts its socket");
        assert_eq!(bytes.settle(), ListAnswer::Deadline);
    }
}

#[test]
fn debug_omits_credentials_before_and_during_the_request() {
    let mut bytes = Bytes::new(Setup::plain(), None, Deadlines::none());
    for _turn in 0..TURNS {
        let debug = format!("{:?}", bytes.exchange);
        assert!(!debug.contains("list-exchange-fixture"));
        assert!(!debug.contains("108, 105, 115, 116"), "credential bytes are omitted too");
        assert!(!debug.contains("Authorization"));
        bytes.tick();
        if bytes.head().ends_with(b"\r\n\r\n") {
            break;
        }
    }
    bytes.exchange.cancel(&mut bytes.requests);
    bytes.until_close();
    assert_eq!(bytes.settle(), ListAnswer::Deadline);
}
