//! First-run gaps at the public fake boundary (client/README.md, 3.5, 4.8, 5.6).
use jig_client_world::harness::App;
use skein_lib::Queue;
use skein_ui::{Role, Tree};
use temper_client_domain::{Address, Choice, Step};
use temper_client_protocol_wire::v1 as wire;
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures,
    front::{Joint, account_acts as harness},
};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

fn draw(root: &Root) -> View {
    let (_, mut view) = Temper::new(fixtures::SEED);
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, root, &mut patches);
    view
}

fn words(tree: &Tree) -> String {
    tree.nodes().iter().flat_map(|node| &node.text).map(|run| String::from_utf8_lossy(&run.text)).collect()
}

fn named(tree: &Tree, role: Role, name: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}

fn act(front: &mut Joint, root: &mut Root, role: Role, name: &[u8], action: skein_ui::Act) {
    let mut view = draw(root);
    let id = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| {
            panic!(
                "missing {}: {:?}",
                String::from_utf8_lossy(name),
                Temper::tree(&view)
                    .nodes()
                    .iter()
                    .map(|node| (node.role, node.name.as_deref().map(String::from_utf8_lossy)))
                    .collect::<Vec<_>>()
            )
        })
        .id;
    let event = Temper::act(&mut view, id, action).expect("bound control");
    harness::event(front, root, event);
}

fn press(front: &mut Joint, root: &mut Root, role: Role, name: &[u8]) {
    act(front, root, role, name, skein_ui::Act::Press);
}

fn connection(front: &mut Joint, root: &mut Root, state: forge::Connection) {
    front.engine.send(engine::Event::Forge(Box::new(forge::Event::Connection { forge: 1, state })));
    front.engine.settle();
    harness::settle(front, root);
}

fn connected(front: &mut Joint, root: &mut Root) {
    connection(front, root, forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() });
}

fn failed_steps(tree: &Tree) {
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
                .flat_map(|child| &child.text)
                .map(|run| String::from_utf8_lossy(&run.text))
                .collect::<String>()
        })
        .filter(|line| {
            ["Fetched Forgejo", "Preparing:", "Starting Forgejo", "Connecting as temper", "Making acme/shop,"]
                .iter()
                .any(|prefix| line.contains(prefix))
        })
        .collect();
    assert_eq!(progress.len(), 5, "five steps: {}", words(tree));
    for (index, line) in progress.iter().enumerate() {
        assert_eq!(line.contains("done"), index < 4, "done at {index}: {line}");
        assert_eq!(line.contains("failed"), index == 4, "failed at {index}: {line}");
    }
    assert!(words(tree).contains("Making acme/shop failed: making the repository"), "{}", words(tree));
}

#[test]
fn qa_fifth_setup_step_fails_on_welcome_and_forges() {
    let (mut front, mut root) = harness::forge_fixture(fixtures::SEED);
    harness::event(&mut front, &mut root, Temper::went(Address::ConnectForge { forge: Some(Choice::Forge(1)) }));
    for (name, value) in [(b"Organization".as_slice(), b"acme".as_slice()), (b"Repository", b"shop")] {
        act(&mut front, &mut root, Role::TextField, name, skein_ui::Act::Input { value: value.into() });
    }
    press(&mut front, &mut root, Role::Button, b"Set up a Forgejo here");
    harness::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Model { way: None } }));
    let (forge, request) = *front.set_ups().last().expect("setup reached the fake host");
    front.engine.send(engine::Event::Forge(Box::new(forge::Event::SetUpAnswered {
        forge,
        request,
        answer: forge::SetUpAnswer::SettingUp,
    })));
    for step in [forge::SetUpStep::Fetch, forge::SetUpStep::Prepare, forge::SetUpStep::Run] {
        connection(&mut front, &mut root, forge::Connection::SettingUp { step: Some(step) });
        assert!(named(Temper::tree(&draw(&root)), Role::Disclosure, b"Each step"));
    }
    connection(&mut front, &mut root, forge::Connection::Connecting);
    connected(&mut front, &mut root);
    front.fail_first_repository(forge);
    harness::settle(&mut front, &mut root);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Model { way: None } });
    failed_steps(Temper::tree(&draw(&root)));
    harness::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Ready }));
    let ready = draw(&root);
    assert!(words(Temper::tree(&ready)).contains("It opens once acme/shop is made."));
    assert!(Temper::tree(&ready).nodes().iter().any(|node| {
        node.role == Role::Button
            && node.name.as_deref().is_some_and(|name| name.starts_with(b"Open "))
            && node.states.as_slice().contains(&skein_ui::State::Disabled)
    }));
    harness::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Model { way: None } }));
    // Model remains available, and Later opens Chats; Forges exposes the failed setup's retry.
    assert!(named(Temper::tree(&draw(&root)), Role::Link, b"Later"));
    press(&mut front, &mut root, Role::Link, b"Later");
    press(&mut front, &mut root, Role::Link, b"Forges");
    let view = draw(&root);
    assert!(named(Temper::tree(&view), Role::Button, b"Set up again"), "{}", words(Temper::tree(&view)));
    // Forges draws no set-up steps for any failure yet (the Forges page's own look, not this row's).
    assert!(words(Temper::tree(&view)).contains("making the repository"), "{}", words(Temper::tree(&view)));
}

fn ended(front: &mut Joint) {
    let mut parts = front.demo.take().expect("demo fixture").into_parts();
    parts.state = wire::DemoState::Ended;
    front.demo = Some(wire::DemoAnswer::new(&wire::CEILINGS, parts).expect("ended answer"));
}

fn no_playground(front: &mut Joint, root: &mut Root) {
    for address in [
        fixtures::chats_page(),
        Address::Welcome { step: Step::Welcome },
        Address::Welcome { step: Step::Model { way: None } },
    ] {
        harness::event(front, root, Temper::went(address.clone()));
        let view = draw(root);
        let tree = Temper::tree(&view);
        assert!(!words(tree).contains("Try the playground"), "{}", words(tree));
        assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Playground")));
        if address == fixtures::chats_page() {
            assert!(words(tree).contains("No project yet"), "{}", words(tree));
            assert!(!named(tree, Role::Button, b"Project 1"));
        }
        assert_eq!(root.person().pages().project(), None);
    }
}

#[test]
fn qa_stale_demo_read_cannot_restore_a_retired_playground() {
    for fresh_first in [false, true] {
        let (mut front, mut root) = harness::demo_fixture(fixtures::SEED, &wire::DemoState::NotStarted);
        let reads = front.demo_reads;
        front.hold_next_demo_read();
        press(&mut front, &mut root, Role::Button, b"Try the playground");
        assert_eq!(front.demo_reads, reads + 1, "post-start Running read was sent and held");
        ended(&mut front);
        if !fresh_first {
            front.hold_next_demo_read();
        }
        connected(&mut front, &mut root);
        assert_eq!(front.demo_reads, reads + 2, "retirement sends a newer read");
        no_playground(&mut front, &mut root);
        assert!(front.release_demo_read(), "release the older Running answer");
        harness::settle(&mut front, &mut root);
        no_playground(&mut front, &mut root);
        if !fresh_first {
            assert!(front.release_demo_read(), "release the newer Ended answer");
            harness::settle(&mut front, &mut root);
            no_playground(&mut front, &mut root);
        }
        assert!(!front.release_demo_read(), "all scripted answers delivered");
    }
}

fn surviving_projects(root: &Root) {
    let view = draw(root);
    let tree = Temper::tree(&view);
    for name in [b"Project 2".as_slice(), b"Project 3"] {
        assert!(named(tree, Role::Button, name), "both real projects remain: {}", words(tree));
    }
    for node in tree
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Button && matches!(node.name.as_deref(), Some(b"Project 2" | b"Project 3")))
    {
        assert_eq!(
            node.states.as_slice().contains(&skein_ui::State::Pressed),
            node.name.as_deref() == Some(b"Project 2")
        );
    }
    assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Playground")));
    assert!(!named(tree, Role::Button, b"Project 1"));
    assert_eq!(root.person().pages().project(), Some(2), "observed choice: first surviving project");
}

#[test]
fn qa_retirement_selects_among_two_surviving_projects_live_and_after_reload() {
    let (mut front, mut root) = harness::demo_fixture_with_projects(fixtures::SEED);
    harness::event(&mut front, &mut root, Temper::went(fixtures::chats_page()));
    assert!(named(Temper::tree(&draw(&root)), Role::Button, b"Playground"));
    ended(&mut front);
    connected(&mut front, &mut root);
    surviving_projects(&root);
    harness::reopen(&mut front, &mut root, fixtures::chats_page(), false);
    surviving_projects(&root);
}
