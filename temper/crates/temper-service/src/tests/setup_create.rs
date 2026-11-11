//! The service's local setup sequence against the shared fake forge's records.
use super::*;
use jig_domain_people as people;
use skein_fake_forge_domain as git;
use skein_fake_forge_forgejo as fake;
use skein_forge_kit::{
    fixtures as forge_fixtures,
    pipe::{Cut, Pipe},
};
use skein_lib::ReplyTo;
use skein_lib::stream::Down;

pub(super) struct Forge {
    pub(super) service: fake::Service,
    env: Env<fake::Limits>,
    pub(super) git: git::Domain,
    pub(super) config: git::Config,
    pub(super) socket: Option<(Token, bool)>,
    server: Option<fake::Server>,
    only_setup: bool,
    pipe: Pipe,
    events: Queue<fake::Event>,
    down: Queue<Down>,
    pub(super) requests: List<Box<[u8]>>,
}
impl Forge {
    pub(super) fn new(refuse_file: bool) -> Self {
        Self::with_sources(refuse_file, Box::new([]))
    }
    pub(super) fn with_sources(refuse_file: bool, sources: Box<[fake::Source]>) -> Self {
        let limits = forge_fixtures::fake_limits();
        let config = fake::Config {
            hooks: None,
            base: b"http://127.0.0.1:9".as_slice().into(),
            tokens: List::with_capacity(0),
            users: List::with_capacity(0),
            repositories: List::with_capacity(0),
            max_response_items: 32,
            default_paging_num: 30,
            sources,
            faults: fake::Faults { reset: 0, drop: 0, server_error: 0 },
        };
        let mut service = fake::Service::new(config, &limits, 7).unwrap();
        let store = service.setup_mut();
        let admin = store.account(b"temper-admin", true, b"");
        store.token(
            admin,
            b"handover",
            &[b"write:admin", b"read:user", b"write:organization", b"write:repository"],
            Secret::new(b"test-admin".as_slice().into()),
        );
        store.account(b"owner", true, b"");
        let bot = store.account(b"temper", false, b"temper:bot");
        store.token(
            bot,
            b"temper-api",
            &[b"read:user", b"write:repository"],
            Secret::new(b"test-api".as_slice().into()),
        );
        store.token(
            bot,
            b"temper-git",
            &[b"read:user", b"write:repository"],
            Secret::new(b"test-git".as_slice().into()),
        );
        let mut config = forge_fixtures::fake_config();
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        config.limits.repositories = 4;
        config.limits.name_bytes = 512;
        config.limits.content_bytes = if refuse_file { 128 } else { 4096 };
        Self {
            service,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            git: git::Domain::new(&config, 7),
            config,
            socket: None,
            server: None,
            only_setup: false,
            pipe: Pipe::new(7, Cut::Never),
            events: Queue::with_capacity(8),
            down: Queue::with_capacity(32),
            requests: List::with_capacity(4096),
        }
    }
    fn permit_file(&mut self) {
        self.config.limits.content_bytes = 4096;
    }
    fn turn(&mut self, service: &mut crate::Service) {
        while service.engine.as_mut().unwrap().notes.pop().is_some() {}
        self.env.now = self.env.now.checked_add(Duration::from_millis(1)).unwrap();
        let engine = service.engine.as_mut().unwrap();
        engine.env.now = self.env.now;
        advance(engine);
        engine.forge.iterate(self.env.now, Wall::EPOCH);
        self.turn_io(engine);
        self.turn_server(engine);
        for _ in 0_u32..4 {
            root_turn(engine);
        }
        self.service.reclaim();
    }
    fn record_requests(&mut self) {
        for line in self.pipe.request_lines() {
            self.requests.push(Box::from(line.as_slice())).unwrap();
        }
    }
    pub(super) fn turn_io(&mut self, engine: &mut crate::engine::Engine) {
        for setup in [true, false] {
            if self.only_setup && !setup {
                continue;
            }
            let io = if setup { &mut engine.setup.io } else { &mut engine.forge.io };
            let mut pending = List::with_capacity(32);
            while let Some(request) = io.pop() {
                pending.push(request).unwrap();
            }
            for request in pending.into_boxed() {
                match request {
                    skein_io::Request::Connect { owner, .. } => {
                        assert!(self.socket.is_none());
                        self.socket = Some((owner, setup));
                        self.pipe = Pipe::new(7, Cut::Never);
                        let mut server = fake::Server::new(owner, &self.env.limits).unwrap();
                        fake::start(&mut server, &mut self.service, &self.env, &mut self.events, &mut self.down);
                        self.server = Some(server);
                        for event in [
                            skein_io::Event::Connecting { owner, socket: Token::new(123) },
                            skein_io::Event::Connected { owner },
                        ] {
                            if setup {
                                engine.setup.up(event);
                            } else {
                                engine.forge.up(event);
                            }
                        }
                    }
                    skein_io::Request::Stream { down, .. } => self.pipe.client_down(down),
                    skein_io::Request::Close { .. } | skein_io::Request::Abort { .. } => {
                        let (owner, _) = self.socket.unwrap();
                        self.record_requests();
                        self.socket = None;
                        self.server = None;
                        // Queued server outputs belong to the closed connection.
                        while self.down.pop().is_some() {}
                        while self.events.pop().is_some() {}
                        if setup {
                            engine.setup.up(skein_io::Event::Closed { owner });
                        } else {
                            engine.forge.up(skein_io::Event::Closed { owner });
                        }
                    }
                    skein_io::Request::Listen { .. }
                    | skein_io::Request::Bind { .. }
                    | skein_io::Request::Reject { .. }
                    | skein_io::Request::Output { .. }
                    | skein_io::Request::Spawn { .. }
                    | skein_io::Request::SpawnAlone { .. }
                    | skein_io::Request::Signal { .. }
                    | skein_io::Request::Usage { .. }
                    | skein_io::Request::Window { .. } => panic!("unexpected forge io request"),
                }
            }
        }
    }
    pub(super) fn turn_server(&mut self, engine: &mut crate::engine::Engine) {
        if let Some(server) = &mut self.server {
            fake::resume_with_domain(
                server,
                &mut self.service,
                fake::Hosted { domain: &mut self.git, config: &self.config },
                &self.env,
                &mut self.events,
                &mut self.down,
            );
            let _head = server.take_head();
            while let Some(down) = self.down.pop() {
                self.pipe.server_down(down);
            }
            if let Some(up) = self.pipe.server_up() {
                fake::up_with_domain(
                    server,
                    &mut self.service,
                    fake::Hosted { domain: &mut self.git, config: &self.config },
                    &self.env,
                    up,
                    &mut self.events,
                    &mut self.down,
                );
            }
            let _head = server.take_head();
            if let Some(up) = self.pipe.client_up() {
                let (owner, setup) = self.socket.unwrap();
                if setup {
                    engine.setup.up(skein_io::Event::Stream { owner, up });
                } else {
                    engine.forge.up(skein_io::Event::Stream { owner, up });
                }
            }
            while let Some(event) = self.events.pop() {
                match event {
                    fake::Event::Domain(event) => {
                        let mut replies = Queue::uncapped();
                        let env = Env { now: self.env.now, wall: Wall::EPOCH, limits: self.config };
                        git::step(&mut self.git, &env, event, &mut replies);
                        git::fire(&mut self.git, &env, &mut replies);
                        while let Some(reply) = replies.pop() {
                            if let git::Request::Reply { .. } = reply {
                                fake::down(
                                    server,
                                    &mut self.service,
                                    &self.env,
                                    reply,
                                    &mut self.events,
                                    &mut self.down,
                                );
                            }
                        }
                        let _head = server.take_head();
                        self.git.reclaim();
                    }
                    fake::Event::Close => {
                        fake::closed(server, &mut self.service, &self.env, &mut self.events, &mut self.down);
                    }
                    fake::Event::Closed => {}
                }
            }
        }
    }
    fn settle(&mut self, service: &mut crate::Service) {
        for _ in 0_u32..20_000 {
            self.turn(service);
            let engine = service.engine.as_ref().unwrap();
            if engine.setup.runs.is_empty() && connected(engine) {
                self.record_requests();
                return;
            }
            if failed(engine) && engine.setup.runs.is_empty() && self.socket.is_none() {
                return;
            }
        }
        panic!("local setup did not settle: {:?}", service.engine.as_ref().unwrap().domain.forge_connection(1));
    }
}
fn keep_access(store: &mut crate::store::Store, forge: &Forge) {
    for (item, bytes) in [
        (Item::AdminToken, b"test-admin".as_slice()),
        (Item::ApiToken, b"test-api"),
        (Item::GitToken, b"test-git"),
        (Item::WebhookSecret, b"test-hook"),
    ] {
        kept_record(store, 1, item, bytes);
    }
    kept_record(store, 1, Item::User, &forge.service.setup().find(b"temper").unwrap().id.to_le_bytes());
}
fn configured(forge: &Forge) -> crate::Service {
    let mut config = local_config(true, false);
    config.setup[0].entry.asked = true;
    config.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7101).into()) });
    let mut store = fixtures::records_store();
    keep_access(&mut store, forge);
    let endpoints = fixtures::configured_endpoints(&config, fixtures::forge_runtime()).unwrap();
    let mut service =
        fixtures::configured_prepared(config, &fixtures::seeds(), None, endpoints, None, Some(256)).unwrap();
    fixtures::replace_store(&mut service, store);
    service.engine.as_mut().unwrap().box_link = crate::route::box_link::Link::new(256, true);
    service.engine.as_mut().unwrap().setup.lifecycle.local_mut(1).unwrap().pending = false;
    // Bind is covered independently by setup_bound_webhook; use the checked
    // configured listener here so the fake socket loop owns only forge traffic.
    service.engine.as_mut().unwrap().setup.waiting_for_webhooks = false;
    while service.engine.as_mut().unwrap().forge.io.pop().is_some() {}
    restore_local(&mut service);
    service
}
fn ask(service: &mut crate::Service, number: u64) {
    ask_copy(service, number, false);
}
fn ask_copy(service: &mut crate::Service, number: u64, playground: bool) {
    ask_first(
        service,
        number,
        domain::SetUpFirst::Empty {
            playground,
            org: if playground { b"playground".as_slice().into() } else { b"work".as_slice().into() },
            repository: if playground { b"widget".as_slice().into() } else { b"hello".as_slice().into() },
        },
    );
}
fn ask_first(service: &mut crate::Service, number: u64, first: domain::SetUpFirst) {
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Forge(Box::new(domain::Event::Connection {
        forge: 1,
        state: domain::Connection::SetUpFailed { why: b"retry".as_slice().into() },
    })));
    engine.step(root::Event::Forge(Box::new(domain::Event::Application {
        role: None,
        carried: None,
        request: Token::new(number),
        token: Token::new(number.checked_add(1).unwrap()),
        retention: Duration::from_secs(60),
        ask: domain::Ask::SetUp { forge: domain::Making::Kept(1), first },
    })));
    // Drive the root's journal seam, including its child continuations.
    for _ in 0_u32..256 {
        root_turn(engine);
        if engine.box_link.write.is_some() {
            break;
        }
    }
    assert!(engine.box_link.write.is_some(), "admitted intent reaches routing: {:?}", engine.domain.forges());
    engine.box_link.write.take().expect("start line");
    let mut out = Queue::with_capacity(3);
    engine.box_link.read(b"started 1\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    read_handover(service);
}
fn read_handover(service: &mut crate::Service) {
    let read = service.take_handover_read().unwrap();
    service.handover_read(1, read.run, Ok(None));
}
#[test]
fn reconcile_then_create_then_connect_makes_the_first_repository_and_bot_workflow() {
    let mut forge = Forge::new(false);
    let mut service = configured(&forge);
    ask(&mut service, 91);
    forge.settle(&mut service);
    let engine = service.engine.as_ref().unwrap();
    assert!(!failed(engine), "{:?}: {:?}", engine.domain.forge_connection(1), forge.requests);
    assert_eq!(engine.domain.forges().rows[0].first.as_ref().unwrap().state, domain::FirstState::Made { id: 1 });
    assert!(connected(engine));
    let reads = &forge.requests;
    let mut admin_reads = List::with_capacity(64);
    let mut creation = None;
    for (at, line) in reads.as_slice().iter().enumerate() {
        if line.starts_with(b"GET /api/v1/user ") {
            admin_reads.push(at).unwrap();
        }
        if creation.is_none() && line.starts_with(b"GET /api/v1/orgs/work ") {
            creation = Some(at);
        }
    }
    let creation = creation.unwrap();
    let admin_reads = admin_reads.as_slice();
    assert!(admin_reads[0] < admin_reads[1] && admin_reads[1] < creation && creation < *admin_reads.last().unwrap());
    assert_eq!(forge.service.setup().organization(b"work").unwrap().owner.as_ref(), b"owner");
    assert!(forge.service.setup().is_private(1));
    assert_eq!(forge.service.setup().collaborator(1, b"temper"), Some(skein_forge::forgejo::request::Grant::Write));
    assert_eq!(forge.service.setup().repository_hooks(1).count(), 1);
    let head = forge.git.branch(b"work/hello", b"main").unwrap();
    let object = forge.git.object(head).unwrap();
    assert_eq!(object.tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"# hello\n");
    assert_eq!(
        object.tree.get(temper_forge_setup::STARTER_PATH).unwrap().as_ref(),
        temper_forge_setup::starter_workflow(b"temper-host").unwrap().as_ref()
    );
    assert!(object.tree.get(temper_forge_setup::PLAYGROUND_PATH).is_none());
    assert_eq!(object.author.as_ref().unwrap().name.as_ref(), b"temper");
    assert_eq!(object.author, object.committer);
}
#[test]
fn no_first_intent_reconciles_and_connects_without_create() {
    let mut forge = Forge::new(false);
    let mut service = configured(&forge);
    lifecycle::started(service.engine.as_mut().unwrap(), 1);
    read_handover(&mut service);
    forge.settle(&mut service);
    assert!(service.engine.as_ref().unwrap().domain.forges().rows[0].first.is_none());
    for line in forge.requests.as_slice() {
        assert!(!line.starts_with(b"GET /api/v1/orgs/") && !line.starts_with(b"POST /api/v1/orgs/"));
    }
    assert!(forge.service.setup().organization(b"work").is_none());
}
#[test]
fn a_file_stop_names_the_step_and_a_second_ask_finishes_once() {
    let mut forge = Forge::new(true);
    let mut service = configured(&forge);
    ask(&mut service, 91);
    forge.settle(&mut service);
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.forge.starts.is_empty());
    assert_eq!(
        engine.domain.forges().rows[0].first.as_ref().unwrap().state,
        domain::FirstState::Failed { step: domain::CreateStep::File }
    );
    assert_eq!(
        engine.domain.forge_connection(1),
        domain::Connection::SetUpFailed { why: b"making work/hello: File (Refused(413))".as_slice().into() }
    );
    let readme = forge.git.branch(b"work/hello", b"main").unwrap();
    assert!(forge.git.object(readme).unwrap().tree.get(temper_forge_setup::STARTER_PATH).is_none());
    forge.permit_file();
    ask(&mut service, 93);
    forge.settle(&mut service);
    assert_eq!(
        service.engine.as_ref().unwrap().domain.forges().rows[0].first.as_ref().unwrap().state,
        domain::FirstState::Made { id: 1 },
        "{:?}",
        service.engine.as_ref().unwrap().domain.forge_connection(1)
    );
    assert!(connected(service.engine.as_ref().unwrap()));
    let head = forge.git.branch(b"work/hello", b"main").unwrap();
    assert_eq!(forge.git.object(head).unwrap().parent, Some(readme));
    assert_eq!(
        forge.git.object(head).unwrap().tree.get(temper_forge_setup::STARTER_PATH).unwrap().as_ref(),
        temper_forge_setup::starter_workflow(b"temper-host").unwrap().as_ref()
    );
}

fn root_turn(engine: &mut crate::engine::Engine) {
    // These asks enter at the setup routing seam; the root's application
    // acknowledgement is exercised separately in the root setup tests.
    let mut events = Queue::with_capacity(engine.events.len());
    while let Some(event) = engine.events.pop() {
        if let root::Event::Forge(forge) = &event
            && let domain::Event::SetUpAnswered { .. } = forge.as_ref()
        {
            continue;
        }
        events.push(event);
    }
    while let Some(event) = events.pop() {
        engine.events.push(event);
    }
    crate::engine::iterate(engine, engine.env.now, Wall::EPOCH);
}

fn connected(engine: &crate::engine::Engine) -> bool {
    match engine.domain.forge_connection(1) {
        domain::Connection::Connected { .. } => true,
        domain::Connection::NotConnected
        | domain::Connection::NotSetUp
        | domain::Connection::SettingUp { .. }
        | domain::Connection::SetUpFailed { .. }
        | domain::Connection::Connecting
        | domain::Connection::Stopped { .. } => false,
    }
}
fn failed(engine: &crate::engine::Engine) -> bool {
    match engine.domain.forge_connection(1) {
        domain::Connection::SetUpFailed { .. } => true,
        domain::Connection::NotConnected
        | domain::Connection::NotSetUp
        | domain::Connection::SettingUp { .. }
        | domain::Connection::Connected { .. }
        | domain::Connection::Connecting
        | domain::Connection::Stopped { .. } => false,
    }
}

#[test]
fn playground_create_copies_the_fake_source_beside_the_starter_and_keeps_forgejos_readme() {
    let mut forge = Forge::new(false);
    let mut service = configured(&forge);
    ask_copy(&mut service, 91, true);
    forge.settle(&mut service);
    let engine = service.engine.as_ref().unwrap();
    assert!(connected(engine), "{:?}", engine.domain.forge_connection(1));
    assert!(engine.domain.forges().rows[0].first.as_ref().unwrap().playground);
    let head = forge.git.branch(b"playground/widget", b"main").unwrap();
    let object = forge.git.object(head).unwrap();
    assert_eq!(object.tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"# widget\n");
    assert_eq!(
        object.tree.get(temper_forge_setup::STARTER_PATH).unwrap().as_ref(),
        temper_forge_setup::starter_workflow(b"temper-host").unwrap().as_ref()
    );
    assert_eq!(
        object.tree.get(temper_forge_setup::PLAYGROUND_PATH).unwrap().as_ref(),
        temper_forge_setup::PLAYGROUND_MAIN
    );
    assert!(object.tree.get(b".forgejo/workflows/check.yml".as_slice()).is_none());
}

#[path = "setup_making.rs"]
mod making_tests;

#[test]
fn deferred_setups_connect_without_create_or_a_first_repository() {
    for (first, starts_with) in [
        (domain::SetUpFirst::Nothing, domain::StartsWith::Nothing),
        (
            domain::SetUpFirst::Copies {
                addresses: Box::new([Box::from(b"https://source.example/owner/repo".as_slice())]),
            },
            domain::StartsWith::Copies {
                addresses: Box::new([Box::from(b"https://source.example/owner/repo".as_slice())]),
            },
        ),
    ] {
        let mut forge = Forge::new(false);
        let mut service = configured(&forge);
        let mut record = runtime_record(domain::Kind::Forgejo, domain::Mode::Owned);
        record.number = 1;
        let engine = service.engine.as_mut().unwrap();
        engine.step(root::Event::Forge(Box::new(domain::Event::Restore { record: domain::Stored::Forge(record) })));
        {
            ask_first(&mut service, 91, first.clone());
            forge.settle(&mut service);
            let engine = service.engine.as_ref().unwrap();
            let row = engine.domain.forges().rows[0].clone();
            assert!(connected(engine));
            assert_eq!(row.first, None);
            assert_eq!(row.starts_with, starts_with);
            assert!(engine.setup.runs.is_empty());
            for line in forge.requests.as_slice() {
                assert!(
                    !line.starts_with(b"POST /api/v1/orgs") && !line.starts_with(b"GET /api/v1/orgs/"),
                    "no Create: {line:?}"
                );
            }
        }
    }
}

fn owner_machine() -> temper_forge_setup::machine::OwnedMachine {
    temper_forge_setup::machine::OwnedMachine {
        sign_in: None,
        listen: ([127, 0, 0, 1], 9).into(),
        remote: b"http://127.0.0.1:9".as_slice().into(),
        person: temper_forge_setup::setup::Account {
            login: b"owner".as_slice().into(),
            email: b"owner@example.test".as_slice().into(),
        },
        public: None,
        fetch: temper_forge_setup::setup::Fetch { forgejo: false, runner: false, mirror: None },
        forgejo: b"/bin/forgejo".as_slice().into(),
        runner: None,
    }
}

fn fresh_owned() -> crate::Service {
    let mut config = fixtures::engine();
    // Adoption carries seven forge options and their resource name.
    config.jig.limits.core.authority.segments = 7;
    config.jig.limits.core.people.amendment_bytes = 512;
    fixtures::fill_files(&mut config);
    let project_owner = config.jig.owners[0].clone();
    let mut owner = project_owner.clone();
    owner.project = 0;
    config.jig.owners = Box::new([owner, project_owner]);
    config.machine.owned = Some(owner_machine());
    let io = config.jig.limits.io;
    let endpoints = crate::forge::prepare(&config, List::with_capacity(0)).unwrap();
    let mut service =
        crate::Service::prepared(config, &fixtures::seeds(), None, endpoints, None, io, Some(256), None).unwrap();
    service.engine.as_mut().unwrap().store = fixtures::memory_store();
    restore_local(&mut service);
    assert!(service.engine.as_ref().unwrap().domain.forges().rows.is_empty());
    service
}

fn owner_reply(service: &mut crate::Service, input: root::PartyInput) -> jig_domain_people::Reply {
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Party(input));
    let mut answer = None;
    for _ in 0_u32..256 {
        while engine.notes.pop().is_some() {}
        while let Some(event) = engine.events.pop() {
            engine.step(event);
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        while let Some(request) = engine.requests.pop() {
            if let root::Request::Party(root::PartyRequest::Deliver(root::Delivery::WebReply { reply, .. })) = request {
                assert!(answer.is_none());
                answer = Some(reply);
            } else {
                crate::route::request(engine, request);
            }
        }
        engine.step(root::Event::Resume);
        engine.store.iterate();
        crate::engine::answer(engine);
        engine.domain.reclaim();
    }
    answer.expect("owner request answered after durability")
}

fn sign_owner(service: &mut crate::Service) {
    let reply = owner_reply(
        service,
        root::PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(91)),
            identity: people::Identity {
                key: people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
                login: b"deployment-owner".as_slice().into(),
                name: b"Owner".as_slice().into(),
            },
        },
    );
    let people::Reply::SignedIn { person, sign_in, .. } = reply else { panic!("sign in: {reply:?}") };
    assert_eq!((person, sign_in), (1, 1));
}

fn start_owned(service: &mut crate::Service, forge: &Forge) {
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Forge(Box::new(domain::Event::Application {
        role: Some(people::Role::Owner),
        carried: None,
        request: Token::new(93),
        token: Token::new(94),
        retention: Duration::from_secs(60),
        ask: domain::Ask::SetUp {
            forge: domain::Making::New(domain::ForgeAsk::Owned),
            first: domain::SetUpFirst::Empty {
                playground: false,
                org: b"work".as_slice().into(),
                repository: b"hello".as_slice().into(),
            },
        },
    })));
    for _ in 0_u32..256 {
        root_turn(engine);
    }
    engine.box_link.write.take().expect("box start");
    let mut out = Queue::with_capacity(3);
    engine.box_link.read(b"started 1\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    keep_access(&mut engine.store, forge);
    let read = service.take_handover_read().expect("owned box handover");
    service.handover_read(1, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
}

fn set_up_owner() -> (Forge, crate::Service) {
    let mut forge = Forge::new(false);
    let mut service = fresh_owned();
    sign_owner(&mut service);
    start_owned(&mut service, &forge);
    for _ in 0_u32..2000 {
        if let Some(remove) = service.take_handover_removal() {
            service.handover_removed(remove.forge, remove.run, Ok(()));
        }
        forge.turn(&mut service);
        let engine = service.engine.as_ref().unwrap();
        if connected(engine) && engine.setup.runs.is_empty() {
            break;
        }
    }
    forge.record_requests();
    let engine = service.engine.as_ref().unwrap();
    assert!(connected(engine), "{:?}", engine.domain.forge_connection(1));
    let mut person_read = false;
    for line in forge.requests.as_slice() {
        person_read |= line.starts_with(b"GET /api/v1/users/owner ");
    }
    assert!(person_read, "reconcile reads the configured forge account");
    assert_owner_link(&mut service, &forge);
    (forge, service)
}

fn assert_owner_link(service: &mut crate::Service, forge: &Forge) {
    let rows = fixtures::load(&mut service.engine.as_mut().unwrap().store, root::Range::Forge);
    let mut link = None;
    for row in &rows {
        if let root::Record::Forge { row } = row
            && let domain::Stored::Forge(record) = row.as_ref()
            && record.number == 1
        {
            link = record.person;
        }
    }
    assert_eq!(
        link,
        Some(domain::Link { person: 1, user: forge.service.setup().find(b"owner").unwrap().id }),
        "setup must durably link the signed-in sole owner before approval eligibility"
    );
}

fn adopt_first(forge: &mut Forge, service: &mut crate::Service) {
    // Hold the connector's idle stream while the setup host probes adoption access.
    let parked =
        (forge.socket.take(), forge.server.take(), core::mem::replace(&mut forge.pipe, Pipe::new(7, Cut::Never)));
    forge.only_setup = true;
    let request = root::Application::Forge(domain::Ask::Adopt(domain::Adoption {
        project: 1,
        home: true,
        forge: 1,
        host: b"127.0.0.1:9".as_slice().into(),
        owner: b"work".as_slice().into(),
        name: b"hello".as_slice().into(),
        prefix: b"temper".as_slice().into(),
        role: domain::Role::Owned,
        landing: b"main".as_slice().into(),
        ci: true,
        checks: Box::new([]),
    }));
    service.engine.as_mut().unwrap().step(root::Event::Party(root::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: 1,
        key: [95; 16],
        digest: [95; 32],
        scope: jig_domain_people::ApplicationScope::Project(1),
        request,
        carried: None,
    }));
    let mut probing = false;
    for _ in 0_u32..2000 {
        forge.turn(service);
        probing |= !service.engine.as_ref().unwrap().setup.runs.is_empty();
        if probing && service.engine.as_ref().unwrap().setup.runs.is_empty() && forge.socket.is_none() {
            break;
        }
    }
    assert!(probing, "adoption probes current access");
    (forge.socket, forge.server, forge.pipe) = parked;
    forge.only_setup = false;
    for _ in 0_u32..2000 {
        forge.turn(service);
        if service.engine.as_ref().unwrap().domain.adopted_ids(1).as_ref() == [1] {
            return;
        }
    }
    panic!("first repository adopted");
}

fn approval_value(people: u32) -> domain::settings::Value {
    use domain::settings::{LandingRule, RuleApproval, RuleGate, Value};
    Value::Rule(LandingRule {
        sets: Box::new([]),
        pattern: domain::GatePattern {
            segments: Box::new([Box::from(b"forge".as_slice())]),
            last: domain::GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: true,
        up_to_date: true,
        gates: Box::new([RuleGate { number: 1, blocking: true, freshness: domain::JudgeFreshness::Clean }]),
        approvals: Box::new([RuleApproval { role: 0, people, freshness: domain::JudgeFreshness::Clean }]),
    })
}

fn propose_approval(service: &mut crate::Service, key: u8, against: u32, people: u32) -> jig_domain_people::Reply {
    use jig_domain_core::settings as s;
    let value = approval_value(people);
    let bytes = temper_engine_smith::kinds::encode(&value);
    owner_reply(
        service,
        root::PartyInput::ProposeSettings {
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: 1,
            key: [key; 16],
            digest: [key; 32],
            change: Box::new(s::Proposed {
                against,
                items: Box::new([s::Item {
                    setting: s::Setting::Application { kind: 1, scope: 0, number: 1 },
                    edit: s::Edit::Set(s::Value::Application(bytes.clone())),
                }]),
                reason: b"Every change gets one owner review; approval survives clean automatic updates."
                    .as_slice()
                    .into(),
            }),
            kinds: Box::new([root::KindValue {
                kind: 1,
                scope: 0,
                number: 1,
                value: domain::settings::SizedValue { value, size: u32::try_from(bytes.len()).unwrap() },
            }]),
        },
    )
}

fn pending_approval(service: &mut crate::Service, key: u8, against: u32, required: u32) -> u64 {
    let reply = propose_approval(service, key, against, required);
    let people::Reply::Outcome(people::Outcome::Proposed { proposal, .. }) = reply else {
        panic!("linked owner can propose approval: {reply:?}");
    };
    assert!(
        service.engine.as_ref().unwrap().domain.tasks().person_proposal(1, proposal).is_some(),
        "pending proposal card"
    );
    proposal
}

fn accept_approval(service: &mut crate::Service, key: u8, proposal: u64) -> people::Reply {
    owner_reply(
        service,
        root::PartyInput::Ask {
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: 1,
            key: [key; 16],
            models: Box::new([]),
            ask: people::Ask::DecideProposal {
                project: 0,
                proposer: people::Proposer::Party(1),
                proposal,
                decision: people::ProposalDecision::Accept,
                running: Some(people::Running::Stays),
            },
        },
    )
}

fn assert_approval_kept(service: &mut crate::Service) {
    let rows = fixtures::load(&mut service.engine.as_mut().unwrap().store, root::Range::Forge);
    for row in &rows {
        if let root::Record::Forge { row } = row
            && let domain::Stored::Setting(setting) = row.as_ref()
            && setting.key.kind == 1
            && setting.key.number == 1
            && setting.key.scope == 0
            && setting.key.slot == domain::settings::SettingSlot::Change
            && let domain::settings::Edit::Set(value) = &setting.edit
        {
            assert_eq!(value.value, approval_value(1));
            return;
        }
    }
    panic!("effective owner approval rule is durable");
}

#[test]
fn a_fresh_owned_box_can_propose_and_accept_owner_approval() {
    let (mut forge, mut service) = set_up_owner();
    adopt_first(&mut forge, &mut service);
    let proposal = pending_approval(&mut service, 96, 1, 1);
    let reply = accept_approval(&mut service, 97, proposal);
    let people::Reply::Outcome(people::Outcome::ProposalDecided { choice, .. }) = reply else {
        panic!("owner accepts: {reply:?}");
    };
    assert_eq!(choice, people::ProposalChoice::Accepted);
    assert_approval_kept(&mut service);
    let proposal = pending_approval(&mut service, 98, 2, 2);
    let reply = accept_approval(&mut service, 99, proposal);
    let people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) = reply else {
        panic!("two people refused: {reply:?}");
    };
    assert!(contains(&why, b"requires 2 distinct known forge people; currently 1 eligible"));
    assert_approval_kept(&mut service);
}
