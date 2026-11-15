//! Ended tasks are converted and counted within each page's room (protocol/store/jobs.md, 5.4).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::match_like_matches_macro,
    reason = "ordinary Rust step tests"
)]
use super::fixtures;
use crate::{JobPage, JobRoom, JobRow, JobWrite, codec::tasks, jobs, keys, tasks_v13 as wire};
use jig_domain_core_vocabulary::Key;
use jig_domain_tasks::{Key as TaskKey, Stored};
use skein_lib::{List, Reader, Writer};

pub(super) fn row(number: u64, before_talk: bool) -> JobRow {
    let mut record = fixtures::task_record();
    record.number = number;
    record.phase =
        jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Report {
            words: b"done".as_slice().into(),
        }));
    record.talk = if before_talk {
        None
    } else {
        Some(jig_domain_tasks::Talk {
            party: 31,
            opened: skein_lib::Wall::from_nanos(11),
            last: skein_lib::Wall::from_nanos(17),
        })
    };
    let record = tasks::record::task_record(&record, &wire::CEILINGS, None).unwrap();
    let value =
        if before_talk { wire::Value::Ended(record.into_parts().record) } else { wire::Value::EndedTalk(record) };
    let row = wire::Row::new(&wire::CEILINGS, wire::RowParts { value, models: List::with_capacity(0) }).unwrap();
    let mut writer = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    JobRow { key: keys::encode(&Key::Tasks(TaskKey::Ended(number)), 1024).unwrap(), value: writer.finish() }
}
fn page(rows: Vec<JobRow>, writes: u32, bytes: u64) -> JobPage {
    JobPage { rows: rows.into_boxed_slice(), room: JobRoom { writes, bytes }, end: true, layout: None }
}
#[test]
fn ended_talk_pages_fit_both_edges_and_count_the_two_forms_in_order() {
    let original = row(1, true);
    let expected = tasks::decode(&original.value, &wire::CEILINGS, &[]).unwrap();
    let value = tasks::encode(&expected, &wire::CEILINGS, &[]).unwrap();
    let size = JobWrite::Put { key: original.key, value }.bytes();
    for (writes, bytes) in [(1, u64::MAX), (u32::MAX, size)] {
        let page = page(vec![row(1, true), row(2, false), row(3, true)], writes, bytes);
        let answer = jobs::page("ended-talk", &page).unwrap();
        assert_eq!(answer.through.as_deref(), Some(page.rows[1].key.as_ref()));
        assert_eq!(answer.counts.as_ref(), [1, 1]);
        let [JobWrite::Put { key, value }] = answer.writes.as_ref() else { panic!("one put") };
        assert_eq!(key, &page.rows[0].key);
        assert_eq!(tasks::decode(value, &wire::CEILINGS, &[]).unwrap(), expected);
        let wire = wire::Row::decode(&wire::CEILINGS, &mut Reader::new(value)).unwrap();
        assert!({ if let wire::Value::EndedTalk(_) = wire.value() { true } else { false } });
        let answer = jobs::page("ended-talk", &self::page(vec![row(3, true)], writes, bytes)).unwrap();
        assert_eq!(answer.counts.as_ref(), [1, 0]);
        assert!(answer.through.is_none());
    }
    let answer = jobs::page("ended-talk", &page(vec![row(1, false)], 0, 0)).unwrap();
    assert!(answer.writes.is_empty());
    assert_eq!(answer.counts.as_ref(), [0, 1]);
    assert!(answer.through.is_none());
}
#[test]
fn ended_talk_refuses_a_wrong_row_or_a_first_put_that_cannot_fit() {
    let record = Stored::Attempt(jig_domain_tasks::AttemptRow {
        task: 1,
        attempt: 1,
        claimed: None,
        started: None,
        ended: None,
        end: None,
        phases: None,
    });
    let invalid = JobRow { key: row(1, true).key, value: tasks::encode(&record, &wire::CEILINGS, &[]).unwrap() };
    for page in
        [page(vec![invalid], 1, u64::MAX), page(vec![row(1, true)], 0, u64::MAX), page(vec![row(1, true)], 1, 0)]
    {
        let refusal = jobs::page("ended-talk", &page).unwrap_err();
        assert_eq!(refusal.key.as_deref(), Some(page.rows[0].key.as_ref()));
    }
}
