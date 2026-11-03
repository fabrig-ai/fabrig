//! The tools' root joined to the engine root; only the box link is scripted
//! (client/tools.md, 11.2; domain/testing.md, 6.3).
use jig_client::{self as client, contract as c};
use jig_client_world::{harness::ClientId, joint::transport::Transport};
use jig_domain_people as people;
use skein_lib::{Env, List, Queue, Time, Token, Wall};
use std::collections::{BTreeMap, VecDeque};
use temper_client_tools::{self as tools, root};
use temper_client_world::{
    component::{Composition, Incoming},
    fixtures, translate,
};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

const CLIENT: ClientId = ClientId(0);
const FORGE: u16 = 3;
const OTHER: u16 = 4;
const CONNECTED_MODE: u16 = 5;
const AUTOMATIC: u16 = 6;
const OUTPUTS: u32 = 128;
const STEPS: u32 = 256;
// The forge setup world covers four configured forges and their project allocations.
const FORGE_SETUP_WRITES: u32 = 2126;

fn config(under_box: bool) -> engine::Config {
    let mut config = walking::config(fixtures::SEED);
    config.core.connectors = Box::new([fixtures::FORGE]);
    config.under_box = under_box;
    config.forges = Box::new([
        forge::Configured {
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            git: Box::new([]),
            web: None,
            read_only: false,
            number: FORGE,
            kind: forge::Kind::Forgejo,
            mode: forge::Mode::Owned,
            asked: true,
        },
        forge::Configured {
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            git: Box::new([]),
            web: None,
            read_only: false,
            number: OTHER,
            kind: forge::Kind::Forgejo,
            mode: forge::Mode::Owned,
            asked: true,
        },
        forge::Configured {
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            git: Box::new([]),
            web: None,
            read_only: false,
            number: CONNECTED_MODE,
            kind: forge::Kind::Forgejo,
            mode: forge::Mode::Connected,
            asked: false,
        },
        forge::Configured {
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            git: Box::new([]),
            web: None,
            read_only: false,
            number: AUTOMATIC,
            kind: forge::Kind::Forgejo,
            mode: forge::Mode::Owned,
            asked: false,
        },
    ]);
    let mut limits = temper_engine_domain_world::fixtures::forge_limits();
    temper_client_world::front::first_run_projects(&mut config, &mut limits);
    config
}
struct World {
    root: root::Root,
    env: Env<root::Limits>,
    engine: Driver,
    transport: Transport,
    component: Composition,
    out: Queue<root::Request>,
    printed: BTreeMap<u32, tools::jig::Printed>,
    progress: Vec<Box<[u8]>>,
    box_requests: VecDeque<(u16, Token)>,
    box_calls: u32,
    calls: u32,
    documents: Vec<client::Family>,
    saved: BTreeMap<[u8; 16], client::Held>,
}
impl World {
    fn new(under_box: bool) -> Self {
        let mut limits = temper_engine_domain_world::fixtures::forge_limits();
        limits.authority = temper_engine_domain_world::walking::limits().authority;
        limits.people.projects = 3;
        limits.people.holdings = 3;
        // Four configured forges distinguish both ineligible modes from the two asked ones.
        let extra = 4 - limits.forge.client.repositories;
        limits.forge.client.repositories = 4;
        // The forge client's output cohort counts two outputs per configured forge.
        limits.forge.output += extra * 2;
        // Frame project allocations fit the tools preset, as the shared front requires.
        limits.documents.lines = tools::limits::client().client.domain.projects;
        limits.journal.writes = FORGE_SETUP_WRITES;
        assert!(engine::valid(&limits), "the story profile covers the larger cohort");
        let store = temper_engine_domain_world::fixtures::restored_over(Store::new(), config(under_box), &limits);
        let configuration = config(under_box);
        let mut transport = Transport::new(c::VersionRange {
            first: jig_client_protocol_wire::VERSION,
            last: jig_client_protocol_wire::VERSION,
        });
        transport.models.clone_from(&configuration.core.settings.models);
        transport.providers = configuration.core.settings.llm_providers.clone();
        let component = Composition::new(&configuration, tools::limits::client().client.domain);
        let mut engine = Driver::configured(store, [config(under_box), configuration], &limits);
        engine.settle();

        transport.bind(
            CLIENT,
            fixtures::PARTY,
            people::Identity {
                key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
        );

        engine.delivered.clear();
        engine.immediate.clear();
        let limits = root::Limits::new(tools::limits::client(), tools::limits::tools());
        let root = root::Root::unlinked(&limits, fixtures::SEED).expect("client profile");
        let mut world = Self {
            root,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            engine,
            transport,
            component,
            out: Queue::with_capacity(OUTPUTS),
            printed: BTreeMap::new(),
            progress: Vec::new(),
            box_requests: VecDeque::new(),
            box_calls: 0,
            calls: 0,
            documents: Vec::new(),
            saved: BTreeMap::new(),
        };
        for forge in [FORGE, OTHER, AUTOMATIC] {
            world.connection(forge, forge::Connection::NotSetUp);
        }
        world.component.sign_in(
            CLIENT,
            people::Identity {
                key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
        );
        world.settle();
        world.client(client::Event::StartUnlinked { kept: List::with_capacity(0) });
        world.client(client::Event::SignIn { with: client::With::Transport });
        world.client(client::Event::Up { up: world.transport.hello() });
        world.settle();
        world
    }
    fn client(&mut self, event: client::Event) {
        root::step(&mut self.root, &self.env, root::Event::Client { event }, &mut self.out);
    }
    fn call(&mut self, words: &[&[u8]]) -> u32 {
        let tools::jig::call::Parsed::Call { call, json } = tools::table::parse(words).expect("story call") else {
            panic!("call")
        };
        self.calls += 1;
        root::step(
            &mut self.root,
            &self.env,
            root::Event::Call { number: self.calls, call, json, value: None },
            &mut self.out,
        );
        self.settle();
        self.calls
    }
    fn answer(&mut self, number: u32) -> tools::jig::Printed {
        self.printed.remove(&number).expect("answered call")
    }
    fn invoke(&mut self, words: &[&[u8]]) -> tools::jig::Printed {
        let call = self.call(words);
        self.answer(call)
    }
    fn connection(&mut self, forge: u16, state: forge::Connection) {
        self.engine.send(engine::Event::Forge(Box::new(forge::Event::Connection { forge, state })));
        self.settle();
    }
    fn box_started(&mut self) {
        let (forge, request) = self.box_requests.pop_front().expect("root asked the box");
        self.connection(forge, forge::Connection::SettingUp { step: None });
        self.engine.send(engine::Event::Forge(Box::new(forge::Event::SetUpAnswered {
            forge,
            request,
            answer: forge::SetUpAnswer::SettingUp,
        })));
        self.settle();
    }
    fn callback(&mut self, event: temper_client_domain::Event) {
        let temper_client_domain::Event::Client { event } = event else { panic!("client callback") };
        if let client::Event::Up { up: client::Up::FamilyDelivered { record, .. } } = &event {
            self.documents.push(record.clone());
        }
        self.client(event);
    }
    fn packet(&mut self, packet: Incoming) {
        let Incoming::Callback { client, event } = packet else {
            assert!(matches!(packet, Incoming::Authenticated { .. }));
            return;
        };
        assert_eq!(client, CLIENT);
        self.callback(translate::callback(event));
    }
    fn incoming(&mut self, request: engine::Request) {
        self.component.incoming(&mut self.transport, request);
    }
    fn settle(&mut self) {
        for _ in 0..STEPS {
            let mut moved = false;
            while let Some(request) = self.out.pop() {
                moved = true;
                match request {
                    root::Request::Client {
                        request: client::Request::Down { down: client::Down::Save { record } },
                    } => {
                        self.saved.insert(record.key, record.request);
                    }
                    root::Request::Client { request: client::Request::Down { down: client::Down::Erase { key } } } => {
                        self.saved.remove(&key);
                    }
                    root::Request::Client { request: request @ client::Request::Down { .. } } => {
                        let client::Request::Down { down } = request else { unreachable!("down request") };
                        self.component.send(CLIENT, down);
                    }
                    root::Request::Printed { number, printed } => {
                        assert!(self.printed.insert(number, printed).is_none(), "one answer");
                    }
                    root::Request::Record { .. } => unreachable!("forge tests do not move accounts"),
                    root::Request::Ready => {}
                    root::Request::Progress { line } => self.progress.push(line),
                    root::Request::Client { request } => panic!("unlinked transport: {request:?}"),
                    root::Request::Kept { .. } | root::Request::Fact { .. } => {
                        panic!("no restored calls or unknown owners")
                    }
                }
            }
            let packets = self.component.pump(&mut self.transport);
            let events = self.component.effects();
            moved |= !packets.is_empty() || !events.is_empty();
            for event in events {
                self.engine.send(event);
            }
            self.engine.settle();
            let refused: Vec<_> = self
                .engine
                .immediate
                .iter()
                .filter_map(|request| {
                    if let engine::Request::Party(engine::PartyRequest::WatchRefused { watcher, .. }) = request {
                        Some(*watcher)
                    } else {
                        None
                    }
                })
                .collect();
            self.component.admitted(&refused);
            for packet in packets {
                self.packet(packet);
            }
            let delivered = std::mem::take(&mut self.engine.delivered);
            moved |= !delivered.is_empty();
            for delivery in delivered {
                self.incoming(engine::Request::Party(engine::PartyRequest::Deliver(delivery)));
            }
            let immediate = std::mem::take(&mut self.engine.immediate);
            moved |= !immediate.is_empty();
            for request in immediate {
                if let engine::Request::SetUp { forge, request, .. } = request {
                    self.box_calls += 1;
                    self.box_requests.push_back((forge, request));
                } else {
                    self.incoming(request);
                }
            }
            if !moved {
                return;
            }
        }
        panic!("joined roots did not settle");
    }
    fn setup(&mut self) -> u32 {
        self.call(&[b"forge", b"setup", b"3"])
    }
    fn started(&mut self) {
        let call = self.setup();
        assert!(!self.printed.contains_key(&call));
        self.box_started();
        assert!(!self.printed.contains_key(&call), "setup waits for its first repository");
        assert_eq!(self.progress.last().expect("setup progress").as_ref(), b"setting up\n");
    }
    fn read(&mut self) -> tools::jig::Printed {
        self.invoke(&[b"forge", b"list", b"--json"])
    }
    fn last_rows(&self) -> Box<[temper_client_domain::forge::forges::Row]> {
        temper_client_domain::forge::forges::rows(
            self.documents.last().expect("forges delivery"),
            &self.env.limits.domain.forge,
        )
        .expect("domain rows")
    }
}
fn contains(bytes: &[u8], words: &[u8]) -> bool {
    skein_lib::bytes::find(bytes, words).is_some()
}
fn row(world: &World, forge: u16) -> temper_client_domain::forge::forges::Row {
    world.last_rows().into_iter().find(|row| row.number == forge).expect("configured row")
}

#[test]
fn under_the_box_setup_is_followed_through_each_step_to_settled() {
    use temper_client_domain::forge::forges as d;
    let mut world = World::new(true);
    let read = world.read();
    assert_eq!(read.status, 0);
    let initial = row(&world, FORGE);
    assert_eq!(
        (initial.mode, initial.connection, initial.offer),
        (d::Mode::Owned, d::Connection::NotSetUp, d::Offer::Offered)
    );
    world.started();
    let wait = world.call(&[b"forge", b"list", b"--until", b"settled"]);
    assert!(!world.printed.contains_key(&wait));
    assert_eq!(row(&world, FORGE).connection, d::Connection::SettingUp { step: None });
    for (step, expected) in [(forge::SetUpStep::Prepare, d::Step::Prepare), (forge::SetUpStep::Run, d::Step::Run)] {
        world.connection(FORGE, forge::Connection::SettingUp { step: Some(step) });
        assert!(!world.printed.contains_key(&wait));
        assert_eq!(row(&world, FORGE).connection, d::Connection::SettingUp { step: Some(expected) });
    }
    world.connection(FORGE, forge::Connection::NotConnected);
    let printed = world.answer(wait);
    assert_eq!(printed.status, 0);
    assert!(contains(&printed.stdout, b"Not connected"));
    assert!(printed.stdout.ends_with(b"met\n"));
    assert_eq!(world.box_calls, 1);
    assert!(world.saved.is_empty());
}
#[test]
fn without_the_box_the_row_and_setup_refusal_say_to_start_temper_with_the_box() {
    use temper_client_domain::forge::{forges::Offer, words};
    let mut world = World::new(false);
    world.read();
    assert_eq!(row(&world, FORGE).offer, Offer::NotUnderTheBox);
    let printed = world.invoke(&[b"forge", b"list"]);
    assert!(contains(&printed.stdout, b"Not set up: start temper with temper box"));
    assert!(!contains(&printed.stdout, b"temper forge fetch"));
    let printed = world.invoke(&[b"forge", b"setup", b"3"]);
    assert_eq!(printed.status, 2);
    assert!(contains(&printed.stderr, b"refused: not under the box"));
    assert!(contains(&printed.stderr, words::UNDER_THE_BOX));
    assert!(!contains(&printed.stderr, b"temper forge fetch"));
    assert_eq!(world.box_calls, 0);
}
#[test]
fn a_forge_not_owned_or_not_asked_does_not_offer_setup() {
    let mut world = World::new(true);
    for number in [b"5".as_slice(), b"6", b"9"] {
        let printed = world.invoke(&[b"forge", b"setup", number]);
        assert_eq!((printed.status, printed.stderr.as_ref()), (2, b"refused: not offered\n".as_slice()));
    }
    assert_eq!(world.box_calls, 0);
    assert!(world.saved.is_empty());
}

#[test]
fn a_second_setup_of_the_same_or_another_forge_is_busy() {
    let mut world = World::new(true);
    world.started();
    for number in [b"3".as_slice(), b"4"] {
        let printed = world.invoke(&[b"forge", b"setup", number]);
        assert_eq!((printed.status, printed.stderr.as_ref()), (2, b"refused: busy\n".as_slice()));
    }
    assert_eq!(world.box_calls, 1);
    assert!(world.box_requests.is_empty());
    assert!(world.saved.is_empty());
}
#[test]
fn a_closed_box_pipe_fails_the_setup_and_withdraws_its_offer() {
    use temper_client_domain::forge::forges as d;
    let mut world = World::new(true);
    world.started();
    let wait = world.call(&[b"forge", b"list", b"--until", b"settled"]);
    assert!(!world.printed.contains_key(&wait));
    world.engine.send(engine::Event::Forge(Box::new(forge::Event::BoxClosed)));
    world.settle();
    let printed = world.answer(wait);
    assert_eq!(printed.status, 0);
    assert!(contains(&printed.stdout, b"Set up failed: the box closed the pipe"));
    let failed = row(&world, FORGE);
    assert_eq!(
        failed.connection,
        d::Connection::SetUpFailed { why: b"the box closed the pipe".as_slice().into(), at: None }
    );
    assert_eq!(failed.offer, d::Offer::NotUnderTheBox);
    let printed = world.invoke(&[b"forge", b"setup", b"3"]);
    assert_eq!(printed.status, 2);
    assert!(contains(&printed.stderr, b"refused: not under the box"));
    assert_eq!(world.box_calls, 1);
}
#[test]
fn a_failed_setup_settles_with_its_why_and_can_be_asked_again() {
    use temper_client_domain::forge::forges as d;
    let mut world = World::new(true);
    world.started();
    let wait = world.call(&[b"forge", b"list", b"--until", b"settled"]);
    world.connection(FORGE, forge::Connection::SetUpFailed { why: b"runner did not start".as_slice().into() });
    let printed = world.answer(wait);
    assert_eq!(printed.status, 0);
    assert!(contains(&printed.stdout, b"Set up failed: runner did not start"));
    let failed = row(&world, FORGE);
    assert_eq!(
        failed.connection,
        d::Connection::SetUpFailed { why: b"runner did not start".as_slice().into(), at: None }
    );
    assert_eq!(failed.offer, d::Offer::Offered);
    world.started();
    assert_eq!(world.box_calls, 2);
    assert!(world.saved.is_empty());
    world.connection(FORGE, forge::Connection::NotConnected);
}

#[test]
fn setup_fields_reach_the_engine_and_unknown_project_names_are_admitted() {
    use temper_client_domain::forge::forges as d;
    let mut world = World::new(true);
    let call =
        world.call(&[b"forge", b"setup", b"3", b"--org", b"team", b"--repository", b"widget", b"--project", b"1"]);
    assert!(!world.printed.contains_key(&call));
    world.box_started();
    assert_eq!(world.progress.last().expect("setup progress").as_ref(), b"setting up\n");
    assert!(!world.printed.contains_key(&call), "admitted setup waits for made");
    world.read();
    assert_eq!(
        row(&world, FORGE).first,
        Some(d::First {
            playground: false,
            org: b"team".as_slice().into(),
            repository: b"widget".as_slice().into(),
            state: d::FirstState::Making,
            label: Box::from(b"temper-host".as_slice()),
        })
    );
    world.engine.send(engine::Event::Forge(Box::new(forge::Event::BoxClosed)));
    world.settle();
    let failed = world.answer(call);
    assert_eq!(failed.status, 1);
    assert!(contains(&failed.stderr, b"Set up failed: the box closed the pipe"));
    let mut world = World::new(true);
    let next = world.call(&[b"forge", b"setup", b"3", b"--project", b"Next"]);
    assert!(!world.printed.contains_key(&next));
    assert_eq!(world.box_calls, 1, "new project is admitted to the box");
    world.read();
    assert_eq!(row(&world, FORGE).first.as_ref().expect("first repository").repository.as_ref(), b"hello");
}
#[test]
fn invalid_setup_names_are_refused_without_a_box_effect() {
    let mut world = World::new(true);
    for flag in [b"--org".as_slice(), b"--repository"] {
        let printed = world.invoke(&[b"forge", b"setup", b"3", flag, b""]);
        assert_eq!(printed.status, 2);
        assert!(contains(
            &printed.stderr,
            b"invalid: the organization, repository or project name is empty or too long"
        ));
    }
    assert_eq!(world.box_calls, 0);
}

#[test]
fn a_fresh_deployment_keeps_a_github_row_only_when_connected_and_removes_it_when_confirmed() {
    use super::connect_forge::{node, view};
    use jig_client_world::harness::App;
    use skein_ui::{Act, Role};
    use temper_client_domain::{self as client, Address};
    use temper_client_world::{app::Temper, front::account_acts as accounts};
    let (mut front, mut root) = accounts::fresh_fixture(temper_client_world::fixtures::SEED);
    accounts::event(&mut front, &mut root, Temper::went(Address::Forges));
    assert!(matches!(root.person().forges(), client::forge::forges::State::Rows(rows) if rows.is_empty()));
    accounts::event(&mut front, &mut root, Temper::went(Address::ConnectForge { forge: None }));
    let mut rendered = view(&root);
    let id = node(&rendered, Role::Button, b"Connect GitHub");
    let action = temper_client_view::act(&mut rendered, id, Act::Press).expect("connect GitHub");
    accounts::event(&mut front, &mut root, client::Event::Act { action }.into());
    accounts::event(&mut front, &mut root, Temper::went(Address::Forges));
    let client::forge::forges::State::Rows(rows) = root.person().forges() else { panic!("forge records") };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, client::forge::forges::Kind::GitHub);
    for confirmed in [false, true] {
        let mut rendered = view(&root);
        let id = node(&rendered, Role::Button, b"Remove");
        let action = temper_client_view::act(&mut rendered, id, Act::Press).expect("remove control");
        accounts::event(&mut front, &mut root, client::Event::Act { action }.into());
        let client::forge::forges::State::Rows(rows) = root.person().forges() else { panic!("forge records") };
        assert_eq!(rows.is_empty(), confirmed, "the first press only opens confirmation");
    }
}
