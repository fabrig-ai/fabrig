//! The real jig set splits a store's whole projections once (protocol/store/parts.md, 4–5).
use super::*;
use crate::tests::projection_parts::{goal, key, legacy, records, task};
use crate::{Owner, PatchSet, PatchSets};
use alloc::collections::VecDeque;
use jig_domain_core_vocabulary::{CoreRecord, ProjectionRecord};

fn store(disk: &Disk, set: PatchSet) -> Store {
    let mut limits = fixtures::store_limits();
    limits.store.budget = 2_097_152;
    limits.store.ops = 128;
    let mut store = disk.store_with_limits(&limits);
    store.patch_sets(PatchSets { sets: Box::leak(Box::new([crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, set])) });
    store
}
fn check_values(disk: &mut Disk, store: &mut Store) {
    for number in [11, 21] {
        let mut loaded = Vec::new();
        for row in records(number) {
            let expected = jig_domain_core_vocabulary::Record::Core(CoreRecord::Projection(row.clone()));
            let events = disk.request(store, Request::Get { owner: Token::new(7), key: Get::Jig(expected.key()) });
            let [Event::Got { row: Some(Row::Jig(found)), .. }] = events.as_ref() else { panic!("read: {events:?}") };
            assert_eq!(found, &expected);
            loaded.push(row);
        }
        let [
            ProjectionRecord::GoalHot { goal: hot_goal, pending, changed, fed_at },
            ProjectionRecord::TaskHot { task: hot_task, .. },
            ProjectionRecord::GoalWords { words: goal_words, .. },
            ProjectionRecord::TaskWords { words: task_words, .. },
        ] = loaded.as_slice()
        else {
            unreachable!()
        };
        assert_eq!(hot_goal.clone().join(goal_words.clone()), goal(number));
        assert_eq!(hot_task.clone().join(task_words.clone()), task(number));
        assert_eq!(pending.as_ref(), &[2, 3]);
        assert!(*changed);
        assert_eq!(*fed_at, Some(19));
    }
}

#[test]
fn projection_parts_open_splits_whole_rows_keeps_milestones_and_runs_once() {
    let mut disk = Disk::new(&[]);
    let set = crate::JIG_STORE_PATCH_SET;
    let old_set = PatchSet {
        jobs: &[],
        families: Box::leak(
            set.families
                .iter()
                .map(|family| crate::KeptFamily { types: &[], current: family.texts.last().unwrap(), ..*family })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        ..set
    };
    let mut before = store(&disk, old_set);
    assert_opened(&disk.open(&mut before), 0);
    let committed = disk.request(&mut before, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { number: 1, .. }] = committed.as_ref() else { panic!("commit: {committed:?}") };
    let milestone =
        fixtures::core_rows::milestone(jig_domain_core_vocabulary::MilestoneId::Lifecycle { task: 12, position: 13 });
    let milestone_key = key(&milestone);
    let milestone_bytes = crate::codec::core::encode(&CoreRecord::Projection(milestone), &before.codec_limits).unwrap();
    let mut ops = Vec::new();
    for number in [11, 21] {
        for row in legacy(number) {
            ops.push(skein_kv::Op::Put { key: row.key, value: row.value });
        }
    }
    ops.push(skein_kv::Op::Put { key: milestone_key.clone(), value: milestone_bytes.clone() });
    registry::raw(&mut disk, &mut before, ops);
    assert_eq!(jobs::done(&mut before, Owner::Jig, "projection-parts"), None);
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, crate::JIG_STORE_PATCH_SET);
    let mut events = VecDeque::from(disk.open(&mut current).into_vec());
    let mut ran = 0_u32;
    let mut converted = 0;
    let mut partial = 0_u32;
    let mut last = None;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: Owner::Jig, job, page } => {
                let answer = crate::jobs::page(&job, &page).unwrap();
                if job.as_ref() == "projection-parts" {
                    assert!(answer.writes.len() <= usize::try_from(page.room.writes).unwrap());
                    converted += answer.writes.len() / 2;
                    if !page.rows.is_empty() && answer.through.as_ref() != page.rows.last().map(|r| &r.key) {
                        partial += 1;
                    }
                }
                events.extend(disk.request(&mut current, Request::Job { owner, answer: Ok(answer) }));
            }
            Event::Ran { of: Owner::Jig, job, commits, .. } => {
                if job.as_ref() == "projection-parts" {
                    ran += 1;
                    assert_eq!(commits, 5);
                }
            }
            Event::Opened { last: number, pending: true, .. } => last = Some(number),
            Event::Census { .. } => {}
            other => panic!("open: {other:?}"),
        }
    }
    assert_eq!((ran, converted), (1, 4));
    assert!(partial > 0, "two puts per row stop before the page's end");
    assert!(jobs::done(&mut current, Owner::Jig, "projection-parts").is_some());
    check_values(&mut disk, &mut current);
    assert_eq!(fixtures::get_kv(&mut current, &milestone_key), Some(milestone_bytes.clone()));
    let switched = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = switched.as_ref() else { panic!("switch: {switched:?}") };
    let mut again = store(&disk, crate::JIG_STORE_PATCH_SET);
    let opened = disk.open(&mut again);
    assert_opened(&opened, last.unwrap());
    let Some(Event::Opened { pending: false, .. }) = opened.first() else { panic!("reopen: {opened:?}") };
    check_values(&mut disk, &mut again);
    assert_eq!(fixtures::get_kv(&mut again, &milestone_key), Some(milestone_bytes));
}
