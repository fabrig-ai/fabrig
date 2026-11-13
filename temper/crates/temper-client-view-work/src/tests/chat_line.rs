use super::{fixtures, fixtures::first_run as f};

use crate::{
    look,
    pages::{
        chat::{self, Drawn, Speakers},
        chat_line::Run,
    },
};

use jig_client::{
    contract as c,
    thread::{Reading, Thread},
};

use skein_lib::{List, Token, Wall};

use skein_ui::{Builder, Role, Tree};

const CALLS: u32 = 16;

const TEXT: u32 = 4096;

const BYTES: u64 = 65536;

fn call(number: u64, tool: &[u8], input: &[u8], outcome: Option<&[u8]>) -> c::Call {
    let c::Block::Call(call) = f::call(number, tool, input, outcome) else { panic!("call") };
    call
}

fn current() -> c::CurrentCall {
    c::CurrentCall {
        identity: c::CallIdentity { owner: Token::new(1), sequence: 1, retry: 0, id: f::text(b"current") },
        tool: f::text(b"settings"),
        input: f::text(br#"{"setting":{"kind":"template","number":2}}"#),
        cut: false,
        began: None,
    }
}

fn line(
    calls: &[c::Call],
    summaries: &[(u64, c::Text)],
    current: Option<&c::CurrentCall>,
    running: bool,
    seconds: u64,
) -> Tree {
    line_with_words(calls, summaries, current, running, seconds, &fixtures::browser_words())
}

fn line_with_words(
    calls: &[c::Call],
    summaries: &[(u64, c::Text)],
    current: Option<&c::CurrentCall>,
    running: bool,
    seconds: u64,
    words: &crate::words::Words,
) -> Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut run = Run::with_capacity(CALLS);
    for (at, call) in calls.iter().enumerate() {
        run.call(
            call,
            Wall::from_nanos(
                u64::try_from(at).unwrap().checked_mul(seconds).unwrap().checked_mul(1_000_000_000).unwrap(),
            ),
            None,
        );
    }
    for (call, summary) in summaries {
        run.result(*call, summary);
    }
    run.flush(&mut b, current, running, words, false);
    b.finish()
}

fn disclosure(tree: &Tree) -> &skein_ui::Node {
    for node in tree.nodes() {
        if node.role == Role::Disclosure {
            return node;
        }
    }
    panic!("run disclosure")
}

fn thread(messages: &[(&[c::Message], u64)]) -> Thread {
    thread_after(messages, None)
}

fn thread_after(messages: &[(&[c::Message], u64)], before: Option<(c::Activity, u64)>) -> Thread {
    let mut thread = Thread::new(1, &jig_client::thread::Limits { lines: CALLS, bytes: BYTES, text: TEXT, live: TEXT });
    thread.reading = Some(Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(CALLS);
    if let Some((line, seconds)) = before {
        rows.push(c::DecodedActivityLine {
            source: Box::new(c::ActivityLine {
                task: 1,
                position: 1,
                at: Wall::from_nanos(seconds.checked_mul(1_000_000_000).unwrap()),
                line,
            }),
            decoded: None,
        })
        .unwrap();
    }
    for (at, (messages, seconds)) in messages.iter().enumerate() {
        let position = rows.len().checked_add(1).unwrap();
        let at = u32::try_from(at.checked_add(1).unwrap()).unwrap();
        let mut turn = f::turn(u64::from(position), 1, at, messages, TEXT);
        turn.source.at = Wall::from_nanos(seconds.checked_mul(1_000_000_000).unwrap());
        if let Some(decoded) = &mut turn.decoded {
            decoded.at = turn.source.at;
        }
        rows.push(turn).unwrap();
    }
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    thread
}

fn conversation(thread: &Thread, drawn: Drawn<'_>) -> Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    chat::conversation_with_times(
        &mut b,
        thread,
        &fixtures::browser_words(),
        (CALLS, TEXT),
        Speakers { person: None, requester: true },
        (drawn, Some(&mut crate::words::Times::new(Wall::EPOCH))),
        &mut crate::bindings::Bindings::new(128),
    );
    b.finish()
}

#[test]
fn a_run_spans_turns_and_ends_at_a_text() {
    let calls = [f::message(c::Speaker::Assistant, &[f::call(1, b"read", b"{}", None)])];
    let ending = [f::message(
        c::Speaker::Assistant,
        &[f::call(2, b"read", b"{}", None), c::Block::Text(f::text(b"answer")), f::call(3, b"wait", b"{}", None)],
    )];
    let tree = conversation(&thread(&[(&calls, 0), (&ending, 29)]), Drawn::Chat { request: None, delegations: None });
    let mut lines = List::with_capacity(CALLS);
    for node in tree.nodes() {
        if node.role == Role::Disclosure {
            lines.push(node).unwrap();
        }
    }
    assert_eq!(lines.len(), 2);
    assert_eq!(lines.get(0).unwrap().name.as_deref(), Some("Read 2 files".as_bytes()));
    assert_eq!(lines.get(0).unwrap().description.as_deref(), Some(b"29 s".as_slice()));
    let mut turns = 0_usize;
    for node in tree.nodes() {
        if node.key.is_some() && node.styles.as_slice().contains(&look::TURN) {
            turns = turns.checked_add(1).unwrap();
        }
    }
    assert_eq!(turns, 3, "two runs and one answer, no calls-only turn shell");
}

fn call_messages() -> [c::Message; 1] {
    [f::message(c::Speaker::Assistant, &[f::call(1, b"read", b"{}", None)])]
}

fn answer_messages() -> [c::Message; 1] {
    [f::message(c::Speaker::Assistant, &[c::Block::Text(f::text(b"answer"))])]
}

fn chat_tree(source: &Thread) -> Tree {
    conversation(source, Drawn::Chat { request: None, delegations: None })
}

#[test]
fn a_run_of_one_turn_measures_from_the_line_before() {
    let tree = chat_tree(&thread(&[(&answer_messages(), 12), (&call_messages(), 41)]));
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"29 s".as_slice()));
}

#[test]
fn a_run_after_words_measures_from_them() {
    let source = thread_after(
        &[(&call_messages(), 41)],
        Some((
            c::Activity::Words {
                message: 1,
                by: c::Requester::Person(1),
                kind: c::MessageKind::Words,
                words: f::text(b"please read"),
            },
            12,
        )),
    );
    assert_eq!(disclosure(&chat_tree(&source)).description.as_deref(), Some(b"29 s".as_slice()));
}

#[test]
fn a_hidden_line_says_when_a_run_began() {
    let mut source = thread(&[(&answer_messages(), 12), (&call_messages(), 3641)]);
    let mut rows = List::with_capacity(CALLS);
    let mut last = source.lines.rows.get(0).unwrap().clone();
    last.source.position = 3;
    rows.push(last).unwrap();
    rows.push(c::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: 1,
            position: 2,
            at: Wall::from_nanos(3_612_000_000_000),
            line: c::Activity::RunBegan { attempt: 1, why: c::RunWhy::Made, resumed: false },
        }),
        decoded: None,
    })
    .unwrap();
    rows.push(source.lines.rows.get(1).unwrap().clone()).unwrap();
    source.lines.rows = rows;
    assert_eq!(disclosure(&chat_tree(&source)).description.as_deref(), Some(b"29 s".as_slice()));
}

#[test]
fn a_run_across_turns_measures_from_the_line_before_its_first() {
    let calls = call_messages();
    let tree = chat_tree(&thread(&[(&answer_messages(), 5), (&calls, 10), (&calls, 20), (&calls, 30)]));
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"25 s".as_slice()));
}

#[test]
fn a_run_at_the_windows_head_has_no_line_to_measure_from() {
    let calls = call_messages();
    assert_eq!(disclosure(&chat_tree(&thread(&[(&calls, 10)]))).description, None);
    assert_eq!(
        disclosure(&chat_tree(&thread(&[(&calls, 10), (&calls, 20)]))).description.as_deref(),
        Some(b"10 s".as_slice())
    );
}

#[test]
fn a_turns_second_run_has_no_time() {
    let split = [f::message(
        c::Speaker::Assistant,
        &[f::call(1, b"read", b"{}", None), c::Block::Text(f::text(b"answer")), f::call(2, b"wait", b"{}", None)],
    )];
    let tree = chat_tree(&thread(&[(&answer_messages(), 12), (&split, 41)]));
    let mut descriptions = List::with_capacity(CALLS);
    for node in tree.nodes() {
        if node.role == Role::Disclosure {
            descriptions.push(node.description.as_deref()).unwrap();
        }
    }
    assert_eq!(descriptions.as_slice(), [Some(b"29 s".as_slice()), None]);
}

#[test]
fn a_run_under_a_second_has_no_time() {
    let mut source = thread(&[(&answer_messages(), 12), (&call_messages(), 12)]);
    let last = source.lines.rows.get_mut(0).unwrap();
    last.source.at = Wall::from_nanos(12_500_000_000);
    last.decoded.as_mut().unwrap().at = last.source.at;
    assert_eq!(disclosure(&chat_tree(&source)).description, None);
}

#[test]
fn a_running_run_says_the_call_under_way_and_takes_the_dots() {
    let tree = line(&[call(1, b"settings", b"{}", None)], &[], Some(&current()), true, 29);
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"checking template 2".as_slice()));
    assert!(disclosure(&tree).styles.as_slice().contains(&look::WORK_RUNNING));
    let mut task = super::chat::asking_task();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
    let root = super::chat::asking_task_root(task, true, 1);
    let messages = [f::message(c::Speaker::Assistant, &[f::call(1, b"read", b"{}", None)])];
    let tree = conversation(
        &thread(&[(&answer_messages(), 12), (&messages, 41)]),
        Drawn::ChatPage { request: None, root: &root },
    );
    assert_eq!(disclosure(&tree).description, None);
    assert!(disclosure(&tree).styles.as_slice().contains(&look::WORK_RUNNING));
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::TYPING));
    }
}

#[test]
fn an_agents_page_folds_results_into_rows() {
    let messages = [
        f::message(c::Speaker::Assistant, &[f::call(1, b"read", b"{}", None)]),
        f::message(
            c::Speaker::Tool,
            &[c::Block::Result { call: 1, summary: f::text(b"first line\nbody"), failed: false }],
        ),
    ];
    let tree = conversation(&thread(&[(&messages, 0)]), Drawn::Lines);
    assert!(fixtures::find(&tree, Role::Paragraph, b"read: first line").is_some());
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::BY_TOOL));
    }
}

#[test]
fn a_card_and_a_write_are_not_counted_and_end_a_run() {
    let object = c::ObjectKey::Proposal(fixtures::chat::PROPOSAL);
    let mut proposal = call(2, b"propose", b"{}", None);
    proposal.objects = List::with_capacity(1);
    proposal.objects.push(object).unwrap();
    let first = [f::message(
        c::Speaker::Assistant,
        &[f::call(1, b"read", b"{}", None), c::Block::Call(proposal), f::call(3, b"wait", b"{}", None)],
    )];
    let second = [f::message(
        c::Speaker::Assistant,
        &[
            f::call(4, b"note", b"{}", Some("Note 1 revision 1 written · 12 lines".as_bytes())),
            f::call(5, b"read", b"{}", None),
        ],
    )];
    let source = thread(&[(&first, 0), (&second, 0)]);
    let root = fixtures::chat::proposal_root_with_activity(Some(c::ActivityPage {
        rows: source.lines.rows.clone(),
        before: None,
        after: None,
        cut: false,
    }));
    let tree = conversation(&source, Drawn::ChatPage { request: None, root: &root });
    let mut names = List::with_capacity(CALLS);
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&look::WORK_RUN) {
            names.push(node.name.as_deref().unwrap()).unwrap();
        }
    }
    assert_eq!(names.as_slice(), &[b"Read 1 file".as_slice(), b"wait", b"Read 1 file"]);
    assert!(fixtures::find(&tree, Role::Button, b"Show").is_some());
}

#[test]
fn a_hidden_line_keeps_the_run_together() {
    let mut source = Thread::new(1, &jig_client::thread::Limits { lines: CALLS, bytes: BYTES, text: TEXT, live: TEXT });
    source.reading = Some(Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(3);
    rows.push(f::turn(1, 1, 1, &[f::message(c::Speaker::Assistant, &[f::call(1, b"read", b"{}", None)])], TEXT))
        .unwrap();
    rows.push(c::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: 1,
            position: 2,
            at: Wall::EPOCH,
            line: c::Activity::Milestone { position: 2 },
        }),
        decoded: None,
    })
    .unwrap();
    rows.push(f::turn(3, 1, 2, &[f::message(c::Speaker::Assistant, &[f::call(2, b"read", b"{}", None)])], TEXT))
        .unwrap();
    source.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let tree = conversation(&source, Drawn::Chat { request: None, delegations: None });
    assert_eq!(disclosure(&tree).name.as_deref(), Some("Read 2 files".as_bytes()));
}
