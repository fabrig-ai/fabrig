//! Summaries preserve a bounded task line, never the full payload (protocol/store/retention.md, 3).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary Rust summary contract tests"
)]
use super::revisions::{Harness, task, to};
use crate::*;
use skein_lib::Wall;

#[test]
fn summary_titles_keep_only_the_first_line_and_cut_at_a_character_boundary() {
    let mut h = Harness::new();
    h.make(vec![task(12, &[])]);
    let mut record = h.domain.task(12).unwrap().clone();
    record.phase = Phase::Ended(Ending::Done(TaskResult::Report { words: b"report".as_slice().into() }));
    for (words, title, cut) in [
        ("Fix the bug\nmore".to_owned(), "Fix the bug".to_owned(), false),
        (String::new(), String::new(), false),
        ("a".repeat(256), "a".repeat(256), false),
        (format!("{}éx", "a".repeat(255)), "a".repeat(255), true),
        (format!("{}\nmore", "é".repeat(128)), "é".repeat(128), false),
    ] {
        record.spec.words = words.into_bytes().into();
        let summary = TaskSummary::of(&record);
        assert_eq!(summary.title.as_ref(), title.as_bytes());
        assert_eq!(summary.title_cut, cut);
    }
}

#[test]
fn ending_saves_summary_and_next_stage_beside_the_ended_record() {
    for turned in [false, true] {
        let mut h = Harness::new();
        h.make(vec![task(12, &[])]);
        domain::task_mut(&mut h.domain, 12).unwrap().record.ever_turned = turned;
        h.env.wall = Wall::from_nanos(1000);
        h.send(Event::Cancel {
            reply_to: to(),
            by: Party::Person(9),
            operation: Cancel { task: 12, reason: b"cancelled".as_slice().into(), grace: None, message: None },
        });
        h.send(Event::EffectsSettled { task: 12 });
        h.send(Event::Settled { task: 12 });
        let Stored::Ended(record) = &h.stored[&Key::Ended(12)] else { panic!("ended record") };
        assert_eq!(h.stored[&Key::Summary(12)], Stored::Summary(Box::new(TaskSummary::of(record))));
        let row = RetentionRow {
            stage: if turned { RetentionStage::Turns } else { RetentionStage::Record },
            since: h.env.wall,
            number: 12,
        };
        assert_eq!(h.stored[&row.key()], Stored::Retention(row));
    }
}

#[test]
fn summary_results_keep_the_chip_words_and_omit_fields_state_and_job_bytes() {
    let mut h = Harness::new();
    h.make(vec![task(12, &[])]);
    let mut record = h.domain.task(12).unwrap().clone();
    for (result, kind, words) in [
        (TaskResult::Job { bytes: Box::new([255; 1024]) }, SummaryResultKind::Job, b"".as_slice()),
        (
            TaskResult::Approved { state: Box::new([255; 1024]), words: Some(b"answer".as_slice().into()) },
            SummaryResultKind::Approved,
            b"answer".as_slice(),
        ),
        (
            TaskResult::Verdict {
                code: 7,
                fields: Box::new([ResultField { name: b"field".as_slice().into(), value: Box::new([255; 1024]) }]),
                words: b"verdict".as_slice().into(),
            },
            SummaryResultKind::Verdict { code: 7 },
            b"verdict".as_slice(),
        ),
        (
            TaskResult::Change {
                connector: 1,
                kind: 2,
                resource: 3,
                title: ChangeTitle::new(b"title".as_slice().into()),
                words: b"not the title".as_slice().into(),
            },
            SummaryResultKind::Change { connector: 1, kind: 2, resource: 3 },
            b"title".as_slice(),
        ),
        (
            TaskResult::NoChange { words: b"no change".as_slice().into() },
            SummaryResultKind::NoChange,
            b"no change".as_slice(),
        ),
        (TaskResult::Report { words: b"report".as_slice().into() }, SummaryResultKind::Report, b"report".as_slice()),
        (
            TaskResult::Failure { reason: b"failure".as_slice().into() },
            SummaryResultKind::Failed,
            b"failure".as_slice(),
        ),
    ] {
        record.phase = Phase::Ended(Ending::Cancelled {
            by: Party::Person(9),
            at: Wall::from_nanos(1000),
            reason: "é".repeat(129).into_bytes().into(),
            result: Some(result),
        });
        let summary = TaskSummary::of(&record);
        assert_eq!(summary.ending, SummaryEnding::Cancelled { by: Party::Person(9), at: Wall::from_nanos(1000) });
        assert_eq!(summary.reason, "é".repeat(128).into_bytes().into_boxed_slice());
        assert!(summary.reason_cut);
        assert_eq!(summary.result, Some(SummaryResult { kind, words: words.into(), words_cut: false }));
    }
}
