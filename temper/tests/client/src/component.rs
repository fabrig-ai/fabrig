//! Temper's clients component and link joined in memory
//! (temper/docs/design/testing/checks.md, 3; jig/docs/design/protocol/client/sign-in.md).
use jig_client::contract as c;
use jig_client_kit::harness::ClientId;
use jig_client_kit::joint::transport::{KeyedOwner, Reply, Transport, Watched};
use jig_client_protocol_link as link;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients::{self as clients, Event, Request};
use skein_lib::{Env, Queue, Secret, Time, Token, Wall};
use std::collections::{BTreeMap, BTreeSet};
use temper_engine_domain as root;

pub struct Composition {
    pub(crate) latest_link: Option<Secret>,
    pub(crate) local_link: Option<Secret>,
    component: clients::Component,
    env: Env<clients::Limits>,
    out: Queue<Request>,
    links: BTreeMap<ClientId, link::Link>,
    browsers: BTreeSet<ClientId>,
    asked: BTreeMap<ClientId, (Secret, jig_client::ApprovalCode, Wall)>,
    approval_signers: BTreeMap<Token, ClientId>,
    signing_identities: BTreeMap<ClientId, people::Identity>,
    authenticated: BTreeMap<ClientId, (people::Identity, u64)>,
    heard: BTreeMap<ClientId, Vec<jig_client::Event>>,
    pub(crate) linked: bool,
    pub(crate) playground: Option<temper_client_protocol_wire::v1::PlaygroundAnswer>,
    linked_changed: bool,
    link_out: BTreeMap<ClientId, Queue<link::Request>>,
    receiving: BTreeMap<ClientId, link::Limits>,
    exchanges: BTreeMap<Token, (ClientId, Token)>,
    streams: BTreeMap<ClientId, Token>,
    secrets: BTreeMap<clients::SecretOwner, clients::Kept>,
    next: u64,
    pending_reply: Option<Reply>,
    pending_watch: Option<(ClientId, Token, Watched)>,
    minting: BTreeMap<Token, ClientId>,
    pub carried: BTreeMap<u64, Secret>,
    next_carried: u64,
    limits: link::Limits,
    callbacks: Vec<Incoming>,
    events: Vec<root::Event>,
    admissions: Vec<Token>,
}

impl Composition {
    #[must_use]
    pub fn new(configuration: &root::Config, limits: jig_client::DomainLimits) -> Self {
        Self::build(configuration, limits, false, None, None, 0)
    }

    /// One verified application identity provider, as a box's sign-in supplies it.
    #[must_use]
    pub fn with_provider(configuration: &root::Config, limits: jig_client::DomainLimits, provider: u16) -> Self {
        Self::build(configuration, limits, false, None, None, provider)
    }

    #[must_use]
    pub fn new_with_approval(configuration: &root::Config, limits: jig_client::DomainLimits) -> Self {
        Self::build(configuration, limits, true, None, None, 0)
    }

    #[must_use]
    pub fn with_printed_link(
        configuration: &root::Config,
        limits: jig_client::DomainLimits,
        identity: people::Identity,
        secret: Secret,
    ) -> Self {
        Self::build(configuration, limits, false, Some(identity), Some(secret), 0)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the world's constructor names every limit and setting of the joint component"
    )]
    fn build(
        configuration: &root::Config,
        limits: jig_client::DomainLimits,
        approval: bool,
        identity: Option<people::Identity>,
        secret: Option<Secret>,
        provider: u16,
    ) -> Self {
        let (now, wall) = (Time::ZERO, Wall::EPOCH);
        let env = Env {
            now,
            wall,
            limits: clients::Limits {
                public: clients::PublicLimits::OFF,
                streams: limits.watches.max(64),
                watches_per_stream: limits.watches,
                queued_bytes: u32::try_from(limits.document_bytes).expect("world document bound"),
                identity_bytes: limits.text.max(4096),
                families: 2,
                providers: 1,
                mints: 64,
                provider_codes: 64,
                provider_tokens: 64,
                exchanges: limits.requests.requests.max(64),
                exchanges_per_client: limits.requests.requests.max(64),
                message_bytes: u32::try_from(limits.document_bytes).expect("world message bound"),
                wire: jig_client_protocol_wire::v19::CEILINGS,
                link_span: skein_lib::Duration::from_secs(300),
            },
        };
        let settings = clients::Settings {
            private: clients::Private::none(),
            provider: None,
            addresses: Box::new([]),
            link: identity,
            link_secret: secret.clone(),
            local_secret: None,
            approval,
            deployment_provider: 2,
            heartbeat: skein_lib::Duration::from_secs(3600),
            rows: limits.rows,
            served: list([
                clients::Served {
                    family: None,
                    first: jig_client_protocol_wire::VERSION,
                    last: jig_client_protocol_wire::VERSION,
                },
                clients::Served {
                    family: Some(temper_client_protocol_wire::FORGE),
                    first: temper_client_protocol_wire::CLIENT_VERSION,
                    last: temper_client_protocol_wire::CLIENT_VERSION,
                },
            ]),
            providers: list([provider]),
            seed: Secret::new(Box::new([7; 32])),
            stream: 31,
            models: if configuration.core.defaults.models.is_empty() {
                configuration
                    .core
                    .settings
                    .models
                    .iter()
                    .map(|model| clients::Model {
                        new_until: None,
                        number: 1,
                        source: jig_domain_core::settings::ModelSource::Stock,
                        estimated: false,
                        effort: model.default_effort().map(skein_lib::bytes::copy_of),
                        shown: None,
                        picker: true,
                        name: model.name.clone(),
                        provider: None,
                        window: model.window,
                        output: model.output,
                        efforts: model.efforts.clone(),
                        prices: clients::Prices {
                            input: model.input_price,
                            cached: model.cached_price,
                            cache_write: model.cache_write_price,
                            output: model.output_price,
                            unit: model.price_unit,
                        },
                        description: None,
                    })
                    .collect()
            } else {
                clients::translate::llm::catalogue(&configuration.core.defaults.models)
            },
            key_bytes: configuration.core.settings.key_bytes,
            redirect_bytes: configuration.core.settings.redirect_bytes,
            moved_bytes: u64::from(configuration.core.settings.key_bytes).max(70000),
            public: skein_lib::List::with_capacity(0),
            public_bounds: jig_client_protocol_wire::PublicIndexBounds {
                goals: 0,
                ended: 0,
                lines: 0,
                depth: 0,
                projects: 0,
            },
        };
        let mut out = Queue::with_capacity(256);
        let component = clients::Component::new(settings, &env, &mut out).expect("joint component configuration");
        Self {
            latest_link: secret,
            local_link: None,
            component,
            env,
            out,
            links: BTreeMap::new(),
            browsers: BTreeSet::new(),
            asked: BTreeMap::new(),
            approval_signers: BTreeMap::new(),
            signing_identities: BTreeMap::new(),
            authenticated: BTreeMap::new(),
            heard: BTreeMap::new(),
            linked: false,
            playground: None,
            linked_changed: false,
            link_out: BTreeMap::new(),
            receiving: BTreeMap::new(),
            exchanges: BTreeMap::new(),
            streams: BTreeMap::new(),
            secrets: BTreeMap::new(),
            next: 1,
            pending_reply: None,
            pending_watch: None,
            minting: BTreeMap::new(),
            carried: BTreeMap::new(),
            next_carried: 1 << 63,
            limits: link::Limits::new(limits, limits.requests.requests.max(64)).expect("joint link bounds"),
            callbacks: Vec::new(),
            events: Vec::new(),
            admissions: Vec::new(),
        }
    }

    fn token(&mut self) -> Token {
        let token = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("transport names left");
        token
    }

    pub fn take_linked_changed(&mut self) -> bool {
        std::mem::take(&mut self.linked_changed)
    }

    pub fn effects(&mut self) -> Vec<root::Event> {
        std::mem::take(&mut self.events)
    }

    pub fn admitted(&mut self, refused: &[Token]) {
        for watcher in std::mem::take(&mut self.admissions) {
            if !refused.contains(&watcher) {
                self.event(Event::Admitted { watcher });
            }
        }
    }

    pub fn incoming(&mut self, transport: &mut Transport, request: root::Request) {
        let root::Request::Party(request) = request else { return };
        let event = match request {
            root::PartyRequest::Deliver(root::Delivery::WebReply { to, reply }) => {
                self.reply(transport, to, reply);
                return;
            }
            root::PartyRequest::Deliver(root::Delivery::Forges { watcher, forges }) => {
                let mut record =
                    temper_service::family::forges(forges, self.linked, &temper_client_protocol_wire::v1::CEILINGS)
                        .expect("forge document");
                crate::front::Joint::with_playground_forges(&mut record, self.playground.as_ref());
                Some(Event::Application(clients::FromApplication::Delivery {
                    watcher,
                    family: temper_client_protocol_wire::FORGE,
                    record,
                }))
            }
            root::PartyRequest::Deliver(root::Delivery::Ways { watcher, document }) => {
                Some(Event::Application(clients::FromApplication::Delivery {
                    watcher,
                    family: temper_client_protocol_wire::FORGE,
                    record: temper_service::family::ways_panel(*document, &temper_client_protocol_wire::v1::CEILINGS)
                        .expect("ways document"),
                }))
            }
            root::PartyRequest::Deliver(root::Delivery::Approvals { watcher, document }) => {
                Some(Event::Application(clients::FromApplication::Delivery {
                    watcher,
                    family: temper_client_protocol_wire::FORGE,
                    record: temper_service::family::approvals(
                        *document,
                        &crate::fixtures::forge_settings(),
                        &temper_client_protocol_wire::v1::CEILINGS,
                    )
                    .expect("approvals document"),
                }))
            }
            root::PartyRequest::Deliver(root::Delivery::Trials { watcher, rows }) => {
                Some(Event::Application(clients::FromApplication::Delivery {
                    watcher,
                    family: temper_client_protocol_wire::FORGE,
                    record: temper_service::family::trials(rows, &temper_client_protocol_wire::v1::CEILINGS)
                        .expect("trial document"),
                }))
            }
            root::PartyRequest::Deliver(root::Delivery::ForgeDocument { watcher, document }) => {
                Some(Event::Application(clients::FromApplication::Delivery {
                    watcher,
                    family: temper_client_protocol_wire::FORGE,
                    record: temper_service::family::document(
                        *document,
                        &crate::fixtures::forge_settings(),
                        &temper_client_protocol_wire::v1::CEILINGS,
                    )
                    .expect("forge document"),
                }))
            }
            root::PartyRequest::Deliver(root::Delivery::Panel { watcher, panel }) => {
                Some(Event::Application(clients::FromApplication::Delivery {
                    watcher,
                    family: temper_client_protocol_wire::FORGE,
                    record: temper_service::family::panel(
                        *panel,
                        &crate::fixtures::forge_settings(),
                        &temper_client_protocol_wire::v1::CEILINGS,
                    )
                    .expect("forge panel"),
                }))
            }
            root::PartyRequest::Deliver(root::Delivery::Application { to, outcome }) => {
                Some(self.application_answer(transport, to, outcome))
            }
            root::PartyRequest::Deliver(root::Delivery::ForgeRead { reply, answer }) => {
                Some(Self::forge_answer(transport, reply, answer))
            }
            root::PartyRequest::Deliver(root::Delivery::Read { mut reply, answer }) => {
                transport.take_reply(reply_token(&mut reply));
                Some(Event::Read { reply, answer })
            }
            request @ (root::PartyRequest::Catalogue { .. }
            | root::PartyRequest::SignInRefused { .. }
            | root::PartyRequest::WatchRefused { .. }
            | root::PartyRequest::RunText { .. }
            | root::PartyRequest::Deliver(_)) => temper_service::clients::translation::to_component(request),
        };
        if let Some(event) = event {
            self.event(event);
        }
    }

    fn application_answer(
        &self,
        transport: &mut Transport,
        mut to: skein_lib::ReplyTo,
        outcome: Option<root::ApplicationAnswer>,
    ) -> Event {
        transport.take_reply(reply_token(&mut to));
        Event::Application(clients::FromApplication::Answered {
            to,
            family: temper_client_protocol_wire::FORGE,
            record: temper_service::family::answer(
                outcome,
                &temper_client_protocol_wire::v1::CEILINGS,
                &self.env.limits.wire,
            )
            .expect("forge answer"),
        })
    }
    fn forge_answer(transport: &mut Transport, mut reply: skein_lib::ReplyTo, answer: root::ForgeReadAnswer) -> Event {
        transport.take_reply(reply_token(&mut reply));
        Event::Application(clients::FromApplication::Read {
            reply,
            family: temper_client_protocol_wire::FORGE,
            record: temper_service::family::forge_read(answer, &[], &temper_client_protocol_wire::v1::CEILINGS),
        })
    }

    pub fn drop_watches(&mut self) {
        for client in self.streams.keys().copied().collect::<Vec<_>>() {
            self.link(client, link::Event::StreamGone);
        }
    }

    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }

    pub fn receiving(&mut self, client: ClientId, limits: jig_client::DomainLimits) {
        let limits = link::Limits::new(limits, limits.requests.requests.max(64)).expect("client link bounds");
        self.receiving.insert(client, limits);
    }

    pub fn link_client(&mut self, client: ClientId, limits: jig_client::DomainLimits) {
        self.receiving(client, limits);
        self.heard.entry(client).or_default();
    }

    pub fn heard(&mut self, client: ClientId) -> Vec<jig_client::Event> {
        std::mem::take(self.heard.get_mut(&client).expect("linked client"))
    }

    pub fn carry(&mut self, client: ClientId, request: jig_client::Request) {
        match request {
            jig_client::Request::Exchange { exchange, credential, message } => {
                let name = self.token();
                self.exchanges.insert(name, (client, exchange));
                let message = match message {
                    link::Message::ApprovalAsk(bytes) => clients::Message::ApprovalAsk(bytes),
                    link::Message::ApprovalRead(bytes) => clients::Message::ApprovalRead(bytes),
                    link::Message::ApprovalDecide(bytes) => clients::Message::ApprovalDecide(bytes),
                    link::Message::SignInApproval(bytes) => clients::Message::SignInApproval(bytes),
                    link::Message::Authorize(bytes) => clients::Message::Authorize(bytes),
                    link::Message::Request(bytes) => clients::Message::Request(bytes),
                    link::Message::Read(bytes) => clients::Message::Read(bytes),
                    link::Message::Watch(bytes) => clients::Message::Watch(bytes),
                    link::Message::Unwatch(bytes) => clients::Message::Unwatch(bytes),
                    link::Message::StreamGone(bytes) => clients::Message::Gone(bytes),
                    link::Message::SignInLink(bytes) => clients::Message::SignInLink(bytes),
                    link::Message::SignInService => clients::Message::SignInService(Box::new([])),
                    link::Message::SignOut => clients::Message::SignOut(Box::new([])),
                };
                self.event(Event::Transport(clients::FromTransport::Exchange {
                    browser: self.browsers.contains(&client),
                    exchange: name,
                    credential,
                    message,
                }));
            }
            jig_client::Request::OpenStream { credential } => {
                let stream = self.token();
                self.streams.insert(client, stream);
                self.event(Event::Transport(clients::FromTransport::StreamOpen { stream, credential }));
            }
            jig_client::Request::CloseStream => {
                if let Some(stream) = self.streams.remove(&client) {
                    self.event(Event::Transport(clients::FromTransport::StreamGone { stream }));
                }
            }
            jig_client::Request::Save { .. } | jig_client::Request::Erase { .. } => {
                self.heard.get_mut(&client).expect("linked client's store").push(jig_client::Event::Stored);
            }
            jig_client::Request::Told { .. }
            | jig_client::Request::Approval(_)
            | jig_client::Request::Authorized(_)
            | jig_client::Request::ServiceCredential { .. } => {}
        }
    }

    fn link(&mut self, client: ClientId, event: link::Event) {
        if let Some(heard) = self.heard.get_mut(&client) {
            heard.push(match event {
                link::Event::Reply { exchange, message } => jig_client::Event::Reply { exchange, message },
                link::Event::Lost { exchange } => jig_client::Event::Lost { exchange },
                link::Event::Streamed { message } => jig_client::Event::Streamed { message },
                link::Event::StreamGone => jig_client::Event::StreamGone,
                link::Event::AskApproval
                | link::Event::ReadApproval { .. }
                | link::Event::DecideApproval { .. }
                | link::Event::SignIn { .. }
                | link::Event::SignOut
                | link::Event::Authorize { .. }
                | link::Event::Start { .. }
                | link::Event::Save { .. }
                | link::Event::Erase { .. }
                | link::Event::Stored
                | link::Event::NotStored
                | link::Event::Send { .. }
                | link::Event::Read { .. }
                | link::Event::Open { .. }
                | link::Event::Continue { .. }
                | link::Event::Close { .. } => panic!("a linked client's own link speaks for it"),
            });
            return;
        }
        let limits = self.receiving.get(&client).unwrap_or(&self.limits);
        let link = self.links.entry(client).or_insert_with(|| link::Link::new(limits).expect("joint link"));
        let out = self.link_out.entry(client).or_insert_with(|| Queue::with_capacity(256));
        link.step(self.env.now, event, out);
    }

    pub fn event(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
    }

    pub fn as_browser(&mut self, client: ClientId) {
        self.browsers.insert(client);
    }

    pub fn approval(&mut self, client: ClientId, act: jig_client::ApprovalAct) {
        let event = match act {
            jig_client::ApprovalAct::Ask => link::Event::AskApproval,
            jig_client::ApprovalAct::Read { code } => link::Event::ReadApproval { code },
            jig_client::ApprovalAct::Decide { code, approve } => link::Event::DecideApproval { code, approve },
        };
        self.link(client, event);
    }

    pub fn ask_approval(&mut self, client: ClientId) {
        self.approval(client, jig_client::ApprovalAct::Ask);
    }

    #[must_use]
    pub fn asked(&self, client: ClientId) -> Option<(jig_client::ApprovalCode, Wall)> {
        self.asked.get(&client).map(|(_, code, expires)| (*code, *expires))
    }

    pub fn take_approval(&mut self, client: ClientId) {
        let (secret, _, _) = self.asked.get(&client).expect("client asked for approval");
        let secret = secret.clone();
        self.pending_reply = Some(Reply::SignIn { client });
        self.link(client, link::Event::SignIn { with: link::With::Approval { secret } });
    }

    /// Bind the identity answered by the engine before the front releases parked work.
    pub fn bind_authenticated(&mut self, client: ClientId, party: u64, transport: &mut Transport) -> bool {
        let Some((identity, sign_in)) = self.authenticated.remove(&client) else { return false };
        if !transport.is_bound(client) {
            transport.bind(client, party, identity);
        }
        transport.signed_in(client, party, sign_in);
        true
    }

    pub fn sign_in(&mut self, client: ClientId, identity: people::Identity) {
        let exchange = self.token();
        self.minting.insert(exchange, client);
        self.pending_reply = Some(Reply::SignIn { client });
        self.event(Event::Mint { exchange, identity });
    }

    pub fn sign_out(&mut self, client: ClientId, sign_in: u64) {
        self.pending_reply = Some(Reply::SignOut { client, sign_in });
        self.link(client, link::Event::SignOut);
    }

    pub fn send(&mut self, client: ClientId, down: jig_client::Down) {
        let event = match down {
            jig_client::Down::Send { attempt, key, request, value } => {
                let owner = match &request {
                    jig_client::Ask::Jig(_) => KeyedOwner::Jig,
                    jig_client::Ask::Family { record, .. } => KeyedOwner::Application { connector: record.family },
                };
                self.pending_reply = Some(Reply::Keyed { client, attempt, key, owner });
                link::Event::Send { attempt, key, ask: request, value }
            }
            jig_client::Down::Read { read, what } => {
                self.pending_reply = Some(match &what {
                    jig_client::Readable::Jig(what) => Reply::Read { client, read, what: what.clone() },
                    jig_client::Readable::Family { record, scope, .. } => {
                        Reply::ApplicationRead { client, read, subject: family_subject(record, *scope) }
                    }
                });
                link::Event::Read { read, what }
            }
            jig_client::Down::Open { watch, subject } => {
                let watched = match &subject {
                    jig_client::Watchable::Jig(subject) => Watched::Jig(subject.clone()),
                    jig_client::Watchable::Family { scope, record, .. } => Watched::Application {
                        subject: family_subject(record, *scope),
                        scope: match scope {
                            c::Scope::Own => people::ApplicationScope::Own,
                            c::Scope::Deployment => people::ApplicationScope::Deployment,
                            c::Scope::Project(project) => people::ApplicationScope::Project(*project),
                        },
                    },
                };
                self.pending_watch = Some((client, watch, watched));
                link::Event::Open { watch, subject }
            }
            jig_client::Down::Continue { watch, subject } => {
                self.pending_watch = Some((client, watch, Watched::Jig(subject.clone())));
                link::Event::Continue { watch, subject }
            }
            jig_client::Down::Close { watch } => link::Event::Close { watch },
            jig_client::Down::Save { .. } | jig_client::Down::Erase { .. } => panic!("storage stays with the shell"),
        };
        self.link(client, event);
    }

    pub fn detach(&mut self, client: ClientId) {
        if let Some(stream) = self.streams.remove(&client) {
            self.event(Event::Transport(clients::FromTransport::StreamGone { stream }));
        }
        let credential = self.links.remove(&client).and_then(|link| link.credential().cloned());
        if let Some(credential) = credential {
            self.link(client, link::Event::SignIn { with: link::With::Held(credential) });
        }
        self.link_out.remove(&client);
        self.heard.remove(&client);
        self.exchanges.retain(|_, (owner, _)| *owner != client);
    }

    fn core(&mut self, transport: &mut Transport, mut event: core::Event) {
        // Only unlinked clients need the harness's typed reply and watch routes.
        // Linked clients correlate the component's wire replies in their own link.
        if let core::Event::SignIn { reply_to, identity, .. } = &mut event
            && let token = reply_token(reply_to)
            && let Some(client) = self.approval_signers.remove(&token)
        {
            self.signing_identities.insert(client, identity.clone());
            transport.route_reply(token, Reply::SignIn { client });
        } else if let core::Event::People(people::Event::Whose { reply_to, .. }) = &mut event
            && let Some(Reply::SignIn { client }) = self.pending_reply.take()
        {
            let token = reply_token(reply_to);
            self.approval_signers.insert(token, client);
            transport.route_reply(token, Reply::SignIn { client });
        } else if let core::Event::SignIn { reply_to, .. }
        | core::Event::People(
            people::Event::Ask { reply_to, .. } | people::Event::SignOut { reply_to, .. },
        )
        | core::Event::Key { reply_to, .. }
        | core::Event::ProposeSettings { reply_to, .. }
        | core::Event::ChangeModel { reply_to, .. }
        | core::Event::ChangeSettings { reply_to, .. } = &mut event
        {
            if let Some(reply) = self.pending_reply.take() {
                transport.route_reply(reply_token(reply_to), reply);
            }
        } else if let core::Event::Read { reply, .. } = &mut event {
            if let Some(route) = self.pending_reply.take() {
                transport.route_reply(reply_token(reply), route);
            }
        } else if let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = &event {
            if let Some((client, native, subject)) = self.pending_watch.take() {
                transport.route_watch(*watcher, client, native, subject);
            }
        } else if let core::Event::Unwatch { watcher } = &event {
            transport.end_watch(*watcher);
        }
        let watcher = if let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = &event {
            Some(*watcher)
        } else {
            None
        };
        self.events.push(crate::translate::party(event));
        if let Some(watcher) = watcher {
            self.admissions.push(watcher);
        }
    }

    pub fn reply(&mut self, transport: &mut Transport, to: skein_lib::ReplyTo, reply: people::Reply) {
        let token = to.into_token();
        if let Some(Reply::SignIn { client }) = transport.reply(token)
            && let people::Reply::SignedIn { person, sign_in, .. } = &reply
        {
            if let Some(identity) = self.signing_identities.remove(client) {
                self.authenticated.insert(*client, (identity, *sign_in));
            } else {
                transport.signed_in(*client, *person, *sign_in);
            }
        }
        transport.take_reply(token);
        self.event(Event::Reply { to: skein_lib::ReplyTo::new(token), reply });
    }

    pub fn pump(&mut self, transport: &mut Transport) -> Vec<Incoming> {
        self.component.fire(&self.env);
        loop {
            let mut active = false;
            let clients: Vec<_> = self.link_out.keys().copied().collect();
            for client in clients {
                while let Some(request) = self.link_out.get_mut(&client).expect("link output").pop() {
                    active = true;
                    self.link_request(client, request, transport);
                }
            }
            while let Some(request) = self.out.pop() {
                active = true;
                self.request(request, transport);
            }
            active |= self.component.up(&self.env, &mut self.out);
            if !active {
                break;
            }
        }
        let signed_out: Vec<_> = self
            .callbacks
            .iter()
            .filter_map(|incoming| if let Incoming::SignedOut { client, .. } = incoming { Some(*client) } else { None })
            .collect();
        // One client step consumes the link's sign-out batch before rendering.
        // SignedOut retires all watches together on the unlinked client.
        self.callbacks.retain(|incoming| {
            !matches!(incoming,
                Incoming::Callback { client, event: jig_client_test_domain::Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { end: c::WatchEnd::SignedOut, .. } }
                } } if signed_out.contains(client)
            )
        });
        std::mem::take(&mut self.callbacks)
    }

    fn application(&mut self, request: clients::ToApplication, transport: &mut Transport) {
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let request = match request {
            clients::ToApplication::Read { reply, family, version, record, .. }
                if self.playground.is_some()
                    && family == wire::FORGE
                    && version == wire::CLIENT_VERSION
                    && wire::decode_reading(&record, &w::CEILINGS)
                        .is_some_and(|reading| *reading.read() == w::Read::Playground) =>
            {
                self.event(Event::Application(clients::FromApplication::Read {
                    reply,
                    family,
                    record: self.playground.as_ref().map(wire::playground_answer),
                }));
                return;
            }
            request => request,
        };
        match temper_service::family::to_root(
            &jig_protocol_config::Directories::working(),
            request,
            &crate::fixtures::forge_settings(),
            None,
            &temper_service::family::Deployment { welcome: true, owned: true, program_login: None },
            None,
            &temper_client_protocol_wire::v1::CEILINGS,
        ) {
            Ok(mut event) => {
                if let root::Event::Party(
                    root::PartyInput::Application { reply_to, .. }
                    | root::PartyInput::ForgeRead { reply: reply_to, .. },
                ) = &mut event
                {
                    if let Some(reply) = self.pending_reply.take() {
                        transport.route_reply(reply_token(reply_to), reply);
                    }
                } else if let root::Event::Party(root::PartyInput::Watch { watcher, .. }) = &event {
                    if let Some((client, native, subject)) = self.pending_watch.take() {
                        transport.route_watch(*watcher, client, native, subject);
                    }
                    self.admissions.push(*watcher);
                }
                self.events.push(event);
            }
            Err(event) => self.event(*event),
        }
    }

    fn transport_reply(&mut self, exchange: Token, reply: clients::Reply) {
        match reply {
            clients::Reply::StreamsFull { size } => {
                let client = self
                    .streams
                    .iter()
                    .find_map(|(client, stream)| (*stream == exchange).then_some(*client))
                    .expect("opening stream");
                self.link(
                    client,
                    link::Event::Streamed {
                        message: link::Streamed::Full(link::StreamsFull { bound: link::StreamBound::Component, size }),
                    },
                );
            }
            clients::Reply::Record(message) => {
                if let Some((client, local)) = self.exchanges.remove(&exchange) {
                    self.link(client, link::Event::Reply { exchange: local, message });
                }
            }
            clients::Reply::NotARecord | clients::Reply::Conflict => {
                if let Some((client, local)) = self.exchanges.remove(&exchange) {
                    self.link(client, link::Event::Lost { exchange: local });
                }
            }
        }
    }
    fn request(&mut self, request: Request, transport: &mut Transport) {
        match request {
            Request::Core(event) => self.core(transport, *event),
            Request::Application(request) => self.application(request, transport),
            Request::Transport(message) => match message {
                clients::ToTransport::Public { .. } => unreachable!("signed-in client sends no public reads"),
                clients::ToTransport::Reply { exchange, reply } => self.transport_reply(exchange, reply),
                clients::ToTransport::Streamed { stream, message } => {
                    let client = self
                        .streams
                        .iter()
                        .find_map(|(client, name)| (*name == stream).then_some(*client))
                        .expect("live stream");
                    let message = match message {
                        clients::Streamed::Record(record) => link::Streamed::Record(record),
                        clients::Streamed::Heartbeat => link::Streamed::Heartbeat,
                    };
                    self.link(client, link::Event::Streamed { message });
                    self.event(Event::Transport(clients::FromTransport::StreamTaken { stream }));
                }
                clients::ToTransport::StreamEnded { stream, why } => {
                    if let Some(client) =
                        self.streams.iter().find_map(|(client, name)| (*name == stream).then_some(*client))
                    {
                        let event = match why {
                            clients::Ended::SignedOut => link::Event::Streamed { message: link::Streamed::SignedOut },
                            clients::Ended::Stopping | clients::Ended::Busy => link::Event::StreamGone,
                        };
                        self.link(client, event);
                        self.streams.remove(&client);
                    }
                }
            },
            Request::Secrets(request) => {
                let answer = match request {
                    clients::SecretsRequest::Keep { owner, value } => {
                        self.secrets.insert(owner, value);
                        clients::SecretsAnswer::Kept { owner }
                    }
                    clients::SecretsRequest::Get { owner } => {
                        clients::SecretsAnswer::Got { owner, value: self.secrets.get(&owner).cloned() }
                    }
                    clients::SecretsRequest::Erase { owner } => {
                        self.secrets.remove(&owner);
                        if let clients::SecretOwner::Carried(number) = owner {
                            self.carried.remove(&number);
                        }
                        clients::SecretsAnswer::Erased { owner }
                    }
                    clients::SecretsRequest::Carry { exchange, value, .. }
                    | clients::SecretsRequest::Hold { exchange, value } => {
                        let number = self.next_carried;
                        self.next_carried = number.checked_add(1).expect("carried names left");
                        self.carried.insert(number, value);
                        clients::SecretsAnswer::Carried { exchange, number: Some(number) }
                    }
                };
                self.event(Event::Secrets(answer));
            }
            Request::Minted { exchange, credential, expires } => {
                let client = self.minting.remove(&exchange).expect("provider exchange");
                self.link(client, link::Event::SignIn { with: link::With::Held(credential) });
                self.callbacks.push(Incoming::Authenticated { client, person: transport.party(client), expires });
            }
            Request::MintRefused { exchange, why } => {
                let client = self.minting.remove(&exchange).expect("provider exchange");
                self.callbacks.push(match why {
                    clients::MintRefusal::NotReady => Incoming::SignInAgain { client, why: people::Again::NotReady },
                    clients::MintRefusal::Busy => Incoming::SignInAgain { client, why: people::Again::Busy },
                    clients::MintRefusal::NotApplication | clients::MintRefusal::Bound => {
                        Incoming::SignInFailed { client, failure: SignInFailure::Capacity }
                    }
                });
            }
            Request::TakeMoved { exchange, source } => {
                let _ = source;
                let record = None;
                self.event(Event::MovedTaken { exchange, record });
            }
            Request::Link { secret } => self.latest_link = Some(secret),
            Request::LocalLink { secret } => self.local_link = Some(secret),
            Request::LinkSignedIn => {
                if !self.linked {
                    self.linked = true;
                    self.linked_changed = true;
                }
            }
            Request::Closed => {}
            Request::Redeemed { .. } | Request::Claimed { .. } => panic!("world has no OAuth issuer"),
        }
    }

    #[expect(clippy::too_many_lines, reason = "the joint transport translates each link request and signing result")]
    fn link_request(&mut self, client: ClientId, request: link::Request, transport: &mut Transport) {
        let up = match request {
            link::Request::StreamsFull { full } => jig_client::Up::StreamsFull { full },
            link::Request::Exchange { exchange, credential, message } => {
                self.carry(client, jig_client::Request::Exchange { exchange, credential, message });
                return;
            }
            link::Request::OpenStream { credential } => {
                self.carry(client, jig_client::Request::OpenStream { credential });
                return;
            }
            link::Request::CloseStream => {
                self.carry(client, jig_client::Request::CloseStream);
                return;
            }
            link::Request::Signing { signing } => {
                self.approval_signers.retain(|_, owner| *owner != client);
                self.signing_identities.remove(&client);
                if matches!(self.pending_reply, Some(Reply::SignIn { client: owner }) if owner == client) {
                    self.pending_reply = None;
                }
                match signing {
                    link::Signing::SignedIn { party } => {
                        self.callbacks.push(Incoming::Authenticated { client, person: party, expires: self.env.wall });
                    }
                    link::Signing::Again => {
                        self.callbacks.push(Incoming::SignInAgain { client, why: people::Again::NotReady });
                    }
                    link::Signing::Busy => {
                        self.callbacks.push(Incoming::SignInAgain { client, why: people::Again::Busy });
                    }
                    link::Signing::Full { sign_ins, earliest, link_until: _ } => {
                        self.callbacks.push(Incoming::SignInAgain {
                            client,
                            why: people::Again::Full { held: sign_ins, room_at: earliest },
                        });
                    }
                    link::Signing::Approval(taking) => {
                        self.callbacks.push(Incoming::Taking { client, taking });
                    }
                    link::Signing::LinkUsed
                    | link::Signing::LinkExpired
                    | link::Signing::LinkUnknown
                    | link::Signing::Refused
                    | link::Signing::Lost => {
                        self.callbacks.push(Incoming::SignInFailed { client, failure: SignInFailure::Capacity });
                    }
                    link::Signing::SignedOut => {
                        if let Some(sign_in) = transport.sign_in(client) {
                            transport.signed_out(client);
                            self.callbacks.push(Incoming::SignedOut { client, sign_in });
                        }
                    }
                }
                return;
            }
            link::Request::Answer { attempt, envelope, answer, why } => {
                jig_client::Up::Answer { attempt, envelope, answer, why }
            }
            link::Request::Read { read, result } => jig_client::Up::ReadDone { read, result },
            link::Request::Watch { watch, event } => jig_client::Up::Watched { watch, event },
            link::Request::Ended { watch, end } => {
                if let Some(watcher) =
                    transport.watch_token(client, watch).or_else(|| transport.application_watch_token(client, watch))
                {
                    transport.end_watch(watcher);
                }
                jig_client::Up::WatchEnded { watch, end }
            }
            link::Request::FamilyDelivery { watch, record } => jig_client::Up::FamilyDelivered { watch, record },
            link::Request::Hello { family: None, versions } => {
                jig_client::Up::Served { served: versions.is_some_and(link::serves), versions }
            }
            link::Request::Hello { family: Some(family), versions } => {
                jig_client::Up::FamilyServed { family, versions }
            }
            link::Request::Save { .. } | link::Request::Erase { .. } => {
                self.link(client, link::Event::Stored);
                return;
            }
            link::Request::Fact { .. } | link::Request::ServiceCredential { .. } => return,
            link::Request::Approval(answer) => {
                if let link::Approval::Asked { secret, code, expires } = &answer {
                    self.asked.insert(client, (secret.clone(), *code, *expires));
                }
                self.callbacks.push(Incoming::Callback {
                    client,
                    event: jig_client_test_domain::Event::Client {
                        event: jig_client::Event::ApprovalAnswer { answer },
                    },
                });
                return;
            }
            link::Request::Authorized(_)
            | link::Request::Kept { .. }
            | link::Request::Dropped
            | link::Request::NotKept { .. } => panic!("joint link request outside transport"),
        };
        self.callbacks.push(Incoming::Callback {
            client,
            event: jig_client_test_domain::Event::Client { event: jig_client::Event::Up { up } },
        });
    }
}

fn reply_token(reply: &mut skein_lib::ReplyTo) -> Token {
    let token = std::mem::replace(reply, skein_lib::ReplyTo::new(Token::new(0))).into_token();
    *reply = skein_lib::ReplyTo::new(token);
    token
}

fn list<const N: usize, T>(values: [T; N]) -> skein_lib::List<T> {
    let mut list = skein_lib::List::with_capacity(u32::try_from(N).expect("small list"));
    for value in values {
        list.push(value).ok().expect("list room");
    }
    list
}

fn family_subject(record: &jig_client::Family, scope: c::Scope) -> jig_client_test_domain::family::Subject {
    use jig_client_test_domain::family::{Kind, Scope, Subject};
    let task = temper_client_protocol_wire::decode_watching(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
        .and_then(|watching| match watching.into_parts().watch {
            temper_client_protocol_wire::v1::Watch::Change(change) => Some(change.task()),
            temper_client_protocol_wire::v1::Watch::Forges
            | temper_client_protocol_wire::v1::Watch::Drafts(_)
            | temper_client_protocol_wire::v1::Watch::Ways(_)
            | temper_client_protocol_wire::v1::Watch::Trials
            | temper_client_protocol_wire::v1::Watch::Approvals
            | temper_client_protocol_wire::v1::Watch::Dismissed
            | temper_client_protocol_wire::v1::Watch::Landing(_)
            | temper_client_protocol_wire::v1::Watch::Changes(_)
            | temper_client_protocol_wire::v1::Watch::Away
            | temper_client_protocol_wire::v1::Watch::Decided(_)
            | temper_client_protocol_wire::v1::Watch::Status(_)
            | temper_client_protocol_wire::v1::Watch::Landings(_) => None,
        })
        .unwrap_or(0);
    let scope = match scope {
        c::Scope::Own | c::Scope::Deployment => Scope::Deployment,
        c::Scope::Project(project) => Scope::Project(project),
    };
    Subject { scope, kind: Kind::Panel(task) }
}

#[derive(Debug)]
pub enum SignInFailure {
    Capacity,
}

#[derive(Debug)]
pub enum Incoming {
    Taking { client: ClientId, taking: jig_client::Taking },
    Authenticated { client: ClientId, person: u64, expires: Wall },
    SignInAgain { client: ClientId, why: people::Again },
    SignInFailed { client: ClientId, failure: SignInFailure },
    Callback { client: ClientId, event: jig_client_test_domain::Event },
    SignedOut { client: ClientId, sign_in: u64 },
}
