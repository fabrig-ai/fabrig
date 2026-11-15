//! Upgrade map fixtures into logs, preserving serving answers (protocol/store/logs.md, 6).
use super::*;
use crate::{Owner, PatchSet};
use alloc::collections::VecDeque;
use jig_domain_core_vocabulary::reads;
use jig_domain_tasks as tasks;

fn read(disk: &mut Disk, store: &mut Store, task: u64, scope: reads::Scope) -> reads::Loaded {
    let events =
        disk.request(store, Request::Read { owner: Token::new(9), range: fixtures::read(fixtures::ALICE, scope) });
    let mut events = events.into_iter();
    let event = events.next();
    let Some(Event::Read { result, .. }) = event else { panic!("task {task}: {event:?}") };
    assert!(events.next().is_none());
    result
}
#[test]
fn history_routes_while_the_first_commits_done_row_waits_for_sync() {
    let mut disk = Disk::new(&[]);
    let mut store = jig_store(&disk, crate::JIG_STORE_PATCH_SET);
    assert_opened(&disk.open(&mut store), 0);
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(1, Box::new([fixtures::header(1)])),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    let row = fixtures::attempt_row(5, 1);
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(2, Box::new([fixtures::header(2), fixtures::save(row.clone())])),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    disk.drive(&mut below, |io, below| crate::up(&mut store, &fixtures::env(), io, &mut above, below));
    let events = fixtures::events(&mut above);
    let [Event::Committed { number: 1, .. }, Event::Committed { number: 2, .. }] = events.as_ref() else {
        panic!("{events:?}")
    };
    let key = crate::keys::encode(&row.key(), fixtures::KEY_LIMIT).unwrap();
    assert!(crate::view::raw_get(&mut store, key).unwrap().is_none());
    assert!(store.stores.store.log_end(crate::logs::history(5).unwrap()) > 0);
}

#[test]
fn task_logs_moves_old_map_rows_in_pages_preserves_reads_and_runs_only_once() {
    let set = crate::JIG_STORE_PATCH_SET;
    let old_jobs = set.jobs.iter().filter(|job| job.name != "task-logs").copied().collect::<Vec<_>>();
    let mut disk = Disk::new(&[]);
    let mut old = jig_store(&disk, PatchSet { jobs: Box::leak(old_jobs.into_boxed_slice()), ..set });
    assert_opened(&disk.open(&mut old), 0);
    let events = disk.request(&mut old, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { .. }] = events.as_ref() else { panic!("{events:?}") };
    let jig_domain_core_vocabulary::Record::Core(jig_domain_core_vocabulary::CoreRecord::ProposalDecision(
        mut decision,
    )) = fixtures::proposal_decision(5, 1)
    else {
        unreachable!()
    };
    decision.proposer = jig_domain_people::Proposer::Task(5);
    let rows = [
        fixtures::ended_of(5, tasks::Party::Person(fixtures::ALICE)),
        fixtures::terminal(5, 1),
        fixtures::history(5, 1),
        fixtures::activity_line(5, 1),
        fixtures::attempt_row(5, 1),
        fixtures::hold_row(5, 1),
        jig_domain_core_vocabulary::Record::Core(jig_domain_core_vocabulary::CoreRecord::ProposalDecision(decision)),
        fixtures::escalation_decision(5, 1),
        fixtures::ended_of(6, tasks::Party::Person(fixtures::ALICE)),
        fixtures::terminal(6, 1),
    ];
    // A binary before this job wrote map rows, not the component's new commit path.
    let ops = rows
        .iter()
        .map(|row| skein_kv::Op::Put {
            key: crate::keys::encode(&row.key(), fixtures::KEY_LIMIT).unwrap(),
            value: crate::codec::encode(row, &old.codec_limits, &[]).unwrap(),
        })
        .collect();
    registry::raw(&mut disk, &mut old, ops);
    let scopes = [
        reads::Scope::Task { task: 5 },
        reads::Scope::History { task: 5, subtree: false },
        reads::Scope::TaskOwnRows { task: 5 },
        reads::Scope::Task { task: 6 },
    ];
    let before = scopes.iter().map(|scope| read(&mut disk, &mut old, 5, scope.clone())).collect::<Vec<_>>();
    let mut disk = Disk::new(&disk.items());
    let mut current = jig_store(&disk, set);
    let mut events = VecDeque::from(disk.open(&mut current).into_vec());
    let mut pages = 0;
    let mut ran = 0_u32;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: Owner::Jig, job, page } => {
                assert_eq!(job.as_ref(), "task-logs");
                let answer = crate::jobs::page(&job, &page).unwrap();
                assert!(answer.writes.len() <= usize::try_from(page.room.writes / 4).unwrap());
                assert!(answer.writes.iter().map(crate::JobWrite::bytes).sum::<u64>() <= page.room.bytes);
                pages += 1;
                events.extend(disk.request(&mut current, Request::Job { owner, answer: Ok(answer) }));
            }
            Event::Ran { of: Owner::Jig, job, commits, .. } => {
                assert_eq!(job.as_ref(), "task-logs");
                assert_eq!(commits, pages);
                ran += 1;
            }
            Event::Opened { pending: true, .. } | Event::Census { .. } => {}
            other => panic!("{other:?}, pages {pages}, ran {ran}"),
        }
    }
    assert!(pages > 2);
    assert_eq!(ran, 1);
    assert!(jobs::done(&mut current, Owner::Jig, "task-logs").is_some());
    for row in rows {
        let key = crate::keys::encode(&row.key(), fixtures::KEY_LIMIT).unwrap();
        assert!(crate::view::raw_get(&mut current, key).unwrap().is_none());
    }
    for (scope, before) in scopes.iter().zip(&before) {
        let after = read(&mut disk, &mut current, 5, scope.clone());
        crate::tests::history_logs::equal(before, &after);
    }
    for task in [5, 6] {
        assert!(current.stores.store.log_end(crate::logs::history(task).unwrap()) > 0);
    }
    let last = current.stores.store.last();
    let switched = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = switched.as_ref() else { panic!("{switched:?}") };
    let mut disk = Disk::new(&disk.items());
    let mut again = jig_store(&disk, set);
    let events = disk.open(&mut again);
    assert_opened(&events, last);
    for (scope, before) in scopes.iter().zip(&before) {
        crate::tests::history_logs::equal(before, &read(&mut disk, &mut again, 5, scope.clone()));
    }
}
