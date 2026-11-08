use jig_client_kit::harness::{FrontFault, Until};
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;
#[test]
fn a_repository_adopted_from_the_navigation_link_is_held_by_the_engine() {
    let mut world = fixtures::adoption_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    world.person(person, fixtures::adopt_and_see_outcome(b"acme", b"shop"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
    assert!(world.front().engine_adopted(1, 7), "the engine holds acme/shop, id 7, in the project, owned");
}
#[test]
fn adopting_a_repository_the_forge_lacks_says_not_found() {
    let mut world = fixtures::adoption_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut steps = fixtures::adopt(b"acme", b"none");
    steps.push(Step::See { find: Find::text(b"The repository was not found."), within: fixtures::WITHIN });
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
    assert!(world.tree(person).nodes().iter().any(|node| node.role == Role::Dialog));
}
#[test]
fn a_second_repository_keeps_the_projects_home() {
    let mut world = fixtures::adoption_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut first = fixtures::adopt(b"acme", b"shop");
    first.push(Step::See { find: Find::text(b"acme/shop"), within: fixtures::WITHIN });
    world.person(person, first);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let mut steps = fixtures::adopt(b"acme", b"second");
    steps.push(Step::See { find: Find::text(b"acme/second"), within: fixtures::WITHIN });
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 2);
}
#[test]
fn a_field_the_form_refuses_sends_nothing_to_the_engine() {
    let mut world = fixtures::adoption_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut steps = fixtures::adopt(b"", b"shop");
    steps.pop();
    steps.pop();
    steps.push(Step::See { find: Find::named(Role::Button, b"Adopt"), within: fixtures::WITHIN });
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 0);
    assert_eq!(world.front().engine.forge.as_ref().expect("fake forge").calls, 0);
}
#[test]
fn a_double_press_of_adopt_adopts_once() {
    let mut world = fixtures::adoption_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut steps = fixtures::adopt(b"acme", b"shop");
    steps.pop().expect("Adopt is last");
    world.person(person, steps);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let node = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Adopt acme/shop"))
        .expect("Adopt")
        .id;
    world.act(person, node, skein_ui::Act::Press);
    world.act(person, node, skein_ui::Act::Press);
    world.person(person, vec![Step::See { find: Find::text(b"acme/shop"), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
}
#[test]
fn a_reload_with_an_adoption_pending_sends_it_once() {
    let mut world = fixtures::adoption_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    world.person(
        person,
        vec![Step::See {
            find: Find::exactly_named(Role::Link, b"How work runs for Project 1"),
            within: fixtures::WITHIN,
        }],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let mut steps = fixtures::adopt(b"acme", b"shop");
    steps.pop().expect("send is last");
    world.person(person, steps);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    world.fault(FrontFault::NoAnswer);
    world.person(
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Adopt acme/shop") },
            Step::Reload,
            Step::See { find: Find::named(Role::Status, b"Live"), within: fixtures::WITHIN },
            Step::Pause { span: skein_lib::Duration::from_millis(50) },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().applications_admitted(), 1);
    assert!(world.kept(person).is_empty(), "the retried adoption's record retired");
}

#[test]
fn ready_adopts_the_first_repository_with_nothing_typed() {
    use jig_client_kit::harness::App;
    use temper_client_domain::Address;
    use temper_client_world::{app::Temper, front::account_acts as accounts};
    let (mut front, mut root, project) = owned_project();
    assert_eq!(root.person().pages().address(), &Address::Home { project });
    assert!(front.engine_adopted(1, 42), "Ready adopts work/hello as owned");
    assert_eq!(root.person().pages().adopting().expect("Ready's adoption").out, None);
    assert_eq!(root.person().pages().adopting().expect("Ready's adoption").ended, None);
    accounts::event(&mut front, &mut root, Temper::went(fixtures::chats_page()));
    let chats = super::connect_forge::view(&root);
    assert!(
        !chats.tree().nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == b"No repository yet"))
    );
}

fn press_adoption(
    front: &mut temper_client_world::front::Joint,
    root: &mut temper_client_world::app::Root,
    role: Role,
    name: &[u8],
) {
    let mut view = super::connect_forge::view(root);
    let node = super::connect_forge::node(&view, role, name);
    let action = temper_client_view::act(
        &mut view,
        node,
        if role == Role::Choice { skein_ui::Act::Toggle } else { skein_ui::Act::Press },
    )
    .unwrap_or_else(|| panic!("bound {role:?} {}", String::from_utf8_lossy(name)));
    temper_client_world::front::account_acts::event(front, root, temper_client_domain::Event::Act { action }.into());
}

#[test]
fn the_same_repository_adopted_again_from_its_page_replaces_its_record() {
    use jig_client_kit::harness::{App, Front};
    use temper_client_domain::{self as d, Address};
    use temper_client_world::{
        app::Temper,
        front::{Joint, account_acts as accounts},
    };
    use temper_engine_domain as engine;
    use temper_engine_domain_forge as forge;
    let mut front = Joint::adoption(fixtures::SEED, fixtures::domain_limits());
    let mut root =
        temper_client_world::app::Root::Person(Box::new(d::unlinked(&fixtures::domain_limits(), fixtures::SEED)));
    front.bind(jig_client_kit::harness::ClientId(0), fixtures::PARTY);
    accounts::event(&mut front, &mut root, Temper::start(fixtures::inbox(), Vec::new(), 0));
    for admitted in 1..=2 {
        accounts::event(&mut front, &mut root, Temper::went(Address::Repositories { project: 1 }));
        press_adoption(&mut front, &mut root, Role::Link, b"Add repositories");
        for (name, value) in
            [(b"Owner".as_slice(), b"acme".as_slice()), (b"Name", b"shop"), (b"Landing branch", b"main")]
        {
            let mut view = super::connect_forge::view(&root);
            let node = super::connect_forge::node(&view, Role::TextField, name);
            let action = temper_client_view::act(&mut view, node, skein_ui::Act::Input { value: value.into() })
                .expect("field act");
            accounts::event(&mut front, &mut root, d::Event::Act { action }.into());
        }
        press_adoption(&mut front, &mut root, Role::Button, b"Adopt acme/shop");
        assert_eq!(root.person().pages().address(), &Address::Repositories { project: 1 });
        assert_eq!(front.applications_admitted(), admitted, "each adoption is admitted under its own key");
        assert!(front.engine_adopted(1, 7));
        let repositories = front.engine.store.rows.values().filter(|row| matches!(row,
            engine::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Repository(repository) if repository.project == 1)
        )).count();
        assert_eq!(repositories, 1, "adopting again replaces the project's repository record");
    }
    let keys: Vec<_> = front
        .engine
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let engine::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Answer {
                key,
                ask,
                ..
            })) = row
                && matches!(ask.as_ref(), jig_domain_people::Ask::Application { .. })
            {
                Some(*key)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(keys.len(), 2);
    assert_ne!(keys.first(), keys.last(), "adopting again uses a fresh request key");
}

#[test]
fn a_search_reads_across_pages_and_adopts_both_picks_into_repositories() {
    let mut world = fixtures::adoption_world(fixtures::SEED);
    world.front().reachable_pages = true;
    let person = world.client(fixtures::PARTY, fixtures::project());
    let mut steps = fixtures::adopt(b"acme", b"shop");
    steps.truncate(8); // Navigation, dialog, then the first read's check.
    steps.extend([
        Step::Type { find: Find::exactly_named(Role::TextField, b"Find by name"), words: b"second".as_slice().into() },
        Step::See { find: Find::exactly_named(Role::Check, b"acme/second"), within: fixtures::WITHIN },
        Step::Press { find: Find::exactly_named(Role::Check, b"acme/second") },
        Step::Type { find: Find::exactly_named(Role::TextField, b"Find by name"), words: b"shop".as_slice().into() },
        Step::See { find: Find::exactly_named(Role::Check, b"acme/shop"), within: fixtures::WITHIN },
        Step::Press { find: Find::exactly_named(Role::Check, b"acme/shop") },
        Step::Type { find: Find::exactly_named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::See { find: Find::exactly_named(Role::Button, b"Adopt 2 repositories"), within: fixtures::WITHIN },
        Step::Press { find: Find::exactly_named(Role::Button, b"Adopt 2 repositories") },
        Step::See { find: Find::text(b"acme/shop"), within: fixtures::WITHIN },
        Step::See { find: Find::text(b"acme/second"), within: fixtures::WITHIN },
    ]);
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().reachable_reads.iter().map(|row| row.2).collect::<Vec<_>>(), [1, 2, 3]);
    assert!(world.front().engine_adopted(1, 7));
    assert!(world.front().engine_adopted(1, 8));
    assert!(!world.tree(person).nodes().iter().any(|node| node.role == Role::Dialog));
}

fn owned_project() -> (temper_client_world::front::Joint, temper_client_world::app::Root, u32) {
    use jig_client_kit::harness::App;
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
    press_adoption(&mut front, &mut root, Role::Button, b"Create your project");
    let project = root.person().pages().project().expect("Ready's project");
    (front, root, project)
}

#[test]
fn a_failed_copy_can_be_filled_again_and_dismissed_from_repositories() {
    use jig_client_kit::harness::App;
    use temper_client_domain::Address;
    use temper_client_world::{app::Temper, front::account_acts as h};
    use temper_engine_domain_forge as forge;
    let (mut front, mut root, project) = owned_project();
    h::event(&mut front, &mut root, Temper::went(Address::Repositories { project }));
    press_adoption(&mut front, &mut root, Role::Link, b"Add repositories");
    press_adoption(&mut front, &mut root, Role::Choice, b"Copy public");
    let mut view = super::connect_forge::view(&root);
    let node = super::connect_forge::node(&view, Role::TextArea, b"Their addresses, one per line");
    let event = temper_client_view::act_event(
        &mut view,
        node,
        skein_ui::Act::Input { value: b"source.example/acme/again".as_slice().into() },
    )
    .unwrap();
    h::event(&mut front, &mut root, event.into());
    press_adoption(&mut front, &mut root, Role::Button, b"Copy acme/again");
    let key = forge::MakingKey { forge: 1, org: b"acme".as_slice().into(), name: b"again".as_slice().into() };
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::MakingStopped {
        key: key.clone(),
        failed: forge::MakingFailed {
            step: forge::MakeStep::Repository,
            reason: forge::MakeFailure::NotAllowed,
            words: b"You can not import from disallowed hosts.".as_slice().into(),
        },
    })));
    front.engine.settle();
    h::settle(&mut front, &mut root);
    let view = super::connect_forge::view(&root);
    assert!(view.tree().nodes().iter().any(|node| {
        node.text
            .iter()
            .any(|run| run.text.as_ref() == "Forgejo said: “You can not import from disallowed hosts.”".as_bytes())
    }));
    press_adoption(&mut front, &mut root, Role::Button, b"Copy again");
    let sheet = root.person().forge().sheet().expect("retry sheet");
    assert_eq!(sheet.source, temper_client_domain::forge::sheet::Source::Copy);
    assert_eq!(sheet.addresses.text.as_ref(), b"https://source.example/acme/again");
    press_adoption(&mut front, &mut root, Role::Button, b"Cancel");
    press_adoption(&mut front, &mut root, Role::Button, b"Dismiss");
    let view = super::connect_forge::view(&root);
    assert!(!view.tree().nodes().iter().any(|node| node.name.as_deref() == Some(b"acme/again")));
    assert!(!front.engine.store.rows.contains_key(&temper_engine_domain::Key::Forge(forge::Key::Making(key))));
}
