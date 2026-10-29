//! A run's live text reaches the chat through the real engine (domain/testing.md, 6.3).
use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::Address;
use temper_client_view::cards::P;
use temper_client_world::fixtures;

const FIRST: &[u8] = b"The answer";
const REST: &[u8] = b" is forty-two.";
const QUESTION: &[u8] = b"What is the answer?";

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: fixtures::WITHIN }
}

fn gone(find: Find) -> Step<Address> {
    Step::Gone { find, within: fixtures::WITHIN }
}

#[test]
fn a_chats_reply_shows_as_it_is_written_then_as_its_committed_turn() {
    let mut world = fixtures::open_world(fixtures::SEED);
    world.front().reply_in_pieces(&[FIRST, REST]);
    let words = fixtures::phrasebooks(&fixtures::domain_limits());
    let provisional = words.say_card(P::Provisional, &[]);
    let whole = [FIRST, REST].concat();
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.person(
        person,
        vec![
            see(Find::named(Role::TextArea, b"Start a chat")),
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: QUESTION.into() },
            Step::Send { find: Find::named(Role::TextArea, b"Start a chat") },
            see(Find::text(&provisional)),
            see(Find::text(FIRST)),
            gone(Find::text(&whole)),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().runs_answered, 0, "the first piece precedes the committed turn");

    assert!(world.front().release_reply());
    world.person(person, vec![see(Find::text(&provisional)), see(Find::text(&whole))]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().runs_answered, 0, "all pieces precede the committed turn");

    assert!(world.front().release_reply());
    world.person(person, vec![see(Find::text(&whole)), gone(Find::text(&provisional))]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().runs_answered, 1);
    assert!(!world.front().release_reply(), "the committed reply is no longer held");
}

#[test]
fn a_tool_only_turn_leaves_no_provisional_text() {
    use skein_lib::Token;
    use smith_domain_session::llm::{Block, Decoded};

    let mut world = fixtures::open_world(fixtures::SEED);
    let task = fixtures::chat(world.front(), QUESTION);
    let attempt = world.front().engine.task(task).expect("running chat").attempt;
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    let words = fixtures::phrasebooks(&fixtures::domain_limits());
    let provisional = words.say_card(P::Provisional, &[]);
    world.person(person, vec![see(Find::named(Role::TextArea, b"Message")), gone(Find::text(&provisional))]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);

    let mut turn = temper_worker_domain_world::protocol::session_turn(1, 0, 0, true);
    turn.messages[0].content = Box::new([Block::ToolCall {
        id: b"wait-1".as_slice().into(),
        name: b"wait".as_slice().into(),
        input: b"{}".as_slice().into(),
        call: Decoded::Historical,
        replay: None,
    }]);
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("tool-only turn encodes");
    world.front().engine.send(temper_engine_domain_world::protocol::turn(
        Token::new(7),
        task,
        attempt,
        1,
        0,
        None,
        &body,
    ));
    world.front().engine.settle();
    world.person(person, vec![see(Find::named(Role::Disclosure, b"1 tool call")), gone(Find::text(&provisional))]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn reply_notification_arrives_off_page_only_after_the_held_turn_commits() {
    let mut world = fixtures::open_world(fixtures::SEED);
    world.front().reply_in_pieces(&[FIRST, REST]);
    let task = fixtures::chat(world.front(), QUESTION);
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(person, vec![see(Find::text(FIRST))]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    world.go(person, fixtures::inbox());
    let replied = Find::named(Role::Heading { level: 3 }, b"replied");
    world.person(person, vec![see(Find::text(b"Needs you")), gone(Find::text(FIRST)), gone(replied.clone())]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);

    assert!(world.front().release_reply(), "release the suffix while retaining the commit");
    world.person(person, vec![gone(replied.clone()), gone(Find::text(&[FIRST, REST].concat()))]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().runs_answered, 0);

    assert!(world.front().release_reply(), "commit the held reply off page");
    world.person(person, vec![see(replied), see(Find::text(QUESTION))]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().runs_answered, 1);
    world.go(person, fixtures::chat_page(task));
    world.person(person, vec![see(Find::text(&[FIRST, REST].concat()))]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}

#[test]
fn a_composer_reset_is_processed_while_the_reply_remains_held() {
    use temper_client_domain::pages::choice::Context;

    let mut world = fixtures::open_world(fixtures::SEED);
    world.front().reply_in_pieces(&[FIRST, REST]);
    let task = fixtures::chat(world.front(), QUESTION);
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(
        person,
        vec![
            see(Find::text(FIRST)),
            Step::Press { find: Find::exactly_named(Role::Button, b"scripted") },
            see(Find::text(FIRST)),
            gone(Find::text(&[FIRST, REST].concat())),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(matches!(
        world.root(person).person().pages().choice().context,
        Some(Context::Task { model_index: Some(0), .. })
    ));
    world
        .person(person, vec![Step::Press { find: Find::named(Role::Button, b"Reset choice") }, see(Find::text(FIRST))]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let draft = world.root(person).person().pages().choice();
    assert!(matches!(draft.context, Some(Context::Task { model_index: None, .. })));
    assert_eq!(draft.requested, draft.original, "reset restores the task's choice while its reply is held");
    assert_eq!(world.front().runs_answered, 0, "the composer acts without committing the reply");
    assert!(world.front().release_reply());
    assert!(world.front().release_reply());
    let words = fixtures::phrasebooks(&fixtures::domain_limits());
    world.person(
        person,
        vec![see(Find::text(&[FIRST, REST].concat())), gone(Find::text(&words.say_card(P::Provisional, &[])))],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
