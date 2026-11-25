//! Replaying controls (smith/docs/design/testing-tiers.md).

use std::fmt::Write;
use std::path::Path;

use skein_fake_machine::Item;
use smith_bench::files::Files;
use smith_bench::{Count, EventCheck, Measure, Refusal, Task};
use smith_bench_fixtures::files::Tree;
use smith_events::{Effect, Event, Record, Source, Verdict};

use crate::common::{self, observe};
use crate::tests::check_probe;

// The authored oversized variant's native and protocol bounds.
const NATIVE_TOOL_PAYLOAD: u64 = 2048;
const CALL_INPUT: u64 = 13848;
// The miniature native refusal has content at and beyond its own bound.
const AT_PAYLOAD: usize = 10;
const SMALL_PAYLOAD: u64 = AT_PAYLOAD as u64;
const OVER_PAYLOAD: usize = 20;
const PAYLOAD: &str = "payload.txt";
// The miniature replay's declared file cap and unexercised protocol bound.
const FILE_BYTES: u64 = 64;
const PROTOCOL_INPUT_BOUND: u64 = 15;
// Retain a recorded size independent of the captured fixture content.
const RECORDED_INPUT_BYTES: u64 = 32_768;

pub(super) const LARGE_WRITE_DOCUMENT: &str = include_str!("../../tasks/probes/large-write/task.toml");
pub(super) const LARGE_WRITE_MANIFEST: &str = "tasks/probes/large-write/task.toml";

pub(super) fn check_large_write(document: &str) -> Result<Task, Refusal> {
    check_probe(
        Path::new(LARGE_WRITE_MANIFEST),
        document,
        &[Item::file(b"source.txt", include_bytes!("../../tasks/probes/large-write/seed/source.txt"))],
    )
}

fn payload() -> &'static Path {
    Path::new(PAYLOAD)
}

fn workspace() -> &'static Path {
    Path::new("/workspace")
}

fn settled_payload(content: &str) -> Tree {
    Tree::new(&[Item::directory(b"workspace"), Item::file(b"workspace/payload.txt", content.as_bytes())])
}

fn recorded_write(call: &str, content: &str, verdict: Verdict) -> Vec<Record> {
    let mut records = common::events::balanced();
    piece(&mut records, "write", call, &serde_json::json!({"path": PAYLOAD, "content": content}), verdict);
    records
}

fn piece(records: &mut Vec<Record>, tool: &str, call: &str, input: &serde_json::Value, verdict: Verdict) {
    let (mut started, mut completed) = common::tool_pair(tool, call);
    started.t_ms = 4;
    completed.t_ms = 4;
    let value = common::edit::tool_started(&mut started);
    value.source = Source::Workspace;
    value.effect = Effect::Write;
    value.input_bytes = RECORDED_INPUT_BYTES;
    value.input = Some(serde_json::to_vec(input).expect("captured arguments").into());
    common::edit::tool_completed(&mut completed).verdict = verdict;
    let at = records.len() - 3;
    records.splice(at..at, [started, completed]);
}

fn append_hello_world(records: &mut Vec<Record>, call: &str) {
    piece(
        records,
        "edit",
        call,
        &serde_json::json!({"path": PAYLOAD, "edits": [{"old": "hello", "new": "hello world"}]}),
        Verdict::Edited,
    );
}

fn protocol_refusal(records: &mut [Record], call: &[u8], input_bytes: u64) {
    let value = common::edit::response_completed(&mut records[4]);
    value.oversized = 1;
    let mut blocks = skein_lib::List::with_capacity(1);
    blocks
        .push(smith_events::Block::Oversized {
            call: call.into(),
            tool: b"write".as_slice().into(),
            bytes: input_bytes,
        })
        .expect("one protocol block");
    value.completion = Some(smith_events::Completion { blocks });
}

#[test]
fn oversized_calls_exceed_the_bound_and_repairs_fit() {
    let task = common::task("probes/large-write");
    let variant = task.with_variant("oversized").expect("small-payload variant");
    let lines = |first, last| {
        let mut result = String::new();
        for index in first..=last {
            writeln!(&mut result, "Line {index:04}: The quick brown fox jumps over the lazy dog.").expect("line");
        }
        result
    };
    let first = lines(1, 280);
    let input = serde_json::to_vec(&serde_json::json!({"path":"payload.txt","content":first})).expect("JSON framing");
    assert_eq!(input.len(), 15995);
    assert!(input.len() * 100 >= 13848 * 115);
    assert_eq!(lines(1, 60).len(), 3360);
    assert_eq!(lines(1, 30).len(), 1680);
    assert!(lines(30, 60).len() <= 2048);
    assert!(!variant.deciding.iter().any(|name| name == "file-digest:payload.txt"));
}

#[test]
fn oversized_write_without_a_call_past_the_bound_is_not_exercised() {
    let records = recorded_write("first", "Line 0001", Verdict::Written);
    let verdict = observe(&records).oversized_write(payload(), NATIVE_TOOL_PAYLOAD, Some(CALL_INPUT), workspace());
    assert!(matches!(&verdict.passed, Measure::Unavailable { reason } if reason.starts_with("not exercised")));
    assert!(verdict.evidence.iter().any(|line| line.starts_with("NotExercised")));
}

#[test]
fn payload_refusal_within_the_call_bound_is_not_exercised() {
    let records = recorded_write("first", &"x".repeat(OVER_PAYLOAD), Verdict::TooLarge);
    let verdict = observe(&records).oversized_write(payload(), SMALL_PAYLOAD, Some(CALL_INPUT), workspace());
    assert!(matches!(&verdict.passed, Measure::Unavailable { reason } if reason.starts_with("not exercised")));
    assert!(verdict.evidence.iter().any(|line| line.starts_with("NotExercised")));
}

#[test]
fn protocol_only_oversized_write_records_identity_and_elided_path() {
    let mut records = common::events::balanced();
    protocol_refusal(&mut records, b"refused", 15995);
    let checked = |records: &[smith_events::Record]| {
        observe(records).oversized_write(payload(), NATIVE_TOOL_PAYLOAD, Some(CALL_INPUT), workspace())
    };
    let positive = checked(&records);
    assert_eq!(positive.passed, Measure::Observed { value: true });
    assert!(positive.evidence.iter().any(|item| item.contains("not carried (arguments elided by design)")));
    assert!(positive.evidence.iter().any(|item| item.contains("call=refused")));
    let original = records.clone();
    for case in ["at-bound", "wrong-tool", "absent-content", "counter", "loss"] {
        records = original.clone();
        if case == "loss" {
            crate::common::edit::session_ended(records.last_mut().expect("ended")).loss.events = 1;
        } else {
            let response = crate::common::edit::response_completed(&mut records[4]);
            match case {
                "absent-content" => response.completion = None,
                "counter" => response.oversized = 2,
                "at-bound" | "wrong-tool" => {
                    if let smith_events::Block::Oversized { bytes, tool, .. } =
                        response.completion.as_mut().expect("captured").blocks.get_mut(0).expect("one block")
                    {
                        if case == "at-bound" {
                            *bytes = CALL_INPUT;
                        } else {
                            *tool = b"edit".as_slice().into();
                        }
                    }
                }
                _ => unreachable!("named negative"),
            }
        }
        if case == "absent-content" || case == "loss" {
            assert!(matches!(checked(&records).passed, Measure::Unavailable { .. }));
        } else {
            assert_eq!(checked(&records).passed, Measure::Observed { value: false });
        }
    }
    records = recorded_write("native", &"x".repeat(OVER_PAYLOAD), Verdict::TooLarge);
    assert_eq!(
        observe(&records).oversized_write(payload(), SMALL_PAYLOAD, None, workspace()).passed,
        Measure::Observed { value: true }
    );
    assert!(matches!(
        observe(&records).oversized_write(payload(), SMALL_PAYLOAD, Some(CALL_INPUT), workspace()).passed,
        Measure::Unavailable { reason } if reason.starts_with("not exercised")
    ));
}

#[test]
fn native_oversized_refusal_rejects_wrong_pairs_small_content_loss_and_absent_capture() {
    let mut records = recorded_write("large", &"x".repeat(OVER_PAYLOAD), Verdict::TooLarge);
    let checked = |rows: &[smith_events::Record]| {
        observe(rows).oversized_write(payload(), SMALL_PAYLOAD, None, workspace()).passed
    };
    assert_eq!(checked(&records), Measure::Observed { value: true });
    let original = records.clone();
    for value in ["run", "conversation", "call", "terminal", "small", "capture", "effect", "loss", "duplicate"] {
        records = original.clone();
        match value {
            "run" | "conversation" | "call" | "terminal" => {
                {
                    let end = crate::common::edit::tool_completed(&mut records[6]);
                    match value {
                        "run" => end.run = 9,
                        "conversation" => end.conversation = 9,
                        "call" => end.call = b"foreign".as_slice().into(),
                        "terminal" => end.verdict = Verdict::Written,
                        _ => unreachable!("bounded negative"),
                    }
                }
                assert_eq!(checked(&records), Measure::Observed { value: false });
            }
            "small" | "capture" | "effect" => {
                {
                    let start = crate::common::edit::tool_started(&mut records[5]);
                    match value {
                        "small" => {
                            start.input = Some(
                                serde_json::to_vec(
                                    &serde_json::json!({"path":"payload.txt","content":"x".repeat(AT_PAYLOAD)}),
                                )
                                .expect("JSON")
                                .into(),
                            );
                        }
                        "capture" => start.input = None,
                        "effect" => start.effect = Effect::Read,
                        _ => unreachable!("bounded negative"),
                    }
                }
                if value == "small" {
                    assert_eq!(checked(&records), Measure::Observed { value: false });
                } else {
                    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
                }
            }
            "loss" => {
                crate::common::edit::session_ended(records.last_mut().expect("ended")).loss.events = 1;
                assert!(matches!(checked(&records), Measure::Unavailable { .. }));
            }
            "duplicate" => {
                records.insert(7, records[6].clone());
                assert_eq!(checked(&records), Measure::Observed { value: false });
            }
            _ => unreachable!("named negatives"),
        }
    }
    // Protocol counters remain protocol-only: this native refusal creates no oversized response block.
    assert_eq!(
        observe(&original).check(&EventCheck::Blocks { oversized: Some(Count::Exactly(0)), cut: None }, "task").passed,
        Measure::Observed { value: true }
    );
}

#[test]
fn oversized_refusal_threshold_must_equal_the_native_tool_payload() {
    let document = LARGE_WRITE_DOCUMENT.replace("content_bytes_above = 2048", "content_bytes_above = 2047");
    let refused = check_large_write(&document).expect_err("native bound mismatch refuses");

    assert!(refused.to_string().contains("authored native tool_payload"));
}

#[test]
fn protocol_probe_judges_accepted_pieces_without_inventing_a_refusal() {
    let files = settled_payload("hello world");
    let mut records = common::events::balanced();
    piece(&mut records, "write", "first", &serde_json::json!({"path": PAYLOAD,"content":"hello"}), Verdict::Written);
    append_hello_world(&mut records, "append");
    let checked = |records: &[Record]| {
        observe(records)
            .file_pieces_for_protocol_probe_with(
                &files,
                payload(),
                SMALL_PAYLOAD,
                FILE_BYTES,
                PROTOCOL_INPUT_BOUND,
                workspace(),
            )
            .passed
    };
    assert_eq!(checked(&records), Measure::Observed { value: true });
    assert!(matches!(
        observe(&records).oversized_write(payload(), SMALL_PAYLOAD, Some(PROTOCOL_INPUT_BOUND), workspace()).passed,
        Measure::Unavailable { reason } if reason.starts_with("not exercised")
    ));

    assert_eq!(
        observe(&records).file_pieces_with(&files, payload(), SMALL_PAYLOAD, FILE_BYTES, workspace()).passed,
        Measure::Observed { value: false }
    );
    let original = records.clone();
    records.retain(|record| {
        !matches!(&record.event, Event::ToolStarted(start) if start.tool.as_ref() == b"edit")
            && !matches!(&record.event, Event::ToolCompleted(end) if end.tool.as_ref() == b"edit")
    });
    assert_eq!(checked(&records), Measure::Observed { value: false });
    records = original.clone();
    common::edit::tool_started(&mut records[7]).input = None;
    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
    records = original;
    files.write(&workspace().join(payload()), b"altered").expect("saved file mismatch");
    assert_eq!(checked(&records), Measure::Observed { value: false });
}

#[test]
fn saved_write_checks_content_size_pairing_and_capture_not_seed() {
    let content = "A line.\nAnother line.\n";
    let files = settled_payload(content);
    let mut records = common::events::balanced();
    piece(&mut records, "write", "w", &serde_json::json!({"path":"./payload.txt","content":content}), Verdict::Written);
    let checked = |records: &[Record], minimum| {
        observe(records).written_file_with(&files, payload(), minimum, workspace()).passed
    };
    assert_eq!(checked(&records, skein_lib::len_u64(content.len())), Measure::Observed { value: true });
    assert_eq!(
        checked(&records, RECORDED_INPUT_BYTES),
        Measure::Observed { value: false },
        "encoded input size cannot replace content size"
    );
    let original = records.clone();
    common::edit::tool_started(&mut records[5]).input = None;
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    records = original.clone();
    common::edit::tool_completed(&mut records[6]).conversation = 7;
    assert_eq!(checked(&records, 1), Measure::Observed { value: false });
    records = original.clone();
    common::edit::tool_started(&mut records[5]).effect = Effect::Read;
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    records = original.clone();
    records.insert(7, records[6].clone());
    assert_eq!(checked(&records, 1), Measure::Observed { value: false });
    records = original.clone();
    common::edit::session_ended(records.last_mut().expect("ended")).loss.events = 1;
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    files.write(&workspace().join(payload()), b"altered").expect("tampered saved file");
    assert_eq!(checked(&original, 1), Measure::Observed { value: false });
}

#[test]
fn oversized_pieces_replay_successful_edits_and_keep_failed_edit_measurement() {
    let files = settled_payload("hello wide world");
    let mut records = common::events::balanced();
    protocol_refusal(&mut records, b"large", OVER_PAYLOAD as u64);
    piece(&mut records, "write", "w", &serde_json::json!({"path": PAYLOAD,"content":"hello"}), Verdict::Written);
    piece(
        &mut records,
        "edit",
        "bad",
        &serde_json::json!({"path": PAYLOAD,"edits":[{"old":"absent","new":"ignored"}]}),
        Verdict::NoMatch,
    );
    append_hello_world(&mut records, "e1");
    piece(
        &mut records,
        "edit",
        "e2",
        &serde_json::json!({"path": PAYLOAD,"edits":[{"old":"world","new":"wide world"}]}),
        Verdict::Edited,
    );
    let checked = |records: &[Record]| {
        observe(records).file_pieces_with(&files, payload(), SMALL_PAYLOAD, FILE_BYTES, workspace())
    };
    let verdict = checked(&records);
    assert_eq!(verdict.passed, Measure::Observed { value: true });
    assert!(verdict.evidence.iter().any(|line| line == "piece call=bad; terminal=NoMatch"));
    let original = records.clone();
    common::edit::response_completed(&mut records[4]).completion = None;
    assert!(matches!(checked(&records).passed, Measure::Unavailable { .. }));
    records = original.clone();
    common::edit::response_completed(&mut records[4]).oversized = 0;
    assert_eq!(checked(&records).passed, Measure::Observed { value: false });
    records = original.clone();
    common::edit::tool_completed(&mut records[12]).call = b"foreign".as_slice().into();
    assert_eq!(checked(&records).passed, Measure::Observed { value: false });
    records = original.clone();
    common::edit::tool_started(&mut records[11]).input = None;
    assert!(matches!(checked(&records).passed, Measure::Unavailable { .. }));
    records = original.clone();
    common::edit::tool_started(&mut records[9]).input =
        Some(br#"{"path":"payload.txt","edits":[{"old":"missing","new":"wrong"}]}"#.as_slice().into());
    assert_eq!(checked(&records).passed, Measure::Observed { value: false });
    files.write(&workspace().join(payload()), b"different pieces").expect("tampered saved file");
    assert_eq!(checked(&original).passed, Measure::Observed { value: false });
}

#[test]
fn payload_too_large_write_then_observed_pieces_is_a_valid_oversized_refusal() {
    let files = settled_payload("hello world");
    let mut records = common::events::balanced();
    piece(
        &mut records,
        "write",
        "large",
        &serde_json::json!({"path": PAYLOAD,"content":"x".repeat(OVER_PAYLOAD)}),
        Verdict::TooLarge,
    );
    piece(&mut records, "write", "w", &serde_json::json!({"path": PAYLOAD,"content":"hello"}), Verdict::Written);
    append_hello_world(&mut records, "e");
    for record in &mut records {
        if let Event::ToolStarted(start) = &mut record.event
            && let Some(input) = &start.input
        {
            start.input_bytes = skein_lib::len_u64(input.len());
        }
    }
    let got = observe(&records).file_pieces_with(&files, payload(), SMALL_PAYLOAD, FILE_BYTES, workspace()).passed;

    assert_eq!(got, Measure::Observed { value: true });
}
