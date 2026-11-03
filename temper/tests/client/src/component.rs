//! Temper's clients component and link joined in memory
//! (jig/docs/design/client/domain/testing.md, 5; temper/docs/design/client/README.md, 8).
use jig_client::contract as c;
use jig_client_protocol_link as link;
use jig_client_world::harness::ClientId;
use jig_client_world::joint::transport::{KeyedOwner, Reply, Transport, Watched};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients::{self as clients, Event, Request};
use skein_lib::{Env, Queue, Secret, Time, Token, Wall};
use std::collections::BTreeMap;
use temper_engine_domain as root;

pub struct Composition {
    component: clients::Component,
    env: Env<clients::Limits>,
    out: Queue<Request>,
    links: BTreeMap<ClientId, link::Link>,
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
            provider: None,
            addresses: Box::new([]),
            link: None,
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
            providers: list([0]),
            seed: Secret::new(Box::new([7; 32])),
            stream: 31,
            models: configuration
                .core
                .settings
                .models
                .iter()
                .map(|model| clients::Model {
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
                .collect(),
            key_bytes: configuration.core.settings.key_bytes,
            redirect_bytes: configuration.core.settings.redirect_bytes,
            moved_bytes: u64::from(configuration.core.settings.key_bytes).max(70000),
        };
        let mut out = Queue::with_capacity(256);
        let component = clients::Component::new(settings, &env, &mut out).expect("joint component configuration");
        Self {
            component,
            env,
            out,
            links: BTreeMap::new(),
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
                Some(Event::Application(clients::FromApplication::Delivery {
                    watcher,
                    family: temper_client_protocol_wire::FORGE,
                    record: temper_service::family::forges(forges, &temper_client_protocol_wire::v1::CEILINGS)
                        .expect("forge document"),
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
            root::PartyRequest::Deliver(root::Delivery::Application { mut to, outcome }) => {
                transport.take_reply(reply_token(&mut to));
                Some(Event::Application(clients::FromApplication::Answered {
                    to,
                    family: temper_client_protocol_wire::FORGE,
                    record: temper_service::family::answer(
                        outcome,
                        &temper_client_protocol_wire::v1::CEILINGS,
                        &self.env.limits.wire,
                    )
                    .expect("forge answer"),
                }))
            }
            root::PartyRequest::Deliver(root::Delivery::Read { mut reply, answer }) => {
                transport.take_reply(reply_token(&mut reply));
                Some(Event::Read { reply, answer })
            }
            request @ (root::PartyRequest::SignInRefused { .. }
            | root::PartyRequest::WatchRefused { .. }
            | root::PartyRequest::RunText { .. }
            | root::PartyRequest::Deliver(_)) => temper_service::clients::translation::to_component(request),
        };
        if let Some(event) = event {
            self.event(event);
        }
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

    fn link(&mut self, client: ClientId, event: link::Event) {
        let limits = self.receiving.get(&client).unwrap_or(&self.limits);
        let link = self.links.entry(client).or_insert_with(|| link::Link::new(limits).expect("joint link"));
        let out = self.link_out.entry(client).or_insert_with(|| Queue::with_capacity(256));
        link.step(self.env.now, event, out);
    }

    pub fn event(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
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
        self.exchanges.retain(|_, (owner, _)| *owner != client);
    }

    fn core(&mut self, transport: &mut Transport, mut event: core::Event) {
        if let core::Event::SignIn { reply_to, .. }
        | core::Event::People(people::Event::Ask { reply_to, .. } | people::Event::SignOut { reply_to, .. })
        | core::Event::Key { reply_to, .. }
        | core::Event::ProposeSettings { reply_to, .. }
        | core::Event::ChangeSettings { reply_to, .. } = &mut event
        {
            transport.route_reply(reply_token(reply_to), self.pending_reply.take().expect("request correlation"));
        } else if let core::Event::Read { reply, .. } = &mut event {
            transport.route_reply(reply_token(reply), self.pending_reply.take().expect("read correlation"));
        } else if let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = &event {
            let (client, native, subject) = self.pending_watch.take().expect("watch correlation");
            transport.route_watch(*watcher, client, native, subject);
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
            transport.signed_in(*client, *person, *sign_in);
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
        match temper_service::family::to_root(
            &jig_protocol_config::Directories::working(),
            request,
            &crate::fixtures::forge_settings(),
            None,
            None,
            &temper_client_protocol_wire::v1::CEILINGS,
        ) {
            Ok(mut event) => {
                if let root::Event::Party(root::PartyInput::Application { reply_to, .. }) = &mut event {
                    transport.route_reply(
                        reply_token(reply_to),
                        self.pending_reply.take().expect("family request correlation"),
                    );
                } else if let root::Event::Party(root::PartyInput::Watch { watcher, .. }) = &event {
                    let (client, native, subject) = self.pending_watch.take().expect("family watch correlation");
                    transport.route_watch(*watcher, client, native, subject);
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
            Request::Link { .. } | Request::Closed => {}
            Request::Redeemed { .. } | Request::Claimed { .. } => panic!("world has no OAuth issuer"),
        }
    }

    fn link_request(&mut self, client: ClientId, request: link::Request, transport: &mut Transport) {
        let up = match request {
            link::Request::StreamsFull { full } => jig_client::Up::StreamsFull { full },
            link::Request::Exchange { exchange, credential, message } => {
                let name = self.token();
                self.exchanges.insert(name, (client, exchange));
                let message = match message {
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
                self.event(Event::Transport(clients::FromTransport::Exchange { exchange: name, credential, message }));
                return;
            }
            link::Request::OpenStream { credential } => {
                let stream = self.token();
                self.streams.insert(client, stream);
                self.event(Event::Transport(clients::FromTransport::StreamOpen { stream, credential }));
                return;
            }
            link::Request::CloseStream => {
                if let Some(stream) = self.streams.remove(&client) {
                    self.event(Event::Transport(clients::FromTransport::StreamGone { stream }));
                }
                return;
            }
            link::Request::Signing { signing } => {
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
                    link::Signing::Refused | link::Signing::Lost => {
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
            | temper_client_protocol_wire::v1::Watch::Landing(_)
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
    Authenticated { client: ClientId, person: u64, expires: Wall },
    SignInAgain { client: ClientId, why: people::Again },
    SignInFailed { client: ClientId, failure: SignInFailure },
    Callback { client: ClientId, event: jig_client_test_domain::Event },
    SignedOut { client: ClientId, sign_in: u64 },
}
