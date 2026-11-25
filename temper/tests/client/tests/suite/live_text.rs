//! Scenarios at the separate piece/commit boundary (testing/checks.md, 3).
use super::terminal::{LineTerminal, Page};
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_world::domain::Verdict;
use temper_client_world::{app::Temper, fixtures, front::Joint};

const WRITING: &[u8] = "▍".as_bytes();
const PARTS: &[&[u8]] = &[b"QA joint prefix", b" and its full suffix."];
const WHOLE: &[u8] = b"QA joint prefix and its full suffix.";

const REQUEST: &str = "Hello";
const PROMPT: &str = "What do you want done?";
const CURSOR: &str = "▍";
const FIRST: &str = "The answer";
const REST: &str = " is forty-two.";
const LIVE: &[&[u8]] = &[FIRST.as_bytes(), REST.as_bytes()];
// Step through each redraw, including removals and inserts around commitment.
const PAGE_STEPS: u32 = 256;
// The two real-loop chat fixtures use this seed.
const LINE_SEED: u64 = 7;

#[test]
fn a_line_terminal_prints_a_chat_from_prefix_to_commit() {
    // Keep both the held answer and the straight-to-commit answer from the real loop.
    line_chat(Some(LIVE), "The answer is forty-two.");
    line_chat(None, "World answer");
}

fn line_chat(pieces: Option<&'static [&'static [u8]]>, reply: &'static str) {
    let mut world = fixtures::terminal::open_chats_world(LINE_SEED);
    if let Some(pieces) = pieces {
        world.front().reply_in_pieces(pieces);
    } else {
        world.front().reply_at_once(reply.as_bytes());
    }
    let client = fixtures::terminal::chats_client(&mut world);
    let mut terminal = LineTerminal::new(LINE_SEED);
    read_until(&mut terminal, &mut world, client, |page| page.contains("New draft or change"));
    let change = control(&terminal.pages.last().expect("Chats page").text, "Change").to_owned();
    terminal.type_line(&change);
    read_until(&mut terminal, &mut world, client, |page| page.contains(PROMPT));
    let field = control(&terminal.pages.last().expect("prompt page").text, PROMPT);
    terminal.type_line(&format!("{field} {REQUEST}"));

    if pieces.is_some() {
        read_until(&mut terminal, &mut world, client, |page| page.contains(FIRST));
        let first = &terminal.pages.last().expect("first piece's page").text;
        assert_eq!(writing(first).as_deref(), Some(FIRST), "first piece under Temper, before the cursor:\n{first}");
        assert!(!first.contains(REST), "the suffix is still held:\n{first}");
        assert_eq!(world.front().runs_answered, 0, "the first piece is still held");
        assert!(world.front().release_reply(), "release the suffix");
        read_until(&mut terminal, &mut world, client, |page| writing(page).as_deref() == Some(reply));
        assert!(world.front().release_reply(), "release the commit");
    }
    read_until(&mut terminal, &mut world, client, |page| {
        page.contains(reply) && !page.contains(CURSOR) && page.contains("waiting on your words")
    });
    terminal.type_line("q");
    for _ in 0..PAGE_STEPS {
        terminal.step(&mut world, client);
        if terminal.drained() {
            check_pages(&terminal.pages, pieces.is_some(), reply);
            return;
        }
    }
    panic!("the terminal did not drain after q");
}

fn read_until(
    terminal: &mut LineTerminal,
    world: &mut World<Temper, Joint>,
    client: ClientId,
    seen: impl Fn(&str) -> bool,
) {
    for _ in 0..PAGE_STEPS {
        if terminal.step(world, client).is_some_and(&seen) {
            return;
        }
    }
    panic!("expected page did not print; last page: {:?}", terminal.pages.last().map(|page| &page.text));
}

fn control<'a>(page: &'a str, label: &str) -> &'a str {
    page.lines()
        .filter_map(|line| line.trim_start_matches([' ', '-']).strip_prefix('[')?.split_once("] "))
        .find(|(number, name)| {
            number.parse::<u32>().is_ok()
                && (name == &label || name.strip_prefix(label).is_some_and(|after| after.starts_with(": ")))
        })
        .unwrap_or_else(|| panic!("missing numbered {label:?} in:\n{page}"))
        .0
}

fn assistant(page: &str) -> Option<(&str, &str)> {
    page.split_once("Conversation:\n")?.1.split_once("Temper\n")
}

fn writing(page: &str) -> Option<String> {
    let (before, turn) = assistant(page)?;
    let (_, indent) = before.rsplit_once('\n')?;
    let (prefix, _) = turn.split_once("▍\n")?;
    Some(prefix.lines().map_while(|line| line.strip_prefix(indent)).collect::<Vec<_>>().join("\n"))
}

fn committed(page: &str, reply: &str) -> bool {
    let Some((_, turn)) = assistant(page) else { return false };
    let mut lines = turn.lines().map(str::trim_ascii);
    lines.next().is_some_and(|clock| {
        matches!(clock.as_bytes(), [b'0'..=b'2', b'0'..=b'9', b':', b'0'..=b'5', b'0'..=b'9']) && &clock[..2] < "24"
    }) && lines.next() == Some(reply)
}

fn check_pages(pages: &[Page], live: bool, reply: &str) {
    let first = pages
        .iter()
        .find(|page| page.text.contains(if live { FIRST } else { reply }))
        .expect("a page prints the reply");
    assert!(first.commands >= 2, "the reply follows the person's commands:\n{}", first.text);
    let commit = pages
        .iter()
        .position(|Page { text, .. }| committed(text, reply) || (text.contains(reply) && !text.contains(CURSOR)))
        .expect("a committed page prints the whole reply");
    let first_commit = &pages[commit].text;
    assert!(committed(first_commit, reply), "the whole reply is the assistant's line after its clock:\n{first_commit}");
    assert!(first_commit.lines().next().expect("chat title").contains(REQUEST), "{first_commit}");

    let mut previous = 0;
    for Page { text, .. } in &pages[..commit] {
        if text.contains(CURSOR) {
            let prefix = writing(text).expect("the cursor follows the provisional assistant line");
            assert!(reply.starts_with(&prefix), "not a prefix of the reply:\n{text}");
            assert!(prefix.len() >= previous, "the provisional text shrank:\n{text}");
            previous = prefix.len();
        }
    }
    for Page { text, .. } in &pages[commit..] {
        assert!(!text.contains(CURSOR), "the cursor remains on or after the committed page:\n{text}");
    }
}

fn whole_reply_before_commit() -> (World<Temper, Joint>, ClientId) {
    // testing/checks.md, 3: the text and its committed turn are separate releases.
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
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert!(matches!(TreeFace::new(world.tree(client)).find(&Find::text(WHOLE)), Found::Absent));
    assert!(world.front().release_reply(), "release the second piece without committing");
    world.person(
        client,
        vec![
            Step::See { find: Find::text(WHOLE), within: fixtures::WITHIN },
            Step::See { find: Find::text(WRITING), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    (world, client)
}

#[test]
fn full_live_text_stays_provisional_until_the_turn_commits() {
    let (mut world, client) = whole_reply_before_commit();
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
fn the_words_stay_on_the_page_at_every_step_of_a_turns_end() {
    // Enough delivery turns for the commit, its activity read and the final redraw.
    const ENDING_STEPS: u32 = 256;
    let (mut world, client) = whole_reply_before_commit();
    assert!(world.front().release_reply(), "release the turn's commit");
    for step in 0..ENDING_STEPS {
        assert_eq!(world.run(Until::Steps(1)).verdict, Verdict::Passed);
        let mut face = TreeFace::new(world.tree(client));
        assert!(!matches!(face.find(&Find::text(WHOLE)), Found::Absent), "words disappeared at ending step {step}");
        if matches!(face.find(&Find::text(WRITING)), Found::Absent) {
            return;
        }
    }
    panic!("the writing mark remained after {ENDING_STEPS} ending steps");
}

#[test]
fn returning_to_the_chat_reads_the_held_prefix_and_its_commit() {
    // client/pages/README.md: leaving closes handles; the page watches what it shows.
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
    let mut whole = pieces[0].to_vec();
    for piece in &pieces[1..] {
        assert!(world.front().release_reply());
        whole.extend_from_slice(piece);
        world.person(
            client,
            vec![
                Step::See { find: Find::text(&whole), within: fixtures::WITHIN },
                Step::See { find: Find::text(WRITING), within: fixtures::WITHIN },
            ],
        );
        assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    }
}

#[test]
fn a_live_answer_past_a_fields_window_arrives_whole() {
    static MIDDLE: [u8; 1024] = [b'm'; 1024];
    static PIECES: &[&[u8]] = &[b"QA beginning ", &MIDDLE];
    live_answer_arrives_whole(PIECES);
}

#[test]
fn a_live_answer_past_the_engines_retained_bytes_arrives_whole() {
    static PIECES: &[&[u8]] = &[b"QA followed from the start ", &FIRST_LONG_PIECE, &SECOND_LONG_PIECE];
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
