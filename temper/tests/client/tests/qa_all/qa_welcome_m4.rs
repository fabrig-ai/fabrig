//! Independent QA from client/README.md 3.5, 4.8, 5.5–5.6 and client/forge.md 5.4.
use jig_client_world::harness::App;
use skein_lib::Queue;
use skein_ui::{Role, Tree};
use temper_client_domain::{Address, Step};
use temper_client_protocol_wire::v1 as wire;
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures,
    front::{Joint, account_acts as harness},
};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

// client/forge.md 5.4: fetching, preparing, starting, connecting, making/adopting.
const SETUP_STEPS: usize = 5;
// The owned forge and its restored writer in the public harness fixtures.
const OWNED_FORGE: u16 = 1;
const FORGE_WRITER: u64 = 1;

fn page(front: &mut Joint, root: &mut Root, address: Address) -> View {
    harness::event(front, root, Temper::went(address));
    let (_, mut view) = Temper::new(fixtures::SEED);
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, root, &mut patches);
    view
}

fn words(tree: &Tree) -> Vec<String> {
    tree.nodes()
        .iter()
        .map(|node| node.text.iter().map(|run| String::from_utf8_lossy(&run.text)).collect::<String>())
        .collect()
}

fn connection(front: &mut Joint, root: &mut Root, state: forge::Connection) {
    front.engine.send(engine::Event::Forge(Box::new(forge::Event::Connection { forge: OWNED_FORGE, state })));
    front.engine.settle();
    harness::settle(front, root);
}

#[test]
fn failed_setup_keeps_steps_and_names_the_failed_one() {
    // Each row is a box/engine failure boundary, including a missing step.
    let cases = [
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Fetch) }, "Fetching", 0),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) }, "Preparing", 1),
        (forge::Connection::SettingUp { step: Some(forge::SetUpStep::Run) }, "Starting", 2),
        (forge::Connection::Connecting, "Connecting", 3),
        (forge::Connection::SettingUp { step: None }, "Set up", SETUP_STEPS),
    ];
    for (state, label, failed) in cases {
        let (mut front, mut root) = harness::forge_fixture(fixtures::SEED);
        connection(&mut front, &mut root, state);
        connection(
            &mut front,
            &mut root,
            forge::Connection::SetUpFailed { why: b"QA stopped the box".as_slice().into() },
        );
        for address in [
            Address::Welcome { step: Step::Model { way: None } },
            Address::ConnectForge { forge: Some(temper_client_domain::Choice::Forge(OWNED_FORGE)) },
        ] {
            let view = page(&mut front, &mut root, address);
            let tree = Temper::tree(&view);
            let text = words(tree);
            let progress: Vec<_> = tree
                .nodes()
                .iter()
                .enumerate()
                .filter(|(_, node)| node.role == Role::Item)
                .map(|(index, node)| {
                    tree.nodes()
                        .iter()
                        .skip(index)
                        .take(usize::try_from(node.size).expect("bounded subtree"))
                        .flat_map(|child| child.text.iter())
                        .map(|run| String::from_utf8_lossy(&run.text))
                        .collect::<String>()
                })
                .filter(|line| {
                    [
                        "Fetching Forgejo",
                        "Fetched Forgejo",
                        "Preparing:",
                        "Starting Forgejo",
                        "Connecting as temper",
                        "Making the first repository",
                    ]
                    .iter()
                    .any(|prefix| line.contains(prefix))
                })
                .collect();
            assert_eq!(progress.len(), SETUP_STEPS, "{label}: five steps survive: {text:?}");
            assert!(
                text.iter().any(|line| line.contains(&format!("{label} failed: QA stopped the box"))),
                "{label}: failure names its step: {text:?}"
            );
            for (index, line) in progress.iter().enumerate() {
                assert_eq!(line.contains("failed"), index == failed, "{label}: failed status at {index}: {line}");
                assert_eq!(
                    line.contains("done"),
                    failed != SETUP_STEPS && index < failed,
                    "{label}: preceding steps done at {index}: {line}"
                );
            }
        }
    }
}

#[test]
fn unavailable_demo_read_does_not_retire_a_project() {
    let (mut front, mut root) = harness::demo_fixture(fixtures::SEED, &wire::DemoState::Running);
    front.demo = None;
    harness::reopen(&mut front, &mut root, fixtures::chats_page(), false);
    let view = page(&mut front, &mut root, fixtures::chats_page());
    let tree = Temper::tree(&view);
    assert!(
        tree.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Project 1")),
        "an absent demo answer leaves the seeded project available: {:?}",
        words(tree)
    );
}

#[test]
fn first_real_forge_hides_the_playground_before_reload() {
    let (mut front, mut root) = harness::demo_fixture(fixtures::SEED, &wire::DemoState::Running);
    retired(&mut front);
    connection(
        &mut front,
        &mut root,
        forge::Connection::Connected { id: FORGE_WRITER, login: b"qa-bot".as_slice().into() },
    );
    let view = page(&mut front, &mut root, Address::Welcome { step: Step::Model { way: None } });
    let tree = Temper::tree(&view);
    let text = words(tree);
    assert!(
        !tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Try the playground instead")),
        "connected forge retires fallback"
    );
    assert!(
        !text.iter().any(|line| line.contains("Scripted API key")),
        "the live session excludes the retired key: {text:?}"
    );
    assert!(
        tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Use an API key")),
        "real key remains offered: {text:?}"
    );
}

fn retired(front: &mut Joint) {
    let mut parts = front.demo.take().expect("demo fixture").into_parts();
    parts.state = wire::DemoState::Ended;
    front.demo = Some(wire::DemoAnswer::new(&wire::CEILINGS, parts).expect("ended demo answer"));
}

#[test]
fn retired_demo_reload_hides_its_provider_and_welcome_card() {
    let (mut front, mut root) = harness::demo_fixture(fixtures::SEED, &wire::DemoState::NotStarted);
    retired(&mut front);
    harness::reopen(&mut front, &mut root, Address::Welcome { step: Step::Welcome }, false);
    let welcome_view = page(&mut front, &mut root, Address::Welcome { step: Step::Welcome });
    let welcome = Temper::tree(&welcome_view);
    assert!(
        !words(welcome).iter().any(|line| line.contains("Try the playground")),
        "retired playground has no welcome card"
    );
    let model_view = page(&mut front, &mut root, Address::Welcome { step: Step::Model { way: None } });
    let model = Temper::tree(&model_view);
    let text = words(model);
    assert!(!text.iter().any(|line| line.contains("Scripted API key")), "retired provider remains excluded: {text:?}");
    assert!(
        model.nodes().iter().any(|node| node.name.as_deref() == Some(b"Use an API key")),
        "the real provider's single key way remains: {text:?}"
    );
    assert!(
        !model.nodes().iter().any(|node| node.name.as_deref() == Some(b"Try the playground instead")),
        "retired playground has no fallback"
    );
}

#[test]
fn running_demo_project_is_named_playground() {
    let (mut front, mut root) = harness::demo_fixture(fixtures::SEED, &wire::DemoState::Running);
    let view = page(&mut front, &mut root, fixtures::chats_page());
    let tree = Temper::tree(&view);
    assert!(
        tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Playground")),
        "project chooser names the running playground: {:?}",
        words(tree)
    );
}

#[test]
fn retired_demo_reload_drops_the_only_navigation_project() {
    let (mut front, mut root) = harness::demo_fixture(fixtures::SEED, &wire::DemoState::Running);
    retired(&mut front);
    harness::reopen(&mut front, &mut root, fixtures::chats_page(), false);
    let view = page(&mut front, &mut root, fixtures::chats_page());
    let tree = Temper::tree(&view);
    let text = words(tree);
    assert!(text.iter().any(|line| line == "No project yet"), "retired demo leaves no selected project: {text:?}");
    for name in [b"Playground".as_slice(), b"Project 1"] {
        assert!(
            !tree.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(name)),
            "retired project is absent from the chooser"
        );
    }
    for name in [b"Board".as_slice(), b"Changes"] {
        assert!(
            !tree.nodes().iter().any(|node| node.name.as_deref() == Some(name)),
            "retired project offers no {}",
            String::from_utf8_lossy(name)
        );
    }
}
