//! A person reads a bounded conversation through the engine and store (client/README.md, 5.15).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Face, Find, Found, Step, Target, TreeFace};
use skein_ui::{LogEdge, Role};
use skein_world::domain::Verdict;
use temper_client_world::{app::Temper, fixtures, front::Joint};

type ChatWorld = World<Temper, Joint>;
const TURNS: usize = 10;

fn words(number: usize) -> Vec<u8> {
    format!("Reading turn {number:02}: {} end {number:02}.", "whole words ".repeat(20)).into_bytes()
}
fn see(text: &[u8]) -> Step<temper_client_domain::Address> {
    Step::See { find: Find::text(text), within: fixtures::WITHIN }
}
fn gone(text: &[u8]) -> Step<temper_client_domain::Address> {
    Step::Gone { find: Find::text(text), within: fixtures::WITHIN }
}
fn near(edge: LogEdge) -> Step<temper_client_domain::Address> {
    Step::Near { find: Find::named(Role::Log, b"Conversation"), edge }
}
fn person(world: &mut ChatWorld, client: ClientId, steps: Vec<Step<temper_client_domain::Address>>) {
    world.person(client, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn run(world: &mut ChatWorld, client: ClientId, steps: Vec<Step<temper_client_domain::Address>>) {
    person(world, client, steps);
    assert_eq!(TreeFace::new(world.tree(client)).find(&Find::exactly_named(Role::Button, b"Older")), Found::Absent);
    assert_eq!(TreeFace::new(world.tree(client)).find(&Find::text(b"omitted")), Found::Absent);
    assert_eq!(
        TreeFace::new(world.tree(client)).find(
            &Find::text(b"left")
                .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() },]))
        ),
        Found::Absent
    );
    for node in world.tree(client).nodes() {
        let text: Vec<u8> = node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()).collect();
        if text.starts_with(b"Reading turn ") {
            assert!((0..TURNS).any(|number| text == words(number)), "a turn is whole: {text:?}");
        }
    }
    for number in held(world, client) {
        assert_eq!(
            TreeFace::new(world.tree(client)).find(&Find::text(&words(number)).nth(1)),
            Found::Absent,
            "a turn appears once"
        );
    }
}
fn long_chat() -> (ChatWorld, ClientId, u64) {
    let mut world = fixtures::reading_world(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"Read the conversation");
    for number in 0..TURNS {
        fixtures::reading_turn(world.front(), task, &words(number));
    }
    let client = fixtures::reading_client(&mut world, task);
    run(&mut world, client, vec![see(&words(TURNS - 1))]);
    (world, client, task)
}
fn held(world: &ChatWorld, client: ClientId) -> Vec<usize> {
    (0..TURNS)
        .filter(|number| matches!(TreeFace::new(world.tree(client)).find(&Find::text(&words(*number))), Found::Node(_)))
        .collect()
}
fn start() -> Find {
    Find::text(b"Read the conversation")
        .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }]))
}
fn back(world: &mut ChatWorld, client: ClientId) {
    for _ in 0..TURNS {
        if matches!(TreeFace::new(world.tree(client)).find(&start()), Found::Node(_)) {
            return;
        }
        run(world, client, vec![near(LogEdge::Head)]);
    }
    panic!("the first turn is reachable");
}

#[test]
fn reading_back_then_forward_recovers_whole_lines_and_follows_new_turns() {
    let (mut world, client, task) = long_chat();
    let mut visited = held(&world, client);
    back(&mut world, client);
    run(
        &mut world,
        client,
        vec![Step::See { find: start(), within: fixtures::WITHIN }, see(&words(0)), gone(&words(TURNS - 1))],
    );
    for _ in 0..TURNS {
        if held(&world, client).contains(&(TURNS - 1)) {
            break;
        }
        visited.extend(held(&world, client));
        run(&mut world, client, vec![near(LogEdge::Foot)]);
    }
    run(&mut world, client, vec![see(&words(TURNS - 1)), gone(&words(0))]);
    visited.extend(held(&world, client));
    visited.sort_unstable();
    visited.dedup();
    assert_eq!(visited, (0..TURNS).collect::<Vec<_>>(), "every turn is recovered");
    fixtures::reading_turn(world.front(), task, b"A new turn at the end");
    run(&mut world, client, vec![see(b"A new turn at the end")]);
}

fn log(world: &ChatWorld, client: ClientId) -> &skein_ui::Node {
    world.tree(client).nodes().iter().find(|node| node.role == Role::Log).expect("conversation log")
}
fn latest() -> Step<temper_client_domain::Address> {
    Step::Press { find: Find::named(Role::Button, b"Latest") }
}

#[test]
fn latest_replaces_the_past_with_the_ends_window() {
    let (mut world, client, _) = long_chat();
    let end = vec![TURNS - 2, TURNS - 1];
    assert!(held(&world, client).ends_with(&end), "opening retains the latest two-row page");
    back(&mut world, client);
    assert!(log(&world, client).states.as_slice().contains(&skein_ui::State::Continues));
    let request = log(&world, client).end_request;
    run(&mut world, client, vec![latest(), see(&words(TURNS - 1)), gone(&words(0))]);
    assert_eq!(held(&world, client), end, "Latest replaces rather than extends the window");
    assert!(!log(&world, client).states.as_slice().contains(&skein_ui::State::Continues));
    assert_ne!(log(&world, client).end_request, request, "Latest asks the shell for the end");
}

#[test]
fn sending_from_the_past_returns_to_the_end_and_shows_the_persons_words() {
    let (mut world, client, _) = long_chat();
    back(&mut world, client);
    let request = log(&world, client).end_request;
    let composer = Find::named(Role::TextArea, b"Message");
    run(
        &mut world,
        client,
        vec![
            Step::Type { find: composer.clone(), words: b"My words from far back".as_slice().into() },
            Step::Send { find: composer },
            see(b"My words from far back"),
            see(&words(TURNS - 1)),
            gone(&words(0)),
        ],
    );
    assert_ne!(log(&world, client).end_request, request, "sending asks the shell for the end");
}

fn reading_snapshot(world: &ChatWorld, client: ClientId) -> Vec<(skein_ui::NodeId, Vec<u8>)> {
    world
        .tree(client)
        .nodes()
        .iter()
        .filter_map(|node| {
            let text: Vec<u8> = node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()).collect();
            (text.starts_with(b"Reading turn ") || text.starts_with(b"Long message ")).then_some((node.id, text))
        })
        .collect()
}

#[test]
fn live_text_and_its_commit_wait_while_the_person_reads_back() {
    const NEW: &[u8] = b"A live answer beyond this window";
    let (mut world, client, task) = long_chat();
    back(&mut world, client);
    let reading = reading_snapshot(&world, client);
    fixtures::reading_piece(world.front(), task, NEW);
    run(&mut world, client, vec![gone(NEW)]);
    assert_eq!(reading_snapshot(&world, client), reading, "live text leaves the words and nodes alone");
    run(&mut world, client, vec![latest(), see(NEW), see("▍".as_bytes())]);
    back(&mut world, client);
    let reading = reading_snapshot(&world, client);
    // The end is away again: a committed reply must also wait there.
    fixtures::reading_turn(world.front(), task, NEW);
    run(&mut world, client, vec![gone(NEW)]);
    assert_eq!(reading_snapshot(&world, client), reading, "the committed turn leaves the words and nodes alone");
    run(&mut world, client, vec![latest(), see(NEW), gone("▍".as_bytes())]);
}

#[test]
fn more_than_sixteen_short_lines_fit_when_read_back() {
    const LINES: usize = 20;
    let mut world = fixtures::world_with_whole_turns(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"A chat of short lines");
    for number in 0..LINES {
        fixtures::reading_turn(world.front(), task, format!("Short line {number:02}.").as_bytes());
    }
    let client = fixtures::whole_turn_client(&mut world, task);
    run(&mut world, client, vec![see(b"Short line 19.")]);
    for _ in 0..LINES {
        if matches!(TreeFace::new(world.tree(client)).find(&Find::text(b"Short line 00.")), Found::Node(_)) {
            break;
        }
        run(&mut world, client, vec![near(LogEdge::Head)]);
    }
    run(&mut world, client, (0..LINES).map(|number| see(format!("Short line {number:02}.").as_bytes())).collect());
}

#[test]
fn a_failed_head_read_says_so_and_the_next_near_retries() {
    let (mut world, client, _) = long_chat();
    let before = held(&world, client);
    world.front().activity_read_ends.push_back(jig_client::contract::ReadNone::Malformed);
    run(&mut world, client, vec![near(LogEdge::Head), see(b"Couldn't load earlier messages")]);
    assert_eq!(held(&world, client), before, "failure keeps the visible page");
    let status = world
        .tree(client)
        .nodes()
        .iter()
        .position(|node| {
            node.role == Role::Status && node.name.as_deref() == Some(b"Loading earlier messages".as_slice())
        })
        .expect("earlier-read status");
    let first = world
        .tree(client)
        .nodes()
        .iter()
        .position(|node| node.text.as_slice().iter().any(|run| run.text.starts_with(b"Reading turn ")))
        .expect("first turn");
    assert!(status < first, "failure is at the head");
    run(&mut world, client, vec![near(LogEdge::Head), gone(b"Couldn't load earlier messages")]);
    assert!(held(&world, client).first() < before.first(), "the retry reads earlier turns");
}

const LONG_MESSAGES: usize = 66;
fn long_words(number: usize, length: usize) -> Vec<u8> {
    let mut words = format!("Long message {number:02}: ").into_bytes();
    words.resize(length - 7, b'x');
    words.extend_from_slice(format!(" end{number:02}.").as_bytes());
    assert_eq!(words.len(), length);
    words
}

fn read_long_messages(agent: bool) {
    use temper_client_domain::Address;
    for length in [538, 1149] {
        let mut world = fixtures::long_messages_world(fixtures::SEED);
        let task = fixtures::long_messages_task(world.front(), agent);
        let messages: Vec<_> = (0..LONG_MESSAGES).map(|number| long_words(number, length)).collect();
        for (number, words) in messages.iter().enumerate() {
            fixtures::long_message(world.front(), task, u8::try_from(number + 1).expect("message key"), words);
        }
        let client = fixtures::long_messages_client(
            &mut world,
            if agent { Address::Task { task } } else { Address::Chat { task } },
        );
        world.person(client, vec![see(messages.last().expect("last message"))]);
        let outcome = world.run(Until::Done);
        assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
        if agent {
            assert!(
                world
                    .tree(client)
                    .nodes()
                    .iter()
                    .any(|node| node.styles.as_slice().contains(&temper_client_view::look::AGENT_PAGE))
            );
        }
        let mut visited = vec![false; LONG_MESSAGES];
        for _ in 0..LONG_MESSAGES {
            let mut face = TreeFace::new(world.tree(client));
            assert_eq!(face.find(&Find::text(b"Couldn't load earlier messages")), Found::Absent);
            for node in world.tree(client).nodes() {
                let text: Vec<u8> = node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()).collect();
                if text.starts_with(b"Long message ") {
                    let number = messages.iter().position(|message| *message == text).expect("a message draws whole");
                    visited[number] = true;
                }
            }
            if visited.iter().all(|seen| *seen) {
                break;
            }
            // Near the head decodes to Older on both pages.
            world.person(client, vec![near(LogEdge::Head)]);
            let outcome = world.run(Until::Done);
            assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
        }
        assert!(visited.iter().all(|seen| *seen), "every {length}-byte message is read to the start: {visited:?}");
    }
}

#[test]
fn sixty_six_long_messages_read_to_their_start() {
    read_long_messages(false);
}

#[test]
fn an_agents_page_reads_long_lines_through_older() {
    read_long_messages(true);
}

// A stored line one byte beyond the reading client's 1,024-byte long_text.
const UNREADABLE_BYTES: usize = 1025;
const RETAINED_BYTES: usize = 538;
const QUIET: skein_lib::Duration = skein_lib::Duration::from_secs(120);
const IMMEDIATE: skein_lib::Duration = skein_lib::Duration::from_secs(3);
// One requested read and one spare expose any duplicate.
const READ_PROBES: usize = 2;
const FAILED: &[u8] = b"Couldn't load earlier messages";

fn ended_page(agent: bool) -> (ChatWorld, ClientId, u64) {
    use temper_engine_domain_world::{store::Store, walking};
    let seed = fixtures::SEED;
    let mut world = fixtures::long_messages_world(seed);
    let mut limits = world.front().engine.env.limits;
    // The sign-in stays live through the quiet interval and the acts after it.
    limits.people.sign_in_lifetime = QUIET.saturating_mul(2);
    let front = Joint::with_engine(
        seed,
        &fixtures::reading_limits(),
        Store::with_limits(&limits),
        [walking::config(seed), walking::config(seed)],
        &limits,
        1,
    );
    *world.front() = front;
    let task = fixtures::long_messages_task(world.front(), agent);
    fixtures::long_message(world.front(), task, 1, &long_words(0, UNREADABLE_BYTES));
    for number in 1..TURNS {
        fixtures::long_message(
            world.front(),
            task,
            u8::try_from(number + 1).expect("message key"),
            &long_words(number, RETAINED_BYTES),
        );
    }
    let client = fixtures::reading_client(&mut world, task);
    if agent {
        world.go(client, temper_client_domain::Address::Task { task });
    }
    person(&mut world, client, vec![see(&long_words(TURNS - 1, RETAINED_BYTES))]);
    for _ in 0..TURNS {
        if matches!(TreeFace::new(world.tree(client)).find(&Find::text(FAILED)), Found::Node(_)) {
            break;
        }
        person(&mut world, client, vec![near(LogEdge::Head)]);
    }
    person(&mut world, client, vec![see(FAILED), gone("Loading earlier messages…".as_bytes())]);
    let kept = reading_snapshot(&world, client);
    assert!(!kept.is_empty(), "the failed read leaves other lines to show");
    // These ends observe any later activity requests; the oversized answer above
    // has already passed through the real link.
    world.front().activity_read_ends.extend([jig_client::contract::ReadNone::Malformed; READ_PROBES]);
    let until = world.now().saturating_add(QUIET);
    let outcome = world.run(Until::Time(until));
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(world.now() >= until, "observe the entire quiet interval");
    assert_eq!(world.front().activity_read_ends.len(), READ_PROBES, "a lasting end asks nothing in 120 seconds");
    assert_eq!(reading_snapshot(&world, client), kept, "the held lines keep their words and nodes");
    person(&mut world, client, vec![see(FAILED), gone("Loading earlier messages…".as_bytes())]);
    (world, client, task)
}

fn send_after_end(world: &mut ChatWorld, client: ClientId, task: u64) {
    const SENT: &[u8] = b"Sent after the read ended";
    world.front().activity_read_ends.clear();
    let composer = Find::named(Role::TextArea, b"Message");
    person(
        world,
        client,
        vec![Step::Type { find: composer.clone(), words: SENT.into() }, Step::Send { find: composer }],
    );
    assert!(
        world.front().engine.task(task).expect("live task").inbox.iter().any(|word| word.words.as_ref() == SENT),
        "the composer still commits its message after a read ends"
    );
}

#[test]
fn a_line_past_what_the_client_takes_ends_its_read_and_says_so() {
    let (mut world, client, task) = ended_page(false);
    let now = world.now();
    person(&mut world, client, vec![latest()]);
    assert!(world.now() < now.saturating_add(IMMEDIATE), "Latest reads within three seconds");
    assert_eq!(world.front().activity_read_ends.len(), 1, "Latest reads the end immediately");
    send_after_end(&mut world, client, task);
}

#[test]
fn an_agents_page_says_so_and_keeps_older() {
    let (mut world, client, task) = ended_page(true);
    person(&mut world, client, vec![near(LogEdge::Head), see(FAILED)]);
    assert_eq!(world.front().activity_read_ends.len(), 1, "the next move near the head retries once");
    send_after_end(&mut world, client, task);
}
