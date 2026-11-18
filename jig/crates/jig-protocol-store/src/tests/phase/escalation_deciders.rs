//! Stored rejection kind repair (protocol/store/jobs.md, 1 and 3).
use super::*;
use crate::{JobPage, JobRoom, JobRow, JobWrite, OwnerSet, jobs};
use jig_domain_core_vocabulary::Record;
use jig_domain_tasks::{self as tasks, Escalation, EscalationDecider, Party, Stored};

const NAME: &str = "escalation-deciders";
const BEFORE: OwnerSet = OwnerSet { jobs: &[], ..crate::JIG_STORE_SET };
fn task(number: u64, requester: Party) -> tasks::TaskRecord {
    let mut task = fixtures::task_record();
    task.number = number;
    task.requester = requester;
    task.proposal = None;
    task.authority.models = tasks::Models::NONE;
    task.escalation = Escalation::Unheld { revision: 0 };
    task
}
fn rejected(number: u64, requester: Party, by: u64) -> tasks::TaskRecord {
    let mut row = task(number, requester);
    row.escalation =
        Escalation::Rejected { revision: 3, by: EscalationDecider::Person(by), reason: b"keep held".as_slice().into() };
    row
}
fn row(task: &tasks::TaskRecord) -> JobRow {
    let record = Record::Tasks(Stored::Live(Box::new(task.clone())));
    JobRow {
        key: crate::keys::encode(&record.key(), fixtures::KEY_LIMIT).unwrap(),
        value: crate::codec::encode(&record, &fixtures::codec_limits(), &[]).unwrap(),
    }
}
fn page(rows: Vec<JobRow>) -> JobPage {
    JobPage { rows: rows.into_boxed_slice(), end: false, layout: None, room: JobRoom { writes: 100, bytes: u64::MAX } }
}
fn begin(ancestors: &[tasks::TaskRecord]) -> (Disk, Store) {
    let mut disk = Disk::new(&[]);
    let mut store = jig_store(&disk, BEFORE);
    assert_opened(&disk.open(&mut store), 0);
    let events = disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1), Write::Wires])));
    let [Event::Committed { number: 1, .. }] = events.as_ref() else { panic!("{events:?}") };
    for (index, task) in ancestors.iter().enumerate() {
        let number = u64::try_from(index).unwrap().checked_add(2).unwrap();
        let events = disk.request(
            &mut store,
            fixtures::commit(
                number,
                Box::new([fixtures::header(number), Write::Save(Record::Tasks(Stored::Live(Box::new(task.clone()))))]),
            ),
        );
        let [Event::Committed { .. }] = events.as_ref() else { panic!("{events:?}") };
    }
    disk.crash(71);
    let mut store = jig_store(&disk, crate::JIG_STORE_SET);
    let events = disk.open(&mut store);
    let [Event::Job { of: crate::Owner::Jig, job, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(job.as_ref(), NAME);
    (disk, store)
}

#[test]
fn a_rejection_by_an_ancestor_changes_only_its_kind_and_only_once() {
    let (_disk, mut store) = begin(&[task(1, Party::Person(99)), task(2, Party::Task(1))]);
    let mut held = rejected(3, Party::Task(2), 1);
    let input = page(vec![row(&held)]);
    let answer = jobs::page(NAME, &input, &mut store).unwrap();
    assert_eq!(answer.counts.as_ref(), [1]);
    assert!(answer.through.is_none());
    let [JobWrite::Put { key, value }] = answer.writes.as_ref() else { panic!("{answer:?}") };
    held.escalation =
        Escalation::Rejected { revision: 3, by: EscalationDecider::Task(1), reason: b"keep held".as_slice().into() };
    let expected = row(&held);
    assert_eq!((key, value), (&expected.key, &expected.value), "no other field changes");
    // A task-kind rejection must not walk again, even if its requester cannot be read.
    store.limits.core.tasks.depth = 0;
    let twice = jobs::page(NAME, &page(vec![expected]), &mut store).unwrap();
    assert!(twice.writes.is_empty());
    assert_eq!(twice.counts.as_ref(), [0]);
}

#[test]
fn non_ancestors_and_unrejected_rows_are_kept() {
    let (_disk, mut store) =
        begin(&[task(1, Party::Person(99)), task(2, Party::Deployment { project: 1 }), task(4, Party::Task(8))]);
    let mut rows = vec![
        row(&rejected(3, Party::Task(1), 99)),  // A person's number at the end of the walk.
        row(&rejected(3, Party::Task(2), 1)),   // A live task, but not an ancestor; deployment stops the walk.
        row(&rejected(3, Party::Task(4), 1)),   // Missing intermediate ancestor.
        row(&rejected(3, Party::Person(1), 1)), // A person with a live task's number.
    ];
    for escalation in [
        Escalation::Unheld { revision: 3 },
        Escalation::Routing { revision: 3 },
        Escalation::Waiting {
            revision: 3,
            holder: tasks::EscalationHolder::Task(1),
            entry: 1,
            since: skein_lib::Wall::EPOCH,
        },
    ] {
        let mut held = task(3, Party::Task(1));
        held.escalation = escalation;
        rows.push(row(&held));
    }
    let answer = jobs::page(NAME, &page(rows), &mut store).unwrap();
    assert!(answer.writes.is_empty());
    assert_eq!(answer.counts.as_ref(), [0]);
}

#[test]
fn decider_writes_stop_at_both_room_edges_and_continue() {
    let (_disk, mut store) = begin(&[]);
    let first = rejected(3, Party::Task(1), 1);
    let second = rejected(4, Party::Task(1), 1);
    let cost = JobWrite::Put { key: row(&first).key, value: row(&first).value }.bytes();
    for room in [JobRoom { writes: 1, bytes: u64::MAX }, JobRoom { writes: 2, bytes: cost }] {
        let mut input = page(vec![row(&first), row(&second)]);
        input.room = room;
        let answer = jobs::page(NAME, &input, &mut store).unwrap();
        assert_eq!(answer.writes.len(), 1);
        assert_eq!(answer.counts.as_ref(), [1]);
        assert_eq!(answer.through, Some(row(&first).key));
        let next = jobs::page(NAME, &page(vec![row(&second)]), &mut store).unwrap();
        assert_eq!(next.writes.len(), 1);
        assert!(next.through.is_none());
    }
    let mut input = page(vec![row(&first)]);
    input.room.writes = 0;
    let refused = jobs::page(NAME, &input, &mut store).unwrap_err();
    assert_eq!(refused.key, Some(row(&first).key));
    assert!(refused.why.contains("edge"));
}

#[test]
fn unreadable_rows_and_overdeep_ancestry_name_their_key() {
    let (_disk, mut store) = begin(&[task(2, Party::Task(2))]);
    let key = row(&task(3, Party::Task(2))).key;
    let unreadable = page(vec![JobRow { key: key.clone(), value: Box::new([0]) }]);
    let refusal = jobs::page(NAME, &unreadable, &mut store).unwrap_err();
    assert_eq!(refusal.key, Some(key.clone()));
    assert!(refusal.why.contains("unreadable"));
    let refusal = jobs::page(NAME, &page(vec![row(&rejected(3, Party::Task(2), 1))]), &mut store).unwrap_err();
    assert_eq!(refusal.key, Some(key));
    assert!(refusal.why.contains("depth"));
}

#[test]
fn jig_refuses_an_unknown_job_name() {
    let (_disk, mut store) = begin(&[]);
    let refusal = jobs::page("not-a-job", &page(vec![]), &mut store).unwrap_err();
    assert!(refusal.key.is_none());
    assert_eq!(refusal.why.as_ref(), "no such jig job");
}
