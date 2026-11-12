//! The clients child and secret-free root handoffs (client/README.md, 2 and client/engine.md; jig's
//! protocol/service.md, 3).
//! Local links stay apart from printed links (client/tui/README.md, 2).
use crate::settings::Seeds;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_engine_clients as kit;
pub use jig_protocol_engine_clients::LINK_SECRET_BYTES;
use jig_protocol_engine_clients_http as http;
use skein_io as io;
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall};
use temper_engine_domain as root;

/// The URL a printed link's secret opens, as serve prints it, with no line feed (protocol/setup/box.md, 2).
#[must_use]
pub fn first_link_url(
    address: ::core::net::SocketAddr,
    public: Option<&crate::startup::application::Origin>,
    secret: &skein_lib::Secret,
) -> Box<[u8]> {
    printed::link_public(address, public, secret)
}
#[cfg(test)]
mod admission_tests;
pub mod bundle;
pub mod fetch;
pub mod forge_sign_in;
mod github;
pub mod google;
#[cfg(test)]
mod playground_tests;
pub(crate) mod policy;
pub(crate) mod printed;
mod provider;
pub mod translation;
pub(crate) const PLAYGROUND_STARTS: u32 = 8;

#[expect(missing_debug_implementations, reason = "clients hold pending OAuth secrets")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "printed-link use and its pending document mark are independent of client lifecycle"
)]
pub struct Clients {
    /// Bound address awaiting the system's note routing.
    pub(crate) listening: Option<::core::net::SocketAddr>,
    pub(crate) address: io::Addr,
    public: Option<crate::startup::application::Origin>,
    directories: config::Directories,
    component: kit::Component,
    initial: Option<kit::Settings>,
    provider_documents: Option<provider::Documents>,
    provider_replies: Map<Token, provider::Reply>,
    next_provider: u64,
    server: crate::server::Server,
    mount: http::Mount,
    mount_env: Env<http::Limits>,
    mount_out: Queue<http::Request>,
    server_out: Queue<crate::server::Routed>,
    waiting: Map<Token, skein_http::server::Call>,
    replies: Map<Token, bundle::Reply>,
    github: Map<Token, github::Reply>,
    pub(crate) google: Option<google::Google>,
    google_replies: Map<Token, google::Reply>,
    pub(crate) github_calls: Queue<(Token, skein_http::server::Call)>,
    pub(crate) query_bytes: u32,
    send: u32,
    pub(super) bundle: Option<bundle::Bundle>,
    pub(crate) forge_sign_in: forge_sign_in::Entries,
    pending_forge_metas: Option<Box<[u8]>>,
    component_closed: bool,
    stopping: Option<()>,

    secret: Option<skein_lib::Secret>,
    local_secret: Option<skein_lib::Secret>,
    linked: bool,
    linked_changed: bool,
    pub(crate) env: Env<kit::Limits>,
    out: Queue<kit::Request>,
    pub(crate) io: Queue<io::Request>,
    pub(crate) links: Queue<Box<[u8]>>,
    pub(crate) local_links: Queue<Box<[u8]>>,
    pub(crate) secrets: Queue<kit::SecretsRequest>,
    pub(crate) closed: bool,
    pending: bool,
    pub(crate) starting: Option<()>,
    pub(crate) early: Queue<root::Event>,
    endings: kit::host::Endings,
    bridge: kit::host::Bridge,
    #[cfg(test)]
    pub(crate) secret_terminals: Option<Queue<kit::SecretOwner>>,
    // The read view for this clients pass (temper/docs/design/protocol/setup/playground.md).
    pub(crate) deployment: crate::family::Deployment,
    pub(crate) program_environment: Box<[u8]>,
    pub(crate) playground: Option<temper_engine_domain_forge::Playground>,
    playground_starts: Map<Token, root::Event>,
    playground_reads: Queue<crate::HandoverRead>,
    playground_ready: Queue<root::Event>,
    next_playground: u64,
    playground_carries: Queue<kit::SecretsRequest>,
}
impl Clients {
    /// The printed link signed a person in since start (client/forge/records.md).
    pub(crate) const fn linked(&self) -> bool {
        self.linked
    }

    pub(crate) fn take_linked_changed(&mut self) -> bool {
        ::core::mem::replace(&mut self.linked_changed, false)
    }

    pub(crate) fn playground_read(&mut self) -> Option<crate::HandoverRead> {
        self.playground_reads.pop()
    }
    pub(crate) fn owns_playground_read(&self, run: Token) -> bool {
        self.playground_starts.contains_key(&run)
    }

    /// The service reads the file at each start and carries its bytes as a
    /// pasted LLM key's (protocol/setup/playground.md). The shell's private read removes
    /// one final LF, as it does for a handover.
    pub(crate) fn playground_read_answer(
        &mut self,
        run: Token,
        answer: Result<Option<skein_lib::Secret>, crate::HandoverUnread>,
    ) {
        let value = match answer {
            Ok(Some(value)) if !value.expose().is_empty() && value.expose().len() <= 4096 => Some(value),
            Ok(Some(_) | None) | Err(_) => None,
        };
        if let Some(value) = value {
            if let Some(root::Event::Party(root::PartyInput::Application { carried, .. })) =
                self.playground_starts.get_mut(&run)
            {
                *carried = Some(core::Carried { number: 0, digest: skein_lib::sha256::digest(value.expose()) });
            }
            self.playground_carries.push(kit::SecretsRequest::Carry {
                exchange: run,
                value,
                kind: kit::CarriedKind::Key,
            });
        } else {
            self.playground_carried(run, None);
        }
    }
    fn playground_carried(&mut self, run: Token, number: Option<u64>) {
        let Some(mut event) = self.playground_starts.remove(&run) else { return };
        if let root::Event::Party(root::PartyInput::Application { carried, .. }) = &mut event {
            match number {
                Some(number) => {
                    carried.as_mut().expect("carried demo digest").number = number;
                }
                None => *carried = None,
            }
        }
        self.playground_ready.push(event);
    }

    #[expect(clippy::too_many_lines, reason = "startup constructs bounded queues beside the client component")]
    #[expect(
        clippy::too_many_arguments,
        reason = "the owned machine supplies the printed link's person beside existing startup inputs"
    )]
    pub fn new(
        directories: &config::Directories,
        config: &config::Engine,
        clients: &crate::startup::application::Clients,
        seeds: &Seeds,
        journal: &root::JournalLimits,
        sign_in: Option<&crate::settings::SignIn>,
        machine: Option<&temper_forge_setup::machine::OwnedMachine>,
        files: &[(Box<[u8]>, skein_lib::Secret)],
    ) -> Result<Self, Problems> {
        if !crate::server::valid(&crate::limits::server(&config.limits)) {
            return Err(config::clients::invalid(What::Bound));
        }
        let config::Address::Addr(listener) = &clients.listener else {
            return Err(config::clients::invalid(What::Form));
        };
        let link_provider = config::clients::link_provider(config);
        let link = if config.clients.link {
            let provider = match sign_in {
                Some(sign_in) => sign_in.provider,
                None => link_provider.ok_or(config::clients::invalid(What::Refers))?,
            };
            let owner = config::clients::link_owner(config, provider).ok_or(config::clients::invalid(What::Refers))?;
            Some(people::Identity {
                key: owner.identity.clone(),
                login: match sign_in {
                    Some(sign_in) => google::member_login(sign_in, &owner.identity.subject)
                        .ok_or(config::clients::invalid(What::Refers))?,
                    None => match machine {
                        Some(machine) if machine.sign_in.is_some() => machine.person.login.clone(),
                        Some(_) | None => Box::from(b"owner".as_slice()),
                    },
                },
                name: match sign_in {
                    Some(sign_in) => google::member_login(sign_in, &owner.identity.subject)
                        .ok_or(config::clients::invalid(What::Refers))?,
                    None => Box::from(b"Owner".as_slice()),
                },
            })
        } else {
            None
        };
        let application = application(
            config,
            link,
            kit::host::served(kit::Served {
                family: Some(temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge)),
                first: temper_client_protocol_wire::CLIENT_VERSION,
                last: temper_client_protocol_wire::CLIENT_VERSION,
            }),
        );
        // The provider owns its signer; construct the initial settings twice without cloning it.
        let settings = || -> Result<kit::Settings, Problems> {
            let provider = match &config.clients.provider {
                Some(provider) => match config::clients::provider_issuer(
                    provider,
                    files,
                    provider::urls(clients, *listener)?,
                    &config.limits.clients,
                ) {
                    Ok(issuer) => Some(issuer),
                    Err(problem) => return Err(provider::problem(problem)),
                },
                None => None,
            };
            Ok(kit::Settings {
                private: temper_engine_smith::private(),
                link_secret: seeds.link_secret.clone(),
                local_secret: seeds.local_secret.clone(),
                provider,
                addresses: match &config.clients.provider {
                    Some(provider) => provider.addresses.clone(),
                    None => Box::new([]),
                },
                ..config::clients::settings(config, &application, seeds.crypto.clone(), seeds.seed)
            })
        };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients };
        let mut out = Queue::uncapped();
        let initial = settings()?;
        let provider_documents = match &initial.provider {
            Some(issuer) => Some(provider::Documents { discovery: issuer.discovery(), keys: issuer.key_set() }),
            None => None,
        };
        if config.limits.clients.public.on() {
            for kind in &initial.public {
                if kind.bound > config.limits.clients.public.answer_bytes {
                    let mut problems = Problems::empty();
                    problems
                        .push(Path::root().member(b"clients").member(b"public").member(b"answer_bytes"), What::Bound);
                    return Err(problems);
                }
            }
        }
        let component = kit::Component::new(initial, &env, &mut out).or(Err(config::clients::invalid(What::Bound)))?;
        Ok(Self {
            listening: None,
            address: *listener,
            public: clients.public.clone(),
            directories: directories.clone(),
            component,
            initial: Some(settings()?),
            provider_documents,
            provider_replies: Map::with_capacity(crate::limits::server(&config.limits).connections),
            next_provider: 1,
            server: crate::server::Server::new(*listener, &crate::limits::server(&config.limits)),
            mount: http::Mount::new(
                http::Settings {
                    max_age: config.limits.clients.public.max_age,
                    ..http_settings(clients.public.as_ref())
                },
                &config.limits.clients_http,
            ),
            mount_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients_http },
            mount_out: Queue::uncapped(),
            server_out: Queue::uncapped(),
            waiting: Map::with_capacity(crate::limits::server(&config.limits).connections),
            replies: Map::with_capacity(crate::limits::server(&config.limits).connections),
            google: None,
            google_replies: Map::with_capacity(crate::limits::server(&config.limits).connections),
            github: Map::with_capacity(crate::limits::server(&config.limits).connections),
            github_calls: Queue::with_capacity(crate::limits::server(&config.limits).connections),
            query_bytes: 2048,
            send: crate::limits::server(&config.limits).http.send,
            bundle: None,
            forge_sign_in: forge_sign_in::Entries::new(&[], &[], None, None, 0),
            pending_forge_metas: None,
            component_closed: false,
            stopping: None,

            secret: None,
            local_secret: None,
            linked: false,
            linked_changed: false,
            env,
            out,
            io: Queue::uncapped(),
            links: Queue::uncapped(),
            local_links: Queue::uncapped(),
            secrets: Queue::uncapped(),
            closed: false,
            pending: true,
            starting: Some(()),
            early: Queue::uncapped(),
            endings: kit::host::Endings::new(journal.commits),
            bridge: kit::host::Bridge::new(Token::new(crate::tokens::CLIENTS).tag()),
            #[cfg(test)]
            secret_terminals: None,
            deployment: crate::family::Deployment { welcome: false, owned: false, program_login: None },
            program_environment: Box::new([]),
            playground: None,
            playground_starts: Map::with_capacity(PLAYGROUND_STARTS),
            playground_reads: Queue::with_capacity(PLAYGROUND_STARTS),
            playground_ready: Queue::with_capacity(PLAYGROUND_STARTS),
            next_playground: 1,
            playground_carries: Queue::with_capacity(PLAYGROUND_STARTS),
        })
    }
    /// Hold the bundle the binary read at start.
    pub(crate) fn adopt_bundle(&mut self, bundle: bundle::Bundle) {
        self.bundle = Some(bundle);
    }
    fn print_link(&mut self) {
        if let Some(listener) = self.server.listening()
            && let Some(secret) = self.secret.take()
        {
            self.links.push(printed::link_public(listener, self.public.as_ref(), &secret));
        }
        if let Some(listener) = self.server.listening()
            && let Some(secret) = self.local_secret.take()
        {
            self.local_links.push(printed::link_public(listener, self.public.as_ref(), &secret));
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        skein_lib::earliest(
            skein_lib::earliest(self.component.next_deadline(), self.server.next_deadline()),
            match &self.google {
                Some(google) => google.next_deadline(),
                None => None,
            },
        )
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.work_pending_with_secrets(now, true)
    }
    // The service holds secret work until its startup mark is durable;
    // listener and transport work can run before that mark.
    pub(crate) fn work_pending_with_secrets(&self, now: Time, secrets_ready: bool) -> bool {
        !self.closed
            && ((match &self.google {
                Some(google) => google.work_pending(),
                None => false,
            }) || self.pending
                || self.stopping.is_none() && self.forge_sign_in.waiting()
                || secrets_ready && !self.secrets.is_empty()
                || self.endings.ready()
                || secrets_ready && !self.playground_carries.is_empty()
                || !self.playground_ready.is_empty()
                || !self.out.is_empty()
                || !self.early.is_empty()
                || !self.github_calls.is_empty()
                || !self.io.is_empty()
                || self.starting.is_none() && !self.links.is_empty()
                || match self.next_deadline() {
                    Some(at) => at <= now,
                    None => false,
                })
    }
    pub fn down(&mut self, event: kit::Event) {
        if let kit::Event::Stop | kit::Event::Close | kit::Event::Abort = &event {
            self.stopping = Some(());
            self.initial = None;
        }
        if let kit::Event::Stop = &event {
            self.server.stop(&mut self.server_out);
        }
        if let kit::Event::Close = &event {
            self.server.close(&mut self.server_out);
        }
        if let kit::Event::Abort = &event {
            self.server.abort(&mut self.server_out);
        }
        if !self.component_closed {
            self.component.down(&self.env, event, &mut self.out);
            self.pending = true;
        }
    }
    pub(crate) fn abort(&mut self, unstarted: bool) {
        self.down(kit::Event::Abort);
        if unstarted {
            self.io = Queue::uncapped();
            self.closed = true;
        }
    }
    #[must_use]
    pub fn opened(&self) -> bool {
        self.server.opened()
    }
    pub fn up(&mut self, event: io::Event) -> bool {
        let owner = event_owner(&event);
        if let Some(owner) = owner
            && owner.raw() & (1_u64 << 59_u32) != 0
        {
            self.google_up(event);
            return true;
        }
        let event = untag(event);
        let mut failed = false;
        if let io::Event::Failed { owner, .. } | io::Event::Closed { owner } = &event
            && *owner == Token::new(0)
            && self.stopping.is_none()
        {
            failed = true;
        }
        if let io::Event::Listening { addr, .. } = &event
            && let Some(address) = addr.socket()
        {
            let mut hosts = List::with_capacity(2);
            if let Some(origin) = &self.public {
                hosts.push(origin.name.clone()).expect("public host");
            }
            hosts.push(printed::address_text(address).expect("bound listener address")).expect("one listener host");
            self.mount.down(&self.mount_env, http::Event::Listening { hosts });
        }
        let previous = self.server.listening();
        // The server must observe the terminal bind event before the engine
        // aborts, or it waits forever for the listener to finish opening.
        self.server.up(event, &mut self.server_out);
        if self.server.listening() != previous {
            self.listening = self.server.listening();
        }
        self.print_link();
        self.pending = true;
        !failed
    }
    fn server_requests(&mut self) {
        for _ in 0..self.server_out.len() {
            match self.server_out.pop().expect("server requests") {
                crate::server::Routed::Io(request) => self.io.push(tag(request)),
                crate::server::Routed::Call { call, head } => {
                    // Claims depend on the prefix alone; the mounted mapping checks the Host.
                    let claimed = http::claims(&http_settings(None), &head);
                    if claimed && !head.target.starts_with(b"/jig/v1/public/") && self.stopping.is_none() {
                        // The existing connection and stall bounds hold this head;
                        // its body is not asked for until the mapping has room.
                        self.waiting.insert(call, head).expect("one waiting head per server connection");
                    } else if claimed && self.mount.room_for(&head) > 0 {
                        self.mount.down(&self.mount_env, http::Event::Call { call, head });
                    } else if !claimed && provider::claims(&head.target) {
                        self.provider_call(call, head);
                    } else if !claimed && google::claims(&head.target) {
                        self.google_call(call, head);
                    } else if !claimed && github::claims(&head.target) {
                        self.github_call(call, head);
                    } else {
                        self.serve_call(call, &head, claimed);
                    }
                }
                crate::server::Routed::Http { call, event } => {
                    if self.waiting.contains_key(&call) {
                        match event {
                            skein_http::server::Event::Closed
                            | skein_http::server::Event::Ended
                            | skein_http::server::Event::Done(_)
                            | skein_http::server::Event::Failed(_)
                            | skein_http::server::Event::Refused(_) => {
                                self.waiting.remove(&call).expect("waiting head");
                                self.server.done(call, &mut self.server_out);
                            }
                            skein_http::server::Event::Body(_)
                            | skein_http::server::Event::Reply(_)
                            | skein_http::server::Event::Call(_) => {
                                unreachable!("waiting heads ask for no body or reply")
                            }
                        }
                    } else if self.provider_replies.contains_key(&call) {
                        self.provider_event(call, event);
                    } else if self.google_replies.contains_key(&call) {
                        self.google_event(call, event);
                    } else if self.github.contains_key(&call) {
                        self.github_event(call, event);
                    } else if self.replies.get(&call).is_some() {
                        self.serve_event(call, event);
                    } else {
                        self.mount.down(&self.mount_env, http::Event::Http { call, event });
                    }
                }
            }
        }
        self.closed = self.component_closed
            && self.waiting.is_empty()
            && self.server.closed()
            && match &self.google {
                Some(google) => google.closed(),
                None => true,
            };
    }
    fn admit_waiting(&mut self) -> bool {
        let mut ready = None;
        for (call, head) in &self.waiting {
            if self.stopping.is_some() || self.mount.room_for(head) > 0 {
                ready = Some(*call);
                break;
            }
        }
        let Some(call) = ready else { return false };
        let head = self.waiting.remove(&call).expect("ready head");
        if self.stopping.is_some() {
            self.serve_call(call, &head, true);
        } else {
            self.mount.down(&self.mount_env, http::Event::Call { call, head });
        }
        true
    }
    fn root(&mut self, value: root::Event, events: &mut Queue<root::Event>) {
        if self.starting.is_some() {
            self.early.push(value);
        } else {
            events.push(value);
        }
    }
    #[expect(clippy::too_many_lines, reason = "exhaustive client routing includes the printed-link sign-in notice")]
    pub fn advance(
        &mut self,
        events: &mut Queue<root::Event>,
        forges: &crate::forges::Forges,
        demo: Option<&crate::settings::Demo>,
        accounts: &mut Option<crate::accounts::Accounts>,
    ) {
        for _ in 0..self.playground_ready.len() {
            if events.room() == 0 {
                break;
            }
            let value = self.playground_ready.pop().expect("ready demo start");
            self.root(value, events);
        }
        for _ in 0..self.out.len() {
            if self.playground_starts.len().saturating_add(self.playground_ready.len()) >= PLAYGROUND_STARTS
                || (self.starting.is_none() && events.room() == 0)
            {
                break;
            }
            match self.out.pop().expect("client requests") {
                kit::Request::Transport(message) => {
                    self.mount.down(&self.mount_env, http::Event::Component(message));
                }
                kit::Request::LinkSignedIn => {
                    if !self.linked {
                        self.linked = true;
                        self.linked_changed = true;
                    }
                }
                kit::Request::LocalLink { secret } => {
                    self.local_secret = Some(secret);
                    self.print_link();
                }
                kit::Request::Link { secret } => {
                    self.secret = Some(secret);
                    self.print_link();
                }
                kit::Request::TakeMoved { exchange, source } => {
                    let record = match accounts {
                        Some(accounts) => accounts.take_moved(source),
                        None => None,
                    };
                    self.down(kit::Event::MovedTaken { exchange, record });
                }
                kit::Request::Secrets(value) => self.secrets.push(value),
                kit::Request::Core(value) => match translation::to_root(*value) {
                    Ok(value) => self.root(root::Event::Party(value), events),
                    Err(value) => self.down(*value),
                },
                kit::Request::Application(value) => {
                    let carried = match &value {
                        kit::ToApplication::Request { carried, .. } => *carried,
                        kit::ToApplication::Read { .. } | kit::ToApplication::Watch { .. } => None,
                    };
                    match crate::family::to_root(
                        &self.directories,
                        value,
                        forges,
                        demo,
                        &self.deployment,
                        self.playground.as_ref(),
                        &temper_client_protocol_wire::v1::CEILINGS,
                    ) {
                        Ok(mut value) => {
                            if let root::Event::Party(root::PartyInput::Application {
                                request: root::Application::Forge(temper_engine_domain_forge::Ask::StartPlayground),
                                ..
                            }) = &value
                                && let Some(demo) = demo
                            {
                                if let root::Event::Party(root::PartyInput::Application { carried, .. }) = &mut value
                                    && let Some(carried) = carried.take()
                                {
                                    self.down(kit::Event::Application(kit::FromApplication::Unused {
                                        number: carried.number,
                                    }));
                                }
                                let run = Token::new((1_u64 << 63) | self.next_playground);
                                self.next_playground = self.next_playground.checked_add(1).expect("demo read sequence");
                                self.playground_starts.insert(run, value).expect("bounded parked starts");
                                self.playground_reads.push(crate::HandoverRead {
                                    forge: 0,
                                    run,
                                    path: demo.key_file.clone(),
                                    max: 4097,
                                });
                            } else {
                                self.root(value, events);
                            }
                        }
                        Err(value) => {
                            if let kit::Event::Application(kit::FromApplication::Answered { .. }) = &*value
                                && let Some(carried) = carried
                            {
                                self.down(kit::Event::Application(kit::FromApplication::Unused {
                                    number: carried.number,
                                }));
                            }
                            self.down(*value);
                        }
                    }
                }
                kit::Request::Minted { exchange, credential, expires } => {
                    self.google_minted(exchange, Some((credential, expires)));
                }
                kit::Request::MintRefused { exchange, .. } => self.google_minted(exchange, None),
                kit::Request::Redeemed { exchange, answer } | kit::Request::Claimed { exchange, answer } => {
                    self.provider_answer(exchange, answer);
                }
                kit::Request::Closed => self.component_closed = true,
            }
        }
    }
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        self.mount_env.now = now;
        self.mount_env.wall = wall;
        self.server.clock(now, wall);
    }
    pub fn iterate(
        &mut self,
        now: Time,
        wall: Wall,
        events: &mut Queue<root::Event>,
        forges: &crate::forges::Forges,
        demo: Option<&crate::settings::Demo>,
        accounts: &mut Option<crate::accounts::Accounts>,
    ) {
        self.clock(now, wall);
        if let Some(settings) = self.initial.take() {
            // Link lifetimes start on the supplied clock, which need not start at zero.
            self.out = Queue::uncapped();
            self.component =
                kit::Component::new(settings, &self.env, &mut self.out).expect("validated client settings");
        }
        if self.closed {
            return;
        }
        self.server.fire(&mut self.server_out);
        let server_pending = self.server.advance(&mut self.server_out);
        self.server_requests();
        let admitted = self.admit_waiting();
        self.google_advance(events);
        self.endings_step(events, forges, demo, accounts);
        self.advance(events, forges, demo, accounts);
        if self.closed {
            return;
        }
        if !self.component_closed {
            self.component.fire(&self.env);
        }
        self.advance(events, forges, demo, accounts);
        self.pending = server_pending || admitted || !self.server_out.is_empty();
        let mount_pending = self.mount.up(&self.mount_env, &mut self.mount_out);
        self.pending |= mount_pending;
        if self.component_closed && !mount_pending && self.mount_out.is_empty() && self.out.is_empty() {
            self.server.finish();
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
            self.pending |= self.component.up(&self.env, &mut self.out);
        }

        self.advance(events, forges, demo, accounts);
    }
}
#[must_use]
pub fn owns(token: Token) -> bool {
    token.raw() & (15_u64 << 60) == crate::tokens::SERVER
}
#[must_use]
pub fn tag(request: io::Request) -> io::Request {
    match request.owner() {
        Some(owner) => {
            assert!(owner.raw() < (1_u64 << 60_u32), "bounded client exchange names");
            request.with_owner(Token::new(owner.raw() | crate::tokens::SERVER))
        }
        None => request,
    }
}
#[must_use]
pub fn untag(mut event: io::Event) -> io::Event {
    match &mut event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Window { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => {
            *owner = Token::new(owner.raw() & !(15_u64 << 60));
        }
        io::Event::Resized | io::Event::Shutdown { .. } => {}
    }
    event
}

impl Clients {
    pub fn commit(&mut self, number: u64, writes: &[root::Write]) {
        for write in writes {
            match write {
                root::Write::Erase(key) => {
                    if let Some(sign_in) = ended(key) {
                        self.endings.ended(number, sign_in);
                    }
                }
                root::Write::DropTranscript { .. }
                | root::Write::Wires
                | root::Write::CopySettingsRevision(_)
                | root::Write::Save(_) => {}
            }
        }
    }
    pub fn committed(&mut self, number: u64) {
        self.endings.committed(number);
    }
    #[must_use]
    pub fn commit_room(&self) -> bool {
        self.endings.room()
    }
    fn endings_step(
        &mut self,
        events: &mut Queue<root::Event>,
        forges: &crate::forges::Forges,
        demo: Option<&crate::settings::Demo>,
        accounts: &mut Option<crate::accounts::Accounts>,
    ) {
        // Only the sign-ins retained when this stage starts can end here.
        for _ in 0..self.endings.waiting() {
            let Some(sign_in) = self.endings.next() else { return };
            self.down(kit::Event::SignInEnded { sign_in });
            self.advance(events, forges, demo, accounts);
        }
    }
}

fn ended(key: &root::Key) -> Option<u64> {
    match key {
        root::Key::Core(key) => kit::host::ending(key),
        root::Key::Wires | root::Key::Forge(_) => None,
    }
}

impl Clients {
    pub fn not_ready(&mut self, value: root::PartyInput) {
        let mut unused = None;
        if let root::PartyInput::Key { carried, .. } = &value {
            unused = Some(carried);
        }
        if let root::PartyInput::Application { carried, .. } = &value {
            unused = carried.as_ref();
        }
        let carried = unused;
        if let Some(carried) = carried {
            self.down(kit::Event::Application(kit::FromApplication::Unused { number: carried.number }));
        }
        if let Some(event) = translation::not_ready(value) {
            self.down(event);
        }
    }
}

impl Clients {
    pub fn secrets_step(&mut self, store: &mut crate::store::Store) {
        for _ in 0..store.host.secrets.len() {
            let answer = store.host.secrets.pop().expect("secrets terminals");
            self.secret_answer(answer);
        }
        self.advance_secrets(store);
    }
    fn advance_secrets(&mut self, store: &mut crate::store::Store) {
        for _ in 0..self.playground_carries.len() {
            let request = self.playground_carries.pop().expect("pending demo carries");
            self.secrets.push(request);
        }
        for _ in 0..self.secrets.len() {
            let request = self.secrets.pop().expect("client secrets requests");
            let (token, request) = self.bridge.ask(request);
            if let Some(answer) = store.secrets(token, request) {
                self.secret_answer(answer);
            }
        }
    }
    fn secret_answer(&mut self, answer: jig_protocol_store::Event) {
        let answer = self.bridge.answer(answer);
        if let kit::SecretsAnswer::Carried { exchange, number } = &answer
            && self.owns_playground_read(*exchange)
        {
            self.playground_carried(*exchange, *number);
            return;
        }
        #[cfg(test)]
        if let Some(trace) = &mut self.secret_terminals {
            match &answer {
                kit::SecretsAnswer::Kept { owner }
                | kit::SecretsAnswer::Erased { owner }
                | kit::SecretsAnswer::Got { owner, .. } => trace.push(*owner),
                kit::SecretsAnswer::Carried { .. } | kit::SecretsAnswer::Failed => {}
            }
        }
        self.down(kit::Event::Secrets(answer));
    }
}

fn http_settings(public: Option<&crate::startup::application::Origin>) -> http::Settings {
    let public = match public {
        Some(origin) => Some(origin.name.clone()),
        None => None,
    };
    http::Settings { max_age: 0, prefix: Box::from(b"/jig".as_slice()), public }
}

fn event_owner(event: &io::Event) -> Option<Token> {
    match event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Window { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => Some(*owner),
        io::Event::Resized | io::Event::Shutdown { .. } => None,
    }
}

fn application(
    config: &config::Engine,
    link: Option<people::Identity>,
    served: List<kit::Served>,
) -> config::clients::Application {
    let mut public = List::with_capacity(3);
    for kind in forge_public_kinds() {
        public.push(kind).expect("three public kinds");
    }
    config::clients::Application { link, served, public, moved_bytes: moved_bytes(&config.limits) }
}

pub(crate) fn moved_bytes(limits: &config::Limits) -> u64 {
    jig_protocol_accounts::moved::bound(&limits.driver.client.document)
}

/// The public kinds use the same ceilings as `family::panel`'s generated codec
/// (temper/docs/design/client/forge/README.md, 9.4).
fn forge_public_kinds() -> [kit::public::PublicKind; 3] {
    use temper_client_protocol_wire::{self as wire, v1};
    let bounds = wire::Bounds {
        segments: v1::CEILINGS.landing_pattern_segments,
        segment_bytes: v1::CEILINGS.landing_pattern_segments_item,
        gates: v1::CEILINGS.landing_rule_gates,
        approvals: v1::CEILINGS.landing_rule_approvals,
        field_bytes: v1::CEILINGS.adoption_owner,
        checks: v1::CEILINGS.adopt_checks,
        contexts: v1::CEILINGS.rule_contexts,
        refusal_bytes: v1::CEILINGS.core_refused_refusal,
    };
    let [change, landings, landing] = wire::public_bounds(&bounds);
    let owner = wire::number(wire::Owner::Forge).checked_add(1).expect("family owner");
    [
        kit::public::PublicKind {
            owner,
            kind: wire::PUBLIC_CHANGE,
            first: wire::PUBLIC_VERSION,
            last: wire::PUBLIC_VERSION,
            task: true,
            bound: change,
        },
        kit::public::PublicKind {
            owner,
            kind: wire::PUBLIC_LANDINGS,
            first: wire::PUBLIC_VERSION,
            last: wire::PUBLIC_VERSION,
            task: false,
            bound: landings,
        },
        kit::public::PublicKind {
            owner,
            kind: wire::PUBLIC_LANDING,
            first: wire::PUBLIC_VERSION,
            last: wire::PUBLIC_VERSION,
            task: true,
            bound: landing,
        },
    ]
}
#[cfg(test)]
#[path = "tests/public.rs"]
mod public_tests;
