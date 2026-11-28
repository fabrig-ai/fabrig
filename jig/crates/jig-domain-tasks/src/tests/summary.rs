//! Summaries preserve a bounded task line, never the full payload (protocol/store/retention.md, 3).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary Rust summary contract tests"
)]
use super::revisions::{Harness, task, to};
use crate::*;
use skein_lib::{Queue, Wall};

fn declared_failure(executor: Executor, reason: &[u8], bound: u32) -> Harness {
    let mut h = Harness::new();
    h.env.limits.result_bytes = bound;
    let mut new = task(12, &[]);
    new.executor = executor;
    new.contract = Contract::Report { words: bound.min(32) };
    h.make(vec![new]);
    let result = TaskResult::Failure { reason: reason.into() };
    let out = match executor {
        Executor::Agent { .. } | Executor::Job { .. } => {
            h.send(Event::Prepare { reply_to: to(), task: 12 });
            h.send(Event::Claim {
                sources: None,
                fresh: None,
                reply_to: to(),
                task: 12,
                attempt: 1,
                resumed: false,
                budget: 1,
                account: None,
                writes: Box::new([]),
                reasons: Reasons::FIRST,
                offered: Reasons::ALL,
                session_end: None,
            });
            h.send(Event::Activation {
                reads_project: false,
                reply_to: to(),
                task: 12,
                attempt: 1,
                end: End::Finished { result, cancel_delegates: false },
                saved: None,
                cause: Cause::Priced { cumulative: 0 },
            })
        }
        Executor::Person(_) => h.send(Event::AnswerPerson { reply_to: to(), task: 12, person: 10, result }),
        Executor::Procedure { .. } => {
            let (_, _, step) = h.domain.procedure_due(12).unwrap();
            let read = h.domain.procedure_offered(12);
            h.send(Event::Procedure {
                reads_project: false,
                reply_to: to(),
                task: 12,
                step,
                read,
                decision: ProcedureDecision::new(ProcedureEnd::Result(result)),
            })
        }
    };
    let Phase::Closing(_) = &h.domain.task(12).unwrap().phase else { panic!("failure refused: {out:?}") };
    h.send(Event::EffectsSettled { task: 12 });
    h.send(Event::Settled { task: 12 });
    h
}

fn assert_failed_summary(h: &Harness, words: &[u8]) {
    let Stored::Ended(record) = &h.stored[&Key::Ended(12)] else { panic!("ended task") };
    assert_eq!(record.phase, Phase::Ended(Ending::Failed { reason: words.into() }));
    let Stored::Summary(summary) = &h.stored[&Key::Summary(12)] else { panic!("task summary") };
    assert_eq!(summary.ending, SummaryEnding::Failed);
    assert_eq!(summary.reason.as_ref(), words);
    assert!(!summary.reason_cut);
}

#[test]
fn an_empty_run_failure_has_a_reason_in_its_summary() {
    let h = declared_failure(Executor::Agent { charter: 1 }, b"", 256);
    assert_failed_summary(&h, b"Its run declared failure without a reason.");
}

#[test]
fn an_empty_person_failure_has_a_reason_in_its_summary() {
    let h = declared_failure(Executor::Person(PersonAddress::Person(10)), b"", 256);
    assert_failed_summary(&h, b"Its person declared failure without a reason.");
}

#[test]
fn an_empty_procedure_failure_has_a_reason_in_its_summary() {
    let h = declared_failure(Executor::Procedure { connector: 1, code: 1 }, b"", 256);
    assert_failed_summary(&h, b"Its procedure declared failure without a reason.");
}

#[test]
fn declared_failure_reasons_are_kept_byte_for_byte() {
    for executor in [
        Executor::Agent { charter: 1 },
        Executor::Person(PersonAddress::Person(10)),
        Executor::Procedure { connector: 1, code: 1 },
    ] {
        let h = declared_failure(executor, b"no access", 256);
        assert_failed_summary(&h, b"no access");
    }
}

fn closer_failure(reason: &[u8], bound: u32) -> Harness {
    let mut h = Harness::new();
    h.env.limits.result_bytes = bound;
    h.make(vec![task(12, &[])]);
    let row = &mut domain::task_mut(&mut h.domain, 12).unwrap().record;
    row.closer = Some(13);
    row.tries.run = h.env.limits.retries.run.retries;
    let mut out = Queue::uncapped();
    closer::ended(&mut h.domain, &h.env, 12, 13, &Ending::Failed { reason: reason.into() }, &mut out);
    closing::progress(&mut h.domain, &h.env, &mut out);
    h.send(Event::EffectsSettled { task: 12 });
    h.send(Event::Settled { task: 12 });
    h
}

#[test]
fn an_empty_closer_failure_omits_the_colon_and_reason() {
    // The closer's own failed ending may carry no words.
    let h = closer_failure(b"", 256);
    assert_failed_summary(&h, b"Its closer failed, past its 2 run tries.");
}

#[test]
fn generated_failure_reasons_keep_only_the_bounded_character_prefix() {
    let h = declared_failure(Executor::Agent { charter: 1 }, b"", 7);
    assert_failed_summary(&h, b"Its run");
    let h = closer_failure("ééé".as_bytes(), 44);
    assert_failed_summary(&h, "Its closer failed, past its 2 run tries: é".as_bytes());
}

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
