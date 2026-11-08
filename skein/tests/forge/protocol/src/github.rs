//! HTTP framing over seeded streams (skein/docs/design/fake-forge.md, 10.6).
use crate::{fixtures, github_fixture as fixture, referee::Accounting};
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_fake_forge_github::{
    inline::{self, Lent},
    server::{self, Server},
    service::{Limits as ServiceLimits, Service},
};
use skein_forge_connection::{Component, Event, Limits, Request};
use skein_forge_kit::{
    fixtures as machine,
    joint::REPO,
    pipe::{Cut, Pipe},
};
use skein_lib::{
    Duration, Env, Queue, ReplyTo, Rng, Secret, Time, Token, Wall,
    stream::{Down, Up},
};
use std::collections::BTreeMap;

// Recovery listing timestamps must be above the observed issues-since cutoff.
const WALL_BASE: Wall = Wall::from_nanos(31_536_000_000_000_000);
const TURN: Duration = Duration::from_nanos(100_000);
// More request heads than one pass over a connection can deliver.
const HEADS: u32 = 64;

#[derive(Default)]
struct Trace {
    own: bool,
    served: bool,
}

struct Peer {
    pipe: Pipe,
    server: Server,
    above: Queue<server::Event>,
    below: Queue<Down>,
    traces: Vec<Trace>,
    entrance: usize,
    sent: Vec<u8>,
    remaining: usize,
    delayed: bool,
    lost: bool,
    graphql: Option<Vec<u8>>,
    received: Vec<u8>,
    receiving_body: usize,
    delivered_heads: usize,
}
impl Peer {
    // Observe bytes at Send, before the pipe cuts or fragments them. Heads are
    // queued whole by HTTP; bodies may arrive in separate sends.
    fn sent(&mut self, bytes: &[u8], secrets: &mut Vec<Secret>) {
        if bytes.is_empty() {
            return;
        }
        if self.remaining > 0 {
            assert!(bytes.len() <= self.remaining, "one request body per Send");
            self.remaining -= bytes.len();
            return;
        }
        if self.sent.is_empty() {
            self.traces.push(Trace::default());
        }
        self.sent.extend_from_slice(bytes);
        let Some(end) = self.sent.windows(4).position(|w| w == b"\r\n\r\n") else { return };
        let mut lines = self.sent[..end].split(|b| *b == b'\n').map(<[u8]>::trim_ascii);
        let line = lines.next().expect("request line");
        let target = line.split(|b| *b == b' ').nth(1).expect("target");
        self.traces.last_mut().expect("first byte observed").own =
            target.starts_with(b"/app/") || target == b"/graphql";
        let mut length = 0;
        for line in lines {
            let colon = line.iter().position(|b| *b == b':').expect("header colon");
            let (name, value) = (&line[..colon], line[colon + 1..].trim_ascii());
            if name.eq_ignore_ascii_case(b"Authorization") {
                let bearer = value.strip_prefix(b"Bearer ").expect("GitHub bearer");
                secrets.push(Secret::new(Box::from(bearer)));
            }
            if name.eq_ignore_ascii_case(b"Content-Length") {
                length = std::str::from_utf8(value).expect("length text").parse::<usize>().expect("length");
            }
        }
        self.remaining = (end + 4 + length).checked_sub(self.sent.len()).expect("one request per Send");
        self.sent.clear();
    }
    // A second framing observer counts complete heads in delivered bytes,
    // independently of both the component and the HTTP server's Call events.
    fn received(&mut self, bytes: &[u8]) {
        self.received.extend_from_slice(bytes);
        loop {
            let consume = self.receiving_body.min(self.received.len());
            if let Some(body) = &mut self.graphql {
                body.extend_from_slice(&self.received[..consume]);
            }
            self.received.drain(..consume);
            self.receiving_body -= consume;
            if self.receiving_body > 0 {
                return;
            }
            if let Some(body) = self.graphql.take() {
                let index = self.delivered_heads.checked_sub(1).expect("delivered request");
                self.traces.get_mut(index).expect("trace").own =
                    body.as_slice() == br#"{"query":"query { viewer { login databaseId } }"}"#;
            }
            let Some(end) = self.received.windows(4).position(|w| w == b"\r\n\r\n") else { return };
            if self.received[..end].split(|b| *b == b' ').nth(1) == Some(b"/graphql".as_slice()) {
                self.graphql = Some(Vec::new());
            }
            for line in self.received[..end].split(|b| *b == b'\n').skip(1) {
                let colon = line.iter().position(|b| *b == b':').expect("header colon");
                if line[..colon].eq_ignore_ascii_case(b"Content-Length") {
                    self.receiving_body = std::str::from_utf8(line[colon + 1..].trim_ascii())
                        .expect("body length")
                        .parse()
                        .expect("body length number");
                }
            }
            self.delivered_heads += 1;
            self.received.drain(..end + 4);
        }
    }
}

pub(crate) struct Github {
    pub component: Component,
    pub env: Env<Limits>,
    pub above: Queue<Event>,
    pub below: Queue<skein_io::Request>,
    pub fake_env: Env<fake::Config>,
    pub domain: fake::Domain,
    pub answers: BTreeMap<u64, skein_forge_connection::Answered>,
    pub observations: Vec<fake::Observation>,
    pub hints: Vec<skein_forge_connection::hooks::Hinted>,
    pub events: Vec<String>,
    pub connected: u64,
    pub closed: bool,
    pub secrets: Vec<Secret>,
    pub mints: u64,
    peers: BTreeMap<Token, Peer>,
    service: Service,
    rng: Rng,
    cut_chance: u32,
    lines: Vec<Vec<u8>>,
    totals: Accounting,
    lose_creation: bool,
    seen_mints: u64,
    seed: u64,
    key: &'static [u8],
}
impl Github {
    pub fn new(seed: u64, cut_chance: u32, log_poll: Duration) -> Self {
        let mut config = machine::fake_config();
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        config.skew = fake::Skew::None;
        config.edit_updates = true;
        let mut domain = fake::Domain::new(&config, seed);
        fake::repository(
            &mut domain,
            &config,
            raw::Setup {
                workflows: Box::new([]),
                id: 2,
                name: Box::from(fixture::PATH),
                default: Box::from(b"main".as_slice()),
                tree: Box::from([]),
                labels: Box::from([]),
                checks: skein_forge_kit::joint::checks(1000),
                protection: None,
                hooked: true,
            },
        );
        fake::grant(&mut domain, fixture::PATH, fixture::USER, raw::Permission::Admin);
        fake::grant(&mut domain, fixture::PATH, 2, raw::Permission::Write);
        let limits = fixture::limits(log_poll);
        let mut world = Self {
            component: Component::new(fixture::settings(), &limits, &fixtures::shared()).expect("GitHub component"),
            env: Env { now: Time::ZERO, wall: WALL_BASE, limits },
            fake_env: Env { now: Time::from_nanos(WALL_BASE.as_nanos()), wall: WALL_BASE, limits: config },
            above: Queue::with_capacity(128),
            below: Queue::with_capacity(128),
            domain,
            answers: BTreeMap::new(),
            observations: Vec::new(),
            hints: Vec::new(),
            events: Vec::new(),
            connected: 0,
            closed: false,
            peers: BTreeMap::new(),
            service: fixture::service(seed, fixture::key()),
            rng: Rng::new(seed),
            cut_chance,
            lines: Vec::new(),
            totals: Accounting { reported: 0, written: 0, served: 0, cut_in_head: 0 },
            secrets: Vec::new(),
            mints: 0,
            lose_creation: false,
            seen_mints: 0,
            seed,
            key: fixture::key(),
        };
        world.component.begin(&world.env, &mut world.above, &mut world.below);
        world.action(Request::Installations { forge: 1, accepted: Box::from([1]) });
        world
    }
    pub fn action(&mut self, request: Request) {
        self.component.down(&self.env, request, &mut self.above, &mut self.below);
    }
    pub fn time(&mut self, now: Time) {
        self.env.now = now;
        self.env.wall = Wall::from_nanos(WALL_BASE.as_nanos() + now.as_nanos());
        self.fake_env.now = Time::from_nanos(self.env.wall.as_nanos());
        self.fake_env.wall = self.env.wall;
    }
    pub fn busy(&self) -> bool {
        !self.below.is_empty()
            || self
                .peers
                .values()
                .any(|p| p.pipe.has_work() || p.server.has_work() || !p.above.is_empty() || !p.below.is_empty())
    }
    pub fn run_for(&mut self, span: Duration) {
        let end = self.env.now.saturating_add(span);
        while self.env.now < end {
            self.tick();
            let next = if self.busy() {
                self.env.now.saturating_add(TURN)
            } else {
                self.component.next_deadline().unwrap_or(end).max(self.env.now.saturating_add(TURN))
            };
            self.time(next.min(end));
        }
        self.tick();
    }
    fn server_env(&self) -> Env<ServiceLimits> {
        self.env.with(fixture::service_limits())
    }
    fn release_hooks(&mut self) {
        inline::release(&mut self.service, &mut self.domain, &self.fake_env);
    }
    fn take_minted(&mut self) {
        let store = self.service.installations_mut();
        while self.seen_mints < store.minted() {
            self.seen_mints += 1;
            self.secrets.push(store.minted_token(self.seen_mints).expect("newly minted token").clone());
            self.mints += 1;
        }
    }
    pub fn tick(&mut self) {
        self.release_hooks();
        while let Some(request) = self.below.pop() {
            self.io(request);
        }
        let env = self.server_env();
        let owners: Vec<_> = self.peers.keys().copied().collect();
        for owner in owners {
            let mut peer = self.peers.remove(&owner).expect("live peer");
            while let Some(down) = peer.below.pop() {
                peer.lost |= peer.delayed && matches!(&down, Down::Send(_));
                if !peer.lost {
                    peer.pipe.server_down(down);
                }
            }
            if let Some(up) = peer.pipe.server_up() {
                if let Up::Bytes(bytes) = &up {
                    peer.received(bytes);
                }
                server::up(&mut peer.server, &mut self.service, &env, up, &mut peer.above, &mut peer.below);
            }
            for _ in 0..HEADS {
                inline::pass(
                    &mut peer.server,
                    &mut self.service,
                    &mut Lent { domain: &mut self.domain, env: &self.fake_env, hooks: true },
                    &env,
                    &mut peer.above,
                    &mut peer.below,
                );
                self.take_minted();
                let Some(call) = peer.server.take_head() else { break };
                let trace = peer.traces.get_mut(peer.entrance).expect("head follows write");
                assert!(!trace.served);
                trace.served = true;
                peer.entrance += 1;
                peer.delayed =
                    self.lose_creation && call.method == skein_http::Method::Post && call.target.ends_with(b"/issues");
                self.lose_creation &= !peer.delayed;
            }
            let up = peer.pipe.client_up();
            self.peers.insert(owner, peer);
            if let Some(up) = up {
                self.up(skein_io::Event::Stream { owner, up });
            }
        }
        if self.component.next_deadline().is_some_and(|at| at <= self.env.now) {
            self.component.fire(&self.env, &mut self.above, &mut self.below);
        }
        while let Some(event) = self.above.pop() {
            self.events.push(format!("{event:?}"));
            match event {
                Event::Answered(answer) => {
                    assert!(self.answers.insert(answer.call.raw(), answer).is_none(), "one terminal");
                }
                Event::Connected { .. } => self.connected += 1,
                Event::Closed => self.closed = true,
                Event::Installation { .. } | Event::GitToken { .. } | Event::Fact { .. } => {}
                Event::Stopped { why, .. } => panic!("GitHub stopped: {why:?}, seed {}", self.seed),
                Event::Hint(hint) => self.hints.push(hint),
                Event::Listening { .. } | Event::ListenerLost => panic!("GitHub has no listener"),
            }
        }
        while let Some(observation) = self.domain.pop_observation() {
            self.observations.push(observation);
        }
        self.domain.reclaim();
        self.component.reclaim();
    }
    pub fn direct(&mut self, op: raw::Op) -> Result<raw::Answer, raw::Error> {
        self.release_hooks();
        let event = fake::Event::Call {
            reply_to: ReplyTo::new(Token::new(77)),
            user: 2,
            repository: Box::from(fixture::PATH),
            op,
        };
        inline::call(&mut self.domain, &self.fake_env, event)
    }

    fn up(&mut self, event: skein_io::Event) {
        self.component.up(&self.env, event, &mut self.above, &mut self.below);
    }
    fn io(&mut self, request: skein_io::Request) {
        match request {
            skein_io::Request::Connect { owner, .. } => {
                let cut =
                    if self.rng.chance(self.cut_chance) { Cut::At(self.rng.between(0, 2048)) } else { Cut::Never };
                let env = self.server_env();
                let mut peer = Peer {
                    pipe: Pipe::new(self.rng.next_u64(), cut),
                    server: Server::new(owner, &env.limits).expect("GitHub server limits"),
                    above: Queue::with_capacity(64),
                    below: Queue::with_capacity(64),
                    traces: Vec::new(),
                    entrance: 0,
                    sent: Vec::new(),
                    remaining: 0,
                    delayed: false,
                    lost: false,
                    graphql: None,
                    received: Vec::new(),
                    receiving_body: 0,
                    delivered_heads: 0,
                };
                server::start(&mut peer.server, &mut self.service, &env, &mut peer.above, &mut peer.below);
                assert!(self.peers.insert(owner, peer).is_none());
                self.up(skein_io::Event::Connecting { owner, socket: owner });
                self.up(skein_io::Event::Connected { owner });
            }
            skein_io::Request::Stream { stream, down } => {
                if let Some(peer) = self.peers.get_mut(&stream) {
                    if let Down::Send(bytes) = &down {
                        peer.sent(bytes, &mut self.secrets);
                    }
                    peer.pipe.client_down(down);
                }
            }
            skein_io::Request::Abort { entity } | skein_io::Request::Close { entity } => self.end(entity),
            other @ (skein_io::Request::Listen { .. }
            | skein_io::Request::Bind { .. }
            | skein_io::Request::Reject { .. }
            | skein_io::Request::Output { .. }
            | skein_io::Request::Spawn { .. }
            | skein_io::Request::SpawnAlone { .. }
            | skein_io::Request::Signal { .. }
            | skein_io::Request::Usage { .. }
            | skein_io::Request::Window { .. }) => panic!("unexpected GitHub io {other:?}"),
        }
    }
    fn end(&mut self, owner: Token) {
        if let Some(peer) = self.peers.remove(&owner) {
            let counts = peer.pipe.request_counts();
            assert_eq!(counts.written, skein_lib::len_u64(peer.traces.len()));
            self.lines.extend_from_slice(peer.pipe.request_lines());
            for (index, trace) in peer.traces.into_iter().enumerate() {
                if !trace.own {
                    self.totals.written += 1;
                    self.totals.served += u64::from(trace.served);
                    self.totals.cut_in_head += u64::from(index >= peer.delivered_heads);
                }
            }
            self.up(skein_io::Event::Closed { owner });
        }
    }
    pub fn accounting(&self) -> Accounting {
        let mut a = Accounting {
            reported: 0,
            written: self.totals.written,
            served: self.totals.served,
            cut_in_head: self.totals.cut_in_head,
        };
        for peer in self.peers.values() {
            for trace in &peer.traces {
                if !trace.own {
                    a.written += 1;
                    a.served += u64::from(trace.served);
                }
            }
        }
        a
    }
    pub fn lines(&self) -> &[Vec<u8>] {
        &self.lines
    }
    pub fn lose_creation(&mut self) {
        self.lose_creation = true;
    }
    pub fn replace(&mut self, component: Component) {
        let owners: Vec<_> = self.peers.keys().copied().collect();
        for owner in owners {
            self.end(owner);
        }
        while let Some(event) = self.above.pop() {
            self.events.push(format!("{event:?}"));
        }
        while self.below.pop().is_some() {}
        self.component = component;
        self.component.begin(&self.env, &mut self.above, &mut self.below);
        self.action(Request::Installations { forge: 1, accepted: Box::from([1]) });
        self.action(Request::Connect { forge: 1, access: fixture::access(self.key) });
    }
    pub fn ambiguous_updates(&mut self) {
        self.service.ambiguous_updates();
    }
    pub fn rotate(&mut self, key: &'static [u8]) {
        self.key = key;
        self.service = fixture::service(self.seed + 1, key);
        self.seen_mints = 0;
        self.action(Request::Connect { forge: 1, access: fixture::access(key) });
    }
    pub fn call(&mut self, call: u64, repository: skein_forge::api::Repository, op: skein_forge::api::Op) {
        assert_eq!(repository, REPO);
        self.action(Request::Call { call: Token::new(call), repository, op });
    }
}
