//! First-repository promises exercised through the person and ctl on the real engine.
//! The kept owned forge follows temper/docs/design/protocol/forge.md, 2.1.
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step};
use skein_lib::Token;
use skein_ui::Role;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{Address, Choice};
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

type ClientWorld = World<Temper, Joint>;
const FORGE: u16 = 1;
const REPOSITORY: u32 = 7;

fn configured(under_box: bool) -> ClientWorld {
    configured_with_makings(under_box, None)
}
fn configured_with_makings(under_box: bool, makings: Option<u32>) -> ClientWorld {
    let mut world = fixtures::world(fixtures::SEED);
    world.randomness(0, 0, 0);
    let mut limits = world.front().engine.env.limits;
    if let Some(makings) = makings {
        limits.forge.makings = makings;
    }
    let [bootstrap, config] = std::array::from_fn(|_| {
        let mut config = walking::config(fixtures::SEED);
        config.forges = Box::new([]);
        config.under_box = under_box;
        config.core.connectors = Box::new([temper_client_protocol_wire::FORGE]);
        temper_client_world::front::first_run_projects(&mut config, &mut limits);
        config
    });
    let mut store = temper_engine_domain_world::fixtures::restored_over(Store::new(), bootstrap, &limits);
    store.seed(
        Box::new([
            engine::Record::Forge {
                row: Box::new(forge::Stored::Forge(forge::ForgeRecord {
                    git_program: None,
                    number: FORGE,
                    kind: forge::Kind::Forgejo,
                    mode: forge::Mode::Owned,
                    state: forge::ForgeState::Kept,
                    place: forge::Place {
                        api: forge::Address { host: fixtures::FORGE_HOST.into(), port: 443, tls: None },
                        git: Box::new([]),
                        web: None,
                    },
                    read_only: false,
                    bot: forge::Bot { login: Box::new([]), name: Box::new([]), email: Box::new([]) },
                    asked: true,
                    first: None,
                })),
            },
            engine::Record::Forge { row: Box::new(forge::Stored::ForgeNumber { last: FORGE }) },
        ]),
        Box::new([]),
    );
    let mut driver = Driver::cold(store, config, &limits);
    driver.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(fixtures::REPOSITORIES));
    driver.hello(Token::new(7));
    driver.settle();
    world.front().engine = driver;
    connection(&mut world, forge::Connection::NotSetUp);
    world
}
fn connection(world: &mut ClientWorld, state: forge::Connection) {
    send(world, forge::Event::Connection { forge: FORGE, state });
}
fn send(world: &mut ClientWorld, event: forge::Event) {
    world.front().engine.send(engine::Event::Forge(Box::new(event)));
}
fn flow() -> Address {
    Address::ConnectForge { forge: Some(Choice::Forge(FORGE)) }
}
fn script(world: &mut ClientWorld, person: ClientId, steps: Vec<Step<Address>>) {
    world.person(person, steps);
    let outcome = world.run(Until::Steps(256));
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
fn see(name: &'static [u8]) -> Step<Address> {
    Step::See { find: Find::text(name), within: fixtures::WITHIN }
}
fn press(name: &'static [u8]) -> Step<Address> {
    Step::Press { find: Find::named(Role::Button, name) }
}
fn fill(name: &'static [u8], words: &[u8]) -> Step<Address> {
    Step::Type { find: Find::named(Role::TextField, name), words: words.into() }
}
fn setup(world: &mut ClientWorld, person: ClientId) {
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextField, b"Organization"), within: fixtures::WITHIN },
            fill(b"Organization", b"acme"),
            fill(b"Repository", b"shop"),
            press(b"Set up a Forgejo here"),
        ],
    );
    wait_for_setup(world);
    acknowledge(world);
}
fn wait_for_setup(world: &mut ClientWorld) {
    for _ in 0..256 {
        if !world.front().set_ups().is_empty() {
            return;
        }
        let _ = world.run(Until::Steps(2));
    }
    panic!("the form did not send SetUp");
}
fn acknowledge(world: &mut ClientWorld) {
    let (forge, request) = *world.front().set_ups().last().expect("admitted setup");
    send(world, forge::Event::SetUpAnswered { forge, request, answer: forge::SetUpAnswer::SettingUp });
    connection(world, forge::Connection::SettingUp { step: None });
}
fn text(world: &ClientWorld, person: ClientId) -> String {
    world
        .tree(person)
        .nodes()
        .iter()
        .flat_map(|node| node.text.iter())
        .map(|run| String::from_utf8_lossy(&run.text))
        .collect::<Vec<_>>()
        .join("\n")
}
fn made(world: &mut ClientWorld) {
    let (forge, request) = *world.front().set_ups().last().expect("admitted setup");
    send(world, forge::Event::SetUpAnswered { forge, request, answer: forge::SetUpAnswer::SettingUp });
    send(world, forge::Event::FirstMade { forge, id: u64::from(REPOSITORY) });
}
fn connected(world: &mut ClientWorld) {
    connection(world, forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() });
}

#[test]
fn empty_organization_is_required_without_starting_the_box() {
    // client/forge.md, 5.4: an empty organization is refused locally, fields kept.
    let mut world = configured(true);
    let person = world.client(fixtures::PARTY, flow());
    script(
        &mut world,
        person,
        vec![
            Step::See { find: Find::named(Role::TextField, b"Organization"), within: fixtures::WITHIN },
            fill(b"Organization", b""),
            press(b"Set up a Forgejo here"),
            see(b"Required."),
        ],
    );
    assert!(world.front().set_ups().is_empty(), "an empty organization must not start the box");
    assert!(
        !world.tree(person).nodes().iter().any(|node| node.role == Role::TextField
            && node.name.as_deref() == Some(b"Into a new project, named".as_slice())),
        "setup asks no project name"
    );
}

#[test]
fn empty_repository_is_required_without_starting_the_box() {
    // client/forge.md, 5.4: an empty repository is refused locally.
    let mut world = configured(true);
    let person = world.client(fixtures::PARTY, flow());
    script(
        &mut world,
        person,
        vec![
            Step::See { find: Find::named(Role::TextField, b"Repository"), within: fixtures::WITHIN },
            fill(b"Repository", b""),
            press(b"Set up a Forgejo here"),
            see(b"Required."),
        ],
    );
    assert!(world.front().set_ups().is_empty(), "an empty repository must not start the box");
}

#[test]
fn without_the_box_the_flow_says_to_start_temper_with_the_box_and_offers_no_setup() {
    // client/forge.md, 5.4: without the box the page says to start temper with the box.
    let mut world = configured(false);
    let person = world.client(fixtures::PARTY, flow());
    script(
        &mut world,
        person,
        vec![
            see(b"Set it up under temper box"),
            Step::Gone { find: Find::named(Role::Button, b"Set up a Forgejo here"), within: fixtures::WITHIN },
        ],
    );
    let page = text(&world, person);
    for expected in [
        "temper was not started by temper box, so it does not start a Forgejo itself.",
        "Start temper with temper box to set one up here.",
    ] {
        assert!(page.contains(expected), "missing {expected}: {page}");
    }
    assert!(!page.contains("temper forge fetch"));
}

#[test]
fn setup_leaves_the_first_repository_unadopted_even_after_connection() {
    // client/tools.md, 5.5; client/forge.md, 5.4: wait for made AND connected.
    let mut world = configured(true);
    let person = world.client(fixtures::PARTY, flow());
    setup(&mut world, person);
    made(&mut world);
    connection(&mut world, forge::Connection::Connecting);
    let outcome = world.run(Until::Steps(256));
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(!world.front().engine_adopted(FORGE, REPOSITORY), "made alone must not adopt before the forge connects");
    connected(&mut world);
    script(&mut world, person, vec![see(b"Connected Forgejo on this machine.")]);
    assert!(!world.front().engine_adopted(FORGE, REPOSITORY), "the asking client leaves the repository unadopted");
}

#[test]
fn a_second_tab_that_never_asked_does_not_adopt_the_made_repository() {
    // client/forge.md, 5.4: the observer adopts nothing; ctl completes its own call.
    let mut world = configured(true);
    let agent = fixtures::agent(&mut world);
    let person = world.client(fixtures::PARTY, flow());
    let number =
        fixtures::give(&mut world, agent, &[b"forge", b"setup", b"1", b"--org", b"acme", b"--repository", b"shop"]);
    wait_for_setup(&mut world);
    made(&mut world);
    connected(&mut world);
    let answer = fixtures::answered(&mut world, agent, number);
    assert_eq!(answer.status, 0, "{answer:?}");
    script(&mut world, person, vec![see(b"Connected Forgejo on this machine.")]);
    assert_eq!(
        world.front().applications_admitted(),
        2,
        "one SetUp and ctl's one Adopt; the observer tab must not adopt"
    );
}

#[test]
fn leaving_the_flow_keeps_the_made_repository_unadopted() {
    // client/forge.md, 5.4: setup adopts nothing on any page.
    let mut world = configured(true);
    let person = world.client(fixtures::PARTY, flow());
    setup(&mut world, person);
    script(&mut world, person, vec![Step::Go { address: Address::Chats { filter: ChatsFilter::All } }]);
    made(&mut world);
    connected(&mut world);
    let _ = world.run(Until::Steps(256));
    assert!(!world.front().engine_adopted(FORGE, REPOSITORY), "leaving the flow adopts nothing");
    let count = world.front().applications_admitted();
    send(&mut world, forge::Event::FirstMade { forge: FORGE, id: u64::from(REPOSITORY) });
    script(&mut world, person, vec![Step::Go { address: flow() }, see(b"Connected Forgejo on this machine.")]);
    assert_eq!(
        world.front().applications_admitted(),
        count,
        "duplicate row updates and returning to Forges must not send another Adopt"
    );
}

#[test]
fn every_create_failure_offers_setup_again_with_the_same_names() {
    // protocol/setup.md, 6.7; client/forge.md, 5.4: failed(step) retries the last fields.
    for (step, why) in [
        (forge::CreateStep::Admin, b"checking the admin's token".as_slice()),
        (forge::CreateStep::Org, b"making the organization".as_slice()),
        (forge::CreateStep::Repository, b"making the repository".as_slice()),
        (forge::CreateStep::File, b"adding the starter workflow".as_slice()),
    ] {
        let mut world = configured(true);
        let person = world.client(fixtures::PARTY, flow());
        setup(&mut world, person);
        send(&mut world, forge::Event::FirstFailed { forge: FORGE, step });
        connection(&mut world, forge::Connection::SetUpFailed { why: why.into() });
        script(&mut world, person, vec![see(b"Making acme/shop failed")]);
        let page = text(&world, person);
        assert!(
            page.contains(std::str::from_utf8(why).expect("failure words")),
            "flow must name the failed step: {page}"
        );
        world.person(person, vec![press(b"Try again")]);
        for _ in 0..256 {
            if world.front().set_ups().len() == 2 {
                break;
            }
            let _ = world.run(Until::Steps(2));
        }
        assert_eq!(world.front().set_ups().len(), 2, "retry sends exactly one new setup at {step:?}");
        acknowledge(&mut world);
        script(&mut world, person, vec![see(b"acme/shop")]);
    }
}

#[test]
fn the_made_repository_waits_until_the_person_adopts_it_from_the_form() {
    let mut world = configured(true);
    let person = world.client(fixtures::PARTY, flow());
    setup(&mut world, person);
    made(&mut world);
    connected(&mut world);
    script(&mut world, person, vec![see(b"acme/shop")]);
    assert!(!world.front().engine_adopted(FORGE, REPOSITORY));
    script(&mut world, person, fixtures::adopt_and_see_outcome(b"acme", b"shop"));
    assert!(world.front().engine_adopted(FORGE, REPOSITORY));
}

fn copy_act(
    front: &mut Joint,
    root: &mut temper_client_world::app::Root,
    action: temper_client_domain::Action,
) -> Vec<jig_client::contract::Ask> {
    temper_client_world::front::account_acts::event_traced(
        front,
        root,
        temper_client_world::app::Event::Root(temper_client_domain::Event::Act { action }),
    )
}
#[test]
fn ready_copies_reach_the_engines_making_rows_in_order() {
    use jig_client_kit::harness::App;
    use temper_client_domain::{Action, Step, forge as client_forge, pages};
    use temper_client_world::front::account_acts as h;
    let (mut front, mut root) = h::fresh_fixture(fixtures::SEED);
    h::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Forge { forge: None } }));
    copy_act(
        &mut front,
        &mut root,
        Action::Forge(client_forge::Act::Forges(client_forge::forges::Act::Start(client_forge::forges::Start::Copies))),
    );
    copy_act(
        &mut front,
        &mut root,
        Action::Forge(client_forge::Act::CopiesWrite {
            text: b"github.com/acme/widget\nsource.invalid/team/tool".as_slice().into(),
        }),
    );
    copy_act(&mut front, &mut root, Action::Forge(client_forge::Act::Forges(client_forge::forges::Act::RunOwned)));
    let (forge, request) = *front.set_ups().last().expect("copy setup admitted");
    front.engine.send(engine::Event::Forge(Box::new(forge::Event::SetUpAnswered {
        forge,
        request,
        answer: forge::SetUpAnswer::SettingUp,
    })));
    front.engine.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge,
        state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    front.engine.settle();
    h::settle(&mut front, &mut root);
    h::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Ready }));
    front.engine.transactions.clear();
    let sent = copy_act(&mut front, &mut root, Action::Pages(pages::Act::MakeDefault));
    assert_eq!(
        sent,
        vec![jig_client::contract::Ask::MakeProject {
            name: jig_client::contract::Text::new(b"Widget".as_slice().into(), 64).unwrap(),
            policy: 1
        }]
    );
    let Address::Home { project } = *root.person().pages().address() else { panic!("project opens while copies wait") };
    let expected = [
        (b"acme".as_slice(), b"widget".as_slice(), b"https://github.com/acme/widget".as_slice(), true),
        (b"team".as_slice(), b"tool".as_slice(), b"https://source.invalid/team/tool".as_slice(), false),
    ];
    let mut order = Vec::new();
    for transaction in &front.engine.transactions {
        for write in transaction {
            if let engine::Write::Save(engine::Record::Forge { row }) = write
                && let forge::Stored::Making(row) = row.as_ref()
                && !order.contains(&row.key)
            {
                order.push(row.key.clone());
            }
        }
    }
    assert_eq!(
        order,
        expected
            .iter()
            .map(|(org, name, _, _)| forge::MakingKey { forge: FORGE, org: (*org).into(), name: (*name).into() })
            .collect::<Vec<_>>()
    );
    for (org, name, address, home) in expected {
        let key = forge::MakingKey { forge: FORGE, org: org.into(), name: name.into() };
        let engine::Record::Forge { row } = &front.engine.store.rows[&engine::Key::Forge(forge::Key::Making(key))]
        else {
            panic!("durable Making")
        };
        let forge::Stored::Making(row) = row.as_ref() else { panic!("Making row") };
        assert_eq!(row.from, forge::MakeFrom::Copy { address: address.into() });
        let into = row.into.as_ref().unwrap();
        assert_eq!((into.project, into.home), (project, home));
        assert_eq!(row.state, forge::MakingState::Waiting);
    }
    assert!(root.person().pages().adopting().is_none());
}

#[test]
fn sheet_copies_wait_for_each_line_then_close_with_both_rows_on_the_watch() {
    let mut world = configured(true);
    connected(&mut world);
    let person = world.client(fixtures::PARTY, Address::Adopt { project: 1 });
    script(
        &mut world,
        person,
        vec![
            Step::See { find: Find::named(Role::Choice, b"Copy public"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Choice, b"Copy public") },
            Step::Type {
                find: Find::named(Role::TextArea, b"Their addresses, one per line"),
                words: b"source.invalid/acme/widget\nnot-an-address".as_slice().into(),
            },
            see(b"Not an address: not-an-address. Write it as github.com/owner/name."),
        ],
    );
    let button = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Copy 2 repositories"))
        .expect("plural copy button");
    assert!(button.states.as_slice().contains(&skein_ui::State::Disabled));
    assert_eq!(world.front().applications_admitted(), 0, "a bad line holds both copies back");
    script(
        &mut world,
        person,
        vec![
            Step::Type {
                find: Find::named(Role::TextArea, b"Their addresses, one per line"),
                words: b"source.invalid/acme/widget\nsource.invalid/team/tool".as_slice().into(),
            },
            press(b"Copy 2 repositories"),
            Step::Gone { find: Find::named(Role::Dialog, b"Add repositories to"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.front().applications_admitted(), 2);
    let root = world.root(person).person();
    assert_eq!(*root.pages().address(), Address::Repositories { project: 1 });
    let row = temper_client_domain::forge::sheet::row(root.forges(), Some(FORGE)).expect("watched forge");
    assert_eq!(row.makings.len(), 2);
    for (making, (org, name, address, home)) in row.makings.iter().zip([
        (b"acme".as_slice(), b"widget".as_slice(), b"https://source.invalid/acme/widget".as_slice(), true),
        (b"team".as_slice(), b"tool".as_slice(), b"https://source.invalid/team/tool".as_slice(), false),
    ]) {
        assert_eq!((making.org.as_ref(), making.name.as_ref()), (org, name));
        let temper_client_protocol_wire::v1::MakeFrom::Copy(copy) = &making.from else { panic!("copy source") };
        assert_eq!(copy.address(), address);
        let into = making.into.as_ref().expect("project destination");
        assert_eq!((into.project(), into.home()), (1, home));
    }
}

#[test]
fn a_refused_sheet_copy_keeps_its_words_and_only_the_unfinished_address() {
    let mut world = configured_with_makings(true, Some(1));
    connected(&mut world);
    let person = world.client(fixtures::PARTY, Address::Adopt { project: 1 });
    script(
        &mut world,
        person,
        vec![
            Step::See { find: Find::named(Role::Choice, b"Copy public"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Choice, b"Copy public") },
            Step::Type {
                find: Find::named(Role::TextArea, b"Their addresses, one per line"),
                words: b"source.invalid/acme/widget\nsource.invalid/team/tool".as_slice().into(),
            },
            press(b"Copy 2 repositories"),
            see(b"Too many repositories are being made. Try again."),
        ],
    );
    assert_eq!(world.front().applications_admitted(), 2);
    let root = world.root(person).person();
    assert_eq!(*root.pages().address(), Address::Adopt { project: 1 });
    let sheet = root.forge().sheet().expect("refused sheet stays open");
    assert_eq!(sheet.addresses.text.as_ref(), b"source.invalid/team/tool");
    assert!(matches!(sheet.sent, temper_client_domain::forge::sheet::Sent::Refused(_)));
    let row = temper_client_domain::forge::sheet::row(root.forges(), Some(FORGE)).expect("watched forge");
    assert_eq!(row.makings.len(), 1);
}
