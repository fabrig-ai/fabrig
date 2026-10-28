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
