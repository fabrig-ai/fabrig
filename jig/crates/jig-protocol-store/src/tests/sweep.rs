//! Typed erasure pages (jig/docs/design/protocol/store.md, 4.3, 7.1 and 11.1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "sweep page fixtures use ordinary Rust collections (programming-model.md, 10.2)"
)]
use super::fixtures;
use crate::{Cursor, Event, Load, Request, Row, Store};
use jig_domain_core::{self as core, reads::Sweep};
use jig_domain_notes as notes;
use skein_lib::Token;

const OWNER: Token = Token::new(8);
const TASK: u64 = 8;
const PROJECT: u32 = 3;
// Two rows force the six task families across several pages.
const PAGE_ROWS: u32 = 2;
const PAGE_BYTES: u64 = 16384;
// The fixture holds at most thirty rows; this catches a stuck cursor.
const MAX_PAGES: u32 = 32;

fn planted(rows: &[core::Record]) -> Store {
    let mut store = fixtures::opened();
    for (at, row) in rows.iter().enumerate() {
        let commit = u64::try_from(at).expect("fixture commits").checked_add(1).expect("fixture commits");
        fixtures::committed(&mut store, Box::new([fixtures::header(commit), fixtures::save(row.clone())]));
    }
    store
}

fn page(store: &mut Store, sweep: Sweep, after: Option<Cursor>) -> (Vec<core::Record>, Option<Cursor>) {
    let events = fixtures::run(
        store,
        Request::Load { owner: OWNER, range: Load::Sweep(sweep), after, rows: PAGE_ROWS, bytes: PAGE_BYTES },
    );
    let mut events = events.into_iter();
    let Some(Event::Loaded { owner, rows, next }) = events.next() else {
        panic!("sweep page");
    };
    assert_eq!(owner, OWNER);
    assert!(events.next().is_none());
    let rows = rows
        .into_vec()
        .into_iter()
        .map(|row| {
            let Row::Jig(record) = row else {
                panic!("typed core row");
            };
            record
        })
        .collect();
    (rows, next)
}

fn swept(store: &mut Store, sweep: Sweep) -> Vec<core::Record> {
    let mut after = None;
    let mut rows = Vec::new();
    for _ in 0..MAX_PAGES {
        let (mut found, next) = page(store, sweep, after.clone());
        for row in &found {
            assert!(sweep.admits(&row.key()), "{row:?}");
        }
        rows.append(&mut found);
        if next.is_none() {
            return rows;
        }
        assert_ne!(next, after, "exclusive cursor progresses");
        after = next;
    }
    panic!("sweep did not finish");
}

fn task_rows(task: u64) -> Vec<core::Record> {
    vec![
        fixtures::terminal(task, 1),
        fixtures::escalation_decision(task, 1),
        fixtures::history(task, 1),
        fixtures::hold_milestone(task, 1),
        fixtures::activity_line(task, 1),
        core::Record::People(jig_domain_people::Stored::EndedRead { task, party: 1, through: 1 }),
    ]
}

#[test]
fn a_task_sweep_loads_every_row_keyed_under_the_task_and_no_other() {
    let mut all = Vec::new();
    for task in [7, TASK, 9] {
        all.extend(task_rows(task));
        all.push(fixtures::ended_task(task));
    }
    let mut store = planted(&all);
    let expected = task_rows(TASK);
    for row in all {
        assert_eq!(Sweep::Task(TASK).admits(&row.key()), expected.contains(&row));
    }
    assert_eq!(swept(&mut store, Sweep::Task(TASK)), expected);
}

#[test]
fn an_ended_sweep_pages_ended_rows_above_its_number() {
    let all = [
        fixtures::ended_task(7),
        fixtures::ended_task(TASK),
        fixtures::ended_task(9),
        fixtures::ended_task(u64::MAX),
        fixtures::activity_line(TASK, 1),
    ];
    let mut store = planted(&all);
    assert_eq!(swept(&mut store, Sweep::Ended { after: 7 }), vec![all[1].clone(), all[2].clone(), all[3].clone()]);
    assert!(swept(&mut store, Sweep::Ended { after: u64::MAX }).is_empty());
}

fn scoped_lines(project: u32) -> Vec<core::Record> {
    [
        notes::Scope::Project { project },
        notes::Scope::Goal { project, goal: TASK },
        notes::Scope::Resources {
            project,
            connector: 1,
            pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
        },
    ]
    .iter()
    .map(|scope| fixtures::notes_line(scope, 1))
    .collect()
}

#[test]
fn a_notes_sweep_loads_the_projects_scoped_lines_alone() {
    let mut all = vec![fixtures::notes_entry(1), fixtures::notes_line(&notes::Scope::Deployment, 1)];
    for project in [0, PROJECT, 4] {
        all.extend(scoped_lines(project));
    }
    let mut store = planted(&all);
    assert_eq!(swept(&mut store, Sweep::Notes { project: PROJECT }), scoped_lines(PROJECT));
    assert_eq!(swept(&mut store, Sweep::Notes { project: 0 }), scoped_lines(0));
}

#[test]
fn a_core_transcript_drop_removes_the_file_and_its_places() {
    use super::fixtures::{Answer, LIVE};
    let turns = [fixtures::turn(LIVE, 1, 1, b"one")];
    let (mut store, _) = fixtures::opened_with_transcripts();
    let mut above = skein_lib::Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(
            2,
            Box::new([
                fixtures::header(2),
                fixtures::save(core::Record::Core(core::CoreRecord::Turn(turns[0].clone()))),
            ]),
        ),
        &mut above,
        &mut below,
    );
    fixtures::written(&mut store, below, &turns);
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(
            3,
            Box::new([fixtures::header(3), crate::Write::from_core(core::Write::DropTranscript(LIVE))]),
        ),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    assert_eq!(store.stores.store.last(), 2, "file removal precedes commit");
    let events = fixtures::script(
        &mut store,
        below,
        &[Answer::Closed, Answer::Opened { len: 0 }, Answer::Removed, Answer::Synced, Answer::Closed],
    );
    let [Event::Committed { number: 3, .. }] = events.as_ref() else {
        panic!("drop commit terminal");
    };
    let place =
        crate::keys::encode_own(&crate::keys::Own::Transcript { task: LIVE }, fixtures::KEY_LIMIT).expect("place key");
    assert!(fixtures::get_kv(&mut store, &place).is_none());
    fixtures::committed(&mut store, Box::new([fixtures::header(4)]));
    let attempt = crate::keys::encode_own(&crate::keys::Own::Attempt { task: LIVE, attempt: 1 }, fixtures::KEY_LIMIT)
        .expect("attempt key");
    assert!(fixtures::get_kv(&mut store, &attempt).is_none());
}

#[test]
fn the_recurring_sweep_admits_only_its_projects_closed_recurring_ledgers() {
    use jig_domain_tasks::{Funder, Stored};
    let mut all = Vec::new();
    let mut expected = Vec::new();
    for project in [2, PROJECT, 4] {
        for funder in [
            Funder::Task(TASK),
            Funder::Pool { project, person: 1, period: 5 },
            Funder::Recurring { project, task: TASK, period: 5 },
            Funder::Recurring { project, task: TASK, period: 6 },
            Funder::Period { project, period: 5 },
        ] {
            let mut row = fixtures::task_funding();
            row.funder = funder;
            row.closed = true;
            let closed = core::Record::Tasks(Stored::ClosedLedger(row));
            if let Funder::Recurring { project: PROJECT, .. } = funder {
                expected.push(closed.clone());
            }
            all.push(closed);
            row.closed = false;
            all.push(core::Record::Tasks(Stored::Ledger(row)));
        }
    }
    all.extend(task_rows(TASK));
    let sweep = Sweep::Recurring { project: PROJECT };
    for row in &all {
        assert_eq!(sweep.admits(&row.key()), expected.contains(row));
    }
    let mut store = planted(&all);
    assert_eq!(swept(&mut store, sweep), expected);
}

#[test]
fn the_proposals_sweep_admits_the_decided_proposals_family_only() {
    use jig_domain_tasks::{PersonProposalState, Stored};
    let mut all = task_rows(TASK);
    let mut expected = Vec::new();
    for number in [1, 2, 3, u64::MAX] {
        let mut row = fixtures::task_person_proposal();
        row.number = number;
        all.push(core::Record::Tasks(Stored::PersonProposal(Box::new(row.clone()))));
        row.state = PersonProposalState::Withdrawn { at: skein_lib::Wall::from_nanos(47) };
        let decided = core::Record::Tasks(Stored::DecidedProposal(Box::new(row)));
        expected.push(decided.clone());
        all.push(decided);
    }
    for row in &all {
        assert_eq!(Sweep::Proposals { after: None }.admits(&row.key()), expected.contains(row));
    }
    let mut store = planted(&all);
    assert_eq!(swept(&mut store, Sweep::Proposals { after: None }), expected);
    let after = Sweep::Proposals { after: Some(2) };
    assert!(!after.admits(&core::Key::Tasks(jig_domain_tasks::Key::DecidedProposal(2))));
    assert!(after.admits(&core::Key::Tasks(jig_domain_tasks::Key::DecidedProposal(3))));
    assert_eq!(swept(&mut store, after), expected[2..].to_vec());
    assert!(swept(&mut store, Sweep::Proposals { after: Some(u64::MAX) }).is_empty());
}
