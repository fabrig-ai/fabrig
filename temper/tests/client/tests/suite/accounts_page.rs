//! Row controls through temper's client and engine, with only the issuer faked.
use jig_client::contract as c;
use skein_ui::{Act, Role, State};
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
fn go(front: &mut Joint, root: &mut Root, address: Address) {
    accounts::event(front, root, d::Event::Went { place: Place::Address(address) }.into());
}
fn press(front: &mut Joint, root: &mut Root, role: Role, name: &[u8], act: Act) {
    let view = view(root);
    let node =
        view.tree().nodes().iter().find(|n| n.role == role && n.name.as_deref() == Some(name)).expect("row control").id;
    let event = temper_client_view::event(&view, node, act).expect("row act");
    accounts::event(front, root, event.into());
}
fn add_key(front: &mut Joint, root: &mut Root) -> u32 {
    press(front, root, Role::Button, b"Add an API key", Act::Press);
    press(
        front,
        root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: b"test-private-key".as_slice().into() },
    );
    press(front, root, Role::Button, b"Add key", Act::Press);
    let source = accounts::kept(front);
    accounts::settle(front, root);
    source
}
fn row_sign_in_press() -> (Joint, Root) {
    let (mut front, mut root, source) = accounts::signed_in_plan(fixtures::SEED);
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::Component(
        jig_domain_accounts::FromComponent::Failed {
            account: source,
            failure: jig_domain_accounts::Failure::SignInAgain,
        },
    )));
    accounts::settle(&mut front, &mut root);
    front.tabs.clear();
    front.shows.clear();
    let drawn = view(&root);
    let node = drawn
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT"))
        .expect("row sign-in button")
        .id;
    let event = temper_client_view::event(&drawn, node, Act::Press).expect("row sign-in press");
    accounts::press_turn(&mut front, &mut root, event.into());
    (front, root)
}
fn row_sign_in_visit(front: &mut Joint, root: &mut Root) {
    accounts::settle(front, root);
    let source = accounts::signing(front);
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::Visited {
        source,
        address: b"https://issuer/visit".as_slice().into(),
    }));
}
#[test]
fn accounts_row_opens_before_the_answer_and_points_once_without_leaving() {
    let (mut front, mut root) = row_sign_in_press();
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }]);
    assert!(
        root.person()
            .pages()
            .accounts()
            .expect("accounts page")
            .request
            .pending
            .as_ref()
            .expect("pending")
            .outcome
            .is_none()
    );
    assert_eq!(root.person().pages().address(), &Address::Accounts);
    assert_eq!(front.leaves.len(), 0);
    row_sign_in_visit(&mut front, &mut root);
    accounts::settle(&mut front, &mut root);
    let expected = [
        d::Tab::Open { provider: accounts::PROVIDER },
        d::Tab::Point { to: b"https://issuer/visit".as_slice().into() },
    ];
    assert_eq!(front.tabs, expected);
    assert_eq!(d::address::print(root.person().pages().address()).as_ref(), b"/accounts");
    assert_eq!(front.shows.len(), 0);
    assert_eq!(front.leaves.len(), 0);
    view(&root);
    accounts::settle(&mut front, &mut root);
    go(&mut front, &mut root, Address::Forges);
    assert_eq!(front.tabs, expected);
}
#[test]
fn accounts_row_refused_answer_closes_the_tab_without_leaving() {
    let (mut front, mut root) = row_sign_in_press();
    row_sign_in_visit(&mut front, &mut root);
    accounts::refused_answer(&mut front, &mut root);
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }, d::Tab::Close]);
    assert_eq!(root.person().pages().address(), &Address::Accounts);
    assert_eq!(front.shows.len(), 0);
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn accounts_row_refused_tab_keeps_its_link_without_pointing() {
    let (mut front, mut root) = row_sign_in_press();
    accounts::event(&mut front, &mut root, d::Event::TabRefused.into());
    row_sign_in_visit(&mut front, &mut root);
    accounts::settle(&mut front, &mut root);
    assert_eq!(front.tabs, [d::Tab::Open { provider: accounts::PROVIDER }]);
    let drawn = view(&root);
    assert!(
        drawn
            .tree()
            .nodes()
            .iter()
            .any(|node| { node.role == Role::Link && node.external.as_deref() == Some(b"https://issuer/visit") })
    );
    assert!(
        drawn
            .tree()
            .nodes()
            .iter()
            .any(|node| { node.text.iter().any(|run| run.text.as_ref() == b"Open the sign-in page") })
    );
    assert_eq!(root.person().pages().address(), &Address::Accounts);
    assert_eq!(front.shows.len(), 0);
    assert_eq!(front.leaves.len(), 0);
}
#[test]
fn a_folded_account_rows_key_joins_that_account_and_is_shown_there() {
    let (mut front, mut root, _) = accounts::signed_in_plan(fixtures::SEED);
    let before = d::pages::accounts::document(root.person().client())
        .expect("account story fixture")
        .accounts
        .get(0)
        .expect("account story fixture")
        .number;
    let folded = view(&root);
    let row = folded
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Disclosure && n.name.as_ref().is_some_and(|n| n.starts_with(b"ChatGPT")))
        .expect("account story fixture");
    assert!(!row.states.as_slice().contains(&State::Expanded));
    let key = add_key(&mut front, &mut root);
    let doc = d::pages::accounts::document(root.person().client()).expect("account story fixture");
    let row = doc.accounts.get(0).expect("account story fixture");
    assert_eq!(row.number, before);
    assert_eq!(doc.accounts.len(), 1);
    assert!(row.sources.iter().any(|s| s.number == key && s.kind == c::AccountKind::Key));
    let drawn = view(&root);
    assert!(drawn.tree().nodes().iter().any(|n| n.text.iter().any(|t| t.text.as_ref() == b"API key")));
    assert!(!root.person().pages().accounts().expect("account story fixture").request.has_key());
    accounts::assert_no_carried(&front);
}
#[test]
fn a_limited_accounts_task_waits_until_its_key_is_moved_first() {
    let (mut front, mut root, plan) = accounts::signed_in_plan(fixtures::SEED);
    let key = add_key(&mut front, &mut root);
    front.engine.hello(skein_lib::Token::new(7));
    front.engine.settle();
    front
        .engine
        .send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::Limited { account: plan, reset: None }));
    accounts::settle(&mut front, &mut root);
    let account = d::pages::accounts::document(root.person().client())
        .expect("account story fixture")
        .accounts
        .get(0)
        .expect("account story fixture")
        .number;
    let waiting = accounts::account_chat(&mut front, account);
    accounts::settle(&mut front, &mut root);
    go(&mut front, &mut root, Address::Chat { task: waiting });
    let task = root.person().pages().chat().expect("created chat").task(root.person().client()).expect("task document");
    assert!(
        matches!(task.run_wait, Some(c::TaskRunWait::Account { account: Some(number), source: Some(source), .. }) if number == account && source == plan),
        "{:?}",
        task.run_wait
    );
    go(&mut front, &mut root, Address::Accounts);
    let limited = view(&root);
    let text: Vec<_> =
        limited.tree().nodes().iter().flat_map(|n| n.text.iter().flat_map(|t| t.text.iter().copied())).collect();
    assert!(
        String::from_utf8(text)
            .expect("account story fixture")
            .contains("temper reached its usage limit on your ChatGPT plan. Tasks on it wait.")
    );
    press(&mut front, &mut root, Role::Button, b"Use the API key first", Act::Press);
    let doc = d::pages::accounts::document(root.person().client()).expect("account story fixture");
    let row = doc.accounts.get(0).expect("account story fixture");
    assert_eq!(row.sources.get(0).expect("account story fixture").number, key);
    assert_eq!(row.sources.get(1).expect("account story fixture").number, plan);
    go(&mut front, &mut root, Address::Chat { task: waiting });
    let task = root
        .person()
        .pages()
        .chat()
        .expect("account story fixture")
        .task(root.person().client())
        .expect("account story fixture");
    assert!(task.run_wait.is_none(), "{:?}", task.run_wait);
}
