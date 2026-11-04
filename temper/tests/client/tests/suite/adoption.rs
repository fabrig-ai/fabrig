use jig_client_world::harness::{FrontFault, Until};
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;
#[test]
fn a_repository_adopted_from_the_navigation_link_is_held_by_the_engine() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    world.person(person, fixtures::adopt_and_see_outcome(b"acme", b"shop"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
    assert!(world.front().engine_adopted(1, 7), "the engine holds acme/shop, id 7, in the project, owned");
}
#[test]
fn adopting_a_repository_the_forge_lacks_says_not_found() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut steps = fixtures::adopt(b"acme", b"none");
    steps.push(Step::See {
        find: Find::text(b"No repository acme/none on forge 1 that temper's forge user can see."),
        within: fixtures::WITHIN,
    });
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
}
#[test]
fn a_second_home_is_refused_by_the_forge_connector() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut first = fixtures::adopt(b"acme", b"shop");
    first.push(Step::See { find: Find::text(b"Adopted acme/shop on forge 1 as owned."), within: fixtures::WITHIN });
    world.person(person, first);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let mut steps = fixtures::adopt(b"acme", b"second");
    steps.push(Step::See {
        find: Find::text(
            b"The project has a home already. Adopt this repository without making it home, or choose a new project.",
        ),
        within: fixtures::WITHIN,
    });
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 2);
}
#[test]
fn a_field_the_form_refuses_sends_nothing_to_the_engine() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut steps = fixtures::adopt(b"", b"shop");
    steps.push(Step::See { find: Find::named(Role::Status, b"Required."), within: fixtures::WITHIN });
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 0);
    assert_eq!(world.front().engine.forge.as_ref().expect("fake forge").calls, 0);
}
#[test]
fn a_double_press_of_adopt_adopts_once() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut steps = fixtures::adopt(b"acme", b"shop");
    steps.pop().expect("Adopt is last");
    world.person(person, steps);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let node = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Adopt"))
        .expect("Adopt")
        .id;
    world.act(person, node, skein_ui::Act::Press);
    world.act(person, node, skein_ui::Act::Press);
    world.person(
        person,
        vec![Step::See { find: Find::text(b"Adopted acme/shop on forge 1 as owned."), within: fixtures::WITHIN }],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
}
#[test]
fn a_reload_with_an_adoption_pending_sends_it_once() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    world.person(
        person,
        vec![Step::See { find: Find::named(Role::Link, b"Adopt a repository"), within: fixtures::WITHIN }],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    world.fault(FrontFault::NoAnswer);
    let mut steps = fixtures::adopt(b"acme", b"shop");
    steps.extend([
        Step::Reload,
        Step::See { find: Find::named(Role::Status, b"Live"), within: fixtures::WITHIN },
        Step::Pause { span: skein_lib::Duration::from_millis(50) },
    ]);
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
    assert!(world.kept(person).is_empty(), "the retried adoption's record retired");
}

#[test]
fn the_first_repository_is_adopted_from_default_with_nothing_typed() {
    use jig_client_world::harness::App;
    use temper_client_domain::{self as d, Address};
    use temper_client_world::{app::Temper, front::account_acts as accounts};
    use temper_engine_domain_forge as forge;
    let (mut front, mut root, mut now) = super::connect_forge::owned_welcome();
    super::connect_forge::accept_owned(&mut front, &mut root, &mut now);
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
    accounts::event(&mut front, &mut root, Temper::went(Address::Welcome { step: d::Step::Ready }));
    press_adoption(&mut front, &mut root, Role::Button, b"Open Default");
    assert_eq!(root.person().pages().address(), &fixtures::chats_page());
    let chats = super::connect_forge::view(&root);
    super::connect_forge::node(&chats, Role::Paragraph, b"No repository yet");
    press_adoption(&mut front, &mut root, Role::Link, b"Adopt a repository");
    press_adoption(&mut front, &mut root, Role::Button, b"Adopt");
    assert!(front.engine_adopted(1, 42), "the engine holds work/hello as owned");
    let outcome = super::connect_forge::view(&root);
    super::connect_forge::node(&outcome, Role::Status, b"Adopted work/hello on forge 1 as owned.");
    accounts::event(&mut front, &mut root, Temper::went(fixtures::chats_page()));
    let chats = super::connect_forge::view(&root);
    assert!(!chats.tree().nodes().iter().any(|node| node.name.as_deref() == Some(b"No repository yet")));
}

fn press_adoption(
    front: &mut temper_client_world::front::Joint,
    root: &mut temper_client_world::app::Root,
    role: Role,
    name: &[u8],
) {
    let mut view = super::connect_forge::view(root);
    let node = super::connect_forge::node(&view, role, name);
    let action = temper_client_view::act(&mut view, node, skein_ui::Act::Press).expect("bound act");
    temper_client_world::front::account_acts::event(front, root, temper_client_domain::Event::Act { action }.into());
}
