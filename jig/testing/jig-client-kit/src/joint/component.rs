//! The clients' component and link joined in memory (client/domain/testing.md, 5;
//! protocol/client/README.md, 2). Only transport names and external secrets live here.
use super::{
    engine::Driver,
    session::{Incoming, SignInFailure},
    transport::{KeyedOwner, Reply, Watched},
};
use crate::harness::ClientId;
use jig_client_domain::contract as c;
use jig_client_protocol_link as link;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients::{self as clients, Event, Request};
use jig_test_domain as root;
use skein_lib::{Env, Queue, Secret, Time, Token, Wall};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) struct Composition {
    component: clients::Component,
    env: Env<clients::Limits>,
    out: Queue<Request>,
    links: BTreeMap<ClientId, link::Link>,
    link_out: BTreeMap<ClientId, Queue<link::Request>>,
    exchanges: BTreeMap<Token, (ClientId, Token)>,
    streams: BTreeMap<ClientId, Token>,
    secrets: BTreeMap<clients::SecretOwner, clients::Kept>,
    next: u64,
    pending_reply: Option<Reply>,
    pending_watch: Option<(ClientId, Token, Watched)>,
    minting: BTreeMap<Token, ClientId>,
    pub(super) carried: BTreeMap<u64, Secret>,
    next_carried: u64,
    limits: link::Limits,
    callbacks: Vec<Incoming>,
    browsers: BTreeSet<ClientId>,
    approval_answers: BTreeMap<ClientId, VecDeque<link::Approval>>,
    approval_takes: BTreeSet<ClientId>,
    approval_signers: BTreeMap<Token, ClientId>,
    approval_signing: BTreeMap<ClientId, VecDeque<link::Signing>>,
}

impl Composition {
    #[expect(
        clippy::too_many_lines,
        reason = "the joint fixture constructs its transport and protocol models together"
    )]
    pub(super) fn new(driver: &Driver, limits: jig_client_domain::Limits) -> Self {
        let (now, wall) = driver.world.clocks();
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
            link_secret: None,
            local_secret: None,
            approval: true,
            addresses: Box::new([]),
            link: None,
            deployment_provider: 2,
            heartbeat: skein_lib::Duration::from_secs(3600),
            rows: limits.rows,
            served: crate::test::list([
                clients::Served {
                    family: None,
                    first: jig_client_protocol_wire::VERSION,
                    last: jig_client_protocol_wire::VERSION,
                },
                clients::Served {
                    family: Some(1),
                    first: jig_client_test_protocol_wire::VERSION,
                    last: jig_client_test_protocol_wire::VERSION,
                },
            ]),
            providers: crate::test::list([0]),
            seed: Secret::new(Box::new([7; 32])),
            stream: 31,
            models: driver
                .transport
                .models
                .iter()
                .map(|model| clients::Model {
                    new_until: None,
                    number: 0,
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
                .collect(),
            key_bytes: driver.key_bytes,
            redirect_bytes: driver.redirect_bytes,
            moved_bytes: u64::from(driver.key_bytes).max(70000),
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
            component,
            env,
            out,
            links: BTreeMap::new(),
            link_out: BTreeMap::new(),
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
            browsers: BTreeSet::new(),
            approval_answers: BTreeMap::new(),
            approval_takes: BTreeSet::new(),
            approval_signers: BTreeMap::new(),
            approval_signing: BTreeMap::new(),
        }
    }

    fn token(&mut self) -> Token {
        let token = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("transport names left");
        token
    }

    pub(super) fn as_browser(&mut self, client: ClientId) {
        self.browsers.insert(client);
    }

    pub(super) fn approval(&mut self, client: ClientId, act: jig_client::ApprovalAct) {
        let event = match act {
            jig_client::ApprovalAct::Ask => link::Event::AskApproval,
            jig_client::ApprovalAct::Read { code } => link::Event::ReadApproval { code },
            jig_client::ApprovalAct::Decide { code, approve } => link::Event::DecideApproval { code, approve },
        };
        self.link(client, event);
    }

    pub(super) fn take_approval(&mut self, client: ClientId, secret: Secret) {
        assert!(self.approval_takes.insert(client), "one approval take per client in flight");
        self.pending_reply = Some(Reply::SignIn { client });
        self.link(client, link::Event::SignIn { with: link::With::Approval { secret } });
    }

    pub(super) fn take_approval_answer(&mut self, client: ClientId) -> Option<link::Approval> {
        self.approval_answers.get_mut(&client)?.pop_front()
    }

    pub(super) fn take_approval_signing(&mut self, client: ClientId) -> Option<link::Signing> {
        self.approval_signing.get_mut(&client)?.pop_front()
    }

    pub(super) fn credential(&self, client: ClientId) -> Option<&Secret> {
        self.links.get(&client)?.credential()
    }

    pub(super) fn next_deadline(&self) -> Option<Time> {
        self.component.next_deadline()
    }

    #[cfg(test)]
    pub(super) fn receiving(&mut self, limits: jig_client_domain::Limits) {
        assert!(self.links.is_empty(), "receiving bounds are supplied before binding");
        self.limits = link::Limits::new(limits, limits.requests.requests.max(64)).expect("joint link bounds");
    }

    pub(super) fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }

    fn link(&mut self, client: ClientId, event: link::Event) {
        let link = self.links.entry(client).or_insert_with(|| link::Link::new(&self.limits).expect("joint link"));
        let out = self.link_out.entry(client).or_insert_with(|| Queue::with_capacity(256));
        link.step(self.env.now, event, out);
    }

    pub(super) fn event(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
    }

    pub(super) fn sign_in(&mut self, client: ClientId, identity: people::Identity) {
        let exchange = self.token();
        self.minting.insert(exchange, client);
        self.pending_reply = Some(Reply::SignIn { client });
        self.event(Event::Mint { exchange, identity });
    }

    pub(super) fn sign_out(&mut self, client: ClientId, sign_in: u64) {
        self.pending_reply = Some(Reply::SignOut { client, sign_in });
        self.link(client, link::Event::SignOut);
    }

    pub(super) fn send(&mut self, client: ClientId, down: jig_client::Down) {
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
                    jig_client::Readable::Family { record, .. } => {
                        Reply::ApplicationRead { client, read, subject: crate::test::family_subject(record, true) }
                    }
                });
                link::Event::Read { read, what }
            }
            jig_client::Down::Open { watch, subject } => {
                let watched = match &subject {
                    jig_client::Watchable::Jig(subject) => Watched::Jig(subject.clone()),
                    jig_client::Watchable::Family { scope, record, .. } => Watched::Application {
                        subject: crate::test::family_subject(record, false),
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

    pub(super) fn detach(&mut self, client: ClientId) {
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

    fn core(&mut self, driver: &mut Driver, mut event: core::Event) {
        if let core::Event::SignIn { reply_to, .. } = &mut event
            && let token = reply_token(reply_to)
            && let Some(client) = self.approval_signers.remove(&token)
        {
            driver.transport.route_reply(token, Reply::SignIn { client });
        } else if let core::Event::People(people::Event::Whose { reply_to, .. }) = &mut event {
            let Some(Reply::SignIn { client }) = self.pending_reply.take() else {
                panic!("approval signer correlation");
            };
            let token = reply_token(reply_to);
            self.approval_signers.insert(token, client);
            driver.transport.route_reply(token, Reply::SignIn { client });
        } else if let core::Event::SignIn { reply_to, .. }
        | core::Event::People(
            people::Event::Ask { reply_to, .. } | people::Event::SignOut { reply_to, .. },
        )
        | core::Event::Key { reply_to, .. }
        | core::Event::ProposeSettings { reply_to, .. }
        | core::Event::ChangeModel { reply_to, .. }
        | core::Event::ChangeSettings { reply_to, .. } = &mut event
        {
            driver
                .transport
                .route_reply(reply_token(reply_to), self.pending_reply.take().expect("request correlation"));
        } else if let core::Event::Read { reply, .. } = &mut event {
            driver.transport.route_reply(reply_token(reply), self.pending_reply.take().expect("read correlation"));
        } else if let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = &event {
            let (client, native, subject) = self.pending_watch.take().expect("watch correlation");
            driver.transport.route_watch(*watcher, client, native, subject);
        } else if let core::Event::Unwatch { watcher } = &event {
            driver.transport.end_watch(*watcher);
        } else if let core::Event::Delivered { watcher } = &event {
            driver.referee.consumed(*watcher);
        }
        let watcher = if let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = &event {
            Some(*watcher)
        } else {
            None
        };
        driver.send(self.env.now, self.env.wall, root::Event::Core(event));
        if let Some(watcher) = watcher {
            let refused = driver.world.client_peer.as_ref().expect("component peer").now.iter().any(|now| {
                matches!(now,
                core::Now::WatchRefused { watcher: actual, .. } if *actual == watcher)
            });
            if !refused {
                self.event(Event::Admitted { watcher });
            }
        }
    }

    pub(super) fn reply(&mut self, driver: &mut Driver, to: skein_lib::ReplyTo, reply: people::Reply) {
        let token = to.into_token();
        if let Some(Reply::SignIn { client }) = driver.transport.reply(token)
            && let people::Reply::SignedIn { person, sign_in, .. } = &reply
        {
            driver.transport.signed_in(*client, *person, *sign_in);
        }
        driver.transport.take_reply(token);
        self.event(Event::Reply { to: skein_lib::ReplyTo::new(token), reply });
    }

    pub(super) fn pump(&mut self, driver: &mut Driver) -> Vec<Incoming> {
        self.component.fire(&self.env);
        loop {
            let mut active = false;
            let clients: Vec<_> = self.link_out.keys().copied().collect();
            for client in clients {
                while let Some(request) = self.link_out.get_mut(&client).expect("link output").pop() {
                    active = true;
                    self.link_request(client, request, driver);
                }
            }
            while let Some(request) = self.out.pop() {
                active = true;
                self.request(request, driver);
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

    fn application(&mut self, request: clients::ToApplication, driver: &mut Driver) {
        let event = jig_test_domain_family(request, &driver.limits);
        match event {
            Ok(mut event) => {
                if let root::Event::Application { reply_to, .. } = &mut event {
                    driver.transport.route_reply(
                        reply_token(reply_to),
                        self.pending_reply.take().expect("application correlation"),
                    );
                } else if let root::Event::ReadApplication { reply, .. } = &mut event {
                    driver.transport.route_reply(
                        reply_token(reply),
                        self.pending_reply.take().expect("application read correlation"),
                    );
                } else if let root::Event::WatchApplication { watcher, .. } = &event {
                    let (client, native, subject) = self.pending_watch.take().expect("application watch correlation");
                    driver.transport.route_watch(*watcher, client, native, subject);
                } else {
                    panic!("application family request");
                }
                let watcher =
                    if let root::Event::WatchApplication { watcher, .. } = &event { Some(*watcher) } else { None };
                driver.send(self.env.now, self.env.wall, event);
                if let Some(watcher) = watcher {
                    self.event(Event::Admitted { watcher });
                }
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
    fn request(&mut self, request: Request, driver: &mut Driver) {
        match request {
            Request::Core(event) => self.core(driver, *event),
            Request::Application(request) => self.application(request, driver),
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
                self.callbacks.push(Incoming::Authenticated {
                    client,
                    person: driver.transport.party(client),
                    expires,
                });
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
                let record = driver.world.take_moved(source);
                self.event(Event::MovedTaken { exchange, record });
            }
            Request::StreamFailed { .. }
            | Request::Link { .. }
            | Request::LocalLink { .. }
            | Request::LinkSignedIn
            | Request::Closed => {}
            Request::Redeemed { .. } | Request::Claimed { .. } => panic!("world has no OAuth issuer"),
        }
    }

    #[expect(clippy::too_many_lines, reason = "transport wiring carries the browser marker beside the credential")]
    fn link_request(&mut self, client: ClientId, request: link::Request, driver: &mut Driver) {
        let up = match request {
            link::Request::StreamsFull { full } => jig_client::Up::StreamsFull { full },
            link::Request::Exchange { exchange, credential, message } => {
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
                if self.approval_takes.remove(&client) {
                    self.approval_signers.retain(|_, owner| *owner != client);
                    if matches!(self.pending_reply, Some(Reply::SignIn { client: owner }) if owner == client) {
                        self.pending_reply = None;
                    }
                    if let link::Signing::SignedIn { party } = &signing {
                        self.callbacks.push(Incoming::Authenticated { client, person: *party, expires: self.env.wall });
                    }
                    self.approval_signing.entry(client).or_default().push_back(signing);
                    return;
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
                    link::Signing::Approval(_)
                    | link::Signing::LinkUsed
                    | link::Signing::LinkExpired
                    | link::Signing::LinkUnknown
                    | link::Signing::Refused
                    | link::Signing::Lost => {
                        self.callbacks.push(Incoming::SignInFailed { client, failure: SignInFailure::Capacity });
                    }
                    link::Signing::SignedOut => {
                        if let Some(sign_in) = driver.transport.sign_in(client) {
                            driver.transport.signed_out(client);
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
                if let Some(watcher) = driver
                    .transport
                    .watch_token(client, watch)
                    .or_else(|| driver.transport.application_watch_token(client, watch))
                {
                    driver.transport.end_watch(watcher);
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
                self.approval_answers.entry(client).or_default().push_back(answer);
                return;
            }
            link::Request::Authorized(_)
            | link::Request::Kept { .. }
            | link::Request::Dropped
            | link::Request::NotKept { .. } => panic!("joint link request outside transport"),
        };
        self.callbacks.push(Incoming::Callback { client, event: crate::test::up(up) });
    }
}

fn jig_test_domain_family(request: clients::ToApplication, limits: &root::Limits) -> Result<root::Event, Box<Event>> {
    use jig_client_test_protocol_wire::codec::{self, Undecodable};
    use jig_protocol_engine_clients::{FromApplication, ToApplication};
    match request {
        ToApplication::Request { reply_to, sign_in, key, family, version, scope, record, digest, carried } => {
            let decoded = if family == 1 && version == jig_client_test_protocol_wire::VERSION {
                codec::request(&record, &limits.connector)
            } else {
                Err(Undecodable)
            };
            let Ok(decoded) = decoded else {
                return Err(Box::new(Event::Application(FromApplication::Undecodable { to: reply_to })));
            };
            Ok(root::Event::Application {
                reply_to,
                sign_in,
                key,
                scope,
                number: family,
                digest,
                value: decoded.value,
                carried,
            })
        }
        ToApplication::Read { reply, sign_in, family, version, scope, window, record } => {
            let decoded = if family == 1 && version == jig_client_test_protocol_wire::VERSION {
                codec::read_value(&record, &limits.connector)
            } else {
                Err(Undecodable)
            };
            let Ok(subject) = decoded else {
                return Err(Box::new(Event::Application(FromApplication::Read { reply, family, record: None })));
            };
            Ok(root::Event::ReadApplication { reply, sign_in, scope, number: family, subject, window })
        }
        ToApplication::Watch { watcher, sign_in, family, version, scope, window, record } => {
            let decoded = if family == 1 && version == jig_client_test_protocol_wire::VERSION {
                codec::subject_value(&record, &limits.connector)
            } else {
                Err(Undecodable)
            };
            let Ok(subject) = decoded else {
                return Err(Box::new(Event::WatchRefused { watcher, why: core::WatchRefusal::Unknown }));
            };
            Ok(root::Event::WatchApplication { watcher, sign_in, scope, number: family, subject, window })
        }
    }
}

fn reply_token(reply: &mut skein_lib::ReplyTo) -> Token {
    let token = std::mem::replace(reply, skein_lib::ReplyTo::new(Token::new(0))).into_token();
    *reply = skein_lib::ReplyTo::new(token);
    token
}
