//! Write predicates judge delivered bytes, call bounds and accepted repair pieces.

use std::fs;

use smith_bench::Measure;
use smith_events::{Effect, Event, Source, Verdict};

use crate::common;
use crate::common::observe;

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
    crate::common::edit::tool_started(&mut records[7]).input = None;
    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
    records = original;
    fs::write(directory.join("payload.txt"), "altered").expect("saved file mismatch");
    assert_eq!(checked(&records), Measure::Observed { value: false });
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
    assert_eq!(checked(&records, skein_lib::len_u64(content.len())), Measure::Observed { value: true });
    assert_eq!(
        checked(&records, 32768),
        Measure::Observed { value: false },
        "encoded input size cannot replace content size"
    );
    let original = records.clone();
    crate::common::edit::tool_started(&mut records[5]).input = None;
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    records = original.clone();
    crate::common::edit::tool_completed(&mut records[6]).conversation = 7;
    assert_eq!(checked(&records, 1), Measure::Observed { value: false });
    records = original.clone();
    crate::common::edit::tool_started(&mut records[5]).effect = Effect::Read;
    assert!(matches!(checked(&records, 1), Measure::Unavailable { .. }));
    records = original.clone();
    records.insert(7, records[6].clone());
    assert_eq!(checked(&records, 1), Measure::Observed { value: false });
    records = original.clone();
    crate::common::edit::session_ended(records.last_mut().expect("ended")).loss.events = 1;
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
    crate::common::edit::response_completed(&mut records[4]).completion = None;
    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
    records = original.clone();
    crate::common::edit::response_completed(&mut records[4]).oversized = 0;
    assert_eq!(checked(&records), Measure::Observed { value: false });
    records = original.clone();
    crate::common::edit::tool_completed(&mut records[12]).call = b"foreign".as_slice().into();
    assert_eq!(checked(&records), Measure::Observed { value: false });
    records = original.clone();
    crate::common::edit::tool_started(&mut records[11]).input = None;
    assert!(matches!(checked(&records), Measure::Unavailable { .. }));
    records = original.clone();
    crate::common::edit::tool_started(&mut records[9]).input =
        Some(br#"{"path":"payload.txt","edits":[{"old":"missing","new":"wrong"}]}"#.as_slice().into());
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
            start.input_bytes = skein_lib::len_u64(input.len());
        }
    }
    let got = observe(&records).file_pieces(std::path::Path::new("payload.txt"), 10, 64, &directory).passed;

    assert_eq!(got, Measure::Observed { value: true });
}
