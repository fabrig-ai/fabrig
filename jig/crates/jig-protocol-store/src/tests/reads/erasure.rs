//! Core erasure reads (playground-reset-r6/tasks/pd-store-reads.md, Acceptance).
use super::{fixtures, load, page, values};
use crate::{Store, Write};
use jig_domain_core::{self as core, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;

const PROJECT: u32 = 2;
const TASK: u64 = 7;

fn ended(number: u64, project: u32) -> core::Record {
    let core::Record::Tasks(tasks::Stored::Ended(mut row)) = fixtures::ended_task(number) else {
        panic!("ended fixture");
    };
    row.project = project;
    core::Record::Tasks(tasks::Stored::Ended(row))
}

fn decided(number: u64, project: u32) -> core::Record {
    let core::Record::Tasks(tasks::Stored::DecidedProposal(mut row)) = fixtures::person_proposal(number, true) else {
        panic!("decided fixture");
    };
    row.project = project;
    core::Record::Tasks(tasks::Stored::DecidedProposal(row))
}

fn project_records() -> [core::Record; 7] {
    [
        ended(1, 1),
        ended(2, PROJECT),
        ended(3, 1),
        ended(4, PROJECT),
        decided(1, PROJECT),
        decided(2, 1),
        decided(3, PROJECT),
    ]
}

fn task_records(task: u64) -> [core::Record; 9] {
    let core::Record::Core(core::CoreRecord::ProposalDecision(mut decision)) = fixtures::proposal_decision(task, 1)
    else {
        panic!("proposal decision fixture");
    };
    decision.proposer = people::Proposer::Task(task);
    [
        fixtures::terminal(task, 1),
        core::Record::Core(core::CoreRecord::ProposalDecision(decision)),
        fixtures::escalation_decision(task, 1),
        fixtures::history(task, 1),
        fixtures::hold_milestone(task, 1),
        fixtures::activity_line(task, 1),
        fixtures::ended_delegate(task, 20),
        fixtures::delegate(task, 21),
        core::Record::People(people::Stored::EndedRead { task, party: fixtures::ALICE, through: 3 }),
    ]
}

fn commit_records(store: &mut Store, records: &[core::Record]) {
    for record in records {
        let number = store.stores.store.last().checked_add(1).expect("fixture commit number");
        fixtures::committed(store, Box::new([fixtures::header(number), fixtures::save(record.clone())]));
    }
}

fn project_store() -> Store {
    let mut store = fixtures::opened();
    commit_records(&mut store, &project_records());
    commit_records(&mut store, &[fixtures::live_task(5), fixtures::person_proposal(4, false)]);
    store.limits.joins = 0;
    store
}

fn task_store() -> Store {
    let mut store = fixtures::opened();
    commit_records(&mut store, &task_records(TASK));
    commit_records(&mut store, &task_records(8));
    commit_records(&mut store, &[fixtures::proposal_decision(TASK, 2), ended(TASK, PROJECT)]);
    store.limits.joins = 0;
    store
}

fn range(scope: reads::Scope, rows: u32) -> reads::Range {
    let mut range = fixtures::read(0, scope);
    range.rows = rows;
    range
}

fn project_range(rows: u32) -> reads::Range {
    range(reads::Scope::ProjectRows { project: PROJECT }, rows)
}

fn task_range(rows: u32) -> reads::Range {
    range(reads::Scope::TaskRows { task: TASK }, rows)
}

fn position(record: &core::Record) -> reads::Position {
    reads::Position::Owner(record.key())
}

fn assert_raw(rows: &[reads::Row]) {
    for row in rows {
        assert_eq!(row.owner, None);
        assert!(row.ancestors.is_empty());
        assert_eq!(row.related, None);
        assert_eq!(row.unjoined, None);
        assert_eq!(row.ended_read, None);
        assert!(row.linked.is_empty());
        assert_eq!(row.inbox, None);
    }
}

#[test]
fn project_rows_find_a_projects_ended_tasks_and_decided_proposals() {
    let mut store = project_store();
    let records = project_records();
    let mut range = project_range(1);
    for (index, record) in records.iter().enumerate() {
        let (rows, next) = page(load(&mut store, range.clone()));
        if [1, 3, 4, 6].contains(&index) {
            assert_eq!(values(&rows).as_ref(), ::core::slice::from_ref(record));
        } else {
            assert!(rows.is_empty());
        }
        assert_raw(&rows);
        let expected = if index == 6 { None } else { Some(position(record)) };
        assert_eq!(next, expected);
        range.after = next;
    }
}

#[test]
fn a_project_rows_page_scans_at_most_its_rows_and_may_return_none() {
    let mut store = project_store();
    let (rows, next) = page(load(&mut store, project_range(1)));
    assert!(rows.is_empty());
    assert_eq!(next, Some(position(&ended(1, 1))));
    let (rows, next) = page(load(&mut store, project_range(2)));
    assert_eq!(values(&rows).as_ref(), [ended(2, PROJECT)]);
    assert_eq!(next, Some(position(&ended(2, PROJECT))));
}

#[test]
fn task_rows_find_every_row_under_the_tasks_prefixes() {
    let mut store = task_store();
    let expected = task_records(TASK);
    let (rows, next) = page(load(&mut store, task_range(20)));
    assert_eq!(values(&rows).as_ref(), expected);
    assert_raw(&rows);
    assert_eq!(next, None);
    let mut range = task_range(1);
    for (index, record) in expected.iter().enumerate() {
        let (rows, next) = page(load(&mut store, range.clone()));
        assert_eq!(values(&rows).as_ref(), ::core::slice::from_ref(record));
        assert_raw(&rows);
        assert_eq!(next, if index == 8 { None } else { Some(position(record)) });
        range.after = next;
    }
}

fn erase(store: &mut Store, rows: &[reads::Row]) {
    for row in rows {
        let number = store.stores.store.last().checked_add(1).expect("fixture commit number");
        fixtures::committed(store, Box::new([fixtures::header(number), Write::Erase(row.value.key())]));
    }
}

#[test]
fn task_rows_after_an_erase_return_the_rest() {
    let mut store = task_store();
    let mut range = task_range(4);
    let (rows, next) = page(load(&mut store, range.clone()));
    assert_eq!(values(&rows).as_ref(), &task_records(TASK)[..4]);
    assert_eq!(next, Some(position(&fixtures::history(TASK, 1))));
    erase(&mut store, &rows);
    range.after = next;
    range.rows = 20;
    let (rows, next) = page(load(&mut store, range));
    assert_eq!(values(&rows).as_ref(), &task_records(TASK)[4..]);
    assert_eq!(next, None);
    let (fresh, next) = page(load(&mut store, task_range(20)));
    assert_eq!(values(&fresh), values(&rows));
    assert_eq!(next, None);
}

#[test]
fn project_rows_after_an_erase_return_the_rest() {
    let mut store = project_store();
    let mut range = project_range(5);
    let (rows, next) = page(load(&mut store, range.clone()));
    assert_eq!(values(&rows).as_ref(), [ended(2, PROJECT), ended(4, PROJECT), decided(1, PROJECT)]);
    assert_eq!(next, Some(position(&decided(1, PROJECT))));
    erase(&mut store, &rows);
    range.after = next;
    let (rows, next) = page(load(&mut store, range));
    assert_eq!(values(&rows).as_ref(), [decided(3, PROJECT)]);
    assert_eq!(next, None);
    let (fresh, next) = page(load(&mut store, project_range(20)));
    assert_eq!(values(&fresh), values(&rows));
    assert_eq!(next, None);
}

fn stored_bytes(store: &mut Store, record: &core::Record) -> u64 {
    let key = crate::keys::encode(&record.key(), u32::MAX).expect("fixture key");
    let value = fixtures::get_kv(store, &key).expect("fixture stored");
    u64::try_from(key.len().checked_add(value.len()).expect("fixture length fits")).expect("fixture length")
}

#[test]
fn project_rows_charge_skipped_rows_to_the_byte_bound() {
    let mut store = project_store();
    let first = ended(1, 1);
    let mut range = project_range(20);
    range.bytes = stored_bytes(&mut store, &first);
    let (rows, next) = page(load(&mut store, range.clone()));
    assert!(rows.is_empty());
    assert_eq!(next, Some(position(&first)));
    range.after = next;
    range.bytes = stored_bytes(&mut store, &ended(2, PROJECT));
    let (rows, next) = page(load(&mut store, range));
    assert_eq!(values(&rows).as_ref(), [ended(2, PROJECT)]);
    assert_eq!(next, Some(position(&ended(2, PROJECT))));
}

#[test]
fn task_rows_stop_before_the_row_past_the_byte_bound() {
    let mut store = task_store();
    let first = fixtures::terminal(TASK, 1);
    let mut range = task_range(20);
    let bytes = stored_bytes(&mut store, &first);
    for bound in [bytes, bytes + stored_bytes(&mut store, &task_records(TASK)[1]) - 1] {
        range.bytes = bound;
        let (rows, next) = page(load(&mut store, range.clone()));
        assert_eq!(values(&rows).as_ref(), ::core::slice::from_ref(&first));
        assert_eq!(next, Some(position(&first)));
    }
    range.bytes = bytes + stored_bytes(&mut store, &task_records(TASK)[1]);
    let (rows, next) = page(load(&mut store, range));
    assert_eq!(values(&rows).as_ref(), &task_records(TASK)[..2]);
    assert_eq!(next, Some(position(&task_records(TASK)[1])));
}
