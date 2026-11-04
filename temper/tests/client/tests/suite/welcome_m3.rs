//! Welcome scenarios from client README 3.5/5.5/5.6 and forge 5.4.
//! The real client and engine are driven only through the public Joint.
use jig_client_world::harness::App;
use skein_lib::Queue;
use skein_ui::{Act, Role, Tree};
use temper_client_domain::{Address, Choice, Step};
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures,
    front::{Joint, account_acts as h},
};

fn draw(root: &Root) -> View {
    let (_, mut view) = Temper::new(fixtures::SEED);
    let mut patches = Queue::with_capacity(Temper::view_limits(&view).patches);
    Temper::build(&mut view, root, &mut patches);
    view
}
fn text(tree: &Tree) -> String {
    tree.nodes()
        .iter()
        .flat_map(|n| n.text.iter())
        .map(|r| String::from_utf8_lossy(&r.text))
        .collect::<Vec<_>>()
        .join("")
}
fn has(root: &Root, role: Role, name: &[u8]) -> bool {
    Temper::tree(&draw(root)).nodes().iter().any(|n| n.role == role && n.name.as_deref() == Some(name))
}
fn act(front: &mut Joint, root: &mut Root, role: Role, name: &[u8], action: Act) {
    let mut view = draw(root);
    let matches = Temper::tree(&view)
        .nodes()
        .iter()
        .filter(|n| n.role == role && n.name.as_deref() == Some(name))
        .map(|n| n.id)
        .collect::<Vec<_>>();
    assert_eq!(
        if name == b"Adopt a repository" { matches.len().min(1) } else { matches.len() },
        1,
        "unique {:?} {:?}\n{}",
        role,
        String::from_utf8_lossy(name),
        text(Temper::tree(&view))
    );
    let event = Temper::act(&mut view, matches[0], action).expect("offered act decodes");
    h::event(front, root, event);
}
fn press(front: &mut Joint, root: &mut Root, role: Role, name: &[u8]) {
    act(front, root, role, name, Act::Press);
}
fn go(front: &mut Joint, root: &mut Root, at: Address) {
    h::event(front, root, Temper::went(at));
}
fn welcome(step: Step) -> Address {
    Address::Welcome { step }
}

#[test]
fn brand_returns_to_chats_from_every_welcome_step_without_starting_the_playground() {
    for step in [Step::Welcome, Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready] {
        let (mut front, mut root) = h::playground_fixture(fixtures::SEED, false);
        go(&mut front, &mut root, welcome(step));
        assert!(has(&root, Role::Link, b"temper chats"));
        assert!(!has(&root, Role::Link, b"Inbox"));
        press(&mut front, &mut root, Role::Link, b"temper chats");
        assert!(has(&root, Role::Link, b"Accounts"), "{}", text(Temper::tree(&draw(&root))));
        assert!(has(&root, Role::Heading { level: 1 }, b"Chats"), "{}", text(Temper::tree(&draw(&root))));
        assert!(front.playground_starts.is_empty(), "brand must only navigate");
    }
}

#[test]
fn setup_model_sentence_names_supported_plans_for_different_providers() {
    for (fixture, empty) in [
        (h::empty_plan_fixture as fn(u64) -> (Joint, Root), true),
        (h::program_fixture as fn(u64) -> (Joint, Root), false),
    ] {
        let (mut front, mut root) = fixture(fixtures::SEED);
        go(&mut front, &mut root, welcome(Step::Welcome));
        if !empty {
            assert_eq!(root.person().pages().address(), &welcome(Step::Forge { forge: None }));
            continue;
        }
        assert_eq!(root.person().pages().address(), &welcome(Step::Welcome));
        let shown = text(Temper::tree(&draw(&root)));
        let expected = "Your model: ChatGPT or Claude";
        assert!(shown.contains(expected), "expected {expected:?}\n{shown}");
        assert_eq!(shown.matches("Your model:").count(), 1);
        assert!(!shown.contains("API key"), "{shown}");
        assert!(!shown.contains(&format!("{expected}.")), "{shown}");
    }
}

#[test]
fn setup_model_sentence_keeps_supported_plans_in_the_playground() {
    let (front, root) = h::playground_fixture(fixtures::SEED, false);
    let shown = text(Temper::tree(&draw(&root)));
    assert!(shown.contains("Your model: ChatGPT or Claude"), "{shown}");
    assert!(!shown.contains("Other fake"), "{shown}");
    assert!(!shown.contains("Scripted"), "{shown}");
    assert!(front.playground_starts.is_empty());
}

#[test]
fn github_ready_offers_adoption_from_the_form_without_reading_repositories() {
    let (mut front, mut root) = Joint::github_fixture(fixtures::SEED);
    assert!(has(&root, Role::Heading { level: 2 }, b"GitHub is connected"));
    assert!(front.reachable_reads.is_empty());
    assert!(!front.engine_adopted(1, 7));
    press(&mut front, &mut root, Role::Link, b"Adopt a repository");
    assert!(has(&root, Role::TextField, b"Owner"));
    assert!(has(&root, Role::TextField, b"Name"));
}
#[test]
fn github_welcome_connection_advances_to_model_without_adoption() {
    let (mut front, mut root) = Joint::github_fixture(fixtures::SEED);
    h::reopen(&mut front, &mut root, welcome(Step::Forge { forge: Some(Choice::Forge(1)) }), false);
    assert!(has(&root, Role::Heading { level: 1 }, b"Connect a model"));
    assert!(!front.engine_adopted(1, 8));
    go(&mut front, &mut root, welcome(Step::Ready));
    assert!(has(&root, Role::Heading { level: 1 }, b"Almost ready"));
    assert!(has(&root, Role::Button, b"Open Default"));
}
