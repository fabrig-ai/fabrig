//! Replaying controls (smith/docs/design/testing.md, 2).

use std::fmt::Write;
use std::path::Path;

use skein_fake_machine::Item;
use smith_bench::{Count, EventCheck, Measure, Refusal, Task};
use smith_events::{Effect, Record, Source, Verdict};

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

fn recorded_write(call: &str, content: &str, verdict: Verdict) -> Vec<Record> {
    let (mut started, mut completed) = common::tool_pair("write", call);
    started.t_ms = 4;
    completed.t_ms = 4;
    let value = common::edit::tool_started(&mut started);
    value.source = Source::Workspace;
    value.effect = Effect::Write;
    // Retain a recorded size independent of the captured fixture content.
    value.input_bytes = 32768;
    let input = serde_json::json!({"path": PAYLOAD, "content": content});
    value.input = Some(serde_json::to_vec(&input).expect("captured arguments").into());
    common::edit::tool_completed(&mut completed).verdict = verdict;
    let mut records = common::events::balanced();
    let at = records.len() - 3;
    records.splice(at..at, [started, completed]);
    records
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
    {
        let value = crate::common::edit::response_completed(&mut records[4]);
        value.oversized = 1;
        let mut blocks = skein_lib::List::with_capacity(1);
        blocks
            .push(smith_events::Block::Oversized {
                call: b"refused".as_slice().into(),
                tool: b"write".as_slice().into(),
                bytes: 15995,
            })
            .expect("one protocol block");
        value.completion = Some(smith_events::Completion { blocks });
    }
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
