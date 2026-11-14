//! Attempt and hold values and the absent open-attempt default (domain/tasks/lifecycle.md, 12).
use super::{fixtures, saved_tasks, tasks_limits};
use crate::{codec::tasks, tasks_v13 as wire};
use jig_domain_tasks as d;
use skein_lib::{Buffer, Reader, Wall, Writer};

#[test]
fn attempts_holds_and_open_attempts_round_trip() {
    for known in [false, true] {
        let at = known.then_some(Wall::from_nanos(17));
        let end = known.then_some(d::EndKind::Failed { class: d::Class::Run, invalid_result: None });
        let rows = [
            d::Stored::Attempt(d::AttemptRow {
                task: 11,
                attempt: 3,
                claimed: at,
                started: at,
                ended: at,
                end,
                phases: known.then_some(5),
            }),
            d::Stored::Hold(d::HoldRow {
                task: 11,
                position: 9,
                at: Wall::from_nanos(17),
                held: d::Held {
                    was: d::Was::Closing(d::Closing {
                        stage: d::Stage::Run { attempt: 3 },
                        ending: d::Ending::Failed { reason: b"failed".as_slice().into() },
                    }),
                    why: d::Hold::Failures(d::Class::Run),
                    source: Some(Box::new(d::HoldSource::Preparation { words: b"reason".as_slice().into() })),
                },
            }),
        ];
        for row in rows {
            assert_eq!(saved_tasks(&row), Ok(row));
        }
        let mut record = fixtures::task_record();
        record.open_attempt = known.then_some(d::OpenAttempt { attempt: 3, claimed: at, started: at, from: 4, end });
        let row = d::Stored::Live(Box::new(record));
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}

#[test]
fn task_runs_written_without_open_attempt_read_with_none() {
    let mut text = None;
    for family in crate::JIG_STORE_PATCH_SET.families {
        if family.name == "jig-store-tasks" {
            text = Some(family.current);
        }
    }
    let text = text.unwrap();
    let old_text = text
        .replace("    open_attempt: option OpenAttempt = none\n", "")
        .replace("    open_attempt: OpenAttempt\n", "");
    assert_ne!(old_text, text);
    let old = skein_codec::schema::parse(old_text.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(text.as_bytes()).unwrap();
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"TaskRun", None).unwrap(),
        &[],
        &new,
        new.root(b"TaskRun", None).unwrap(),
        &[],
    )
    .unwrap();
    let limits = tasks_limits();
    let record = tasks::record::task_record(&fixtures::task_record(), &limits, None)
        .unwrap()
        .into_parts()
        .record
        .into_parts()
        .run;
    assert!(record.open_attempt().is_none());
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    let old_bytes = bytes.get(..bytes.len() - 1).unwrap();
    let mut out = Buffer::with_capacity(u32::try_from(bytes.len()).unwrap());
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    plan.convert(old_bytes, &mut out, &mut work).unwrap();
    let read = wire::TaskRun::decode(&limits, &mut Reader::new(out.as_slice())).unwrap();
    assert_eq!(read, record);
    assert!(read.open_attempt().is_none());
}
