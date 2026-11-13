//! Adversarial scenarios: client/forge/deployment.md; client/pages/README.md and client/forge/records.md and client/forge/family.md, 7 and client/view.md, 1–3 and client/pages/welcome.md, 2
//! ; client/web/README.md 2.3.
use super::model::press;
use jig_client_kit::harness::{ClientId, Fault, Until, World};
use skein_fake_person::{Find, Step as PersonStep};
use skein_lib::Duration;
use skein_ui::{Act, Role, State};
use skein_world::domain::Verdict;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{Address, Choice, Place, Step, address};
use temper_client_view::look;
use temper_client_world::{app::Temper, fixtures as f, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, walking};

type Pages = World<Temper, Joint>;
const OWNED: u16 = 1;
const GITHUB: u16 = 2;
const REMOTE: u16 = 3;
const FAKE: u16 = 4;
const UNKNOWN: u16 = 65535;
// walking::config gives each fake writer user 1.
const BOT_ID: u64 = 1;
// Enough for several 1–3 ms shell deliveries; below the 2 s request obligation.
const PENDING_WINDOW: Duration = Duration::from_millis(100);
const BOT: &[u8] = b"qa-bot";
const FAILURE: &[u8] = b"preparing: fake disk full";
// One writing request for one press, even when pressed twice before its answer.
const ONE_SETUP: usize = 1;

fn flow(choice: Option<Choice>) -> Address {
    Address::ConnectForge { forge: choice }
}
fn at(forge: u16) -> Address {
    flow(Some(Choice::Forge(forge)))
}
fn see(role: Role, name: &[u8]) -> PersonStep<Address> {
    PersonStep::See { find: Find::exactly_named(role, name), within: f::WITHIN }
}
fn gone(role: Role, name: &[u8]) -> PersonStep<Address> {
    PersonStep::Gone { find: Find::exactly_named(role, name), within: f::WITHIN }
}
fn follow(world: &mut Pages, client: ClientId, steps: Vec<PersonStep<Address>>) {
    world.person(client, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn owned_details() -> forge::Owned {
    forge::Owned {
        forgejo: b"17.0.0".as_slice().into(),
        runner: Some(b"13.2.0".as_slice().into()),
        password: b"/fake/forge/person_password".as_slice().into(),
    }
}
fn configured(owner: bool, under_box: bool, mode: forge::Mode) -> Pages {
    configured_with(
        owner,
        under_box,
        &[
            forge::Configured {
                admin_connect: false,
                bot: Box::new([]),
                owned: (mode == forge::Mode::Owned).then(owned_details),
                git: Box::new([]),
                web: None,
                read_only: false,
                number: OWNED,
                kind: forge::Kind::Forgejo,
                mode,
                asked: true,
            },
            forge::Configured {
                admin_connect: false,
                bot: Box::new([]),
                owned: None,
                git: Box::new([]),
                web: None,
                read_only: false,
                number: GITHUB,
                kind: forge::Kind::GitHub,
                mode: forge::Mode::Connected,
                asked: false,
            },
        ],
    )
}
fn configured_with(owner: bool, under_box: bool, rows: &[forge::Configured]) -> Pages {
    configured_accounts(owner, under_box, rows, true)
}
fn configured_accounts(owner: bool, under_box: bool, rows: &[forge::Configured], account: bool) -> Pages {
    let mut world = if owner { f::accounts_world(f::SEED) } else { f::world(f::SEED) };
    let front = world.front();
    let mut store = front.engine.take_store();
    if !account {
        // The charter keeps its model; the restored deployment account goes.
        let accounts = store
            .rows
            .iter()
            .filter(|(_, row)| matches!(row, engine::Record::Core(jig_domain_core::Record::Accounts(_))))
            .map(|(key, _)| key.clone())
            .collect();
        store.seed(Box::new([]), accounts);
    }
    // temper/docs/design/client/forge/deployment.md: kept records supply the fixture's forges.
    let forges = store
        .rows
        .iter()
        .filter(|(_, row)| matches!(row, engine::Record::Forge { .. }))
        .map(|(key, _)| key.clone())
        .collect();
    store.seed(Box::new([]), forges);
    let mut records = Vec::new();
    let mut last = 0;
    for row in rows {
        last = last.max(row.number);
        let record = forge::ForgeRecord {
            person: None,
            git_program: None,
            number: row.number,
            kind: row.kind,
            mode: row.mode,
            state: forge::ForgeState::Kept,
            place: forge::Place {
                api: forge::Address { host: f::FORGE_HOST.into(), port: 443, tls: None },
                git: row.git.clone(),
                web: row.web.clone(),
            },
            read_only: row.read_only,
            bot: forge::Bot { login: row.bot.clone(), name: Box::new([]), email: Box::new([]) },
            asked: row.asked,
            first: None,
        };
        records.push(engine::Record::Forge { row: Box::new(forge::Stored::Forge(record)) });
    }
    if last != 0 {
        records.push(engine::Record::Forge { row: Box::new(forge::Stored::ForgeNumber { last }) });
    }
    if !records.is_empty() {
        store.seed(records.into_boxed_slice(), Box::new([]));
    }
    let mut limits = front.engine.env.limits;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(f::SEED);
        config.core.connectors = Box::new([temper_client_protocol_wire::FORGE]);
        config.forge.writers = rows
            .iter()
            .map(|row| {
                let mut writer = config.forge.writers[0];
                writer.forge = row.number;
                writer.author = BOT_ID;
                writer
            })
            .collect();
        config.forges = rows.to_vec().into_boxed_slice();
        config.under_box = under_box;
        if rows.iter().any(|row| row.admin_connect) {
            temper_client_world::front::first_run_projects(&mut config, &mut limits);
        }
        config
    });
    front.engine = Driver::configured(store, configs, &limits);
    front.engine.settle();
    world
}
fn world() -> Pages {
    configured(true, true, forge::Mode::Owned)
}
fn world_for(number: u16) -> Pages {
    match number {
        REMOTE => configured_with(
            true,
            true,
            &[
                forge::Configured {
                    admin_connect: false,
                    bot: Box::new([]),
                    owned: None,
                    git: Box::new([]),
                    web: None,
                    read_only: false,
                    number: GITHUB,
                    kind: forge::Kind::GitHub,
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
                    number: REMOTE,
                    kind: forge::Kind::Forgejo,
                    mode: forge::Mode::Managed,
                    asked: false,
                },
            ],
        ),
        FAKE => configured_with(
            true,
            true,
            &[
                forge::Configured {
                    admin_connect: false,
                    bot: Box::new([]),
                    owned: None,
                    git: Box::new([]),
                    web: None,
                    read_only: false,
                    number: OWNED,
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
                    number: FAKE,
                    kind: forge::Kind::Forgejo,
                    mode: forge::Mode::Fake,
                    asked: false,
                },
            ],
        ),
        _ => world(),
    }
}
fn connection(world: &mut Pages, number: u16, state: forge::Connection) {
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::Connection { forge: number, state })));
}
fn fetched(world: &mut Pages, number: u16, fetched: forge::SetUpFetched) {
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::Fetched { forge: number, fetched })));
}
fn connected() -> forge::Connection {
    forge::Connection::Connected { id: BOT_ID, login: BOT.into() }
}
// Welcome hides Live. Establish the world's Online observation before entering it.
fn open(world: &mut Pages, address: Address) -> ClientId {
    let client = world.client(f::PARTY, Address::Chats { filter: ChatsFilter::All });
    follow(world, client, vec![see(Role::Heading { level: 1 }, b"Talk to an agent.")]);
    follow(world, client, vec![PersonStep::Go { address }]);
    client
}
fn rendered(nodes: &[skein_ui::Node]) -> String {
    nodes
        .iter()
        .flat_map(|node| {
            node.name
                .iter()
                .map(AsRef::as_ref)
                .chain(node.description.iter().map(AsRef::as_ref))
                .chain(node.text.as_slice().iter().map(|run| run.text.as_ref()))
        })
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}
fn text(world: &Pages, client: ClientId) -> String {
    rendered(world.tree(client).nodes())
}
fn no_flow_steps(world: &Pages, client: ClientId) {
    assert!(
        !world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.role == Role::List && node.name.as_deref() == Some(b"Steps"))
    );
}
fn pending(world: &mut Pages) {
    let _ = world.run(Until::Time(world.now().saturating_add(PENDING_WINDOW)));
}
fn press_pending(world: &mut Pages, client: ClientId, button: &[u8]) {
    let button = node(world, client, Role::Button, button).id;
    world.act(client, button, Act::Press);
    pending(world);
}
fn answer_setup(world: &mut Pages, answer: forge::SetUpAnswer) {
    let (number, request) = world.front().set_ups().last().copied().expect("one setup sent");
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::SetUpAnswered {
        forge: number,
        request,
        answer,
    })));
}
fn node<'a>(world: &'a Pages, client: ClientId, role: Role, name: &[u8]) -> &'a skein_ui::Node {
    world
        .tree(client)
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {role:?} {}: {}", String::from_utf8_lossy(name), text(world, client)))
}

#[test]
fn forge_queries_canonicalize_good_bad_and_repeated_values() {
    for (query, choice, canonical) in [
        ("", None, ""),
        ("?forge=1", Some(Choice::Forge(OWNED)), "?forge=1"),
        ("?forge=65535", Some(Choice::Forge(UNKNOWN)), "?forge=65535"),
        ("?forge=elsewhere", Some(Choice::Elsewhere), "?forge=elsewhere"),
        ("?forge=1&forge=2", Some(Choice::Forge(OWNED)), "?forge=1"),
        ("?forge=elsewhere&forge=1", Some(Choice::Elsewhere), "?forge=elsewhere"),
        ("?forge=01", None, ""),
        ("?forge=65536", None, ""),
        ("?forge=-1", None, ""),
        ("?forge=garbage", None, ""),
        ("?forge=", None, ""),
        ("?forge=1=2", None, ""),
        ("?x=1&forge=3&at=ready", Some(Choice::Forge(REMOTE)), "?forge=3"),
    ] {
        let path = format!("/forges/connect{query}");
        assert_eq!(address::parse(path.as_bytes()), Place::Address(flow(choice)), "{path}");
        assert_eq!(address::print(&flow(choice)).as_ref(), format!("/forges/connect{canonical}").as_bytes());
        let fields = query.trim_start_matches('?');
        let welcome = format!("/welcome?at=forge&{fields}");
        let step = Step::Forge { forge: choice };
        assert_eq!(address::parse(welcome.as_bytes()), Place::Address(Address::Welcome { step }), "{welcome}");
    }
}

#[test]
fn malformed_first_forge_does_not_shadow_a_readable_repeat() {
    for path in ["/forges/connect?forge=garbage&forge=1", "/welcome?at=forge&forge=&forge=1"] {
        let expected = if path.starts_with("/welcome") {
            Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(OWNED)) } }
        } else {
            at(OWNED)
        };
        assert_eq!(address::parse(path.as_bytes()), Place::Address(expected), "client/web/README.md 2.3: {path}");
    }
}

#[test]
fn choose_orders_owned_github_and_elsewhere_and_omits_fakes() {
    let mut world = configured_accounts(true, true, &[], true);
    let client = open(&mut world, flow(None));
    follow(
        &mut world,
        client,
        vec![
            see(Role::Region, b"GitHub"),
            see(Role::Region, b"Forgejo"),
            see(Role::Link, b"Connect your Forgejo"),
            gone(Role::Region, b"Playground forge"),
        ],
    );
    let names = world
        .tree(client)
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Region)
        .filter_map(|node| node.name.as_deref())
        .collect::<Vec<_>>();
    let github = names.iter().position(|name| *name == b"GitHub").expect("GitHub card");
    let owned = names.iter().position(|name| *name == b"Forgejo").expect("owned card");
    assert!(owned < github, "forge/deployment.md: {names:?}");
}

#[test]
fn unknown_and_fake_choices_fall_back_to_choose() {
    for number in [UNKNOWN, FAKE] {
        let mut world = world_for(number);
        let client = open(&mut world, at(number));
        follow(&mut world, client, vec![see(Role::Link, b"Connect your Forgejo"), gone(Role::Link, b"Other forges")]);
    }
}

#[test]
fn owned_details_offer_one_setup_and_name_its_costs() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::NotSetUp);
    let client = open(&mut world, at(OWNED));
    follow(&mut world, client, vec![see(Role::Button, b"Set up a Forgejo here"), see(Role::Link, b"Other forges")]);
    let words = text(&world, client);
    for expected in [
        "Its CI jobs run on this machine, as you, with no isolation.",
        "Forgejo stops when temper box stops. Its repositories are kept in temper’s data directory.",
    ] {
        assert!(words.contains(expected), "missing {expected}: {words}");
    }
}

#[test]
fn not_under_box_says_to_start_temper_with_the_box_without_action() {
    let mut world = configured(true, false, forge::Mode::Owned);
    connection(&mut world, OWNED, forge::Connection::NotSetUp);
    let client = open(&mut world, at(OWNED));
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 2 }, b"Set it up under temper box"),
            gone(Role::Button, b"Set up a Forgejo here"),
        ],
    );
    let words = text(&world, client);
    for expected in [
        "temper was not started by temper box, so it does not start a Forgejo itself.",
        "Start temper with temper box to set one up here.",
    ] {
        assert!(words.contains(expected), "missing {expected}: {words}");
    }
    assert!(!words.contains("temper forge fetch"));
    follow(&mut world, client, vec![PersonStep::Go { address: flow(None) }, see(Role::Button, b"Start over")]);
    assert!(text(&world, client).contains("not finished"));
}

#[test]
fn unoffered_forge_has_no_owned_choose_card() {
    for (mode, asked, state) in [
        (forge::Mode::Managed, false, forge::Connection::NotConnected),
        (forge::Mode::Owned, false, forge::Connection::NotSetUp),
    ] {
        let mut world = configured_with(
            true,
            true,
            &[
                forge::Configured {
                    admin_connect: false,
                    bot: Box::new([]),
                    owned: None,
                    git: Box::new([]),
                    web: None,
                    read_only: false,
                    number: OWNED,
                    kind: forge::Kind::Forgejo,
                    mode,
                    asked,
                },
                forge::Configured {
                    admin_connect: false,
                    bot: Box::new([]),
                    owned: None,
                    git: Box::new([]),
                    web: None,
                    read_only: false,
                    number: GITHUB,
                    kind: forge::Kind::GitHub,
                    mode: forge::Mode::Connected,
                    asked: false,
                },
            ],
        );
        connection(&mut world, OWNED, state);
        let client = open(&mut world, flow(None));
        follow(
            &mut world,
            client,
            vec![see(Role::Button, b"Start over"), gone(Role::Region, b"Forgejo on this machine")],
        );
    }
}

#[test]
fn github_details_offer_only_the_app_of_ones_own_with_a_masked_key() {
    let mut world = world();
    let client =
        world.client_with(f::PARTY, Address::Chats { filter: ChatsFilter::All }, f::domain_limits(), browser_client);
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 1 }, b"Talk to an agent."),
            PersonStep::Go { address: at(GITHUB) },
            see(Role::Button, b"Connect the App"),
        ],
    );
    let words = text(&world, client);
    assert!(!words.contains("organization"), "{words}");
    assert!(!words.contains("Connect GitHub"), "{words}");
    assert!(!words.contains("not available yet"), "{words}");
    node(&world, client, Role::Heading { level: 2 }, b"An App of your own");
    follow(
        &mut world,
        client,
        vec![gone(Role::Heading { level: 2 }, b"GitHub"), gone(Role::Heading { level: 3 }, b"An App of your own")],
    );
    node(&world, client, Role::TextField, b"App id");
    assert!(!node(&world, client, Role::Button, b"Connect the App").states.as_slice().contains(&State::Disabled));
    assert_eq!(node(&world, client, Role::TextField, b"Private key").entry, Some(skein_ui::EntryKind::Secret));
    let mut fresh = configured_accounts(true, true, &[], true);
    let client = open(&mut fresh, flow(None));
    follow(&mut fresh, client, vec![see(Role::Button, b"Connect GitHub")]);
    assert!(!node(&fresh, client, Role::Button, b"Connect GitHub").states.as_slice().contains(&State::Disabled));
    assert!(text(&fresh, client).contains("Your repositories on GitHub. temper makes its own App there; you choose the account and the repositories it may use."));
}

const ADMIN_TOKEN: &[u8] = b"admin-token-placeholder-0123";
pub(super) fn admin_world(mode: forge::Mode) -> Pages {
    // The original deployment's durable store is cold-restored with the new
    // configuration, as after changing temper.json and restarting.
    let mut world = configured_accounts(
        true,
        false,
        &[
            forge::Configured {
                owned: None,
                admin_connect: mode == forge::Mode::Managed,
                bot: Box::new([]),
                git: Box::new([]),
                web: Some(b"https://git.example.org".as_slice().into()),
                read_only: false,
                number: OWNED,
                kind: forge::Kind::Forgejo,
                mode,
                asked: false,
            },
            forge::Configured {
                owned: None,
                admin_connect: false,
                bot: Box::new([]),
                git: Box::new([]),
                web: None,
                read_only: false,
                number: GITHUB,
                kind: forge::Kind::GitHub,
                mode: forge::Mode::Connected,
                asked: false,
            },
        ],
        false,
    );
    world.front().engine.forge =
        Some(temper_engine_domain_world::forge_stand::ForgeStand::with(&[(b"work", b"hello", 42)]));
    world
}
fn admin_input(world: &mut Pages, client: ClientId, field: &[u8], value: &[u8]) {
    let id = node(world, client, Role::TextField, field).id;
    world.act(client, id, Act::Input { value: value.into() });
    pending(world);
}
fn admin_private(world: &mut Pages, client: ClientId) {
    let tree = world.tree(client);
    for node in tree.nodes() {
        for value in [node.name.as_deref(), node.description.as_deref(), node.value.as_ref().map(|v| v.text.as_ref())]
            .into_iter()
            .flatten()
        {
            assert!(skein_lib::bytes::find(value, ADMIN_TOKEN).is_none());
        }
        for run in &node.text {
            assert!(skein_lib::bytes::find(&run.text, ADMIN_TOKEN).is_none());
        }
    }
    let engine = &world.front().engine;
    // Includes durable notes and all engine answers and output logs retained
    // by the fake driver; no real token, forge or process participates.
    for captured in [
        format!("{:?}", engine.store.rows),
        format!("{:?}", engine.transactions),
        format!("{:?}", engine.delivered),
        format!("{:?}", engine.immediate),
    ] {
        assert!(skein_lib::bytes::find(captured.as_bytes(), ADMIN_TOKEN).is_none());
    }
}
#[test]
fn elsewhere_welcome_after_restart_connects_retries_and_leaves_the_repository_unadopted() {
    let mut world = admin_world(forge::Mode::Managed);
    let client = open(&mut world, Address::Welcome { step: Step::Forge { forge: None } });
    follow(&mut world, client, vec![see(Role::Button, b"Start over"), gone(Role::Region, b"GitHub")]);
    let names = world
        .tree(client)
        .nodes()
        .iter()
        .filter(|n| n.role == Role::Region)
        .filter_map(|n| n.name.as_deref())
        .collect::<Vec<_>>();
    assert!(!names.contains(&b"Forgejo at https://git.example.org".as_slice()));
    // Enter by address: the welcome draws no admin card (client/forge/deployment.md).
    follow(
        &mut world,
        client,
        vec![
            PersonStep::Go { address: Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(OWNED)) } } },
            see(Role::TextField, "The site admin’s token".as_bytes()),
        ],
    );
    admin_input(&mut world, client, b"Runner label", b"linux-arm");
    admin_input(&mut world, client, "The site admin’s token".as_bytes(), ADMIN_TOKEN);
    admin_private(&mut world, client);
    press_pending(&mut world, client, b"Connect as its admin");
    follow(&mut world, client, vec![see(Role::Heading { level: 2 }, b"Connecting forge 1 as its admin.")]);
    admin_private(&mut world, client);
    let (_, request) = *world.front().admin_connects.last().expect("admin Connect sent");
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::ConnectEnded {
        forge: OWNED,
        request,
        connected: false,
        why: Some(b"NotAdmin".as_slice().into()),
        stop: Some(forge::AdminStop::NotAdmin),
    })));
    follow(
        &mut world,
        client,
        vec![see(Role::TextField, "The site admin’s token".as_bytes()), see(Role::Button, b"Connect as its admin")],
    );
    assert!(text(&world, client).contains("temper kept nothing of the token."));
    assert_eq!(
        node(&world, client, Role::TextField, b"Runner label").value.as_ref().expect("field value").text.as_ref(),
        b"linux-arm"
    );
    assert_eq!(
        node(&world, client, Role::TextField, "The site admin’s token".as_bytes())
            .value
            .as_ref()
            .expect("field value")
            .text
            .as_ref(),
        b""
    );
    admin_private(&mut world, client);
    admin_input(&mut world, client, "The site admin’s token".as_bytes(), ADMIN_TOKEN);
    press_pending(&mut world, client, b"Connect as its admin");
    follow(&mut world, client, vec![see(Role::Heading { level: 2 }, b"Connecting forge 1 as its admin.")]);
    let (_, request) = *world.front().admin_connects.last().expect("admin Connect sent");
    assert_eq!(world.front().admin_connects.len(), 2);
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::FirstMade { forge: OWNED, id: 42 })));
    connection(&mut world, OWNED, connected());
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::ConnectEnded {
        forge: OWNED,
        request,
        connected: true,
        why: None,
        stop: None,
    })));
    // Connection advances welcome to Model (client/pages/README.md); the durable
    // runner label belongs to this forge's Ready (client/forge/deployment.md).
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 1 }, b"Connect a model"),
            PersonStep::Go { address: at(OWNED) },
            see(Role::Description, "qa-bot · user 1".as_bytes()),
            see(
                Role::Paragraph,
                "Made work/hello. Its CI runs on your forge’s runners labelled linux-arm; with none, a change’s checks wait.".as_bytes(),
            ),
            PersonStep::Go { address: Address::Welcome { step: Step::Ready } },
            see(Role::Heading { level: 1 }, b"Almost ready"),
            see(Role::TextField, b"Your first project"),
            see(Role::Button, b"Create your project"),
        ],
    );
    admin_private(&mut world, client);
    assert!(node(&world, client, Role::Button, b"Create your project").states.as_slice().contains(&State::Disabled));
    admin_private(&mut world, client);
    assert!(!world.front().engine_adopted(OWNED, 42));
}

#[test]
fn setup_steps_show_the_reported_step_and_current_flow_step() {
    for (state, current) in [
        (forge::Connection::SettingUp { step: None }, None),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Fetch) }, Some(0)),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) }, Some(1)),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Run) }, Some(2)),
        (forge::Connection::Connecting, Some(3)),
    ] {
        let mut world = world();
        connection(&mut world, OWNED, state);
        let client = open(&mut world, at(OWNED));
        follow(&mut world, client, vec![see(Role::Disclosure, b"Forgejo on this machine")]);
        let words = text(&world, client);
        let nodes = world.tree(client).nodes();
        let progress = nodes
            .iter()
            .position(|node| {
                node.role == Role::List && node.styles.as_slice().contains(&temper_client_view::look::CONNECT_LIVE)
            })
            .expect("setup progress list");
        let progress = &nodes[progress..progress + usize::try_from(nodes[progress].size).expect("tree size")];
        let items = progress.iter().enumerate().filter(|(_, node)| node.role == Role::Item).collect::<Vec<_>>();
        let labels = [
            if current > Some(0) { "Fetched Forgejo and its runner" } else { "Fetching Forgejo and its runner" },
            "Preparing: its database, temper’s admin, your account",
            "Starting Forgejo and its runner",
            "Connecting as temper",
            "Making the first repository",
        ];
        assert_eq!(items.len(), labels.len(), "{words}");
        for (stage, ((index, item), label)) in items.iter().zip(labels).enumerate() {
            let subtree = &progress[*index..*index + usize::try_from(item.size).expect("tree size")];
            assert!(rendered(subtree).contains(label), "stage {stage}: {words}");
            let under_way = current == Some(stage);
            let later = current.is_none_or(|current| stage > current);
            assert_eq!(item.states.as_slice().contains(&State::Current), under_way, "stage {stage}: {words}");
            assert_eq!(item.styles.as_slice().contains(&temper_client_view::look::CONNECT_LATER), later);
            for (status, expected) in [(b"under way".as_slice(), false), (b"done".as_slice(), false)] {
                assert_eq!(
                    subtree.iter().any(|node| node.role == Role::Status && node.name.as_deref() == Some(status)),
                    expected,
                    "stage {stage}: {words}"
                );
            }
        }
        if current.is_none() {
            node(&world, client, Role::Status, b"under way");
        }
        no_flow_steps(&world, client);
        assert!(world.front().set_ups().is_empty(), "opening a running setup must not ask again");
    }
}

#[test]
fn connecting_and_connected_rows_keep_their_rows_without_flow_steps() {
    for (number, state, name) in [
        (OWNED, forge::Connection::Connecting, b"Forgejo on this machine".as_slice()),
        (GITHUB, forge::Connection::Connecting, b"GitHub"),
        (REMOTE, forge::Connection::Connecting, b"Your Forgejo"),
        (OWNED, connected(), b"Forgejo on this machine"),
        (GITHUB, connected(), b"GitHub"),
        (REMOTE, connected(), b"Your Forgejo"),
    ] {
        let mut world = world_for(number);
        connection(&mut world, number, state);
        let client = open(&mut world, at(number));
        follow(
            &mut world,
            client,
            vec![see(Role::Heading { level: 1 }, b"Where your code lives."), see(Role::Disclosure, name)],
        );
        no_flow_steps(&world, client);
    }
}

#[test]
fn forges_list_kept_unstarted_rows_and_omit_fakes() {
    let mut world = world_for(FAKE);
    connection(&mut world, OWNED, forge::Connection::NotSetUp);
    connection(&mut world, FAKE, connected());
    let client = open(&mut world, Address::Forges);
    follow(
        &mut world,
        client,
        vec![
            see(Role::Link, b"+ Connect a forge"),
            see(Role::Disclosure, b"Forgejo on this machine"),
            gone(Role::Disclosure, b"GitHub"),
            gone(Role::Disclosure, b"Playground forge"),
        ],
    );
}

#[test]
fn forges_rows_are_folded_and_keep_state_account_and_actions() {
    for (state, words, action) in [
        (forge::Connection::SettingUp { step: None }, "Setting up", "Remove"),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Fetch) }, "Setting up: fetching", "Remove"),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) }, "Setting up: preparing", "Remove"),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Run) }, "Setting up: starting", "Remove"),
        (
            forge::Connection::SetUpFailed { why: FAILURE.into() },
            "Set up failed: preparing: fake disk full",
            "Try again",
        ),
        (forge::Connection::Connecting, "Connecting", "Remove"),
        (connected(), "Connected as qa-bot", "Adopt a repository"),
        (
            forge::Connection::Stopped { reason: b"token refused".as_slice().into() },
            "Stopped: token refused",
            "Restart temper box: its preparation mends it",
        ),
    ] {
        let mut world = world();
        let connected = matches!(state, forge::Connection::Connected { .. });
        let live = matches!(
            state,
            forge::Connection::SettingUp { .. } | forge::Connection::Connecting | forge::Connection::SetUpFailed { .. }
        );
        connection(&mut world, OWNED, state);
        let client = open(&mut world, Address::Forges);
        let row = node(&world, client, Role::Disclosure, b"Forgejo on this machine");
        assert_eq!(row.states.as_slice().contains(&State::Expanded), live, "running and failed rows open");
        let drawing = text(&world, client);
        node(&world, client, Role::Term, b"Access");
        node(&world, client, Role::Description, b"Site admin, with temper's own token");
        if connected {
            node(&world, client, Role::Term, b"Account");
            node(&world, client, Role::Description, b"qa-bot \xc2\xb7 user 1");
        }
        // Restored forge records have no host-supplied versions or password path.
        assert!(!drawing.contains("First password"));
        assert!(!drawing.contains("Your first password is in"));
        assert!(!drawing.contains("Runs"));
        assert!(!drawing.contains("Forgejo 17.0.0"));
        for expected in ["Forge 1 · Forgejo", words, action] {
            assert!(drawing.contains(expected), "missing {expected}: {drawing}");
        }
        follow(
            &mut world,
            client,
            vec![PersonStep::Go { address: flow(None) }, gone(Role::Region, b"Forgejo on this machine")],
        );
    }
}

#[test]
fn managed_and_github_connected_rows_keep_their_names_and_accounts() {
    let mut world = world_for(REMOTE);
    connection(&mut world, REMOTE, connected());
    connection(&mut world, GITHUB, connected());
    let client = open(&mut world, Address::Forges);
    follow(&mut world, client, vec![see(Role::Disclosure, b"Your Forgejo"), see(Role::Disclosure, b"GitHub")]);
    let words = text(&world, client);
    assert!(words.contains("qa-bot · user 1"), "Account line: {words}");
}

#[test]
fn navigation_dot_names_and_colours_follow_real_rows_only() {
    for (number, state, label, style) in [
        (OWNED, forge::Connection::SettingUp { step: None }, b"a forge is setting up".as_slice(), look::NAV_LIMITED),
        (GITHUB, forge::Connection::Connecting, b"a forge is setting up", look::NAV_LIMITED),
        (REMOTE, forge::Connection::Connecting, b"a forge is setting up", look::NAV_LIMITED),
        (OWNED, forge::Connection::SetUpFailed { why: FAILURE.into() }, b"a forge needs you", look::NAV_NEEDS),
        (OWNED, forge::Connection::Stopped { reason: FAILURE.into() }, b"a forge needs you", look::NAV_NEEDS),
    ] {
        let mut world = world_for(number);
        connection(&mut world, number, state);
        let client = open(&mut world, Address::Chats { filter: ChatsFilter::All });
        follow(
            &mut world,
            client,
            vec![PersonStep::See {
                find: Find::exactly_named(Role::Text, label).within(Box::new([skein_fake_person::Target::ExactRole {
                    role: Role::Navigation,
                    name: b"Project 1".as_slice().into(),
                }])),
                within: f::WITHIN,
            }],
        );
        let dot = node(&world, client, Role::Text, label);
        let nodes = world.tree(client).nodes();
        let (at, deployment) = nodes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, node)| node.role == Role::Link && node.name.as_deref() == Some(b"Deployment"))
            .expect("rail's Deployment link");
        let size = usize::try_from(deployment.size).expect("Deployment subtree size");
        assert!(nodes.iter().skip(at).take(size).any(|node| node.id == dot.id), "Deployment holds its mark");
        assert!(dot.styles.as_slice().contains(&style), "wrong colour: {dot:?}");
    }
}

#[test]
fn failure_dot_wins_over_progress_then_clears_on_connect() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::SetUpFailed { why: FAILURE.into() });
    connection(&mut world, GITHUB, forge::Connection::Connecting);
    let client = open(&mut world, Address::Chats { filter: ChatsFilter::All });
    follow(&mut world, client, vec![see(Role::Text, b"a forge needs you"), gone(Role::Text, b"a forge is setting up")]);
    connection(&mut world, OWNED, connected());
    connection(&mut world, GITHUB, connected());
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::Installation {
        forge: GITHUB,
        installation: 1,
        state: Some(forge::Installation {
            account: Some((1, b"qa-account".as_slice().into())),
            suspended: false,
            stray: false,
        }),
    })));
    follow(
        &mut world,
        client,
        vec![gone(Role::Text, b"a forge needs you"), gone(Role::Text, b"a forge is setting up")],
    );
}

#[test]
fn fake_failure_never_sets_navigation_dot() {
    let mut world = world_for(FAKE);
    connection(&mut world, FAKE, forge::Connection::SetUpFailed { why: FAILURE.into() });
    let client = open(&mut world, Address::Chats { filter: ChatsFilter::All });
    follow(
        &mut world,
        client,
        vec![gone(Role::Text, b"a forge needs you"), gone(Role::Text, b"a forge is setting up")],
    );
}

#[test]
fn setup_press_is_single_and_disabled_until_box_answers() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::NotSetUp);
    let client = open(&mut world, at(OWNED));
    let button = node(&world, client, Role::Button, b"Set up a Forgejo here").id;
    world.fault(Fault::DoublePress { client, node: button });
    pending(&mut world);
    assert_eq!(world.front().set_ups().len(), ONE_SETUP);
    assert!(node(&world, client, Role::Button, b"Set up a Forgejo here").states.as_slice().contains(&State::Disabled));
    answer_setup(&mut world, forge::SetUpAnswer::Refused(forge::SetUpRefusal::Busy));
    follow(&mut world, client, vec![see(Role::Button, b"Set up a Forgejo here")]);
}

#[test]
fn retry_press_is_single_and_disabled_until_box_answers() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::SetUpFailed { why: FAILURE.into() });
    let client = open(&mut world, Address::Forges);
    let button = node(&world, client, Role::Button, b"Try again").id;
    world.fault(Fault::DoublePress { client, node: button });
    pending(&mut world);
    assert_eq!(world.front().set_ups().len(), ONE_SETUP);
    assert!(node(&world, client, Role::Button, b"Try again").states.as_slice().contains(&State::Disabled));
    answer_setup(&mut world, forge::SetUpAnswer::Refused(forge::SetUpRefusal::Busy));
    follow(&mut world, client, vec![see(Role::Button, b"Try again")]);
}

#[test]
fn every_box_refusal_is_shown_and_reenables_setup() {
    for (refusal, words) in [
        (forge::SetUpRefusal::NotOffered, "not offered"),
        (forge::SetUpRefusal::NotUnderTheBox, "not under the box"),
        (forge::SetUpRefusal::Busy, "busy"),
    ] {
        let mut world = world();
        connection(&mut world, OWNED, forge::Connection::NotSetUp);
        let client = open(&mut world, at(OWNED));
        press_pending(&mut world, client, b"Set up a Forgejo here");
        answer_setup(&mut world, forge::SetUpAnswer::Refused(refusal));
        follow(&mut world, client, vec![see(Role::Button, b"Set up a Forgejo here")]);
        let drawing = text(&world, client);
        assert!(drawing.contains(words), "{refusal:?}: expected {words}: {drawing}");
        assert!(
            !node(&world, client, Role::Button, b"Set up a Forgejo here").states.as_slice().contains(&State::Disabled)
        );
    }
}

#[test]
fn welcome_accepted_setup_moves_to_model_and_follows_its_forge() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::NotSetUp);
    let client = open(&mut world, Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(OWNED)) } });
    world.event(
        client,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Forge(temper_client_domain::forge::Act::SetUpAtOnce { forge: OWNED }),
        }
        .into(),
    );
    pending(&mut world);
    answer_setup(&mut world, forge::SetUpAnswer::SettingUp);
    connection(&mut world, OWNED, forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) });
    follow(
        &mut world,
        client,
        vec![see(Role::Heading { level: 1 }, b"Connect a model"), see(Role::Disclosure, b"Each step")],
    );
    connection(&mut world, GITHUB, connected());
    follow(&mut world, client, vec![see(Role::Disclosure, b"Each step")]);
    assert!(text(&world, client).contains("Forgejo on this machine · work/hello · Setting up: preparing"));
    connection(&mut world, OWNED, connected());
    follow(
        &mut world,
        client,
        vec![
            PersonStep::Go { address: Address::Welcome { step: Step::Ready } },
            see(Role::Heading { level: 1 }, b"Ready"),
            see(Role::TextField, b"Your first project"),
            see(Role::Disclosure, b"Each step"),
            gone(Role::Heading { level: 2 }, b"Setting up Forgejo on this machine"),
            gone(Role::Link, b"Later"),
        ],
    );
    node(&world, client, Role::Paragraph, "Forgejo on this machine · work/hello · Connected as qa-bot (1)".as_bytes());
    assert!(!node(&world, client, Role::Button, b"Create your project").states.as_slice().contains(&State::Disabled));
    assert_eq!(world.front().set_ups().len(), ONE_SETUP);
}

#[test]
fn running_setup_opened_in_second_tab_is_not_resent() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::SettingUp { step: Some(forge::SetUpStep::Run) });
    let _first = open(&mut world, at(OWNED));
    let second = world.client(f::PARTY, at(OWNED));
    follow(&mut world, second, vec![see(Role::Disclosure, b"Forgejo on this machine")]);
    assert_eq!(world.front().set_ups(), []);
}

#[test]
fn strip_exclusions_and_forges_navigation_through_deployment_match_pages() {
    for page in [
        Address::Forges,
        flow(None),
        Address::Accounts,
        Address::Welcome { step: Step::Forge { forge: None } },
        Address::Welcome { step: Step::Model { way: None } },
        Address::Welcome { step: Step::Ready },
    ] {
        let mut world = world();
        let client = open(&mut world, page);
        follow(&mut world, client, vec![gone(Role::Region, b"Setup"), gone(Role::Region, b"Playground")]);
    }
    for page in [Address::Forges, flow(None)] {
        let mut world = world();
        let client = open(&mut world, page);
        follow(&mut world, client, vec![press(Role::Link, b"Deployment")]);
        assert!(world.tree(client).nodes().iter().any(|n| n.role == Role::Link
            && n.name.as_deref() == Some(b"Deployment")
            && n.states.as_slice().contains(&State::Current)));
        assert!(node(&world, client, Role::Link, b"Accounts").states.as_slice().contains(&State::Current));
        follow(
            &mut world,
            client,
            vec![press(Role::Link, b"Forges"), see(Role::Heading { level: 1 }, b"Where your code lives.")],
        );
    }
}

#[test]
#[ignore = "needs: a non-owner identity; `configured(false, ..)` still signs in the deployment's owner, so the forges watch is served"]
fn non_owner_forges_page_explains_the_watch_refusal() {
    let mut world = configured(false, true, forge::Mode::Owned);
    let client = open(&mut world, Address::Forges);
    follow(&mut world, client, vec![see(Role::Link, b"Forges"), gone(Role::Button, b"Set up a Forgejo here")]);
    assert!(text(&world, client).contains("role"), "forges refusal: {}", text(&world, client));
}

#[test]
#[ignore = "needs: Joint per-subject forges watch ledger, hold/release and refused/ended injection; verify one session watch, no retry after terminal and headings before delivery"]
fn session_watch_survives_pages_and_stays_terminal_without_retry() {
    let mut world = world();
    let client = open(&mut world, Address::Forges);
    follow(
        &mut world,
        client,
        vec![
            PersonStep::Go { address: at(OWNED) },
            PersonStep::Go { address: Address::Chats { filter: ChatsFilter::All } },
        ],
    );
    // Hold first delivery: Forges must draw headings only. Refuse/end instead:
    // no rows or stale dot/banner; navigating must not open another watch.
}

#[test]
#[ignore = "needs: forges row wire fields carrying unreadable/mismatched record reasons; today's Connection::NotConnected has no reason"]
fn unreadable_not_connected_row_is_listed_and_sets_red_dot() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::NotConnected);
    let client = open(&mut world, Address::Forges);
    follow(
        &mut world,
        client,
        vec![see(Role::Disclosure, b"Forgejo on this machine"), see(Role::Text, b"a forge needs you")],
    );
}

#[test]
#[ignore = "needs: Joint browser reload retaining the address; its existing reload explicitly resets to Inbox as a terminal restart"]
fn browser_reload_preserves_running_setup_without_resending() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::SettingUp { step: None });
    let client = open(&mut world, at(OWNED));
    world.reload(client);
    follow(&mut world, client, vec![see(Role::Disclosure, b"Forgejo on this machine")]);
    assert_eq!(world.front().set_ups(), []);
}

#[test]
fn accounts_banner_stays_visible_without_the_setup_strip() {
    for (page, _had_strip) in [
        (Address::Home { project: 1 }, true),
        (Address::Settings { project: 1 }, true),
        (Address::Chats { filter: ChatsFilter::All }, true),
        (f::inbox(), true),
        (Address::Adopt { project: 1 }, true),
        (Address::Forges, false),
        (flow(None), false),
    ] {
        // No usable account: the accounts document is not usable.
        let mut world = configured_accounts(
            true,
            true,
            &[
                forge::Configured {
                    admin_connect: false,
                    bot: Box::new([]),
                    owned: None,
                    git: Box::new([]),
                    web: None,
                    read_only: false,
                    number: OWNED,
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
                    number: GITHUB,
                    kind: forge::Kind::GitHub,
                    mode: forge::Mode::Connected,
                    asked: false,
                },
            ],
            false,
        );
        let client = open(&mut world, page);
        let nodes = world.tree(client).nodes();
        let banner = nodes.iter().position(|node| {
            node.text.as_slice().iter().any(|run| run.text.as_ref() == b"Agents need an LLM account to run.")
        });
        assert!(banner.is_some(), "client/view.md 1 requires the accounts banner: {}", text(&world, client));
        let strip = nodes.iter().position(|node| node.role == Role::Region && node.name.as_deref() == Some(b"Setup"));
        assert!(strip.is_none(), "the setup strip is absent");
        if let Some(strip) = strip {
            assert!(banner.expect("banner exists") < strip, "accounts banner precedes setup strip");
        }
    }
}

#[test]
fn failed_setup_flow_keeps_reason_and_offers_one_retry() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::SetUpFailed { why: FAILURE.into() });
    let client = open(&mut world, at(OWNED));
    no_flow_steps(&world, client);
    assert!(text(&world, client).contains("Set up failed: preparing: fake disk full"));
    press_pending(&mut world, client, b"Try again");
    assert_eq!(world.front().set_ups().len(), ONE_SETUP);
    assert!(node(&world, client, Role::Button, b"Try again").states.as_slice().contains(&State::Disabled));
    answer_setup(&mut world, forge::SetUpAnswer::Refused(forge::SetUpRefusal::Busy));
    follow(&mut world, client, vec![see(Role::Button, b"Try again")]);
}

fn browser_client(seed: u64) -> (temper_client_world::app::Root, temper_client_world::app::View) {
    use temper_client_world::app::{Root, View};
    let limits = f::domain_limits();
    let words = temper_client_view::words::Words::english(
        f::FORGE,
        limits.client.domain.text,
        temper_client_view::words::Shell::Browser,
    )
    .expect("browser words fit the client bounds");
    let bounds = f::view_bounds(&limits, &words);
    (
        Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        View::Person(Box::new(temper_client_view::View::new(&bounds, &limits, words, f::FORGE))),
    )
}

#[test]
fn a_forge_row_says_how_far_its_fetch_has_gone() {
    let mut world = world();
    connection(&mut world, OWNED, forge::Connection::SettingUp { step: Some(forge::SetUpStep::Fetch) });
    let client = open(&mut world, Address::Forges);
    for (progress, words) in [
        (
            forge::SetUpFetched { binary: forge::SetUpBinary::Forgejo, done: 42_000_000, total: Some(100_000_000) },
            "Forge 1 · Forgejo · Setting up: fetching Forgejo · 42%",
        ),
        (
            forge::SetUpFetched { binary: forge::SetUpBinary::Forgejo, done: 67_000_000, total: Some(100_000_000) },
            "Forge 1 · Forgejo · Setting up: fetching Forgejo · 67%",
        ),
        (
            forge::SetUpFetched { binary: forge::SetUpBinary::Runner, done: 8_000_000, total: Some(10_000_000) },
            "Forge 1 · Forgejo · Setting up: fetching its runner · 80%",
        ),
        (
            forge::SetUpFetched { binary: forge::SetUpBinary::Runner, done: 41_999_999, total: None },
            "Forge 1 · Forgejo · Setting up: fetching its runner · 41 MB",
        ),
    ] {
        fetched(&mut world, OWNED, progress);
        pending(&mut world);
        let row = node(&world, client, Role::Disclosure, b"Forgejo on this machine");
        assert_eq!(row.description.as_deref(), Some(words.as_bytes()));
    }
    connection(&mut world, OWNED, forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) });
    pending(&mut world);
    let row = node(&world, client, Role::Disclosure, b"Forgejo on this machine");
    assert_eq!(row.description.as_deref(), Some("Forge 1 · Forgejo · Setting up: preparing".as_bytes()));
}
