//! Inbox questions and the conversation around them, client/README.md, 3.5–3.6.
use jig_client_world::harness::{ClientId, Fault, FrontFault, Until, World};
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_lib::Duration;
use skein_ui::Role;
use skein_world::domain::Verdict;
use std::fmt::Write;
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures as f, front::Joint};

type Pages = World<Temper, Joint>;
const QUESTION: &[u8] = b"Which approach first?\nKeep the fallback?";
const ANSWER: &[u8] = b"Use the careful approach.\nKeep the fallback: yes.";

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: f::WITHIN }
}

fn gone(find: Find) -> Step<Address> {
    Step::Gone { find, within: f::WITHIN }
}

fn setup() -> (Pages, ClientId, u64) {
    let mut world = f::world(f::SEED);
    let task = f::question(world.front(), QUESTION);
    let client = world.client(f::PARTY, f::inbox());
    (world, client, task)
}

fn drawing(world: &Pages, client: ClientId) -> String {
    let mut drawing = String::new();
    for node in world.tree(client).nodes() {
        let text: Vec<u8> = node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()).collect();
        writeln!(
            drawing,
            "{:?} size={} name={:?} text={:?}",
            node.role,
            node.size,
            node.name.as_deref().map(String::from_utf8_lossy),
            String::from_utf8_lossy(&text)
        )
        .expect("write drawing to String");
    }
    drawing
}

fn follow(world: &mut Pages, client: ClientId, steps: Vec<Step<Address>>) {
    world.person(client, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}\n{}", drawing(world, client));
}

fn draft(world: &mut Pages, client: ClientId) {
    follow(
        world,
        client,
        vec![
            see(Find::named(Role::TextArea, b"Answer words")),
            Step::Type { find: Find::named(Role::TextArea, b"Answer words"), words: ANSWER.into() },
        ],
    );
}

fn answer() -> Step<Address> {
    Step::Press { find: Find::exactly_named(Role::Button, b"Answer") }
}

fn node(world: &Pages, client: ClientId, find: &Find) -> skein_ui::NodeId {
    let Found::Node(node) = TreeFace::new(world.tree(client)).find(find) else {
        panic!("missing {find:?}\n{}", drawing(world, client));
    };
    node
}

// Inspect only the semantic drawing: every line is one paragraph, in one turn.
// Exact paragraph counts also catch a retry drawing the same answer twice.
fn turn(world: &Pages, client: ClientId, words: &[u8]) -> Vec<String> {
    let lines: Vec<&[u8]> = words.split(|byte| *byte == b'\n').collect();
    let nodes = world.tree(client).nodes();
    for line in &lines {
        assert_eq!(
            nodes.iter().filter(|node| node.role == Role::Paragraph && node.name.as_deref() == Some(*line)).count(),
            1,
            "one paragraph for {:?}\n{}",
            String::from_utf8_lossy(line),
            drawing(world, client)
        );
    }
    let first = nodes
        .iter()
        .position(|node| node.role == Role::Paragraph && node.name.as_deref() == Some(lines[0]))
        .expect("first paragraph");
    let start = (0..first)
        .rev()
        .find(|index| {
            nodes[*index].role == Role::Region
                && *index + usize::try_from(nodes[*index].size).expect("subtree size") > first
        })
        .expect("turn region");
    let end = start + usize::try_from(nodes[start].size).expect("subtree size");
    let paragraphs: Vec<String> = nodes[start..end]
        .iter()
        .filter(|node| node.role == Role::Paragraph)
        .map(|node| String::from_utf8(node.name.as_deref().expect("paragraph name").to_vec()).expect("UTF-8"))
        .collect();
    let expected: Vec<String> = lines.iter().map(|line| String::from_utf8_lossy(line).into_owned()).collect();
    assert_eq!(
        paragraphs.get(1..),
        Some(expected.as_slice()),
        "speaker then each line: {paragraphs:?}\n{}",
        drawing(world, client)
    );
    paragraphs
}

fn assert_person_answer(world: &Pages, client: ClientId) {
    assert_eq!(turn(world, client, ANSWER)[0], "You", "{}", drawing(world, client));
    // The delegate's question remains a separate turn when the answer arrives.
    let question = turn(world, client, QUESTION);
    assert!(question[0].strip_prefix('T').is_some_and(|number| number.parse::<u64>().is_ok()), "{question:?}");
}

#[test]
fn question_card_links_to_chat_whose_wait_is_for_you() {
    let (mut world, client, task) = setup();
    follow(
        &mut world,
        client,
        vec![
            see(Find::text(b"Needs you")),
            see(Find::text(b"Question")),
            see(Find::text(QUESTION)),
            see(Find::text(b"Waiting")),
            see(Find::text(b"Project 1")),
            see(Find::exactly_named(Role::Link, b"Inbox 1")),
            see(Find::exactly_named(Role::Link, format!("Open T{task} ↗").as_bytes())),
            Step::Press { find: Find::exactly_named(Role::Link, format!("Open T{task} ↗").as_bytes()) },
            see(Find::text(b"waiting on an answer from you")),
            Step::Go { address: f::chats_page() },
            see(Find::text(b"waiting on an answer from you")),
        ],
    );
}

#[test]
fn delegate_question_keeps_its_lines_and_its_speaker() {
    let (mut world, client, task) = setup();
    follow(
        &mut world,
        client,
        vec![
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Which approach first?")),
            see(Find::text(b"Keep the fallback?")),
        ],
    );
    let paragraphs = turn(&world, client, QUESTION);
    let delegate = paragraphs[0].strip_prefix('T').and_then(|number| number.parse::<u64>().ok());
    assert!(delegate.is_some_and(|number| number != task), "delegate name: {paragraphs:?}");
}

#[test]
fn question_and_answer_keep_utf8_and_lines_at_the_receiving_bound() {
    let bound = usize::try_from(f::domain_limits().client.domain.text).expect("text bound");
    let mut question = "Café <&>?\n".as_bytes().to_vec();
    question.resize(bound, b'q');
    let mut answer_words = "Oui — café.\n".as_bytes().to_vec();
    answer_words.resize(bound, b'a');
    let mut world = f::world(f::SEED);
    let task = f::question(world.front(), &question);
    let client = world.client(f::PARTY, f::inbox());
    follow(
        &mut world,
        client,
        vec![
            see(Find::text(&question)),
            see(Find::named(Role::TextArea, b"Answer words")),
            Step::Type { find: Find::named(Role::TextArea, b"Answer words"), words: answer_words.clone().into() },
            answer(),
            see(Find::text(b"Answered")),
            Step::Go { address: f::chat_page(task) },
            see(Find::text("Café <&>?".as_bytes())),
            see(Find::text("Oui — café.".as_bytes())),
        ],
    );
    let question_turn = turn(&world, client, &question);
    assert!(
        question_turn[0].strip_prefix('T').is_some_and(|number| number.parse::<u64>().is_ok()),
        "{question_turn:?}"
    );
    assert_eq!(turn(&world, client, &answer_words)[0], "You");
}

#[test]
fn answered_card_stays_until_inbox_is_read_again() {
    let (mut world, client, task) = setup();
    draft(&mut world, client);
    follow(
        &mut world,
        client,
        vec![
            answer(),
            see(Find::text(b"Answered")),
            see(Find::text(QUESTION)),
            gone(Find::exactly_named(Role::Link, b"Inbox 1")),
            see(Find::exactly_named(Role::Link, b"Inbox")),
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
            Step::Go { address: f::inbox() },
            gone(Find::named(Role::TextArea, b"Answer words")),
            gone(Find::text(QUESTION)),
            gone(Find::text(b"Answered")),
        ],
    );
}

#[test]
fn inbox_answer_is_your_multiline_turn_after_navigation_and_reload() {
    let (mut world, client, task) = setup();
    draft(&mut world, client);
    follow(
        &mut world,
        client,
        vec![
            answer(),
            see(Find::text(b"Answered")),
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
        ],
    );
    assert_person_answer(&world, client);
    follow(
        &mut world,
        client,
        vec![
            Step::Go { address: f::inbox() },
            see(Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")),
            Step::Go { address: f::chat_page(task) },
            see(Find::exactly_named(Role::Heading { level: 1 }, format!("T{task} · Question chat").as_bytes())),
            Step::Reload,
            // A client closed on any page opens at the inbox (client/README.md, 3.4).
            see(Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")),
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
        ],
    );
    assert_person_answer(&world, client);
}

#[test]
fn answer_survives_leaving_and_reopening_both_pages() {
    let (mut world, client, task) = setup();
    draft(&mut world, client);
    follow(
        &mut world,
        client,
        vec![
            answer(),
            see(Find::text(b"Answered")),
            Step::Go { address: f::chats_page() },
            see(Find::exactly_named(Role::Link, format!("T{task}").as_bytes())),
            Step::Press { find: Find::exactly_named(Role::Link, format!("T{task}").as_bytes()) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
        ],
    );
    assert_person_answer(&world, client);
    follow(
        &mut world,
        client,
        vec![
            Step::Go { address: f::inbox() },
            see(Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")),
            gone(Find::text(QUESTION)),
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
        ],
    );
    assert_person_answer(&world, client);
}

#[test]
fn double_press_waits_for_confirmation_and_delivers_one_answer() {
    let (mut world, client, task) = setup();
    draft(&mut world, client);
    world.fault(FrontFault::Slow { span: Duration::from_millis(100) });
    let button = node(&world, client, &Find::exactly_named(Role::Button, b"Answer"));
    world.fault(Fault::DoublePress { client, node: button });
    follow(
        &mut world,
        client,
        vec![
            see(Find::text(b"Pending")),
            gone(Find::text(b"Answered")),
            see(Find::text(b"Answered")),
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
        ],
    );
    assert_person_answer(&world, client);
}

#[test]
fn lost_confirmation_retries_without_losing_the_answer() {
    let (mut world, client, task) = setup();
    draft(&mut world, client);
    world.fault(FrontFault::NoAnswer);
    follow(
        &mut world,
        client,
        vec![
            answer(),
            see(Find::text(b"Pending")),
            gone(Find::text(b"Answered")),
            see(Find::text(b"Answered")),
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
        ],
    );
    assert_person_answer(&world, client);
}

#[test]
#[ignore = "the joint world's reload delivers its Inbox jump and a second sign-in late (tests/client/src/front.rs:868-876): a Go right after a reload at the Inbox is undone; needs jig's client-world reload-address hook"]
fn reload_during_pending_preserves_the_answer_request() {
    let (mut world, client, task) = setup();
    draft(&mut world, client);
    world.fault(FrontFault::Slow { span: Duration::from_millis(100) });
    follow(
        &mut world,
        client,
        vec![
            answer(),
            see(Find::text(b"Pending")),
            Step::Reload,
            see(Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")),
            Step::Go { address: f::chat_page(task) },
            see(Find::text(b"Use the careful approach.")),
            see(Find::text(b"Keep the fallback: yes.")),
        ],
    );
    assert_person_answer(&world, client);
}
