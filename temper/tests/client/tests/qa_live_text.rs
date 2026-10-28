//! Design-only QA at the separate piece/commit boundary (domain/testing.md, 6.3).
use jig_client_world::harness::Until;
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_world::domain::heap::Counting;
use temper_client_world::fixtures;

#[global_allocator]
static HEAP: Counting = Counting;

const WRITING: &[u8] = b"Writing...";
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
