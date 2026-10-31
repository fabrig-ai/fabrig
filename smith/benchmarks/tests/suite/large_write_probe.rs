//! Write predicates judge delivered bytes, call bounds and accepted repair pieces.

use std::fmt::Write as _;
use std::fs;

use smith_bench::{Count, EventCheck, Measure};
use smith_events::{Effect, Event, Source, Verdict};

use crate::common;
use crate::common::observe;

#[test]
fn oversized_write_without_a_call_past_the_bound_is_not_exercised() {
    let mut records = common::events::balanced();
    let input = serde_json::json!({"path":"payload.txt","content":"Line 0001"});
    piece(&mut records, "write", "first", &input, Verdict::Written);
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let workspace = scratch.path().to_path_buf();
    let verdict = observe(&records).oversized_write(std::path::Path::new("payload.txt"), 2048, Some(13848), &workspace);
    assert!(matches!(&verdict.passed, Measure::Unavailable { reason } if reason.starts_with("not exercised")));
    assert!(verdict.evidence.iter().any(|line| line.starts_with("NotExercised")));
}

#[test]
fn payload_refusal_within_the_call_bound_is_not_exercised() {
    let mut records = common::events::balanced();
    let input = serde_json::json!({"path":"payload.txt","content":"x".repeat(20)});
    piece(&mut records, "write", "first", &input, Verdict::TooLarge);
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let workspace = scratch.path().to_path_buf();
    let verdict = observe(&records).oversized_write(std::path::Path::new("payload.txt"), 10, Some(13848), &workspace);
    assert!(matches!(&verdict.passed, Measure::Unavailable { reason } if reason.starts_with("not exercised")));
    assert!(verdict.evidence.iter().any(|line| line.starts_with("NotExercised")));
}

#[test]
fn protocol_probe_judges_accepted_pieces_without_inventing_a_refusal() {
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let directory = scratch.path().to_path_buf().join(format!("smith-pieces-unexercised-{}", std::process::id()));
    fs::create_dir(&directory).expect("fresh workspace");
    fs::write(directory.join("payload.txt"), "hello world").expect("settled pieces");
    let mut records = common::events::balanced();
    piece(
        &mut records,
        "write",
        "first",
        &serde_json::json!({"path":"payload.txt","content":"hello"}),
        Verdict::Written,
    );
    piece(
        &mut records,
        "edit",
        "append",
        &serde_json::json!({"path":"payload.txt","edits":[{"old":"hello","new":"hello world"}]}),
        Verdict::Edited,
    );
    let checked = |records: &[smith_events::Record]| {
        observe(records)
            .file_pieces_for_protocol_probe(std::path::Path::new("payload.txt"), 10, 64, 15, &directory)
            .passed
    };
    assert_eq!(checked(&records), Measure::Observed { value: true });
    assert!(matches!(
        observe(&records).oversized_write(std::path::Path::new("payload.txt"), 10, Some(15), &directory).passed,
        Measure::Unavailable { reason } if reason.starts_with("not exercised")
    ));

    assert_eq!(
        observe(&records).file_pieces(std::path::Path::new("payload.txt"), 10, 64, &directory).passed,
        Measure::Observed { value: false }
    );
    let original = records.clone();
    records.retain(|record| {
        !matches!(&record.event, Event::ToolStarted(start) if start.tool.as_ref() == b"edit")
            && !matches!(&record.event, Event::ToolCompleted(end) if end.tool.as_ref() == b"edit")
    });
    assert_eq!(checked(&records), Measure::Observed { value: false });
    records = original.clone();
    {
        let start = crate::common::edit::tool_started(&mut records[7]);
        start.input = None;
    }
    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
    records = original;
    fs::write(directory.join("payload.txt"), "altered").expect("saved file mismatch");
    assert_eq!(checked(&records), Measure::Observed { value: false });
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
fn protocol_only_oversized_write_records_identity_and_elided_path() {
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let directory = scratch.path().to_path_buf().join(format!("smith-protocol-oversized-{}", std::process::id()));
    fs::create_dir(&directory).expect("fresh workspace");
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
        observe(records).oversized_write(std::path::Path::new("payload.txt"), 2048, Some(13848), &directory)
    };
    let positive = checked(&records);
    assert_eq!(positive.passed, Measure::Observed { value: true });
    assert!(positive.evidence.iter().any(|item| item.contains("not carried (arguments elided by design)")));
    assert!(positive.evidence.iter().any(|item| item.contains("call=refused")));
    let original = records.clone();
    for case in ["at-bound", "wrong-tool", "absent-content", "counter", "loss"] {
        records = original.clone();
        if case == "loss" {
            {
                let end = crate::common::edit::session_ended(records.last_mut().expect("ended"));
                end.loss.events = 1;
            }
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
                            *bytes = 13848;
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
    records = common::events::balanced();
    piece(
        &mut records,
        "write",
        "native",
        &serde_json::json!({"path":"payload.txt","content":"x".repeat(20)}),
        Verdict::TooLarge,
    );
    assert_eq!(
        observe(&records).oversized_write(std::path::Path::new("payload.txt"), 10, None, &directory).passed,
        Measure::Observed { value: true }
    );
    assert!(matches!(
        observe(&records).oversized_write(std::path::Path::new("payload.txt"), 10, Some(13848), &directory).passed,
        Measure::Unavailable { reason } if reason.starts_with("not exercised")
    ));
}

#[test]
fn saved_write_checks_content_size_pairing_and_capture_not_seed() {
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let directory = scratch.path().to_path_buf().join(format!("smith-written-file-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh workspace");
    let content = "A line.\nAnother line.\n";
    std::fs::write(directory.join("payload.txt"), content).expect("saved write");
    let mut records = common::events::balanced();
    piece(&mut records, "write", "w", &serde_json::json!({"path":"./payload.txt","content":content}), Verdict::Written);
    let checked = |records: &[smith_events::Record], minimum| {
        observe(records).written_file(std::path::Path::new("payload.txt"), minimum, &directory).passed
    };
    assert_eq!(
        checked(&records, u64::try_from(content.len()).expect("small bytes")),
        Measure::Observed { value: true }
    );
    assert_eq!(
        checked(&records, 32768),
        Measure::Observed { value: false },
        "encoded input size cannot replace content size"
    );
    let original = records.clone();
    {
        let value = crate::common::edit::tool_started(&mut records[5]);
        value.input = None;
    }
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    records = original.clone();
    {
        let value = crate::common::edit::tool_completed(&mut records[6]);
        value.conversation = 7;
    }
    assert_eq!(checked(&records, 1), Measure::Observed { value: false });
    records = original.clone();
    {
        let value = crate::common::edit::tool_started(&mut records[5]);
        value.effect = Effect::Read;
    }
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    records = original.clone();
    records.insert(7, records[6].clone());
    assert_eq!(checked(&records, 1), Measure::Observed { value: false });
    records = original.clone();
    {
        let value = crate::common::edit::session_ended(records.last_mut().expect("ended"));
        value.loss.events = 1;
    }
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    std::fs::write(directory.join("payload.txt"), "altered").expect("tampered saved file");
    assert_eq!(checked(&original, 1), Measure::Observed { value: false });
}

#[test]
fn oversized_pieces_replay_successful_edits_and_keep_failed_edit_measurement() {
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let directory = scratch.path().to_path_buf().join(format!("smith-file-pieces-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh workspace");
    std::fs::write(directory.join("payload.txt"), "hello wide world").expect("saved pieces");
    let mut records = common::events::balanced();
    {
        let value = crate::common::edit::response_completed(&mut records[4]);
        value.oversized = 1;
        let mut blocks = skein_lib::List::with_capacity(1);
        blocks
            .push(smith_events::Block::Oversized {
                call: b"large".as_slice().into(),
                tool: b"write".as_slice().into(),
                bytes: 20,
            })
            .expect("one refusal");
        value.completion = Some(smith_events::Completion { blocks });
    }
    piece(&mut records, "write", "w", &serde_json::json!({"path":"payload.txt","content":"hello"}), Verdict::Written);
    piece(
        &mut records,
        "edit",
        "bad",
        &serde_json::json!({"path":"payload.txt","edits":[{"old":"absent","new":"ignored"}]}),
        Verdict::NoMatch,
    );
    piece(
        &mut records,
        "edit",
        "e1",
        &serde_json::json!({"path":"payload.txt","edits":[{"old":"hello","new":"hello world"}]}),
        Verdict::Edited,
    );
    piece(
        &mut records,
        "edit",
        "e2",
        &serde_json::json!({"path":"payload.txt","edits":[{"old":"world","new":"wide world"}]}),
        Verdict::Edited,
    );
    let checked = |records: &[smith_events::Record]| {
        observe(records).file_pieces(std::path::Path::new("payload.txt"), 10, 64, &directory).passed
    };
    assert_eq!(checked(&records), Measure::Observed { value: true });
    let original = records.clone();
    {
        let value = crate::common::edit::response_completed(&mut records[4]);
        value.completion = None;
    }
    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
    records = original.clone();
    {
        let value = crate::common::edit::response_completed(&mut records[4]);
        value.oversized = 0;
    }
    assert_eq!(checked(&records), Measure::Observed { value: false });
    records = original.clone();
    {
        let value = crate::common::edit::tool_completed(&mut records[12]);
        value.call = b"foreign".as_slice().into();
    }
    assert_eq!(checked(&records), Measure::Observed { value: false });
    records = original.clone();
    {
        let value = crate::common::edit::tool_started(&mut records[11]);
        value.input = None;
    }
    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
    records = original.clone();
    {
        let value = crate::common::edit::tool_started(&mut records[9]);
        value.input = Some(br#"{"path":"payload.txt","edits":[{"old":"missing","new":"wrong"}]}"#.as_slice().into());
    }
    assert_eq!(checked(&records), Measure::Observed { value: false });
    std::fs::write(directory.join("payload.txt"), "different pieces").expect("tampered saved file");
    assert_eq!(checked(&original), Measure::Observed { value: false });
}

fn piece(records: &mut Vec<smith_events::Record>, tool: &str, call: &str, input: &serde_json::Value, verdict: Verdict) {
    let (mut started, mut completed) = common::tool_pair(tool, call);
    started.t_ms = 4;
    completed.t_ms = 4;
    let value = crate::common::edit::tool_started(&mut started);
    value.source = Source::Workspace;
    value.effect = Effect::Write;
    value.input_bytes = 32768;
    value.input = Some(serde_json::to_vec(input).expect("captured arguments").into());
    crate::common::edit::tool_completed(&mut completed).verdict = verdict;
    let at = records.len() - 3;
    records.insert(at, started);
    records.insert(at + 1, completed);
}

#[test]
fn payload_too_large_write_then_observed_pieces_is_a_valid_oversized_refusal() {
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let directory = scratch.path().to_path_buf().join(format!("smith-native-refusal-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh workspace");
    std::fs::write(directory.join("payload.txt"), "hello world").expect("saved pieces");
    let mut records = common::events::balanced();
    piece(
        &mut records,
        "write",
        "large",
        &serde_json::json!({"path":"payload.txt","content":"x".repeat(20)}),
        Verdict::TooLarge,
    );
    piece(&mut records, "write", "w", &serde_json::json!({"path":"payload.txt","content":"hello"}), Verdict::Written);
    piece(
        &mut records,
        "edit",
        "e",
        &serde_json::json!({"path":"payload.txt","edits":[{"old":"hello","new":"hello world"}]}),
        Verdict::Edited,
    );
    for record in &mut records {
        if let Event::ToolStarted(start) = &mut record.event
            && let Some(input) = &start.input
        {
            start.input_bytes = u64::try_from(input.len()).expect("fixture bytes");
        }
    }
    let got = observe(&records).file_pieces(std::path::Path::new("payload.txt"), 10, 64, &directory).passed;

    assert_eq!(got, Measure::Observed { value: true });
}

#[test]
fn native_oversized_refusal_rejects_wrong_pairs_small_content_loss_and_absent_capture() {
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let directory = scratch.path().to_path_buf().join(format!("smith-refusal-negatives-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh workspace");
    let mut records = common::events::balanced();
    piece(
        &mut records,
        "write",
        "large",
        &serde_json::json!({"path":"payload.txt","content":"x".repeat(20)}),
        Verdict::TooLarge,
    );
    let checked = |rows: &[smith_events::Record]| {
        observe(rows).oversized_write(std::path::Path::new("payload.txt"), 10, None, &directory).passed
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
                                serde_json::to_vec(&serde_json::json!({"path":"payload.txt","content":"x".repeat(10)}))
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
                {
                    let end = crate::common::edit::session_ended(records.last_mut().expect("ended"));
                    end.loss.events = 1;
                }
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
    let root = crate::common::repository();
    let source = root.join("tasks/probes/large-write");
    let scratch = skein_scratch::Scratch::new("bench-large_write_probe");
    let directory = scratch.path().to_path_buf().join(format!("smith-refusal-declaration-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh task");
    std::fs::create_dir(directory.join("seed")).expect("seed directory");
    std::fs::copy(source.join("seed/source.txt"), directory.join("seed/source.txt")).expect("same seed bytes");
    let document = std::fs::read_to_string(source.join("task.toml"))
        .expect("current task")
        .replace("content_bytes_above = 2048", "content_bytes_above = 2047");
    std::fs::write(directory.join("task.toml"), document).expect("wrong threshold fixture");
    let refused = smith_bench::check_task(&directory.join("task.toml"), &root.join("../docs/design"))
        .expect_err("native bound mismatch refuses");

    assert!(refused.to_string().contains("authored native tool_payload"));
}
