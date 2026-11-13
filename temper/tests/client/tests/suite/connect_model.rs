use skein_ui::Role;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{self as d, Address, Place};
use temper_client_world::{
    app::Root,
    fixtures,
    front::{Joint, account_acts as accounts},
};
// The joined world supplies this address; no test visits a real issuer.
const ISSUER: &[u8] = b"https://issuer/visit";

fn view(root: &Root) -> temper_client_view::View {
    let limits = fixtures::domain_limits();
    let words = temper_client_view::words::Words::english(
        fixtures::FORGE,
        limits.client.domain.text,
        temper_client_view::words::Shell::Browser,
    )
    .expect("browser words fit");
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
    go(&mut front, &mut root, Address::Chats { filter: ChatsFilter::All });
    assert!(text(&root).contains("Agents need an LLM account to run."));
    press(&mut front, &mut root, Role::Link, b"Use an API key", skein_ui::Act::Press);
    assert_eq!(
        root.person().pages().address(),
        &Address::ConnectModel {
            way: Some(d::ModelWay { source: None, provider: accounts::PROVIDER, kind: d::WayKind::Key })
        }
    );
    assert!(text(&root).contains("A Scripted API key"));
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
    accounts::answer_trial(&mut front, &mut root);
    assert!(text(&root).contains("Added your Scripted API key"));
    assert!(!format!("{:?}", root.person().pages()).contains("test-private-key"));
    press(&mut front, &mut root, Role::Link, b"Start a chat", skein_ui::Act::Press);
    assert!(!text(&root).contains("Agents need an LLM account to run."));
    accounts::assert_no_carried(&front);
}
#[test]
fn a_declared_program_lands_after_its_own_check() {
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
    press(&mut front, &mut root, Role::Button, "I’ve signed in".as_bytes(), skein_ui::Act::Press);
    assert_eq!(
        root.person().pages().address(),
        &Address::ConnectModel {
            way: Some(d::ModelWay { source: None, provider: accounts::PROVIDER, kind: d::WayKind::Program })
        }
    );
    assert!(text(&root).contains("temper program-login claude-alice"));
    accounts::program_checked(&mut front, &mut root, true);
    accounts::answer_trial(&mut front, &mut root);
    assert_eq!(root.person().pages().address(), &Address::Accounts);
    assert!(text(&root).contains("Added your Scripted subscription. temper’s agents now run on it."));
}
#[test]
fn a_creation_key_is_absent_from_the_small_store() {
    use jig_client_kit::harness::Until;
    use skein_fake_person::{Find, Step};
    let mut world = fixtures::model_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, Address::ConnectModel { way: None });
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Link, b"Add key"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"Add key") },
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

fn ask_plan_sign_in(front: &mut Joint, root: &mut Root) -> u32 {
    go(front, root, Address::ConnectModel { way: None });
    press(front, root, Role::Button, b"Continue with ChatGPT", skein_ui::Act::Press);
    accounts::signing(front)
}
fn start_plan_sign_in(front: &mut Joint, root: &mut Root) -> u32 {
    let source = ask_plan_sign_in(front, root);
    visit(front, root, source);
    source
}
fn returned_path(source: u32) -> Vec<u8> {
    format!("/accounts/connect?provider={}&way=sign-in&source={source}", accounts::PROVIDER).into_bytes()
}
#[test]
fn a_press_opens_its_tab_before_any_answer() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    go(&mut front, &mut root, Address::ConnectModel { way: None });
    let view = view(&root);
    let id = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT"))
        .expect("sign-in button")
        .id;
    let event = temper_client_view::event(&view, id, skein_ui::Act::Press).expect("press");
    accounts::press_turn(&mut front, &mut root, event.into());
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }]);
    assert!(
        root.person()
            .pages()
            .connect_model()
            .expect("page")
            .request
            .pending
            .as_ref()
            .expect("pending")
            .outcome
            .is_none()
    );
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn an_answer_points_once_and_replaces_the_page_address() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    let source = start_plan_sign_in(&mut front, &mut root);
    let expected = [d::Tab::Open { provider: accounts::PROVIDER }, d::Tab::Point { to: ISSUER.into() }];
    assert_eq!(front.tabs, expected);
    assert_eq!(d::address::print(root.person().pages().address()).as_ref(), returned_path(source));
    assert_eq!(front.shows.last(), Some(&(root.person().pages().address().clone(), true)));
    assert_eq!(front.leaves.len(), 0);
    view(&root);
    accounts::settle(&mut front, &mut root);
    // An unrelated answer must not replay the sign-in's tab action.
    go(&mut front, &mut root, Address::Forges);
    accounts::settle(&mut front, &mut root);
    assert_eq!(front.tabs, expected);
}
fn unanswered_sign_in() -> (Joint, Root, u32) {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    let source = ask_plan_sign_in(&mut front, &mut root);
    (front, root, source)
}
fn visited(front: &mut Joint, source: u32) {
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::Visited {
        source,
        address: ISSUER.into(),
    }));
}
fn visit(front: &mut Joint, root: &mut Root, source: u32) {
    visited(front, source);
    accounts::settle(front, root);
}
#[test]
fn an_answer_after_leaving_closes_the_tab() {
    let (mut front, mut root, source) = unanswered_sign_in();
    go(&mut front, &mut root, Address::Accounts);
    visit(&mut front, &mut root, source);
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }, d::Tab::Close]);
    assert_eq!(root.person().pages().address(), &Address::Accounts);
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn a_refused_sign_in_closes_the_tab_and_explains_why() {
    let (mut front, mut root, source) = unanswered_sign_in();
    visited(&mut front, source);
    accounts::refused_answer(&mut front, &mut root);
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }, d::Tab::Close]);
    assert!(text(&root).contains("not offered"), "{}", text(&root));
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn a_lost_sign_in_closes_the_tab_and_explains_why() {
    let (mut front, mut root, source) = unanswered_sign_in();
    accounts::lose_next(&mut front);
    visit(&mut front, &mut root, source);
    accounts::lost(&mut front, &mut root);
    accounts::expire_answer(&mut front, &mut root);
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }, d::Tab::Close]);
    assert!(text(&root).contains("Waiting for current account facts."), "{}", text(&root));
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn a_refused_tab_keeps_the_address_without_pointing() {
    let (mut front, mut root, source) = unanswered_sign_in();
    let shown = d::shown(root.person());
    accounts::event(&mut front, &mut root, d::Event::TabRefused.into());
    assert!(d::shown(root.person()) > shown, "the tab refusal redraws the page");
    assert!(root.person().pages().connect_model().expect("page").tab_refused());
    visit(&mut front, &mut root, source);
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }]);
    assert_eq!(
        root.person().pages().connect_model().expect("page").request.sign_in_address(root.person().client()),
        Some(ISSUER)
    );
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn a_fresh_page_at_the_return_address_lands_on_accounts() {
    use d::pages::accounts::SignInState;
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    let source = start_plan_sign_in(&mut front, &mut root);
    let Place::Address(address) = d::address::parse(&returned_path(source)) else { panic!("returned address") };
    accounts::reopen(&mut front, &mut root, address);
    assert_eq!(
        root.person()
            .pages()
            .connect_model()
            .expect("connect model page")
            .request
            .sign_in_state(root.person().client()),
        SignInState::Signing { source }
    );
    assert!(text(&root).contains("Your ChatGPT plan"));
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
        said: jig_domain_accounts::Said::default(),
    }));
    accounts::settle(&mut front, &mut root);
    accounts::answer_trial(&mut front, &mut root);
    assert!(text(&root).contains("Added your ChatGPT plan. temper’s agents now run on it."));
    assert_eq!(root.person().pages().address(), &Address::Accounts);
    assert_eq!(front.shows.last(), Some(&(Address::Accounts, true)));
    assert!(root.person().pages().accounts().unwrap().added.is_some());
    accounts::reopen(&mut front, &mut root, Address::Accounts);
    assert!(root.person().pages().accounts().unwrap().added.is_none());
    assert!(!text(&root).contains("Added your ChatGPT plan."));
    assert_eq!(front.leaves.len(), 0);
}
fn welcome_sign_in() -> (Joint, Root, u32) {
    let (mut front, mut root) = accounts::empty_plan_fixture(fixtures::SEED);
    accounts::reopen(&mut front, &mut root, Address::Welcome { step: d::Step::Model { way: None } });
    press(&mut front, &mut root, Role::Button, b"Continue with ChatGPT", skein_ui::Act::Press);
    let source = accounts::signing(&front);
    visit(&mut front, &mut root, source);
    (front, root, source)
}
#[test]
fn a_welcome_model_answer_names_its_source_and_holds() {
    let (mut front, mut root, source) = welcome_sign_in();
    let expected = [d::Tab::Open { provider: accounts::PROVIDER }, d::Tab::Point { to: ISSUER.into() }];
    assert_eq!(front.tabs, expected);
    assert_eq!(
        root.person().pages().address(),
        &Address::Welcome {
            step: d::Step::Model {
                way: Some(d::ModelWay { provider: accounts::PROVIDER, kind: d::WayKind::SignIn, source: Some(source) })
            }
        }
    );
    assert_eq!(front.shows.last(), Some(&(root.person().pages().address().clone(), true)));
    view(&root);
    accounts::settle(&mut front, &mut root);
    assert_eq!(front.tabs, expected);
    assert_eq!(front.leaves.len(), 0);
}
fn welcome_signed_in(front: &mut Joint, root: &mut Root, source: u32, declined: bool) {
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: b"alice@example.com".as_slice().into(),
        declined,
        said: jig_domain_accounts::Said::default(),
    }));
    accounts::settle(front, root);
}
fn usable_sign_in_account(root: &Root, source: u32) -> u32 {
    let document = d::pages::accounts::document(root.person().client()).expect("engine account facts");
    assert!(document.usable, "the sign-in left a usable model");
    let [account] = document.accounts.as_slice() else { panic!("the sign-in made one account") };
    assert_eq!(account.provider, Some(accounts::PROVIDER));
    assert!(account.plan);
    let [sign_in] = account.sources.as_slice() else { panic!("the account has one sign-in source") };
    assert_eq!(sign_in.number, source);
    assert_eq!(sign_in.kind, jig_client::contract::AccountKind::SignIn);
    assert!(sign_in.usable, "the sign-in's source is usable");
    account.number
}
fn welcome_moves_on(front: &mut Joint, root: &mut Root, source: u32) {
    let document = d::pages::accounts::document(root.person().client()).expect("engine account facts before sign-in");
    assert!(!document.usable, "no model is usable before sign-in");
    assert!(document.accounts.iter().all(|account| account.sources.iter().all(|source| !source.usable)));
    let shown = front.shows.len();
    welcome_signed_in(front, root, source, false);
    let account = usable_sign_in_account(root, source);
    let ready = Address::Welcome { step: d::Step::Ready };
    assert_eq!(root.person().pages().address(), &ready);
    assert_eq!(&front.shows[shown..], &[(ready, true)]);
    accounts::answer_trial(front, root);
    assert!(
        view(root).tree().nodes().iter().any(|node| {
            node.role == Role::Paragraph
                && node.name.as_deref() == Some("ChatGPT plan · alice@example.com · answered in 1 s".as_bytes())
        }),
        "{}",
        text(root)
    );
    accounts::settle(front, root);
    assert_eq!(front.shows.len(), shown + 1);
    assert_eq!(front.leaves.len(), 0);
    accounts::reopen(front, root, Address::Accounts);
    assert_eq!(usable_sign_in_account(root, source), account);
}
#[test]
fn a_held_welcome_sign_in_moves_on_once_when_the_watch_makes_its_account() {
    let (mut front, mut root, source) = welcome_sign_in();
    welcome_moves_on(&mut front, &mut root, source);
}
#[test]
fn a_reloaded_welcome_sign_in_moves_on_once_when_the_watch_makes_its_account() {
    let (mut front, mut root, source) = welcome_sign_in();
    let path = d::address::print(root.person().pages().address());
    let Place::Address(address) = d::address::parse(&path) else { panic!("welcome source address") };
    accounts::reopen(&mut front, &mut root, address);
    assert_eq!(
        root.person().pages().connect_model().expect("model page").request.sign_in_state(root.person().client()),
        d::pages::accounts::SignInState::Signing { source }
    );
    welcome_moves_on(&mut front, &mut root, source);
}
#[test]
fn a_welcome_sign_in_ending_without_a_usable_model_keeps_its_words_and_acts() {
    for declined in [true, false] {
        let (mut front, mut root, source) = welcome_sign_in();
        let address = Address::Welcome {
            step: d::Step::Model {
                way: Some(d::ModelWay { provider: accounts::PROVIDER, kind: d::WayKind::SignIn, source: Some(source) }),
            },
        };
        let shown = front.shows.len();
        let words = if declined {
            welcome_signed_in(&mut front, &mut root, source, true);
            "You signed in without plan usage."
        } else {
            front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::SignInFailed {
                source,
                failure: jig_domain_accounts::Failure::Misconfigured,
            }));
            accounts::settle(&mut front, &mut root);
            "OpenAI refused temper’s sign-in. Check the configuration’s registration."
        };
        assert_eq!(root.person().pages().address(), &address);
        assert_eq!(front.shows.len(), shown);
        assert!(text(&root).contains(words), "{}", text(&root));
        let view = view(&root);
        assert!(
            view.tree()
                .nodes()
                .iter()
                .any(|node| { node.role == Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT") })
        );
        if declined {
            assert!(
                view.tree()
                    .nodes()
                    .iter()
                    .any(|node| { node.role == Role::Link && node.name.as_deref() == Some(b"Use an API key") })
            );
        }
        assert_eq!(front.leaves.len(), 0);
    }
}
#[test]
fn a_typed_welcome_sign_in_way_with_a_usable_model_keeps_continue() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    let Place::Address(address) =
        d::address::parse(format!("/welcome?at=model&provider={}&way=sign-in", accounts::PROVIDER).as_bytes())
    else {
        panic!("welcome sign-in way");
    };
    accounts::reopen(&mut front, &mut root, address.clone());
    assert_eq!(d::pages::welcome::model_usable(root.person()), Some(true));
    assert_eq!(root.person().pages().address(), &address);
    assert!(!front.shows.iter().any(|(address, _)| *address == Address::Welcome { step: d::Step::Ready }));
    assert!(text(&root).contains("Connect a model"));
    assert!(
        view(&root)
            .tree()
            .nodes()
            .iter()
            .any(|node| { node.role == Role::Link && node.name.as_deref() == Some(b"Continue") })
    );
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn a_typed_welcome_ready_reads_the_sign_in_then_its_account() {
    const SUBJECT: &[u8] = b"subject";
    const DISPLAY: &[u8] = b"alice@example.com";
    let (mut front, mut root, source) = welcome_sign_in();
    accounts::reopen(&mut front, &mut root, Address::Welcome { step: d::Step::Ready });
    assert!(text(&root).contains("Model ChatGPT plan · signing in…"));
    assert!(!text(&root).contains("Connect a model"));
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::SignedIn {
        source,
        subject: SUBJECT.into(),
        display: DISPLAY.into(),
        declined: false,
        said: jig_domain_accounts::Said::default(),
    }));
    accounts::settle(&mut front, &mut root);
    accounts::answer_trial(&mut front, &mut root);
    assert!(view(&root).tree().nodes().iter().any(|node| node.role == Role::Paragraph
        && node.name.as_deref() == Some("ChatGPT plan · alice@example.com · answered in 1 s".as_bytes())));
    assert!(!text(&root).contains("Connect a model"));
}
#[test]
fn an_unknown_return_source_is_removed_and_the_way_can_start_again() {
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    let Place::Address(address) = d::address::parse(&returned_path(u32::MAX)) else { panic!("returned address") };
    accounts::reopen(&mut front, &mut root, address);
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
    assert_eq!(front.leaves.len(), 0);
}

#[test]
fn welcome_claude_checks_on_the_wire_from_ready_and_observes_the_host_answer() {
    use jig_client::contract::{AccountChange, Ask, AskAccountHolder};
    let (mut front, mut root) = accounts::empty_program_fixture(fixtures::SEED);
    front.deployment = (true, true);
    let address = Address::Welcome {
        step: d::Step::Model {
            way: Some(d::ModelWay { provider: accounts::PROVIDER, kind: d::WayKind::Program, source: None }),
        },
    };
    accounts::reopen(&mut front, &mut root, address.clone());
    assert!(text(&root).contains("temper program-login claude-person"), "{}", text(&root));
    let view = view(&root);
    let button = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some("I’ve signed in".as_bytes()))
        .expect("sign-in button")
        .id;
    let event = temper_client_view::event(&view, button, skein_ui::Act::Press).expect("check binding");
    let sent = accounts::event_traced(&mut front, &mut root, event.into());
    let [
        Ask::AccountChange { holder: AskAccountHolder::Own, change: AccountChange::DeclareProgram { name, .. } },
        Ask::AccountChange { holder: AskAccountHolder::Own, change: AccountChange::CheckProgram { source } },
    ] = sent.as_slice()
    else {
        panic!("declare then check on the wire: {sent:?}");
    };
    assert_eq!(name.as_ref(), b"claude-person");
    assert_eq!(
        root.person().pages().connect_model().unwrap().request.pending.as_ref().unwrap().ask,
        Ask::AccountChange { holder: AskAccountHolder::Own, change: AccountChange::CheckProgram { source: *source } }
    );
    assert_eq!(
        root.person().pages().connect_model().unwrap().check,
        d::pages::connect_model::ProgramCheck::Checking { source: *source },
        "admission isn't the host's login observation"
    );
    accounts::program_checked(&mut front, &mut root, false);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: d::Step::Ready });
    assert!(text(&root).contains("Claude Code isn’t signed in here yet."));
    press(&mut front, &mut root, Role::Button, "Try again".as_bytes(), skein_ui::Act::Press);
    accounts::program_checked(&mut front, &mut root, false);
    press(&mut front, &mut root, Role::Link, b"Sign in again", skein_ui::Act::Press);
    assert!(text(&root).contains("temper program-login claude-person"));
    assert!(!text(&root).contains("Claude didn’t answer"));
    press(&mut front, &mut root, Role::Button, "I’ve signed in".as_bytes(), skein_ui::Act::Press);
    accounts::program_checked(&mut front, &mut root, true);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: d::Step::Ready });
}

fn owner_command(words: &[u8], expected: &[u8]) {
    for welcome in [false, true] {
        let (mut front, mut root) = accounts::empty_program_fixture(fixtures::SEED);
        front.program_environment = words.into();
        let way = Some(d::ModelWay { source: None, provider: accounts::PROVIDER, kind: d::WayKind::Program });
        let address =
            if welcome { Address::Welcome { step: d::Step::Model { way } } } else { Address::ConnectModel { way } };
        go(&mut front, &mut root, address);
        accounts::settle(&mut front, &mut root);
        let view = view(&root);
        let nodes = view.tree().nodes();
        let position = nodes
            .iter()
            .position(|node| node.role == Role::Code)
            .unwrap_or_else(|| panic!("command code on {:?}: {}", root.person().pages().address(), text(&root)));
        let code = nodes.get(position).expect("command position");
        let line: Vec<_> = nodes
            .iter()
            .skip(position)
            .take(usize::try_from(code.size).expect("command subtree fits"))
            .flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied()))
            .collect();
        assert_eq!(line, expected);
    }
}
#[test]
fn the_owner_sees_the_command_the_box_answers() {
    owner_command(
        b"TEMPER_CONFIG='/config space' TEMPER_DATA='/data' TEMPER_CACHE='/cache' temper",
        b"TEMPER_CONFIG='/config space' TEMPER_DATA='/data' TEMPER_CACHE='/cache' temper program-login claude-person",
    );
}
#[test]
fn the_owner_sees_the_short_command_for_default_directories() {
    owner_command(b"", b"temper program-login claude-person");
}

#[test]
fn a_stopped_welcome_check_stays_on_ready_without_a_model() {
    let (mut front, mut root) = accounts::empty_program_fixture(fixtures::SEED);
    front.deployment = (true, true);
    let address = Address::Welcome {
        step: d::Step::Model {
            way: Some(d::ModelWay { provider: accounts::PROVIDER, kind: d::WayKind::Program, source: None }),
        },
    };
    accounts::reopen(&mut front, &mut root, address);
    press(&mut front, &mut root, Role::Button, "I’ve signed in".as_bytes(), skein_ui::Act::Press);
    accounts::program_checked(&mut front, &mut root, false);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: d::Step::Ready });
    assert_eq!(d::pages::welcome::model_usable(root.person()), Some(false));
}
