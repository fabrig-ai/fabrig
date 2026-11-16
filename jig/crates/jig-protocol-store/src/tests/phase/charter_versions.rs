//! A fresh store marks the charter job done (protocol/store/layouts.md, 6).
use super::*;
use crate::tests::charter_versions::jig_set;
use crate::{Owner, PatchSet, PatchSets};

fn store(disk: &Disk, set: PatchSet) -> Store {
    // This scene runs the charter job; the other jig jobs have their own page fixtures.
    let set = PatchSet {
        families: Box::leak(
            set.families
                .iter()
                .map(|family| {
                    if family.name == "jig-store-settings" {
                        *family
                    } else {
                        crate::KeptFamily { types: &[], retired: &[], ..*family }
                    }
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
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
