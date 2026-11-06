//! Scenarios at the separate piece/commit boundary (domain/testing.md, 6.3).
use jig_client_kit::harness::Until;
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use temper_client_world::fixtures;

const WRITING: &[u8] = "▍".as_bytes();
const PARTS: &[&[u8]] = &[b"QA joint prefix", b" and its full suffix."];
const WHOLE: &[u8] = b"QA joint prefix and its full suffix.";

#[test]
fn full_live_text_stays_provisional_until_the_turn_commits() {
    // domain/testing.md, 6.3: the text and its committed turn are separate releases.
    let mut world = fixtures::open_world(fixtures::SEED);
    world.front().reply_in_pieces(PARTS);
    let task = fixtures::chat(world.front(), b"QA joint live text");
    let client = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(
        client,
        vec![
            Step::See { find: Find::text(PARTS[0]), within: fixtures::WITHIN },
            Step::See { find: Find::text(WRITING), within: fixtures::WITHIN },
        ],
    );
    world.run(Until::Done);
    assert!(matches!(TreeFace::new(world.tree(client)).find(&Find::text(WHOLE)), Found::Absent));
    assert!(world.front().release_reply(), "release the second piece without committing");
    world.person(
        client,
        vec![
            Step::See { find: Find::text(WHOLE), within: fixtures::WITHIN },
            Step::See { find: Find::text(WRITING), within: fixtures::WITHIN },
        ],
    );
    world.run(Until::Done);
    assert!(world.front().release_reply(), "release the turn's commit");
    world.person(
        client,
        vec![
            Step::See { find: Find::text(WHOLE), within: fixtures::WITHIN },
            Step::Gone { find: Find::text(WRITING), within: fixtures::WITHIN },
        ],
    );
    world.run(Until::Done);
    assert!(!world.front().release_reply(), "nothing remains to release after commit");
}

#[test]
fn returning_to_the_chat_reads_the_held_prefix_and_its_commit() {
    // client/README.md, 3.5: leaving closes handles; the page watches what it shows.
    // The real-loop typist cannot follow a numbered task link inside a list row.
    let mut world = fixtures::open_world(fixtures::SEED);
    world.front().reply_in_pieces(PARTS);
    let task = fixtures::chat(world.front(), b"QA leave and return");
    let address = fixtures::chat_page(task);
    let client = world.client(fixtures::PARTY, address.clone());
    world.person(client, vec![Step::See { find: Find::text(PARTS[0]), within: fixtures::WITHIN }]);
    world.run(Until::Done);
    world.go(client, fixtures::chats_page());
    world.person(client, vec![Step::Gone { find: Find::text(WRITING), within: fixtures::WITHIN }]);
    world.run(Until::Done);
    assert!(world.front().release_reply());
    world.go(client, address);
    world.person(
        client,
        vec![
            Step::See { find: Find::text(WHOLE), within: fixtures::WITHIN },
            Step::See { find: Find::text(WRITING), within: fixtures::WITHIN },
        ],
    );
    world.run(Until::Done);
    assert!(world.front().release_reply());
    world.person(
        client,
        vec![
            Step::See { find: Find::text(WHOLE), within: fixtures::WITHIN },
            Step::Gone { find: Find::text(WRITING), within: fixtures::WITHIN },
        ],
    );
    world.run(Until::Done);
}

// Each delivery fits the wire; together these cross the engine's retained tail.
static FIRST_LONG_PIECE: [u8; 40_000] = [b'a'; 40_000];
static SECOND_LONG_PIECE: [u8; 40_000] = [b'z'; 40_000];

fn live_answer_arrives_whole(pieces: &'static [&'static [u8]]) {
    use skein_world::domain::Verdict;
    let mut world = fixtures::world_with_whole_turns(fixtures::SEED);
    world.front().reply_in_pieces(pieces);
    let task = fixtures::chat(world.front(), b"QA whole live answer");
    let client = fixtures::whole_turn_client(&mut world, task);
    world.person(client, vec![Step::See { find: Find::text(pieces[0]), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert!(world.front().release_reply());
    let whole = pieces.concat();
    world.person(
        client,
        vec![
            Step::See { find: Find::text(&whole), within: fixtures::WITHIN },
            Step::See { find: Find::text(WRITING), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}

#[test]
fn a_live_answer_past_a_fields_window_arrives_whole() {
    static MIDDLE: [u8; 1024] = [b'm'; 1024];
    static PIECES: &[&[u8]] = &[b"QA beginning ", &MIDDLE];
    live_answer_arrives_whole(PIECES);
}

#[test]
fn a_live_answer_past_the_engines_retained_bytes_arrives_whole() {
    static PIECES: &[&[u8]] = &[&FIRST_LONG_PIECE, &SECOND_LONG_PIECE];
    live_answer_arrives_whole(PIECES);
}

#[test]
fn a_committed_turn_with_large_results_keeps_every_calls_name() {
    use skein_world::domain::Verdict;
    let mut world = fixtures::world_with_whole_turns(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"QA large results");
    fixtures::turn_with_large_results(world.front(), task);
    let client = fixtures::whole_turn_client(&mut world, task);
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"first_call"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"second_call"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"third_call"), within: fixtures::WITHIN },
            Step::Gone { find: Find::text(b"omitted"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}

#[test]
fn a_committed_turn_draws_no_opaque_block() {
    use skein_world::domain::Verdict;
    let mut world = fixtures::world_with_whole_turns(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"QA opaque block");
    fixtures::large_turn_with_opaque(world.front(), task, 32);
    let client = fixtures::whole_turn_client(&mut world, task);
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"!!!!"), within: fixtures::WITHIN },
            Step::Gone { find: Find::text(b"Content in"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
