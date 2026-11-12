//! The clients child and secret-free root handoffs (protocol/client/README.md, 2 and protocol/client/sign-in.md; service.md, 3).
use crate::{Seeds, family};
use alloc::boxed::Box;
use jig_client_test_protocol_wire::VERSION;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_config::{self as config, Problems, What};
use jig_protocol_engine_clients as kit;
use jig_protocol_engine_clients_http as http;
use jig_test_domain as root;
use skein_io as io;
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall};
mod translation;
pub(super) use translation::{application, immediate, to_component};

#[expect(
    clippy::struct_excessive_bools,
    reason = "component closure, server handoff, io settlement and runnable work are independent"
)]
pub(super) struct Clients {
    component: Option<kit::Component>,
    settings: Option<kit::Settings>,
    pub(super) server: crate::server::Server,
    pub(super) pages: crate::server::Pages,
    pub(super) mount: http::Mount,
    pub(super) refused: Map<Token, ()>,
    mount_env: Env<http::Limits>,
    mount_out: Queue<http::Request>,
    pub(super) server_out: Queue<crate::server::Routed>,
    component_closed: bool,
    server_finished: bool,
    secret: Option<skein_lib::Secret>,
    pub(super) env: Env<kit::Limits>,
    out: Queue<kit::Request>,
    pub(super) io: Queue<io::Request>,
    pub(super) links: Queue<Box<[u8]>>,
    pub(super) secrets: Queue<kit::SecretsRequest>,
    pub(super) closed: bool,
    pending: bool,
    pub(super) starting: Option<()>,
    pub(super) early: Queue<root::Event>,
    endings: kit::host::Endings,
    pub(super) bridge: kit::host::Bridge,
    public_names: Option<()>,
}

// Hide printed sign-in links because they carry secrets (skein/docs/design/runtime/lib.md, 6).
impl ::core::fmt::Debug for Clients {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Clients")
            .field("component", &self.component)
            .field("settings", &self.settings)
            .field("server", &self.server)
            .field("pages", &self.pages)
            .field("mount", &self.mount)
            .field("refused", &self.refused)
            .field("mount_env", &self.mount_env)
            .field("mount_out", &self.mount_out)
            .field("server_out", &self.server_out)
            .field("component_closed", &self.component_closed)
            .field("server_finished", &self.server_finished)
            .field("secret", &self.secret)
            .field("env", &self.env)
            .field("out", &self.out)
            .field("io", &self.io)
            .field("links_entries", &self.links.len())
            .field("secrets", &self.secrets)
            .field("closed", &self.closed)
            .field("pending", &self.pending)
            .field("starting", &self.starting)
            .field("early", &self.early)
            .field("endings", &self.endings)
            .field("bridge", &self.bridge)
            .field("public_names", &self.public_names)
            .finish()
    }
}

#[derive(Debug)]
pub(super) enum Routed {
    Root(Box<root::Event>),
    Application(kit::ToApplication),
    Secrets(kit::SecretsRequest),
    Transport(kit::ToTransport),
    Link(skein_lib::Secret),
    LocalLink,
    TakeMoved { exchange: Token, source: u32 },
    Closed,
}
pub(super) fn to_root(request: kit::Request) -> Routed {
    match request {
        kit::Request::Core(event) => Routed::Root(Box::new(root::Event::Core(*event))),
        kit::Request::Application(value) => Routed::Application(value),
        kit::Request::Secrets(value) => Routed::Secrets(value),
        kit::Request::Transport(message) => Routed::Transport(message),
        kit::Request::Link { secret } => Routed::Link(secret),
        // The reference service keeps no local link (protocol/service.md, 7).
        kit::Request::StreamFailed { .. } | kit::Request::LocalLink { .. } | kit::Request::LinkSignedIn => {
            Routed::LocalLink
        }
        kit::Request::Minted { .. }
        | kit::Request::MintRefused { .. }
        | kit::Request::Redeemed { .. }
        | kit::Request::Claimed { .. } => {
            unreachable!("the test service mints no application sign-ins")
        }
        kit::Request::TakeMoved { exchange, source } => Routed::TakeMoved { exchange, source },
        kit::Request::Closed => Routed::Closed,
    }
}
impl Clients {
    pub fn new(
        config: &config::Engine,
        listener: io::Addr,
        seeds: &Seeds,
        transport: &crate::server::Limits,
    ) -> Result<Self, Problems> {
        if jig_protocol_accounts::moved::bound(&config.limits.driver.client.document)
            > u64::from(config.limits.clients.wire.keyed_value_some)
        {
            return Err(config::clients::invalid(What::Bound));
        }
        let link_provider = config::clients::link_provider(config);
        let link = if config.clients.link {
            let provider = link_provider.ok_or(config::clients::invalid(What::Refers))?;
            let owner = config::clients::link_owner(config, provider).ok_or(config::clients::invalid(What::Refers))?;
            Some(people::Identity {
                key: owner.identity.clone(),
                login: Box::from(b"owner".as_slice()),
                name: Box::from(b"Owner".as_slice()),
            })
        } else {
            None
        };
        let application = config::clients::Application {
            link,
            served: kit::host::served(kit::Served { family: Some(1), first: VERSION, last: VERSION }),
            public: List::with_capacity(0),
            moved_bytes: jig_protocol_accounts::moved::bound(&config.limits.driver.client.document),
        };
        let settings = config::clients::settings(config, &application, seeds.crypto.clone(), seeds.seed);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients };
        let out = Queue::uncapped();
        Ok(Self {
            refused: Map::with_capacity(transport.connections),
            component: None,
            settings: Some(settings),
            server: crate::server::Server::new(listener, transport),
            pages: crate::server::Pages::new(transport),
            mount: http::Mount::new(
                http::Settings { max_age: config.limits.clients.public.max_age, ..http_settings() },
                &config.limits.clients_http,
            ),
            mount_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients_http },
            mount_out: Queue::uncapped(),
            server_out: Queue::uncapped(),
            component_closed: false,
            server_finished: false,
            secret: None,
            env,
            out,
            io: Queue::uncapped(),
            links: Queue::uncapped(),
            secrets: Queue::uncapped(),
            closed: false,
            pending: true,
            starting: Some(()),
            early: Queue::uncapped(),
            endings: kit::host::Endings::new(u32::MAX),
            bridge: kit::host::Bridge::new(crate::tokens::CLIENTS),
            public_names: None,
        })
    }
    pub fn start(&mut self, now: Time, wall: Wall) -> Result<(), Problems> {
        self.env.now = now;
        self.env.wall = wall;
        if let Some(settings) = self.settings.take() {
            self.component = Some(
                kit::Component::new(settings, &self.env, &mut self.out)
                    .or(Err(config::clients::invalid(What::Bound)))?,
            );
        }
        Ok(())
    }
    fn print_link(&mut self) {
        if let Some(listener) = self.server.listening()
            && let Some(secret) = self.secret.take()
        {
            self.links.push(printed(listener, &secret));
        }
    }
    pub fn opened(&self) -> bool {
        self.server.opened()
    }
    pub fn next_deadline(&self) -> Option<Time> {
        let component = match &self.component {
            Some(component) => component.next_deadline(),
            None => None,
        };
        skein_lib::earliest(component, self.server.next_deadline())
    }
    pub fn work_pending(&self, now: Time) -> bool {
        !self.closed
            && (self.pending
                || !self.secrets.is_empty()
                || self.endings.ready()
                || !self.out.is_empty()
                || !self.early.is_empty()
                || !self.io.is_empty()
                || !self.links.is_empty()
                || match self.next_deadline() {
                    Some(at) => at <= now,
                    None => false,
                })
    }
    pub(super) fn catalogue(&mut self, models: Box<[core::settings::CatalogueModel]>) {
        let models = kit::translate::llm::catalogue(&models);
        if let Some(settings) = &mut self.settings {
            settings.models = models;
        } else {
            self.down(kit::Event::Catalogue { models });
        }
    }
    pub fn down(&mut self, event: kit::Event) {
        if let kit::Event::Stop = &event {
            self.server.stop(&mut self.server_out);
        }
        if let kit::Event::Close = &event {
            self.server.close(&mut self.server_out);
        }
        if let kit::Event::Abort = &event {
            self.server.abort(&mut self.server_out);
        }
        if !self.component_closed
            && let Some(component) = &mut self.component
        {
            component.down(&self.env, event, &mut self.out);
            self.pending = true;
        }
    }
    pub fn up(&mut self, event: io::Event) -> bool {
        let event = untag(event);
        if let io::Event::Failed { owner, .. } = &event
            && *owner == Token::new(0)
        {
            return false;
        }
        if let io::Event::Listening { addr, .. } = &event
            && let Some(address) = addr.socket()
        {
            let mut hosts = List::with_capacity(1);
            hosts.push(http::address_text(address).expect("bound listener address")).expect("one listener host");
            self.mount.down(&self.mount_env, http::Event::Listening { hosts });
        }
        self.server.up(event, &mut self.server_out);
        self.print_link();
        self.pending = true;
        true
    }
    pub(super) fn server_requests(&mut self) {
        for _ in 0..self.server_out.len() {
            match self.server_out.pop().expect("server requests") {
                crate::server::Routed::Io(request) => self.io.push(tag(request)),
                crate::server::Routed::Call { call, head } => {
                    let answer = self.pages.route(&head);
                    if answer != crate::server::Answer::Mount {
                        self.pages.call(&mut self.server, call, &head, answer, &mut self.server_out);
                    } else if self.mount.room_for(&head) > 0 {
                        self.mount.down(&self.mount_env, http::Event::Call { call, head });
                    } else {
                        self.refused.insert(call, ()).expect("bounded refused server calls");
                        self.server.answer(call, skein_http::server::Request::Discard, &mut self.server_out);
                        self.server.answer(
                            call,
                            skein_http::server::Request::Respond(self.mount.refusal(&head)),
                            &mut self.server_out,
                        );
                    }
                }
                crate::server::Routed::Http { call, event } => {
                    if self.pages.owns(call) {
                        self.pages.event(&mut self.server, call, event, &mut self.server_out);
                    } else if self.refused.get(&call).is_some() {
                        match event {
                            skein_http::server::Event::Done(_)
                            | skein_http::server::Event::Failed(_)
                            | skein_http::server::Event::Closed => {
                                self.refused.remove(&call);
                                self.server.done(call, &mut self.server_out);
                            }
                            skein_http::server::Event::Call(_)
                            | skein_http::server::Event::Ended
                            | skein_http::server::Event::Body(_)
                            | skein_http::server::Event::Reply(_)
                            | skein_http::server::Event::Refused(_) => {}
                        }
                    } else {
                        self.mount.down(&self.mount_env, http::Event::Http { call, event });
                    }
                }
            }
        }
        self.closed = self.component_closed && self.server.closed();
    }
    fn root(&mut self, value: root::Event, events: &mut Queue<root::Event>) {
        if self.starting.is_some() {
            self.early.push(value);
        } else {
            events.push(value);
        }
    }
    pub fn advance(
        &mut self,
        events: &mut Queue<root::Event>,
        bound: &jig_test_connector::Limits,
        rows: u32,
        accounts: &mut Option<crate::accounts::Accounts>,
    ) {
        for _ in 0..self.out.len() {
            match to_root(self.out.pop().expect("client requests")) {
                Routed::Transport(message) => {
                    self.mount.down(&self.mount_env, http::Event::Component(message));
                }
                Routed::LocalLink => {}
                Routed::Link(value) => {
                    self.secret = Some(value);
                    self.print_link();
                }
                Routed::TakeMoved { exchange, source } => {
                    let record = match accounts {
                        Some(accounts) => accounts.take_moved(source),
                        None => None,
                    };
                    self.down(kit::Event::MovedTaken { exchange, record });
                }
                Routed::Secrets(kit::SecretsRequest::Get { owner: kit::SecretOwner::PublicNames }) => {
                    self.public_names = Some(());
                }
                Routed::Secrets(value) => self.secrets.push(value),
                Routed::Root(value) => self.root(*value, events),
                Routed::Application(value) => match family::to_root(value, bound, rows) {
                    Ok(value) => self.root(value, events),
                    Err(value) => self.down(*value),
                },
                Routed::Closed => {
                    self.component_closed = true;
                    self.public_names = None;
                }
            }
        }
    }
    pub fn iterate(
        &mut self,
        now: Time,
        wall: Wall,
        events: &mut Queue<root::Event>,
        bound: &jig_test_connector::Limits,
        rows: u32,
        accounts: &mut Option<crate::accounts::Accounts>,
    ) {
        self.env.now = now;
        self.env.wall = wall;
        self.mount_env.now = now;
        self.mount_env.wall = wall;
        self.server.clock(now, wall);
        if self.closed {
            return;
        }
        self.server.fire(&mut self.server_out);
        let server_pending = self.server.advance(&mut self.server_out);
        self.server_requests();
        self.endings_step(events, bound, rows, accounts);
        self.advance(events, bound, rows, accounts);
        if self.closed {
            return;
        }
        if !self.component_closed {
            self.component.as_mut().expect("clients started at first service clock").fire(&self.env);
        }
        self.advance(events, bound, rows, accounts);
        self.pending = server_pending || !self.server_out.is_empty();
        let mount_pending = self.mount.up(&self.mount_env, &mut self.mount_out);
        self.pending |= mount_pending;
        if self.component_closed
            && !self.server_finished
            && !mount_pending
            && self.mount_out.is_empty()
            && self.out.is_empty()
        {
            self.server.finish();
            self.server_finished = true;
            self.pending = true;
        }
        if let Some(request) = self.mount_out.pop() {
            self.pending = true;
            match request {
                http::Request::Component(message) => self.down(kit::Event::Transport(message)),
                http::Request::Http { call, request } => self.server.answer(call, request, &mut self.server_out),
            }
        }
        if !self.component_closed {
            self.pending |=
                self.component.as_mut().expect("clients started at first service clock").up(&self.env, &mut self.out);
        }
        self.advance(events, bound, rows, accounts);
    }
}
pub(super) fn owns(token: Token) -> bool {
    token.tag() == crate::tokens::CLIENTS
}
pub(super) fn tag(request: io::Request) -> io::Request {
    crate::tokens::tag(request, crate::tokens::CLIENTS)
}
pub(super) fn untag(event: io::Event) -> io::Event {
    crate::tokens::untag(event)
}

impl Clients {
    pub fn root_event(&mut self, value: &root::Event) -> Option<Token> {
        match value {
            root::Event::Core(value) => translation::watcher(value),
            root::Event::WatchApplication { watcher, .. } => Some(*watcher),
            root::Event::Committed { number, .. } => {
                self.committed(*number);
                None
            }
            root::Event::StartRecord(_)
            | root::Event::StartMissing { .. }
            | root::Event::StartPageRefused { .. }
            | root::Event::StartBegin { .. }
            | root::Event::StartDone(_)
            | root::Event::Link { .. }
            | root::Event::ResultRead { .. }
            | root::Event::Stop
            | root::Event::ReadApplication { .. }
            | root::Event::Application { .. }
            | root::Event::WriterRead { .. }
            | root::Event::ConnectorTimer { .. }
            | root::Event::Timer(..)
            | root::Event::Credential { .. }
            | root::Event::Connector { .. }
            | root::Event::ProjectionEffect { .. }
            | root::Event::TopicCall { .. }
            | root::Event::EffectCall { .. }
            | root::Event::CallAnswered { .. }
            | root::Event::Text { .. }
            | root::Event::Turn { .. }
            | root::Event::Answer { .. }
            | root::Event::TranscriptLoaded { .. }
            | root::Event::Failed { .. } => None,
        }
    }
}

impl Clients {
    pub fn commit(&mut self, number: u64, writes: &Queue<root::Write>) {
        for write in writes {
            match write {
                root::Write::Erase(key) => {
                    if let Some(sign_in) = ended(key) {
                        self.endings.ended(number, sign_in);
                    }
                }
                root::Write::DropTranscript { .. }
                | root::Write::Wires
                | root::Write::Save(_)
                | root::Write::CopySettingsRevision(_) => {}
            }
        }
    }
    pub fn committed(&mut self, number: u64) {
        self.endings.committed(number);
    }
    fn endings_step(
        &mut self,
        events: &mut Queue<root::Event>,
        bound: &jig_test_connector::Limits,
        rows: u32,
        accounts: &mut Option<crate::accounts::Accounts>,
    ) {
        // Only the sign-ins retained when this stage starts can end here.
        for _ in 0..self.endings.waiting() {
            let Some(sign_in) = self.endings.next() else { return };
            self.down(kit::Event::SignInEnded { sign_in });
            self.advance(events, bound, rows, accounts);
        }
    }
}

fn ended(key: &root::Key) -> Option<u64> {
    match key {
        root::Key::Core(key) => kit::host::ending(key),
        root::Key::Connector { .. } => None,
    }
}

impl Clients {
    pub fn not_ready(&mut self, value: root::Event) {
        let event = match value {
            root::Event::Core(value) => translation::starting(value),
            root::Event::Application { reply_to, .. } => {
                Some(kit::Event::Reply { to: reply_to, reply: people::Reply::Again(people::Again::NotReady) })
            }
            root::Event::ReadApplication { reply, .. } => {
                Some(kit::Event::Read { reply, answer: core::reads::Answer::None(core::reads::Why::Busy) })
            }
            root::Event::WatchApplication { watcher, .. } => {
                Some(kit::Event::WatchRefused { watcher, why: core::WatchRefusal::Busy })
            }
            root::Event::StartRecord(_)
            | root::Event::StartMissing { .. }
            | root::Event::StartPageRefused { .. }
            | root::Event::StartBegin { .. }
            | root::Event::StartDone(_)
            | root::Event::Link { .. }
            | root::Event::ResultRead { .. }
            | root::Event::Stop
            | root::Event::WriterRead { .. }
            | root::Event::ConnectorTimer { .. }
            | root::Event::Timer(..)
            | root::Event::Credential { .. }
            | root::Event::Connector { .. }
            | root::Event::ProjectionEffect { .. }
            | root::Event::TopicCall { .. }
            | root::Event::EffectCall { .. }
            | root::Event::CallAnswered { .. }
            | root::Event::Text { .. }
            | root::Event::Turn { .. }
            | root::Event::Answer { .. }
            | root::Event::TranscriptLoaded { .. }
            | root::Event::Committed { .. }
            | root::Event::Failed { .. } => None,
        };
        if let Some(event) = event {
            self.down(event);
        }
    }
}

impl Clients {
    pub(super) fn secrets_step(&mut self, store: &mut crate::store::Store) {
        // Answer retained requests before creating the next correlations.
        for _ in 0..store.host.secrets.len() {
            let answer = store.host.secrets.pop().expect("secrets terminals");
            self.secret_answer(answer);
        }
        self.advance_secrets(store);
    }
    fn advance_secrets(&mut self, store: &mut crate::store::Store) {
        if self.public_names.is_some() && store.host.mark_durable() {
            self.public_names = None;
            self.secrets.push(kit::SecretsRequest::Get { owner: kit::SecretOwner::PublicNames });
        }
        for _ in 0..self.secrets.len() {
            let request = self.secrets.pop().expect("client secrets requests");
            let (token, request) = self.bridge.ask(request);
            if let Some(answer) = store.host.secrets(token, request) {
                self.secret_answer(answer);
            }
        }
    }
    fn secret_answer(&mut self, answer: jig_protocol_store::Event) {
        let answer = self.bridge.answer(answer);
        self.down(kit::Event::Secrets(answer));
    }
}
pub(super) fn printed(address: ::core::net::SocketAddr, secret: &skein_lib::Secret) -> Box<[u8]> {
    http::printed_link(address, &http::link_path(&http_settings(), secret))
}

pub(super) fn http_settings() -> http::Settings {
    http::Settings { max_age: 0, prefix: Box::from(b"/jig".as_slice()), public: None }
}

#[cfg(test)]
#[path = "clients/public.rs"]
mod public_tests;
