//! Whole-task erasure reads (jig/docs/design/protocol/store.md, 11.1;
//! jig/docs/design/protocol/store.md, 4.3, 7.3 and 11.1).
use super::fixtures;
use crate::{Event, Request, Store, Write, codec, keys};
use alloc::boxed::Box;
use jig_domain_core::{self as core, Record, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Token, Wall};

const TASK: u64 = 7;
const PAGE_ROWS: u32 = 2;
const MAX_ROWS: u32 = 64;

fn row_bytes(row: &reads::Row) -> u64 {
    if let Some(bytes) = row.owned_bytes() {
        return bytes;
    }
    let Record::People(people::Stored::EndedRead { .. }) = &row.value else { panic!("fixed ended-read row") };
    u64::try_from(::core::mem::size_of::<reads::Row>())
        .expect("row carrier")
        .checked_add(u64::try_from(::core::mem::size_of::<Record>()).expect("record carrier"))
        .expect("fixed row size")
}

fn save(store: &mut Store, record: Record) {
    let commit = store.stores.store.last().checked_add(1).expect("fixture commit");
    fixtures::committed(store, Box::new([fixtures::header(commit), fixtures::save(record)]));
}

fn load(store: &mut Store, range: reads::Range) -> (Box<[reads::Row]>, Option<reads::Position>) {
    let owner = Token::new(7);
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(store, &fixtures::env(), Request::Read { owner, range }, &mut above, &mut below);
    assert!(below.store.is_empty());
    assert!(below.secrets.is_empty());
    assert!(below.transcripts.is_empty(), "erasure never joins a turn's transcript");
    let event = above.pop();
    let Some(Event::Read { owner: found, result: reads::Loaded::Page { rows, next } }) = event else {
        panic!("erasure read answers a page: {event:?}");
    };
    assert_eq!(found, owner);
    assert!(above.is_empty());
    (rows, next)
}

fn all(store: &mut Store, mut range: reads::Range) -> Box<[Record]> {
    let mut found = List::with_capacity(MAX_ROWS);
    let mut previous = None;
    for _ in 0..MAX_ROWS {
        let (rows, next) = load(store, range.clone());
        assert!(rows.len() <= usize::try_from(range.rows).expect("row bound"));
        let mut bytes = 0_u64;
        for row in rows {
            bytes = bytes.checked_add(row_bytes(&row)).expect("page size");
            assert!(bytes <= range.bytes);
            assert!(row.owner.is_none());
            assert!(row.ancestors.is_empty());
            assert!(row.related.is_none());
            assert!(row.linked.is_empty());
            let key = keys::encode(&row.value.key(), fixtures::KEY_LIMIT).expect("fixture key");
            if let Some(previous) = previous {
                assert!(previous < key, "every row appears once, in encoded key order");
            }
            previous = Some(key);
            found.push(row.value).expect("fixture rows");
        }
        match next {
            Some(next) => {
                assert_ne!(range.after, Some(next.clone()), "a cut page advances");
                range.after = Some(next);
            }
            None => return found.into_boxed(),
        }
    }
    panic!("fixture pages finish");
}

fn ended_store() -> Store {
    let mut store = fixtures::opened_with_budget(1_048_576);
    // Save out of order and include other families next to the ended records.
    for number in [8, 6, 7] {
        save(&mut store, fixtures::ended_task(number));
    }
    save(&mut store, fixtures::live_task(5));
    save(&mut store, fixtures::history(TASK, 1));
    store.limits.joins = 0;
    store
}

#[test]
fn ended_tasks_page_every_ended_record_once() {
    let mut store = ended_store();
    let mut range = fixtures::read(fixtures::ALICE, reads::Scope::EndedTasks);
    range.rows = PAGE_ROWS;
    let found = all(&mut store, range);
    assert_eq!(found.as_ref(), [fixtures::ended_task(6), fixtures::ended_task(7), fixtures::ended_task(8)]);
}

#[test]
fn ended_tasks_continue_from_a_cut_page() {
    let mut store = ended_store();
    let range = fixtures::read(fixtures::ALICE, reads::Scope::EndedTasks);
    let (rows, next) = load(&mut store, range.clone());
    assert!(next.is_none());
    let first = rows.first().expect("ended rows");
    let decoded_bytes = row_bytes(first);
    let record = &first.value;
    let encoded_bytes = keys::encode(&record.key(), fixtures::KEY_LIMIT)
        .expect("key")
        .len()
        .checked_add(codec::encode(record, &store.codec, &store.names).expect("value").len())
        .expect("encoded row size");
    for (row_bound, byte_bound, encoded_bound) in [
        (1, range.bytes, store.limits.store.page.bytes),
        (range.rows, decoded_bytes, store.limits.store.page.bytes),
        (range.rows, range.bytes, u32::try_from(encoded_bytes).expect("encoded page bound")),
    ] {
        let mut cut = range.clone();
        cut.rows = row_bound;
        cut.bytes = byte_bound;
        store.limits.store.page.bytes = encoded_bound;
        let (page, next) = load(&mut store, cut.clone());
        assert_eq!(page.len(), 1);
        assert_eq!(next, Some(reads::Position::Owner(fixtures::ended_task(6).key())));
        let found = all(&mut store, cut);
        assert_eq!(found.as_ref(), [fixtures::ended_task(6), fixtures::ended_task(7), fixtures::ended_task(8)]);
    }
}

fn task_rows(task: u64) -> Box<[Record]> {
    let mut proposal = fixtures::core_rows::proposal();
    proposal.proposer = people::Proposer::Task(task);
    Box::new([
        fixtures::terminal(task, 1),
        fixtures::terminal(task, 3),
        Record::Core(core::CoreRecord::ProposalDecision(proposal)),
        fixtures::escalation_decision(task, 1),
        fixtures::history(task, 1),
        fixtures::history(task, 2),
        fixtures::hold_milestone(task, 1),
        fixtures::activity_line(task, 1),
        Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
            task,
            position: 2,
            at: Wall::from_nanos(23),
            line: tasks::Line::Turn { reply: 11, attempt: 3, turn: 2, read: Some(9) },
        })),
        Record::People(people::Stored::EndedRead { task, party: fixtures::ALICE, through: 3 }),
        Record::People(people::Stored::EndedRead { task, party: 12, through: 4 }),
    ])
}

fn task_store() -> Store {
    let mut store = fixtures::opened_with_budget(1_048_576);
    for task in [8, 7, 6] {
        for row in task_rows(task) {
            save(&mut store, row);
        }
    }
    // Equal party and task numbers must not select a party's proposal decision.
    save(&mut store, fixtures::proposal_decision(TASK, 13));
    save(&mut store, fixtures::ended_task(TASK));
    save(&mut store, fixtures::run_proof(TASK));
    save(&mut store, fixtures::call(TASK, 1));
    store.limits.joins = 1;
    store
}

#[test]
fn task_rows_name_only_their_task() {
    let mut store = task_store();
    let mut range = fixtures::read(fixtures::ALICE, reads::Scope::TaskRows { task: TASK });
    range.rows = PAGE_ROWS;
    assert_eq!(all(&mut store, range), task_rows(TASK));
}

#[test]
fn task_rows_include_their_accepted_party_proposal_decision_and_update() {
    let mut store = fixtures::opened_with_budget(1_048_576);
    let owner = 23;
    let proposal = 43;
    let object = tasks::UpdateObject::Proposal { proposal: tasks::Newest::of(proposal) };
    let expected = Box::new([
        fixtures::proposal_decision(fixtures::ALICE, proposal),
        fixtures::person_proposal(proposal, true),
        fixtures::update(fixtures::ALICE, 19, owner, object),
    ]);
    for record in expected.iter() {
        save(&mut store, record.clone());
    }
    save(&mut store, fixtures::update(fixtures::ALICE, 20, 24, object));
    store.limits.joins = 1;
    let mut range = fixtures::read(fixtures::ALICE, reads::Scope::TaskRows { task: owner });
    range.rows = 1;
    assert_eq!(all(&mut store, range).as_ref(), expected.as_ref());
}

#[test]
fn task_rows_cover_each_task_keyed_family() {
    let mut store = task_store();
    let mut range = fixtures::read(fixtures::ALICE, reads::Scope::TaskRows { task: TASK });
    range.rows = MAX_ROWS;
    let (rows, next) = load(&mut store, range.clone());
    assert!(next.is_none());
    assert_eq!(rows.len(), task_rows(TASK).len());
    // Each cut occurs between families as well as within a family. Stored turn
    // lines keep their position, time and reply without any transcript present.
    for row in rows {
        let mut cut = range.clone();
        cut.rows = 1;
        cut.bytes = row_bytes(&row);
        cut.after = range.after.clone();
        let (mut page, mut next) = load(&mut store, cut.clone());
        if page.is_empty() {
            cut.after = next;
            (page, next) = load(&mut store, cut);
        }
        let [loaded] = page.as_ref() else { panic!("one row fits the cut page") };
        assert_eq!(loaded.value, row.value);
        range.after = Some(reads::Position::Owner(row.value.key()));
        if next.is_none() {
            assert_eq!(row.value, task_rows(TASK).last().expect("last family").clone());
        }
    }
}

#[test]
fn task_rows_of_an_erased_task_are_empty() {
    let mut store = task_store();
    for record in task_rows(TASK) {
        let commit = store.stores.store.last().checked_add(1).expect("fixture commit");
        fixtures::committed(&mut store, Box::new([fixtures::header(commit), Write::Erase(record.key())]));
    }
    let commit = store.stores.store.last().checked_add(1).expect("fixture commit");
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(commit), Write::Erase(fixtures::ended_task(TASK).key())]),
    );
    let (rows, next) = load(&mut store, fixtures::read(fixtures::ALICE, reads::Scope::TaskRows { task: TASK }));
    assert!(rows.is_empty());
    assert!(next.is_none());
    let range = fixtures::read(fixtures::ALICE, reads::Scope::TaskRows { task: 8 });
    assert_eq!(all(&mut store, range), task_rows(8));
}

#[test]
fn task_rows_continue_across_families_after_byte_cuts() {
    let mut store = task_store();
    let mut range = fixtures::read(fixtures::ALICE, reads::Scope::TaskRows { task: TASK });
    range.rows = MAX_ROWS;
    let (rows, next) = load(&mut store, range.clone());
    assert!(next.is_none());
    let mut decoded_bound = 0;
    let mut encoded_bound = 0;
    for row in rows {
        decoded_bound = decoded_bound.max(row_bytes(&row));
        let record = &row.value;
        let bytes = keys::encode(&record.key(), fixtures::KEY_LIMIT)
            .expect("key")
            .len()
            .checked_add(codec::encode(record, &store.codec, &store.names).expect("value").len())
            .expect("encoded row size");
        encoded_bound = encoded_bound.max(u32::try_from(bytes).expect("encoded page bound"));
    }
    range.bytes = decoded_bound;
    let (first, next) = load(&mut store, range.clone());
    assert!(!first.is_empty());
    assert!(next.is_some());
    assert_eq!(all(&mut store, range.clone()), task_rows(TASK));
    range.bytes = 65_536;
    store.limits.store.page.bytes = encoded_bound;
    let (first, next) = load(&mut store, range.clone());
    assert!(!first.is_empty());
    assert!(next.is_some());
    assert_eq!(all(&mut store, range), task_rows(TASK));
}
