//! A settings-v3 store is seeded and converted once (protocol/store/layouts.md, 6).
use super::*;
use crate::tests::charter_versions::jig_set;
use crate::tests::charter_versions::{key, old, record};
use crate::{Owner, PatchSet, PatchSets};
use alloc::collections::VecDeque;

fn store(disk: &Disk, set: PatchSet) -> Store {
    // This scene runs the charter job; the other jig jobs have their own page fixtures.
    let set = PatchSet {
        jobs: Box::leak(
            set.jobs
                .iter()
                .copied()
                .filter(|job| job.name == "charter-versions")
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        ..set
    };
    let mut limits = fixtures::store_limits();
    // The real family's schema texts and registered roots live in this map.
    limits.store.budget = 2_097_152;
    limits.store.ops = 128;
    let mut store = disk.store_with_limits(&limits);
    store.patch_sets(PatchSets { sets: Box::leak(Box::new([crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, set])) });
    store
}
fn check_values(disk: &mut Disk, store: &mut Store) {
    for expected in [record(false, true), record(true, true)] {
        let events = disk.request(store, Request::Get { owner: Token::new(7), key: Get::Jig(expected.key()) });
        let [Event::Got { row: Some(Row::Jig(found)), .. }] = events.as_ref() else { panic!("read: {events:?}") };
        assert_eq!(found, &expected);
        let bytes = fixtures::get_kv(store, &key(&expected)).unwrap();
        assert_eq!(&bytes[..2], &[0, 4], "puts take the new binary layout id");
    }
}
#[test]
fn charter_versions_open_seeds_runs_two_rows_marks_done_and_reopens_without_writes() {
    let mut disk = Disk::new(&[]);
    let set = jig_set();
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
    registry::raw(
        &mut disk,
        &mut before,
        [record(false, true), record(true, true)]
            .iter()
            .map(|r| skein_kv::Op::Put { key: key(r), value: old(r) })
            .collect(),
    );
    let seed = crate::keys::Own::Layout { family: "jig-store-settings".into(), number: 4, id: 3 };
    let seed = crate::keys::encode_own(&seed, before.limits.store.key).unwrap();
    assert!(fixtures::get_kv(&mut before, &seed).is_none());
    assert_eq!(jobs::done(&mut before, Owner::Jig, "charter-versions"), None);
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, jig_set());
    let mut events = VecDeque::from(disk.open(&mut current).into_vec());
    let mut ran = 0_u32;
    let mut pages = 0_u32;
    let mut last = None;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: Owner::Jig, job, page } if job.as_ref() != "charter-versions" => {
                assert!(page.rows.is_empty(), "other jobs have no fixture rows");
                events
                    .extend(disk.request(&mut current, Request::Job { owner, answer: crate::jobs::page(&job, &page) }));
            }
            Event::Ran { of: Owner::Jig, job, rows, .. } if job.as_ref() != "charter-versions" => {
                assert_eq!(rows, 0, "other jobs have no fixture rows");
            }
            Event::Job { owner, of: Owner::Jig, job, page } => {
                if job.as_ref() == "charter-versions" {
                    if pages == 0 {
                        let layout = page.layout.as_ref().unwrap();
                        assert_eq!(layout.text.as_ref(), crate::tests::charter_versions::KEPT.as_bytes());
                        assert_eq!(layout.root.as_ref(), "CharterVersion");
                        assert_eq!(page.rows.len(), 2);
                        // The seed exists before any page reaches its owner.
                        let seed = crate::keys::Own::Layout { family: "jig-store-settings".into(), number: 4, id: 3 };
                        let seed = crate::keys::encode_own(&seed, current.limits.store.key).unwrap();
                        assert!(fixtures::get_kv(&mut current, &seed).is_some());
                    } else {
                        assert!(page.layout.is_none());
                        assert!(page.end);
                    }
                    pages += 1;
                }
                let answer = crate::jobs::page(&job, &page);
                events.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::Ran { of: Owner::Jig, job, rows, commits, .. } => {
                if job.as_ref() == "charter-versions" {
                    assert_eq!((rows, commits), (2, 2));
                    ran += 1;
                }
            }
            Event::Opened { last: number, pending: true, .. } => last = Some(number),
            Event::Census { .. } => {}
            other => panic!("open after {pages} pages: {other:?}"),
        }
    }
    assert_eq!((ran, pages), (1, 2));
    assert!(jobs::done(&mut current, Owner::Jig, "charter-versions").is_some());
    check_values(&mut disk, &mut current);
    let switched = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = switched.as_ref() else { panic!("switch: {switched:?}") };
    let contents = disk.contents();
    let mut again = store(&disk, jig_set());
    let opened = disk.open(&mut again);
    assert_opened(&opened, last.unwrap());
    let Some(Event::Opened { pending: false, .. }) = opened.first() else { panic!("reopen: {opened:?}") };
    check_values(&mut disk, &mut again);
    // Recovery may create an empty segment, but adds no committed writes.
    assert_eq!(committed_bytes(disk.contents()), committed_bytes(contents));
}
type Contents = Vec<(Box<[u8]>, Kind, u32, Box<[u8]>)>;
fn committed_bytes(contents: Contents) -> Contents {
    contents.into_iter().filter(|(_, kind, _, bytes)| *kind == Kind::File && !bytes.is_empty()).collect()
}
#[test]
fn an_empty_stores_first_commit_marks_charter_versions_done_without_running_it() {
    let mut disk = Disk::new(&[]);
    let mut current = store(&disk, jig_set());
    assert_opened(&disk.open(&mut current), 0);
    let committed = disk.request(&mut current, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { number: 1, .. }] = committed.as_ref() else { panic!("commit: {committed:?}") };
    assert_eq!(jobs::done(&mut current, Owner::Jig, "charter-versions"), Some(1));
    let mut again = store(&disk, jig_set());
    assert_opened(&disk.open(&mut again), 1);
}
