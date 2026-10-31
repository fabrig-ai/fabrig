//! The production component and scripted HTTP/SSE peers under the agent world's clock.
//! Only public requests, terminals and peer-decoded queries cross the shared world seam.
//! Contract: smith/docs/design/protocol/llm.md, section 10.
use crate::wire::{Configuration, Observed};
use skein_fake_llm_domain as fake;
use skein_fake_llm_protocol::{documents, provider};
use skein_io::{Event as IoEvent, Request as IoRequest};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, Intake, Queue, Time, Token, Wall};
use smith_agent_world::backend::{Backend, Progress};
use smith_domain::{Event, GrantName, Request, llm, run};
use smith_protocol_llm::{Component, ComponentLimits, ConfiguredEndpoint, Endpoints, FromDomain, Receiving, ToDomain};
use std::collections::{BTreeMap, VecDeque};

/// One logical call's public receiving contract and independently received HTTP request.
#[derive(Debug)]
pub struct Call {
    pub owner: Token,
    pub receiving: Receiving,
    pub request: Vec<u8>,
    pub started: (Time, Wall),
    /// The component's terminal observations, independent of pooled socket retirement.
    pub observed: Vec<(Time, Wall, Observed)>,
    pub retired: Option<(Time, Wall)>,
}

struct Socket {
    owner: Token,
    server: provider::Server,
    up: Queue<provider::Event>,
    down: Queue<Down>,
    incoming: Intake,
    outgoing: Intake,
    demand: Option<(Read, u32)>,
    peer_demand: Option<(Read, u32)>,
    request: Vec<u8>,
}

/// One production component owns calls and connection reuse; the fixture owns only byte delivery.
pub struct Protocol {
    component: Component,
    env: Env<ComponentLimits>,
    above: Queue<ToDomain>,
    below: Queue<IoRequest>,
    service: provider::Service,
    peer_env: Env<provider::Limits>,
    credentials: Box<[skein_llm::Credential]>,
    provider: fake::Domain,
    provider_env: Env<fake::Config>,
    replies: Queue<fake::Request>,
    sockets: BTreeMap<Token, Socket>,
    next_socket: u64,
    pending: VecDeque<usize>,
    active: BTreeMap<Token, usize>,
    calls: Vec<Call>,
    contract: Option<run::outcome::OutcomeSpec>,
    deliver: Option<run::outcome::ChangeSpec>,
    closing: bool,
    seed: u64,
    referee: skein_world::domain::Referee<crate::referee::Meeting>,
}
impl core::fmt::Debug for Protocol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Protocol").field("calls", &self.calls).finish_non_exhaustive()
    }
}
impl Protocol {
    pub(crate) fn new(
        configuration: Configuration,
        translation: smith_protocol_llm::Limits,
        scripts: Box<[fake::api::Script]>,
        seed: u64,
        domain: &smith_domain::Limits,
    ) -> Self {
        let mut limits =
            smith_protocol_llm::fixture::component(translation, Duration::from_secs(60)).expect("receiving bounds");
        limits.connection.calls = 64;
        limits.connection.connections = 64;
        limits.connection.per_endpoint = 64;
        limits.connection.io.sockets = 64;
        limits.llms = 64;
        limits.contract_bytes = 65536;
        // The root supplies each call's tighter receiving contract at Complete.
        limits.receiving = Receiving {
            max_completion_bytes: domain.session.completion_bytes,
            max_completion_blocks: domain.session.completion_blocks,
            max_failure_bytes: domain.session.failure_bytes,
            decoded_call_bytes: domain.decoded_call_bytes,
        };
        let peer_limits = skein_llm_world::fake::limits(&translation.client);
        let service = provider::Service::new(
            provider::Config {
                echo: skein_llm::openai::Echo::NONE,
                usage_fields: configuration.usage_fields,
                provider: match configuration.endpoint.provider {
                    skein_llm::Provider::OpenAi | skein_llm::Provider::OpenAiCodex => documents::Provider::OpenAi,
                    skein_llm::Provider::Anthropic => documents::Provider::Anthropic,
                },
                path: configuration.endpoint.target.clone(),
                headers: Box::new([]),
                without_credential: false,
            },
            &peer_limits,
        )
        .expect("scripted HTTP peer");
        let endpoints = Endpoints::new(
            Box::new([ConfiguredEndpoint {
                name: llm::Endpoint(0),
                connect: Duration::from_secs(2),
                handshake: Duration::from_secs(2),
                destination: skein_llm_connection::Endpoint {
                    limits: translation.client,
                    credential: skein_llm::client::CredentialLimits { value: 128, account_id: 128 },
                    address: "127.0.0.1:443".parse().expect("fixture address"),
                    transport: skein_llm_connection::Transport::Plaintext,
                    llm: configuration.endpoint,
                },
                identity: smith_protocol_llm::IdentityProfile::Plain,
                models: Box::new([]),
            }]),
            1,
        )
        .expect("fixture endpoint");
        let mut component = Component::new(&limits, endpoints).expect("production component");
        let credential = skein_llm::Credential {
            value: configuration.credential.value.clone(),
            account_id: configuration.credential.account_id.clone(),
            scheme: configuration.credential.scheme,
        };
        component
            .grant(GrantName { account: 0, generation: 1 }, credential, Time::from_nanos(u64::MAX))
            .expect("fixture grant");
        let config = fake::Config { calls: 256, cache_entries: 256, ..skein_llm_world::fake::config() };
        Self {
            component,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::uncapped(),
            below: Queue::uncapped(),
            service,
            peer_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: peer_limits },
            credentials: Box::new([configuration.credential]),
            provider: fake::Domain::try_scripted(&config, 3, scripts).expect("bounded scripts"),
            provider_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config },
            replies: Queue::uncapped(),
            sockets: BTreeMap::new(),
            next_socket: 100,
            pending: VecDeque::new(),
            active: BTreeMap::new(),
            calls: Vec::new(),
            contract: None,
            deliver: None,
            closing: false,
            seed,
            referee: skein_world::domain::Referee::new(crate::referee::Meeting::default()),
        }
    }
    pub(crate) fn bindings(&self) -> &[Call] {
        &self.calls
    }
    fn at(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        self.peer_env.now = now;
        self.peer_env.wall = wall;
        self.provider_env.now = now;
        self.provider_env.wall = wall;
    }
    fn drain_measures(&mut self) {
        while let Some(measure) = self.component.pop_measure() {
            if let smith_protocol_llm::Measure::Prompt { owner, .. } = measure {
                self.component.prompt_recorded(&self.env, owner, &mut self.above, &mut self.below);
            }
        }
    }
}
impl Backend for Protocol {
    fn contract(&mut self, outcome: run::outcome::OutcomeSpec, deliver: Option<run::outcome::ChangeSpec>) {
        self.contract = Some(outcome);
        self.deliver = deliver;
    }
    fn start(&mut self, request: Request, now: Time, wall: Wall) -> Vec<fake::api::Query> {
        self.at(now, wall);
        let Request::Complete {
            owner,
            grant,
            prompt,
            deadlines,
            max_completion_bytes,
            max_completion_blocks,
            max_failure_bytes,
            decoded_call_bytes,
            ..
        } = request
        else {
            panic!("backend accepts Complete")
        };
        let bounds = Receiving { max_completion_bytes, max_completion_blocks, max_failure_bytes, decoded_call_bytes };
        let index = self.calls.len();
        assert!(self.active.insert(owner, index).is_none());
        self.pending.push_back(index);
        self.calls.push(Call {
            owner,
            receiving: bounds,
            request: Vec::new(),
            started: (now, wall),
            observed: Vec::new(),
            retired: None,
        });
        self.component.from_domain(
            &self.env,
            FromDomain::Complete {
                owner,
                grant,
                prompt,
                deadlines,
                bounds,
                outcome: self.contract.clone().expect("host supplied charter contract"),
                deliver: self.deliver.clone(),
                observation: None,
                deltas: smith_protocol_llm::Deltas { text: false, input: false },
            },
            &mut self.above,
            &mut self.below,
        );
        Vec::new()
    }
    fn advance(&mut self, now: Time, wall: Wall, finished: bool) -> Progress {
        self.at(now, wall);
        let mut progress = Progress::default();
        self.drain_measures();
        if finished && !self.closing {
            self.component.close(&self.env, &mut self.above, &mut self.below);
            self.closing = true;
            progress.immediate = true;
        }
        self.component.fire(&self.env, &mut self.above, &mut self.below);
        self.drain_measures();
        let batch = self.below.len();
        for _ in 0..batch {
            let request = self.below.pop().expect("finite IO batch");
            progress.immediate = true;
            match request {
                IoRequest::Connect { owner, .. } => {
                    let socket = Token::new(self.next_socket);
                    self.next_socket += 1;
                    let mut binding = Socket {
                        owner,
                        server: provider::Server::new(socket, &self.peer_env.limits).expect("peer socket"),
                        up: Queue::uncapped(),
                        down: Queue::uncapped(),
                        incoming: Intake::with_capacity(32768),
                        outgoing: Intake::with_capacity(32768),
                        demand: None,
                        peer_demand: None,
                        request: Vec::new(),
                    };
                    provider::start(
                        &mut binding.server,
                        &mut self.service,
                        &self.credentials,
                        &self.peer_env,
                        &mut binding.up,
                        &mut binding.down,
                    );
                    self.sockets.insert(socket, binding);
                    self.component.from_below(
                        &self.env,
                        IoEvent::Connecting { owner, socket },
                        &mut self.above,
                        &mut self.below,
                    );
                    self.drain_measures();
                    self.component.from_below(
                        &self.env,
                        IoEvent::Connected { owner },
                        &mut self.above,
                        &mut self.below,
                    );
                }
                IoRequest::Stream { stream, down } => {
                    let socket = self.sockets.get_mut(&stream).expect("live socket");
                    match down {
                        Down::Demand { read, room } => {
                            socket.demand = if read == Read::Nothing && room == 0 { None } else { Some((read, room)) }
                        }
                        Down::Send(bytes) => {
                            socket.request.extend_from_slice(&bytes);
                            socket.incoming.append(&bytes).expect("bounded request intake");
                        }
                        Down::Finish => {}
                    }
                }
                IoRequest::Close { entity } | IoRequest::Abort { entity } => {
                    let mut socket = self.sockets.remove(&entity).expect("one socket close");
                    provider::close(
                        &mut socket.server,
                        &mut self.service,
                        &self.peer_env,
                        &mut socket.up,
                        &mut socket.down,
                    );
                    let _ = provider::closed(
                        &mut socket.server,
                        &mut self.service,
                        &self.peer_env,
                        &mut socket.up,
                        &mut socket.down,
                    );
                    self.component.from_below(
                        &self.env,
                        IoEvent::Closed { owner: socket.owner },
                        &mut self.above,
                        &mut self.below,
                    );
                }
                other => panic!("unexpected IO request: {other:?}"),
            }
            self.drain_measures();
        }
        while self.provider.is_due(now) {
            fake::fire(&mut self.provider, &self.provider_env, &mut self.replies);
        }
        while let Some(reply) = self.replies.pop() {
            let (socket, reply) = self.service.target(reply).expect("live provider reply route");
            if let Some(binding) = self.sockets.get_mut(&socket) {
                provider::down(
                    &mut binding.server,
                    &mut self.service,
                    &self.credentials,
                    &self.peer_env,
                    reply,
                    &mut binding.up,
                    &mut binding.down,
                );
            }
            progress.immediate = true;
        }
        let mut incoming = Vec::new();
        for socket in self.sockets.values_mut() {
            if socket.server.has_work() {
                provider::resume(
                    &mut socket.server,
                    &mut self.service,
                    &self.credentials,
                    &self.peer_env,
                    &mut socket.up,
                    &mut socket.down,
                );
                progress.immediate = true;
            }
            while let Some(down) = socket.down.pop() {
                match down {
                    Down::Demand { read, room } => {
                        socket.peer_demand = if read == Read::Nothing && room == 0 { None } else { Some((read, room)) }
                    }
                    Down::Send(bytes) => {
                        socket.outgoing.append(&bytes).expect("bounded response intake");
                    }
                    Down::Finish => {}
                }
            }
            if let Some((read, room)) = socket.peer_demand {
                let event = if room > 0 && socket.outgoing.room() >= room {
                    Some(Up::Room)
                } else if room == 0 {
                    socket.incoming.meet(read).map(Up::Bytes)
                } else {
                    None
                };
                if let Some(event) = event {
                    socket.peer_demand = None;
                    provider::up(
                        &mut socket.server,
                        &mut self.service,
                        &self.credentials,
                        &self.peer_env,
                        event,
                        &mut socket.up,
                        &mut socket.down,
                    );
                    progress.immediate = true;
                }
            }
            if let Some((read, room)) = socket.demand {
                let event = if room > 0 && socket.incoming.room() >= room {
                    Some(Up::Room)
                } else if room == 0 {
                    socket.outgoing.meet(read).map(Up::Bytes)
                } else {
                    None
                };
                if let Some(up) = event {
                    socket.demand = None;
                    incoming.push(IoEvent::Stream { owner: socket.owner, up });
                    progress.immediate = true;
                }
            }
            while let Some(event) = socket.up.pop() {
                if let provider::Event::Domain(event) = event {
                    if let fake::Event::Call { query, .. } = &event {
                        let index = self.pending.pop_front().expect("one pending root request per query");
                        let call = &mut self.calls[index];
                        call.request = core::mem::take(&mut socket.request);
                        let request = crate::referee::NativeRequest::from_http(&call.request);
                        self.referee.observe(
                            now,
                            crate::referee::Seen::Request { owner: call.owner, request },
                            &mut Vec::new(),
                        );
                        progress.queries.push(query.clone());
                    }
                    fake::step(&mut self.provider, &self.provider_env, event, &mut self.replies);
                    progress.immediate = true;
                }
            }
        }
        for event in incoming {
            self.drain_measures();
            self.component.from_below(&self.env, event, &mut self.above, &mut self.below);
        }
        self.drain_measures();
        while let Some(event) = self.above.pop() {
            let (owner, event, observed) = match event {
                ToDomain::Completed { owner, completion } => {
                    (owner, Event::Completed { owner, completion }, Observed::Completed(owner))
                }
                ToDomain::Failed { owner, failure, evidence, detail, report } => {
                    (owner, Event::Failed { owner, failure, evidence, detail, report }, Observed::Failed(owner))
                }
                ToDomain::Cancelled { owner } => (owner, Event::Cancelled { owner }, Observed::Cancelled(owner)),
                ToDomain::Text { .. } => continue,
                other => panic!("unexpected component terminal: {other:?}"),
            };
            let index = self.active.remove(&owner).expect("one terminal per root request");
            self.pending.retain(|pending| *pending != index);
            self.calls[index].observed.push((now, wall, observed));
            self.calls[index].retired = Some((now, wall));
            progress.terminals.push((owner, event));
            progress.immediate = true;
        }
        self.provider.reclaim();
        self.service.reclaim();
        self.component.reclaim();
        self.referee.assert_holding(self.seed);
        progress.immediate |= self.has_events();
        progress
    }
    fn cancel(&mut self, owner: Token, now: Time, wall: Wall) -> bool {
        self.at(now, wall);
        self.drain_measures();
        self.component.from_domain(&self.env, FromDomain::Cancel { owner }, &mut self.above, &mut self.below);
        true
    }
    fn has_events(&self) -> bool {
        self.component.has_work() || !self.below.is_empty() || !self.above.is_empty() || !self.replies.is_empty()
    }
    fn is_settled(&self) -> bool {
        self.closing && self.component.closed() && self.sockets.is_empty() && self.active.is_empty()
    }
    fn next_deadline(&self) -> Option<Time> {
        self.component.next_deadline().into_iter().chain(self.provider.next_deadline()).min()
    }
    fn assert_settled(&self) {
        assert!(self.is_settled());
        self.referee.assert_passed(self.seed);
    }
}
