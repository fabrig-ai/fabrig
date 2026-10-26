use skein_ui::Role;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{self as d, Address, Place};
use temper_client_world::{
    app::Root,
    fixtures,
    front::{Joint, account_acts as accounts},
};
fn view(root: &Root) -> temper_client_view::View {
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(
        &mut view,
        root.person(),
        fixtures::NOW,
        &mut skein_lib::Queue::with_capacity(bounds.tree.patches),
    );
    view
}
fn text(root: &Root) -> String {
    let view = view(root);
    let text: Vec<u8> =
        view.tree().nodes().iter().flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied())).collect();
    String::from_utf8(text).expect("view text")
}
fn go(front: &mut Joint, root: &mut Root, address: Address) {
    accounts::event(front, root, d::Event::Went { place: Place::Address(address) }.into());
}
fn press(front: &mut Joint, root: &mut Root, role: Role, name: &[u8], act: skein_ui::Act) {
    let view = view(root);
    let id = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .expect("flow control")
        .id;
    let event = temper_client_view::event(&view, id, act).expect("flow binding");
    accounts::event(front, root, event.into());
}
#[test]
fn the_banner_key_way_opens_details_and_ready_removes_the_banner() {
    let (mut front, mut root) = accounts::empty_fixture(fixtures::SEED);
    go(&mut front, &mut root, Address::Chats { filter: ChatsFilter::All, start: false });
    assert!(text(&root).contains("Agents need an LLM account to run."));
    press(&mut front, &mut root, Role::Link, b"Use an API key", skein_ui::Act::Press);
    assert_eq!(
        root.person().pages().address(),
        &Address::ConnectModel {
            way: Some(d::ModelWay { source: None, provider: accounts::PROVIDER, kind: d::WayKind::Key })
        }
    );
    assert!(text(&root).contains("Details"));
    press(
        &mut front,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        skein_ui::Act::Input { value: b"test-private-key".as_slice().into() },
    );
    press(&mut front, &mut root, Role::Button, b"Add key", skein_ui::Act::Press);
    accounts::kept(&mut front);
    accounts::settle(&mut front, &mut root);
    assert!(text(&root).contains("Added your Scripted API key"));
    assert!(!format!("{:?}", root.person().pages()).contains("test-private-key"));
    press(&mut front, &mut root, Role::Link, b"Start a chat", skein_ui::Act::Press);
    assert!(!text(&root).contains("Agents need an LLM account to run."));
    accounts::assert_no_carried(&front);
}
#[test]
fn declaring_a_program_login_shows_its_command_on_ready() {
    let (mut front, mut root) = accounts::program_fixture(fixtures::SEED);
    go(&mut front, &mut root, Address::ConnectModel { way: None });
    press(&mut front, &mut root, Role::Link, b"Use your Scripted subscription", skein_ui::Act::Press);
    press(
        &mut front,
        &mut root,
        Role::TextField,
        b"Name",
        skein_ui::Act::Input { value: b"claude-alice".as_slice().into() },
    );
    press(&mut front, &mut root, Role::Button, b"Use your Scripted subscription", skein_ui::Act::Press);
    assert!(text(&root).contains("Declared your Scripted subscription · claude-alice"));
    assert!(text(&root).contains("temper program-login claude-alice"));
    assert!(text(&root).contains("Claude Code asks you to sign in. temper never reads its login."));
}
#[test]
fn a_creation_key_is_absent_from_the_small_store() {
    use jig_client_world::harness::Until;
    use skein_fake_person::{Find, Step};
    let mut world = fixtures::model_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, Address::ConnectModel { way: None });
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Link, b"Use an API key"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"Use an API key") },
            Step::See { find: Find::exactly_named(Role::Button, b"Add key"), within: fixtures::WITHIN },
            Step::Type {
                find: Find::exactly_named(Role::TextField, b"Key (pasted once)"),
                words: b"test-private-key".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Add key") },
            Step::See { find: Find::text(b"Added your Scripted API key"), within: fixtures::WITHIN },
        ],
    );
    let mut kept = false;
    for _ in 0..4096 {
        world.run(Until::Steps(2));
        if world.front().engine.protocol.accounts.iter().any(|request| {
            matches!(request, jig_domain_accounts::Request::Signing(jig_domain_accounts::Signing::Keep { .. }))
        }) {
            accounts::kept(world.front());
            kept = true;
            break;
        }
    }
    assert!(kept, "the fake credential store received the carried key");
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(world.kept(person).is_empty(), "a creation key has no small-store record");
}

fn start_plan_sign_in(front: &mut Joint, root: &mut Root) -> u32 {
    go(front, root, Address::ConnectModel { way: None });
    press(front, root, Role::Button, b"Continue with ChatGPT", skein_ui::Act::Press);
    let source = accounts::signing(front);
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::Visited {
        source,
        address: b"https://issuer/visit".as_slice().into(),
    }));
    accounts::settle(front, root);
    source
}
fn returned_path(source: u32) -> Vec<u8> {
    format!("/accounts/connect?provider={}&way=sign-in&source={source}", accounts::PROVIDER).into_bytes()
}
#[test]
fn a_departing_page_returns_then_leaves_once_and_a_rebuild_asks_nothing() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    accounts::reopen(&mut front, &mut root, Address::ConnectModel { way: None }, true);
    let source = start_plan_sign_in(&mut front, &mut root);
    assert_eq!(front.engine.protocol.returns, [(source, returned_path(source).into_boxed_slice())]);
    assert_eq!(front.leaves, [Box::from(b"https://issuer/visit".as_slice())]);
    assert!(text(&root).contains("Signing in"));
    view(&root);
    accounts::settle(&mut front, &mut root);
    assert_eq!(front.leaves, [Box::from(b"https://issuer/visit".as_slice())]);
    assert_eq!(front.engine.protocol.returns, [(source, returned_path(source).into_boxed_slice())]);
}
#[test]
fn a_refused_return_keeps_the_page_and_its_sign_in_link() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    accounts::reopen(&mut front, &mut root, Address::ConnectModel { way: None }, true);
    front.refuse_return = true;
    start_plan_sign_in(&mut front, &mut root);
    assert!(front.leaves.is_empty());
    assert!(view(&root).tree().nodes().iter().any(|node| node.external.as_deref() == Some(b"https://issuer/visit")));
}
#[test]
fn a_non_departing_page_asks_no_return_and_never_leaves() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    start_plan_sign_in(&mut front, &mut root);
    assert!(front.engine.protocol.returns.is_empty());
    assert!(front.leaves.is_empty());
}
#[test]
fn a_fresh_page_at_the_return_address_follows_the_watch_to_ready_without_departing() {
    use d::pages::accounts::SignInState;
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    let source = start_plan_sign_in(&mut front, &mut root);
    let Place::Address(address) = d::address::parse(&returned_path(source)) else { panic!("returned address") };
    accounts::reopen(&mut front, &mut root, address, true);
    assert_eq!(
        root.person()
            .pages()
            .connect_model()
            .expect("connect model page")
            .request
            .sign_in_state(root.person().client()),
        SignInState::Signing { source }
    );
    assert!(text(&root).contains("Signing in"));
    assert!(
        !view(&root)
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT"))
    );
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: b"alice@example.com".as_slice().into(),
        declined: false,
    }));
    accounts::settle(&mut front, &mut root);
    assert!(text(&root).contains("Signed in to ChatGPT as alice@example.com"));
    assert_eq!(
        root.person()
            .pages()
            .connect_model()
            .expect("connect model page")
            .request
            .sign_in_state(root.person().client()),
        SignInState::Ended { source }
    );
    assert!(front.engine.protocol.returns.is_empty());
    assert!(front.leaves.is_empty());
}
fn departing_welcome_sign_in() -> (Joint, Root, u32) {
    let (mut front, mut root) = accounts::empty_plan_fixture(fixtures::SEED);
    accounts::reopen(&mut front, &mut root, Address::Welcome { step: d::Step::Model { way: None } }, true);
    press(&mut front, &mut root, Role::Button, b"Continue with ChatGPT", skein_ui::Act::Press);
    let source = accounts::signing(&front);
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::Visited {
        source,
        address: b"https://issuer/visit".as_slice().into(),
    }));
    accounts::settle(&mut front, &mut root);
    (front, root, source)
}
#[test]
fn a_departing_welcome_model_step_returns_to_ready() {
    let (front, _, source) = departing_welcome_sign_in();
    assert_eq!(front.engine.protocol.returns, [(source, Box::from(b"/welcome?at=ready".as_slice()))]);
    assert_eq!(front.leaves, [Box::from(b"https://issuer/visit".as_slice())]);
}
#[test]
fn a_welcome_sign_in_come_back_at_ready_reads_it_under_way_then_its_account() {
    const SUBJECT: &[u8] = b"subject";
    const DISPLAY: &[u8] = b"alice@example.com";
    let (mut front, mut root, source) = departing_welcome_sign_in();
    let (_, returned) = front.engine.protocol.returns.last().expect("welcome return");
    let Place::Address(address) = d::address::parse(returned) else { panic!("returned welcome address") };
    accounts::reopen(&mut front, &mut root, address, true);
    assert!(text(&root).contains("Model: ChatGPT · signing in"));
    assert!(!text(&root).contains("Connect a model"));
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::SignedIn {
        source,
        subject: SUBJECT.into(),
        display: DISPLAY.into(),
        declined: false,
    }));
    accounts::settle(&mut front, &mut root);
    assert!(text(&root).contains("Model: ChatGPT · alice@example.com · OpenAI · held by Person · scripted"));
    assert!(!text(&root).contains("Connect a model"));
}
#[test]
fn an_unknown_return_source_is_removed_and_the_way_can_start_again() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    let Place::Address(address) = d::address::parse(&returned_path(u32::MAX)) else { panic!("returned address") };
    accounts::reopen(&mut front, &mut root, address, true);
    assert_eq!(
        d::address::print(root.person().pages().address()).as_ref(),
        format!("/accounts/connect?provider={}&way=sign-in", accounts::PROVIDER).as_bytes()
    );
    assert!(
        view(&root)
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT"))
    );
    assert!(front.leaves.is_empty());
    assert!(front.engine.protocol.returns.is_empty());
}
