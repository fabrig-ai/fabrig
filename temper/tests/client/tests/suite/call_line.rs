//! Calls read through the client and engine (temper/docs/design/client/README.md, 3.5 and 5.9).
use jig_client::contract as c;
use jig_client_kit::harness::{ClientId, Front, Until, World};
use skein_fake_person::{Find, Step};
use skein_lib::Token;
use skein_ui::{Node, Role};
use skein_world::domain::Verdict;
use smith_domain_session::llm::{Block, Decoded, Message, Returned, Role as Speaker, ToolCall};
use temper_client_domain::Address;
use temper_client_world::{
    app::{Event, Temper},
    fixtures,
    front::Joint,
};

type CallsWorld = World<Temper, Joint>;
type Row<'a> = (&'a [u8], &'a [u8], &'a [u8]);
const WORKER: Token = Token::new(7);
const QUESTION: &[u8] = b"Read the documentation and check the settings";
const ANSWER: &[u8] = b"The documentation and settings agree.";
const HOLD: Row<'_> = (b"read_docs", br#"{"topic":"hold"}"#, b"Hold overview");
const CHANGE: Row<'_> = (b"read_docs", br#"{"topic":"change"}"#, b"Change overview");
const SETTINGS: Row<'_> = (b"settings", br#"{"setting":{"kind":"template","number":2}}"#, b"Template ready");

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: fixtures::WITHIN }
}

fn follow(world: &mut CallsWorld, person: ClientId, steps: Vec<Step<Address>>) {
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn chat() -> (CallsWorld, u64, ClientId) {
    let mut world = fixtures::open_world(fixtures::SEED);
    let task = fixtures::chat(world.front(), QUESTION);
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    follow(&mut world, person, vec![see(Find::exactly_named(Role::TextArea, b"Message"))]);
    (world, task, person)
}

/// Commit the same call/result messages that a worker's session sends.
fn commit(world: &mut CallsWorld, task: u64, rows: &[Row<'_>], text: Option<&[u8]>) {
    let driver = &mut world.front().engine;
    let run = driver.task(task).expect("running task");
    let attempt = run.attempt;
    let number = run.turn.checked_add(1).expect("story turn");
    let spent = run.run_spent;
    let mut calls = Vec::new();
    let mut results = Vec::new();
    for (index, (tool, input, outcome)) in rows.iter().enumerate() {
        let id = format!("call-{number}-{index}").into_bytes().into_boxed_slice();
        calls.push(Block::ToolCall(ToolCall {
            id: id.clone(),
            name: (*tool).into(),
            input: (*input).into(),
            call: Decoded::Historical,
            replay: None,
        }));
        results.push(Block::ToolResult {
            id,
            result: Returned::Text { text: (*outcome).into(), error: false, replay: None },
        });
    }
    let mut messages = Vec::new();
    if !calls.is_empty() {
        messages.push(Message { role: Speaker::Assistant, content: calls.into_boxed_slice() });
        messages.push(Message { role: Speaker::User, content: results.into_boxed_slice() });
    }
    if let Some(text) = text {
        messages.push(Message {
            role: Speaker::Assistant,
            content: Box::new([Block::Text { text: text.into(), replay: None }]),
        });
    }
    let mut turn = temper_worker_domain_world::protocol::session_turn(number, spent, 0, number == 1);
    turn.messages = messages.into_boxed_slice();
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("call turn encodes");
    driver.send(temper_engine_domain_world::protocol::turn(WORKER, task, attempt, number, spent, None, &body));
    driver.settle();
}

/// Supply the run delivery's call while the joint world's decoder cannot stay unanswered.
/// All other delivery fields and all committed turns come from the engine.
fn outstanding(world: &mut CallsWorld, task: u64) {
    let now = world.now();
    let mut supplied = false;
    for (person, mut event) in world.front().poll(now, fixtures::NOW) {
        if let Event::Root(temper_client_domain::Event::Client {
            event:
                jig_client::Event::Up {
                    up: jig_client::Up::Watched { event: c::WatchEvent::Delivered(c::Delivery::RunState(run)), .. },
                },
        }) = &mut event
            && run.task == task
        {
            let text = |bytes: &[u8]| {
                c::Text::new(bytes.into(), fixtures::domain_limits().client.domain.turn_text)
                    .expect("call snapshot fits")
            };
            run.call = Some(c::CurrentCall {
                identity: c::CallIdentity {
                    owner: Token::new(task),
                    sequence: run.committed_turn,
                    retry: 0,
                    id: text(b"checking-template"),
                },
                tool: text(SETTINGS.0),
                input: text(SETTINGS.1),
                cut: false,
                began: None,
            });
            supplied = true;
        }
        world.event(person, event);
    }
    assert!(supplied, "engine supplied the task's run delivery");
}

fn begin_checking(world: &mut CallsWorld, task: u64, person: ClientId) {
    commit(world, task, &[], Some(b"Checking the configuration."));
    outstanding(world, task);
    follow(world, person, vec![see(Find::exactly_named(Role::Disclosure, b"Checking template 2"))]);
}

fn conversation(world: &CallsWorld, person: ClientId) -> &[Node] {
    let nodes = world.tree(person).nodes();
    let at = nodes
        .iter()
        .position(|node| node.role == Role::Log && node.name.as_deref() == Some(b"Conversation"))
        .expect("conversation log");
    let log = nodes.get(at).expect("conversation node");
    let end = at.checked_add(usize::try_from(log.size).expect("log size")).expect("log end");
    nodes.get(at..end).expect("conversation subtree")
}

fn lines(world: &CallsWorld, person: ClientId, expected: &[&[u8]]) {
    let names: Vec<_> = conversation(world, person)
        .iter()
        .filter(|node| node.role == Role::Disclosure)
        .map(|node| node.name.as_deref().expect("named call line"))
        .collect();
    assert_eq!(names, expected, "one disclosure per uninterrupted run of calls");
}

fn description(world: &CallsWorld, person: ClientId, name: &[u8], expected: &[u8]) {
    let line = conversation(world, person)
        .iter()
        .find(|node| node.role == Role::Disclosure && node.name.as_deref() == Some(name))
        .expect("call line");
    assert_eq!(line.description.as_deref(), Some(expected), "the call under way is the line's description");
}

#[test]
fn calls_across_turns_read_as_one_line() {
    let (mut world, task, person) = chat();
    commit(&mut world, task, &[HOLD], None);
    commit(&mut world, task, &[CHANGE, CHANGE], None);
    commit(&mut world, task, &[SETTINGS], None);
    commit(&mut world, task, &[], Some(ANSWER));
    let name = b"Read 2 docs, checked settings";
    follow(&mut world, person, vec![see(Find::exactly_named(Role::Disclosure, name)), see(Find::text(ANSWER))]);
    lines(&world, person, &[name]);
    let nodes = conversation(&world, person);
    let line = nodes.iter().position(|node| node.name.as_deref() == Some(name)).expect("call line");
    let answer =
        nodes.iter().position(|node| node.text.iter().any(|run| run.text.as_ref() == ANSWER)).expect("answer text");
    assert!(answer > line, "the answer follows the call line");
}

#[test]
fn a_text_ends_the_line() {
    let (mut world, task, person) = chat();
    commit(&mut world, task, &[HOLD], None);
    commit(&mut world, task, &[], Some(ANSWER));
    commit(&mut world, task, &[SETTINGS], None);
    follow(
        &mut world,
        person,
        vec![
            see(Find::exactly_named(Role::Disclosure, b"Read the page hold")),
            see(Find::text(ANSWER)),
            see(Find::exactly_named(Role::Disclosure, b"Checked template 2")),
        ],
    );
    lines(&world, person, &[b"Read the page hold", b"Checked template 2"]);
}

#[test]
fn the_persons_words_end_the_line() {
    const WORDS: &[u8] = b"Check the template too";
    let (mut world, task, person) = chat();
    commit(&mut world, task, &[HOLD], None);
    follow(
        &mut world,
        person,
        vec![
            see(Find::exactly_named(Role::Disclosure, b"Read the page hold")),
            Step::Type { find: Find::exactly_named(Role::TextArea, b"Message"), words: WORDS.into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Send") },
            see(Find::text(WORDS)),
        ],
    );
    commit(&mut world, task, &[SETTINGS], None);
    follow(&mut world, person, vec![see(Find::exactly_named(Role::Disclosure, b"Checked template 2"))]);
    lines(&world, person, &[b"Read the page hold", b"Checked template 2"]);
}

#[test]
fn the_line_counts_up_and_says_the_call_under_way() {
    let (mut world, task, person) = chat();
    begin_checking(&mut world, task, person);
    lines(&world, person, &[b"Checking template 2"]);
    for (row, name) in [
        (HOLD, b"Read the page hold".as_slice()),
        (CHANGE, b"Read 2 docs"),
        ((b"read_docs".as_slice(), br#"{"topic":"index"}"#.as_slice(), b"Index overview".as_slice()), b"Read 3 docs"),
    ] {
        commit(&mut world, task, &[row], None);
        outstanding(&mut world, task);
        follow(&mut world, person, vec![see(Find::exactly_named(Role::Disclosure, name))]);
        lines(&world, person, &[name]);
        description(&world, person, name, "checking template 2…".as_bytes());
    }
}

#[test]
fn opened_it_holds_one_row_a_call() {
    const OUTCOME: &[u8] = b"Hold overview\nPrivate second line\nPrivate third line";
    let (mut world, task, person) = chat();
    commit(&mut world, task, &[(HOLD.0, HOLD.1, OUTCOME), CHANGE, SETTINGS], None);
    follow(
        &mut world,
        person,
        vec![
            see(Find::exactly_named(Role::Disclosure, b"Read 2 docs, checked settings")),
            see(Find::exactly_named(Role::Paragraph, b"read_docs hold: Hold overview")),
            see(Find::exactly_named(Role::Paragraph, b"read_docs change: Change overview")),
            see(Find::exactly_named(Role::Paragraph, b"settings template 2: Template ready")),
        ],
    );
    let nodes = conversation(&world, person);
    let line = nodes.iter().position(|node| node.role == Role::Disclosure).expect("call line");
    let end = line
        .checked_add(usize::try_from(nodes.get(line).expect("call line node").size).expect("line size"))
        .expect("line end");
    let rows: Vec<_> = nodes
        .get(line..end)
        .expect("call subtree")
        .iter()
        .filter(|node| node.role == Role::Paragraph)
        .map(|node| node.name.as_deref().expect("call row name"))
        .collect();
    assert_eq!(
        rows,
        [
            b"read_docs hold: Hold overview".as_slice(),
            b"read_docs change: Change overview",
            b"settings template 2: Template ready"
        ]
    );
    for node in world.tree(person).nodes() {
        for later in [b"Private second line".as_slice(), b"Private third line"] {
            assert!(node.name.as_deref().is_none_or(|name| skein_lib::bytes::find(name, later).is_none()));
            assert!(node.text.iter().all(|run| skein_lib::bytes::find(&run.text, later).is_none()));
        }
    }
}

#[test]
fn an_agents_page_says_the_call_under_way() {
    let mut world = fixtures::open_world(fixtures::SEED);
    let delegates = fixtures::goal_with_dependent_delegates(world.front());
    let task = delegates.first;
    let person = world.client(fixtures::PARTY, fixtures::task_page(task));
    follow(&mut world, person, vec![see(Find::text(b"Findings in progress"))]);
    begin_checking(&mut world, task, person);
    lines(&world, person, &[b"Checking template 2"]);
    description(&world, person, b"Checking template 2", "checking template 2…".as_bytes());
}
