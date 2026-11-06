use jig_client_kit::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;
#[test]
fn a_model_choice_reaches_the_next_run_document() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Link, b"New chat"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"New chat") },
            Step::See { find: Find::exactly_named(Role::Button, b"scripted"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"scripted") },
            Step::Type { find: Find::named(Role::TextArea, b"What do you want done?"), words: Box::from(*b"Hello") },
            Step::Send { find: Find::named(Role::TextArea, b"What do you want done?") },
            Step::See { find: Find::exactly_named(Role::Disclosure, b"Model"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Heading { level: 1 }, " · ".as_bytes()), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let byline = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Disclosure && node.name.as_deref() == Some(b"Model"))
        .expect("the chat's byline");
    assert_eq!(byline.description.as_deref(), Some(b"scripted".as_slice()));
    let root = world.root(person).person();
    let task = root.pages().chat().expect("created chat").task(root.client()).expect("next-run document");
    assert_eq!(task.llm.as_ref().expect("explicit chosen model").main.model.bytes(), b"scripted");
}
