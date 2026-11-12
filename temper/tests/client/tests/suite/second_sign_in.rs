//! A second provider's sign-in after the welcome's (temper/docs/design/client/web/testing.md, 2).
use skein_ui::Role;
use temper_client_domain::{self as d, Address, Place};
use temper_client_world::{
    app::Root,
    fixtures,
    front::{Joint, account_acts as accounts},
};

// The joined world's synthetic issuer and its one subject; no real issuer is visited.
const ISSUER: &[u8] = b"https://issuer/visit";
const SUBJECT: &[u8] = b"subject";
const SECOND_PROVIDER: u32 = accounts::PROVIDER + 1;

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
fn press(front: &mut Joint, root: &mut Root, name: &[u8]) {
    let view = view(root);
    let id = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("offered sign-in button {:?}: {}", String::from_utf8_lossy(name), text(root)))
        .id;
    let event = temper_client_view::event(&view, id, skein_ui::Act::Press).expect("sign-in binding");
    accounts::event(front, root, event.into());
}
fn visit(front: &mut Joint, root: &mut Root, source: u32) {
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::Visited {
        source,
        address: ISSUER.into(),
    }));
    accounts::settle(front, root);
}
fn signed_in(front: &mut Joint, root: &mut Root, source: u32) {
    front.engine.send(temper_engine_domain::Event::Account(jig_domain_accounts::Event::SignedIn {
        source,
        subject: SUBJECT.into(),
        display: Box::new([]),
        declined: false,
        said: jig_domain_accounts::Said::default(),
    }));
    accounts::settle(front, root);
}

#[test]
fn a_second_providers_sign_in_reaches_accounts_after_the_welcomes() {
    let (mut front, mut root) = accounts::two_sign_ins_fixture(fixtures::SEED);
    accounts::reopen(&mut front, &mut root, Address::Welcome { step: d::Step::Model { way: None } });
    press(&mut front, &mut root, b"Continue with Welcome fake");
    let first = accounts::signing(&front);
    visit(&mut front, &mut root, first);
    signed_in(&mut front, &mut root, first);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: d::Step::Ready });
    accounts::answer_trial(&mut front, &mut root);

    go(&mut front, &mut root, Address::ConnectModel { way: None });
    press(&mut front, &mut root, b"Continue with Other fake");
    let second = accounts::signing(&front);
    assert_ne!(second, first, "the second sign-in has its own source");
    visit(&mut front, &mut root, second);
    assert_eq!(
        front.tabs,
        [
            d::Tab::Open { provider: accounts::PROVIDER },
            d::Tab::Point { to: ISSUER.into() },
            d::Tab::Open { provider: SECOND_PROVIDER },
            d::Tab::Point { to: ISSUER.into() },
        ]
    );
    assert_eq!(
        root.person().pages().address(),
        &Address::ConnectModel {
            way: Some(d::ModelWay { provider: SECOND_PROVIDER, kind: d::WayKind::SignIn, source: Some(second) })
        }
    );
    signed_in(&mut front, &mut root, second);
    assert_eq!(root.person().pages().address(), &Address::Accounts);
    let words = text(&root);
    assert!(words.contains("Added your Other fake plan"), "{words}");
    assert!(!words.contains("the same account again"), "{words}");
    assert_eq!(front.leaves.len(), 0);
    accounts::answer_trial(&mut front, &mut root);
}
