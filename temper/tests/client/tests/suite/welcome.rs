//! Welcome states (temper/docs/design/client/README.md, 3.5 and 10).
use jig_client_world::harness::App;
use skein_lib::Queue;
use skein_ui::Role;
use temper_client_domain::{self as d, Address, Step};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::account_acts as accounts,
};

fn has(root: &Root, role: Role, name: &[u8]) -> bool {
    let (_, mut view) = Temper::new(fixtures::SEED);
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, root, &mut patches);
    Temper::tree(&view).nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}
#[test]
fn empty_welcome_offers_the_playground_and_setup() {
    let (_, root) = accounts::empty_welcome_fixture(fixtures::SEED);
    assert_eq!(d::pages::welcome::decision(root.person()), Some(d::pages::welcome::FirstRun::Empty));
    assert!(has(&root, Role::Button, b"Try the playground"));
    assert!(has(&root, Role::Link, b"Set up temper"));
}
#[test]
fn unfinished_welcome_opens_at_forge_without_offering_the_playground() {
    let (_, root) = accounts::welcome_fixture(fixtures::SEED);
    assert_eq!(
        d::pages::welcome::decision(root.person()),
        Some(d::pages::welcome::FirstRun::Unfinished { step: Step::Forge { forge: None } })
    );
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Forge { forge: None } });
    assert!(!has(&root, Role::Button, b"Try the playground"));
}
#[test]
fn whole_welcome_moves_to_chats_after_the_first_repository_is_adopted() {
    use super::connect_forge::{accept_owned, owned_welcome};
    use temper_engine_domain_forge as forge;
    let (mut front, mut root, mut now) = owned_welcome();
    accept_owned(&mut front, &mut root, &mut now);
    for event in [
        forge::Event::Connection {
            forge: 1,
            state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
        },
        forge::Event::FirstMade { forge: 1, id: 42 },
    ] {
        front.engine.send(temper_engine_domain::Event::Forge(Box::new(event)));
    }
    front.engine.settle();
    accounts::settle(&mut front, &mut root);
    assert!(front.engine_adopted(1, 42));
    assert_eq!(d::pages::welcome::decision(root.person()), Some(d::pages::welcome::FirstRun::Whole));
    accounts::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Welcome }));
    assert_eq!(root.person().pages().address(), &fixtures::chats_page());
}
