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
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut run = Run::with_capacity(CALLS);
    for (at, call) in calls.iter().enumerate() {
        run.call(
            call,
            Wall::from_nanos(
                u64::try_from(at).unwrap().checked_mul(seconds).unwrap().checked_mul(1_000_000_000).unwrap(),
            ),
        );
    }
    for (call, summary) in summaries {
        run.result(*call, summary);
    }
    run.flush(&mut b, current, running, &fixtures::words());
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
    let mut thread = Thread::new(1, &jig_client::thread::Limits { lines: CALLS, bytes: BYTES, text: TEXT, live: TEXT });
    thread.reading = Some(Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(CALLS);
    for (at, (messages, seconds)) in messages.iter().enumerate() {
        let at = u32::try_from(at.checked_add(1).unwrap()).unwrap();
        let mut turn = f::turn(u64::from(at), 1, at, messages, TEXT);
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
        &mut crate::frame::Bindings::new(128),
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
    assert_eq!(lines.get(0).unwrap().name.as_deref(), Some("read ×2".as_bytes()));
    assert_eq!(lines.get(0).unwrap().description.as_deref(), Some(b"29 s".as_slice()));
    let mut turns = 0_usize;
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&look::TURN) {
            turns = turns.checked_add(1).unwrap();
        }
    }
    assert_eq!(turns, 3, "two runs and one answer, no calls-only turn shell");
}
#[test]
fn read_docs_counts_topics_not_calls() {
    let tree = line(
        &[
            call(1, b"read_docs", br#"{"topic":"hold"}"#, None),
            call(2, b"read_docs", br#"{"topic":"hold","from":12}"#, None),
            call(3, b"read_docs", br#"{"search":"word"}"#, None),
        ],
        &[],
        None,
        false,
        0,
    );
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Read 2 docs".as_slice()));
}
#[test]
fn one_call_alone_says_what_it_touched() {
    for (tool, input, expected) in [
        (b"read_docs".as_slice(), br#"{"topic":"hold"}"#.as_slice(), b"Read the page hold".as_slice()),
        (b"read_docs", br#"{"search":"word"}"#, b"Searched the docs for word"),
        (b"settings", br#"{"setting":{"kind":"template","number":2}}"#, b"Checked template 2"),
    ] {
        assert_eq!(
            disclosure(&line(&[call(1, tool, input, None)], &[], None, false, 0)).name.as_deref(),
            Some(expected)
        );
    }
}
#[test]
fn a_tool_with_no_words_is_its_name_counted() {
    assert_eq!(
        disclosure(&line(
            &[call(1, b"read_file", b"{}", None), call(2, b"read_file", b"{}", None)],
            &[],
            None,
            false,
            0
        ))
        .name
        .as_deref(),
        Some("read_file ×2".as_bytes())
    );
}
#[test]
fn a_failed_call_is_said_on_the_line_and_its_row() {
    let mut failed = call(1, b"read_file", b"{}", Some(b"denied\nbody"));
    failed.failed = true;
    let tree = line(&[failed.clone()], &[], None, false, 0);
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"1 failed".as_slice()));
    assert!(disclosure(&tree).styles.as_slice().contains(&look::WORK_FAILED));
    assert!(fixtures::find(&tree, Role::Paragraph, b"read_file: failed: denied").is_some());
    for node in tree.nodes() {
        if node.role == Role::Paragraph {
            assert!(node.styles.as_slice().contains(&look::WORK_FAILED));
        }
    }
    let tree = line(&[failed.clone()], &[], Some(&current()), true, 0);
    assert_eq!(disclosure(&tree).styles.len(), 5);
    failed.outcome = None;
    let tree = line(&[failed], &[], None, false, 0);
    assert!(fixtures::find(&tree, Role::Paragraph, b"read_file: failed: ").is_some());
}
#[test]
fn a_run_of_one_turn_has_no_time() {
    let tree = line(&[call(1, b"read", b"{}", None), call(2, b"read", b"{}", None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).description, None);
}
#[test]
fn a_running_run_says_the_call_under_way_and_takes_the_dots() {
    let tree = line(&[call(1, b"settings", b"{}", None)], &[], Some(&current()), true, 29);
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"checking template 2".as_slice()));
    assert!(disclosure(&tree).styles.as_slice().contains(&look::WORK_RUNNING));
    let mut task = super::chat::configuring_task();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
    let root = super::chat::configuring_task_root(task, true, 1);
    let messages = [f::message(c::Speaker::Assistant, &[f::call(1, b"read", b"{}", None)])];
    let tree = conversation(&thread(&[(&messages, 0)]), Drawn::ChatPage { request: None, root: &root });
    assert!(disclosure(&tree).styles.as_slice().contains(&look::WORK_RUNNING));
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::TYPING));
    }
}
#[test]
fn a_call_under_way_with_none_ended_is_a_line() {
    let tree = line(&[], &[], Some(&current()), true, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Checking template 2".as_slice()));
    assert!(fixtures::find(&tree, Role::Paragraph, b"settings template 2").is_some());
}
#[test]
fn a_row_never_holds_a_results_body() {
    let tree = line(&[call(1, b"read", b"{}", Some(b"summary\nsecret body"))], &[], None, false, 0);
    assert!(fixtures::find(&tree, Role::Paragraph, b"read: summary").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"secret body").is_none());
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
    assert_eq!(names.as_slice(), &[b"read".as_slice(), b"wait", b"read"]);
    assert!(fixtures::find(&tree, Role::Button, b"Show").is_some());
}
#[test]
fn the_terminal_prints_the_name_description_and_rows() {
    let tree =
        line(&[call(1, b"read", b"{}", Some(b"first")), call(2, b"read", b"{}", Some(b"last"))], &[], None, false, 29);
    let mut print = skein_ui_text::Print::new(&fixtures::tree_limits()).unwrap();
    skein_ui_text::render(&tree, b"temper", None, &mut print);
    assert!(skein_lib::bytes::find(print.text.as_slice(), "read ×2: 29 s".as_bytes()).is_some());
    assert!(skein_lib::bytes::find(print.text.as_slice(), b"`read`: first").is_some());
    assert!(skein_lib::bytes::find(print.text.as_slice(), b"`read`: last").is_some());
}
#[test]
fn rows_and_names_end_on_character_and_phrase_boundaries() {
    // The contract bounds a row at 160 bytes and a name at 256 (README.md, 3.5).
    let tool = [b'x'; 256];
    let tree = line(&[call(1, &tool, b"{}", Some(b"outcome"))], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_ref().unwrap().len(), 256);
    for node in tree.nodes() {
        if node.role == Role::Paragraph {
            assert_eq!(node.name.as_ref().unwrap().len(), 160);
        }
    }
    let tree = line(&[call(1, &tool, b"{}", None), call(2, b"another", b"{}", None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some("…".as_bytes()));
    let mut input = skein_lib::Writer::new(268);
    input.put(b"{\"topic\":\"").unwrap();
    input.put(&tool).unwrap();
    input.put(b"\"}").unwrap();
    let tree = line(&[call(1, b"read_docs", &input.finish(), None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_ref().unwrap().len(), 256);
    let mut name = skein_lib::Writer::new(258);
    name.put(&[b'x'; 252]).unwrap();
    name.put("ééé".as_bytes()).unwrap();
    let tree = line(&[call(1, &name.finish(), b"{}", None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_ref().unwrap().get(252..), Some("…".as_bytes()));
}
#[test]
fn settings_and_minutes_keep_their_counted_words() {
    let tree = line(&[call(1, b"settings", b"{}", None), call(2, b"settings", b"{}", None)], &[], None, false, 240);
    assert_eq!(disclosure(&tree).name.as_deref(), Some("Checked settings 2×".as_bytes()));
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"4 min".as_slice()));
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
    assert_eq!(disclosure(&tree).name.as_deref(), Some("read ×2".as_bytes()));
}
