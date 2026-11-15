//! Opening folds old milestone ranges once, before serving (protocol/store/jobs.md, 2–3).
use super::*;
use crate::tests::milestones::{ended_attempt, ended_rows, idle_hold, row};
use crate::{Owner, PatchSet, PatchSets};
use alloc::collections::VecDeque;
use jig_domain_tasks::{Active, AttemptRow, Held, Hold, HoldRow, Phase as TaskPhase, Stored, Was};
use skein_lib::Wall;

fn store(disk: &Disk, set: PatchSet) -> Store {
    let mut limits = fixtures::store_limits();
    limits.store.budget = 2_097_152;
    limits.store.ops = 128;
    limits.commits.writes = 7;
    let mut store = disk.store_with_limits(&limits);
    store.patch_sets(PatchSets { sets: Box::leak(Box::new([crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, set])) });
    store
}
fn range(store: &mut Store, tag: u8) -> Vec<Stored> {
    let rows = crate::pages::reader::walk(store, &[skein_kv::Range::prefix(&[1, 1, tag])], None, 128).unwrap();
    assert!(rows.next.is_none());
    rows.rows
        .iter()
        .map(|row| crate::codec::tasks::decode(&row.row.value, &crate::tasks_v13::CEILINGS, &[]).unwrap())
        .collect()
}
#[test]
fn milestones_open_folds_ended_and_live_tasks_in_pages_marks_done_and_never_reruns() {
    let mut disk = Disk::new(&[]);
    let mut before = store(&disk, PatchSet { jobs: &[], ..crate::JIG_STORE_PATCH_SET });
    assert_opened(&disk.open(&mut before), 0);
    let committed = disk.request(&mut before, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { number: 1, .. }] = committed.as_ref() else { panic!("commit: {committed:?}") };
    let mut rows = ended_rows();
    rows.push(row(7, 9, TaskPhase::Ended(jig_domain_tasks::Ending::Failed { reason: Box::new([]) }), 18));
    rows.push(row(8, 1, TaskPhase::Active(Active::Claimed { attempt: 1 }), 20));
    rows.push(row(8, 2, TaskPhase::Active(Active::Running { attempt: 1 }), 21));
    rows.push(row(
        8,
        3,
        TaskPhase::Held { was: Was::Active(Active::Running { attempt: 1 }), why: Hold::Stopped, source: None },
        22,
    ));
    registry::raw(
        &mut disk,
        &mut before,
        rows.into_iter().map(|r| skein_kv::Op::Put { key: r.key, value: r.value }).collect(),
    );
    assert_eq!(jobs::done(&mut before, Owner::Jig, "milestones"), None);
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, crate::JIG_STORE_PATCH_SET);
    let mut events = VecDeque::from(disk.open(&mut current).into_vec());
    let mut pages = 0;
    let mut ran = 0_u32;
    let mut last = None;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: Owner::Jig, job, page } => {
                let answer = crate::jobs::page(&job, &page).unwrap();
                assert!(answer.writes.len() <= usize::try_from(page.room.writes).unwrap());
                assert!(answer.writes.iter().map(crate::JobWrite::bytes).sum::<u64>() <= page.room.bytes);
                if job.as_ref() == "milestones" {
                    pages += 1;
                    assert!(page.layout.is_none());
                    if page.end {
                        assert!(answer.writes.is_empty());
                    }
                } else {
                    assert!(["charter-versions", "retention-rows", "retention-proposals"].contains(&job.as_ref()));
                    assert!(page.end);
                }
                events.extend(disk.request(&mut current, Request::Job { owner, answer: Ok(answer) }));
            }
            Event::Ran { of: Owner::Jig, job, commits, .. } => {
                if job.as_ref() == "milestones" {
                    assert_eq!(commits, pages);
                    ran += 1;
                }
            }
            Event::Opened { last: number, pending: true, .. } => last = Some(number),
            Event::Census { .. } => {}
            other => panic!("milestone open: {other:?}"),
        }
    }
    assert_eq!(ran, 1);
    assert!(pages > 2, "a fold that exceeds room needs several pages");
    assert_eq!(range(&mut current, 10), []);
    assert_eq!(
        range(&mut current, 22),
        [
            ended_attempt(),
            Stored::Attempt(AttemptRow {
                task: 8,
                attempt: 1,
                claimed: Some(Wall::from_nanos(20)),
                started: Some(Wall::from_nanos(21)),
                ended: None,
                end: None,
                phases: None,
            })
        ]
    );
    assert_eq!(
        range(&mut current, 23),
        [
            idle_hold(),
            Stored::Hold(HoldRow {
                task: 8,
                position: 3,
                held: Held { was: Was::Active(Active::Running { attempt: 1 }), why: Hold::Stopped, source: None },
                at: Wall::from_nanos(22),
            })
        ]
    );
    assert!(jobs::done(&mut current, Owner::Jig, "milestones").is_some());
    let switched = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = switched.as_ref() else { panic!("switch: {switched:?}") };
    let number = current.stores.store.last();
    let mut again = store(&disk, crate::JIG_STORE_PATCH_SET);
    let opened = disk.open(&mut again);
    assert_opened(&opened, last.unwrap());
    let Some(Event::Opened { pending: false, .. }) = opened.first() else { panic!("reopen: {opened:?}") };
    assert_eq!(again.stores.store.last(), number);
    assert_eq!(range(&mut again, 10), []);
}
#[test]
fn an_empty_stores_first_commit_marks_milestones_done_without_running_it() {
    let mut disk = Disk::new(&[]);
    let mut current = store(&disk, crate::JIG_STORE_PATCH_SET);
    assert_opened(&disk.open(&mut current), 0);
    let committed = disk.request(&mut current, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { number: 1, .. }] = committed.as_ref() else { panic!("commit: {committed:?}") };
    assert_eq!(jobs::done(&mut current, Owner::Jig, "milestones"), Some(1));
    let mut again = store(&disk, crate::JIG_STORE_PATCH_SET);
    assert_opened(&disk.open(&mut again), 1);
}
