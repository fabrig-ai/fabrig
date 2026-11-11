//! Deletion through the web's pages and sheets against the real engine
//! (temper/docs/design/client/pages/settings.md, 1; home.md, 1).
use jig_client_kit::harness::Until;
use skein_fake_person::{Find, Step, Target};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::{Address, address};
use temper_client_world::{app::{Root, View}, fixtures, front::Joint};

const PROJECT: u32 = 1;

fn browser(seed: u64) -> (Root, View) {
    let limits = fixtures::domain_limits();
    let mut words = fixtures::phrasebooks(&limits);
    words.shell = temper_client_view::words::Shell::Browser;
    let bounds = fixtures::view_bounds(&limits, &words);
    (Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        View::Person(Box::new(temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE))))
}

#[test]
fn an_owner_ends_live_work_deletes_the_project_and_reads_its_old_address() {
    let mut world = fixtures::world(fixtures::SEED);
    *world.front() = Joint::adoption(fixtures::SEED, fixtures::domain_limits());
    world.randomness(0, 0, 0);
    let task = fixtures::chat(world.front(), b"A live chat in the project to delete");
    let person = world.client_with(fixtures::PARTY, Address::Settings { project: PROJECT }, fixtures::domain_limits(), browser);
    let see = |find| Step::See { find, within: fixtures::WITHIN };
    let press = |name: &[u8]| Step::Press { find: Find::exactly_named(Role::Button, name) };
    world.person(person, vec![see(Find::exactly_named(Role::Button, b"Delete project"))]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let name = world.root(person).person().pages().me(world.root(person).person().client()).unwrap()
        .projects.iter().find(|row| row.project == PROJECT).unwrap().name.bytes().to_vec();
    // Make the surviving project through the same switcher the person uses.
    world.person(person, vec![
        Step::Type { find: Find::exactly_named(Role::TextField, b"New project"), words: b"Survivor".as_slice().into() },
        press(b"Create"),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Survivor")),
        Step::Go { address: Address::Settings { project: PROJECT } },
        see(Find::exactly_named(Role::Button, b"Delete project")),
        press(b"Delete project"),
    ]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let sheet = format!("Delete {}?", String::from_utf8(name.clone()).unwrap());
    let in_sheet = |find: Find| find.within(Box::new([Target::ExactRole { role: Role::Dialog, name: sheet.as_bytes().into() }]));
    world.person(person, vec![
        see(Find::exactly_named(Role::Dialog, sheet.as_bytes())),
        Step::Press { find: in_sheet(Find::exactly_named(Role::Button, b"Delete project")) },
        see(in_sheet(Find::text(format!("This project has live tasks; end them first: T{task}").as_bytes()))),
    ]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(world.tree(person).nodes().iter().any(|node| node.role == Role::Dialog && node.name.as_deref() == Some(sheet.as_bytes())));
    // Another tab closes the chat while the refusal and its sheet stay open.
    let chat = world.client_with(fixtures::PARTY, fixtures::chat_page(task), fixtures::domain_limits(), browser);
    world.person(chat, vec![
        see(Find::exactly_named(Role::Button, b"Close")), press(b"Close"),
        see(Find::exactly_named(Role::Dialog, b"Close this chat?")), press(b"Close chat"),
        Step::Pause { span: skein_lib::Duration::from_secs(6) },
        see(Find::text(b"Closed at")),
    ]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(world.front().engine.task(task).is_none(), "the chat ended before deletion");
    world.person(person, vec![
        Step::Press { find: in_sheet(Find::exactly_named(Role::Button, b"Delete project")) },
        Step::Gone { find: Find::exactly_named(Role::Dialog, sheet.as_bytes()), within: fixtures::WITHIN },
        see(Find::exactly_named(Role::Disclosure, b"All projects")),
        Step::Gone { find: Find::exactly_named(Role::Button, &name), within: fixtures::WITHIN },
        see(Find::exactly_named(Role::Button, b"Survivor")),
    ]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let root = world.root(person).person();
    assert_eq!(address::print(root.pages().address()).as_ref(), b"/home");
    assert!(!root.pages().me(root.client()).unwrap().projects.iter().any(|row| row.project == PROJECT));
    let temper_client_domain::Place::Address(deleted_address) = address::parse(format!("/projects/{PROJECT}").as_bytes()) else { panic!("project address") };
    world.person(person, vec![
        Step::Go { address: deleted_address },
        see(Find::text(format!("Project {PROJECT} was deleted.").as_bytes())),
        see(Find::exactly_named(Role::Link, b"Chats")),
    ]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
