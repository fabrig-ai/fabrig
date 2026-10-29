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
fn signed_in() -> (Joint, Root, u32) {
    use jig_domain_accounts as a;
    let (mut front, mut root) = accounts::plan_fixture(fixtures::SEED);
    go(&mut front, &mut root, Address::ConnectModel { way: None });
    press(&mut front, &mut root, Role::Button, b"Continue with ChatGPT", Act::Press);
    let source = accounts::signing(&front);
    front.engine.send(temper_engine_domain::Event::Account(a::Event::Visited {
        source,
        address: b"https://issuer/visit".as_slice().into(),
    }));
    accounts::settle(&mut front, &mut root);
    front.engine.send(temper_engine_domain::Event::Account(a::Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: b"alice@example.com".as_slice().into(),
        declined: false,
    }));
    accounts::settle(&mut front, &mut root);
    go(&mut front, &mut root, Address::Accounts);
    (front, root, source)
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
#[test]
fn a_folded_account_rows_key_joins_that_account_and_is_shown_there() {
    let (mut front, mut root, _) = signed_in();
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
    assert!(row.sources.iter().any(|s| s.number == key && s.kind == c::OwnerAccountKind::Key));
    let drawn = view(&root);
    assert!(drawn.tree().nodes().iter().any(|n| n.text.iter().any(|t| t.text.as_ref() == b"API key")));
    assert!(!root.person().pages().accounts().expect("account story fixture").request.has_key());
    accounts::assert_no_carried(&front);
}
#[test]
fn a_limited_accounts_task_waits_until_its_key_is_moved_first() {
    let (mut front, mut root, plan) = signed_in();
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
