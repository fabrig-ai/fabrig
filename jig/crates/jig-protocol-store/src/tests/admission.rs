//! Family admission, independent of patch scheduling (protocol/store/opening.md, 3).
use super::owners::SETS;
use crate::patching::plan::{self, Unplanned};
use crate::{KeptFamily, KeptMap, Owner, PatchSet, PatchSets};
#[test]
fn duplicate_owners_are_refused_beside_a_single_owner() {
    const SETS_DUPLICATE: PatchSets = PatchSets { sets: &[SETS.sets[0], SETS.sets[0], SETS.sets[1]] };
    assert_eq!(
        plan::bounds(&SETS_DUPLICATE).expect_err("duplicate owner"),
        Unplanned { owner: Owner::Application, patch: 0 }
    );
    plan::bounds(&SETS).expect("single owners");
}
// component-v3.schema bounds: 64 counts, each family at most 64 bytes.
const MANIFEST_FAMILIES: usize = 64;
const MANIFEST_NAME: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789ab";
const MANIFEST_FAMILY: KeptFamily = KeptFamily {
    retired: &[],
    current: "",
    types: &[],
    flights: &[],
    frames: None,
    name: MANIFEST_NAME,
    map: KeptMap::Store,
    texts: &[],
    keys: &[],
};
const MANIFEST_SET: PatchSet = PatchSet {
    jobs: &[],
    fresh: None,
    owner: Owner::Application,
    name: "manifest-bounds",
    families: &[],
    level: None,
    patches: &[],
};

#[test]
fn manifest_bounds_accept_the_count_and_name_ceilings() {
    let sets = PatchSets { sets: &[PatchSet { families: &[MANIFEST_FAMILY; MANIFEST_FAMILIES], ..MANIFEST_SET }] };
    plan::bounds(&sets).expect("manifest family count and name ceilings fit");
}

#[test]
fn manifest_bounds_refuse_a_count_or_name_past_the_ceiling() {
    const HALF: usize = MANIFEST_FAMILIES / 2;
    const PAST_HALF: usize = HALF + 1;
    for sets in [
        PatchSets {
            sets: &[
                PatchSet { families: &[MANIFEST_FAMILY; HALF], ..MANIFEST_SET },
                PatchSet { owner: Owner::Jig, families: &[MANIFEST_FAMILY; PAST_HALF], ..MANIFEST_SET },
            ],
        },
        PatchSets {
            sets: &[PatchSet {
                families: &[KeptFamily {
                    name: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abc",
                    ..MANIFEST_FAMILY
                }],
                ..MANIFEST_SET
            }],
        },
    ] {
        plan::bounds(&sets).expect_err("invalid sets");
    }
}

#[test]
fn two_families_holding_each_other_refuse_admission() {
    const A: &str = "family cycle-a 1\nversioned record A { b: bytes 64 holds cycle-b B }\n";
    const B: &str = "family cycle-b 2\nversioned record B { a: bytes 64 holds cycle-a A }\n";
    let sets = PatchSets {
        sets: &[PatchSet {
            jobs: &[],
            fresh: None,
            owner: Owner::Application,
            name: "cycle",
            level: Some(1),
            patches: &[],
            families: &[
                KeptFamily {
                    retired: &[],
                    current: A,
                    types: &[],
                    flights: &[],
                    frames: None,
                    map: KeptMap::Store,
                    name: "cycle-a",
                    texts: &[A],
                    keys: &[],
                },
                KeptFamily {
                    retired: &[],
                    current: B,
                    types: &[],
                    flights: &[],
                    frames: None,
                    map: KeptMap::Store,
                    name: "cycle-b",
                    texts: &[B],
                    keys: &[],
                },
            ],
        }],
    };
    assert_eq!(plan::bounds(&sets).expect_err("hold cycle"), Unplanned { owner: Owner::Application, patch: 0 });
}
