use jig_client_world::harness::{Front, Until};
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn signing_in_shows_the_inbox_from_the_engine_root() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Status, b"Live"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Status, b"Person"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Inbox"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().sign_ins, 1, "one sign-in reached the engine root");
}

#[test]
fn a_double_press_of_start_makes_one_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: Box::from(*b"Hello") },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    // Both presses reach the domain before the disabled Start is drawn.
    let node = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Start chat"))
        .expect("Start")
        .id;
    world.act(person, node, skein_ui::Act::Press);
    world.act(person, node, skein_ui::Act::Press);
    world.person(person, vec![Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1, "one chat task for the two presses");
}
#[test]
fn enter_in_the_start_composer_starts_one_chat_titled_by_its_first_line() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: Box::from(*b"Hello\nthere") },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    // Both Enters reach the domain before the view blocks a pending submission.
    let field = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::TextArea && node.name.as_deref() == Some(b"Start a chat"))
        .expect("chat composer")
        .id;
    world.act(person, field, skein_ui::Act::Submit);
    world.act(person, field, skein_ui::Act::Submit);
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"T1 \xc2\xb7 Hello"),
                within: fixtures::WITHIN,
            },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1, "one chat task for the two Enters");
}
#[test]
fn a_chat_started_on_the_chats_page_is_answered_by_the_run() {
    let mut world = fixtures::world(fixtures::SEED);
    world.front().engine.hello(skein_lib::Token::new(7));
    world.front().answer_runs = true;
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: Box::from(*b"Hello") },
            Step::Press { find: Find::named(Role::Button, b"Start chat") },
            Step::See { find: Find::text(b"Answered by the run"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}
#[test]
fn signed_out_mid_request_is_made_once_after_signing_in() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.person(
        person,
        vec![Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: fixtures::WITHIN }],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    world.fault(jig_client_world::harness::FrontFault::SignedOut);
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: Box::from(*b"Hello") },
            Step::Press { find: Find::named(Role::Button, b"Start chat") },
            Step::See { find: Find::text(b"Signed out"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 0);
    world.reload(person);
    let now = world.now();
    world.front().sign_in(person, now, skein_lib::Wall::from_nanos(now.as_nanos()));
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    world.go(person, fixtures::chats_page());
    world.person(person, vec![Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}

#[test]
fn a_question_needing_the_person_is_shown_first_in_the_inbox() {
    let mut world = fixtures::world(fixtures::SEED);
    fixtures::question(world.front(), b"Which provider first?");
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Needs you"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Which provider first?"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
