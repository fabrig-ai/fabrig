use jig_client_world::harness::Until;
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
            Step::See { find: Find::exactly_named(Role::Button, b"scripted"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"scripted") },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: Box::from(*b"Hello") },
            Step::Send { find: Find::named(Role::TextArea, b"Start a chat") },
            Step::See { find: Find::exactly_named(Role::Paragraph, b"Using scripted"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let root = world.root(person).person();
    let task = root.pages().chat().expect("created chat").task(root.client()).expect("next-run document");
    assert_eq!(task.llm.as_ref().expect("explicit chosen model").main.model.bytes(), b"scripted");
}
