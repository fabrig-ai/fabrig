//! Family admission, independent of patch scheduling (protocol/store/opening.md, 3).
use super::owners::SETS;
use crate::upgrade::admission;
use crate::{KeptFamily, KeptMap, Owner, OwnerSet, OwnerSets};
#[test]
fn duplicate_owners_are_refused_beside_a_single_owner() {
    const SETS_DUPLICATE: OwnerSets = OwnerSets { sets: &[SETS.sets[0], SETS.sets[0], SETS.sets[1]] };
    assert_eq!(admission::bounds(&SETS_DUPLICATE, &[]).expect_err("duplicate owner"), Owner::Application);
    admission::bounds(&SETS, &[]).expect("single owners");
}
#[test]
fn prepared_texts_match_self_parsing_admission_and_refusal() {
    const TEXT: &str = "family fixture 1\nversioned record Kept { n: u8 }\n";
    const OTHER: &str = "family fixture 1\nversioned record Other { n: u8 }\n";
    const FAMILY: KeptFamily =
        KeptFamily { current: TEXT, keys: &[crate::Keys { from: &[10], to: &[11], root: "Kept" }], ..MANIFEST_FAMILY };
    const ACCEPT: OwnerSets = OwnerSets { sets: &[OwnerSet { families: &[FAMILY], ..MANIFEST_SET }] };
    const CHANGED: OwnerSets =
        OwnerSets { sets: &[OwnerSet { families: &[KeptFamily { current: OTHER, ..FAMILY }], ..MANIFEST_SET }] };
    const REFUSE: OwnerSets = OwnerSets {
        sets: &[OwnerSet {
            families: &[KeptFamily { keys: &[crate::Keys { from: &[11], to: &[10], root: "Kept" }], ..FAMILY }],
            ..MANIFEST_SET
        }],
    };
    let prepared = [skein_codec::schema::parse(TEXT.as_bytes()).expect("prepared fixture")];
    for (sets, expected) in [(ACCEPT, Ok(())), (CHANGED, Err(Owner::Application)), (REFUSE, Err(Owner::Application))] {
        assert_eq!(admission::bounds(&sets, &[]), expected, "self-parsed");
        assert_eq!(admission::bounds(&sets, &prepared), expected, "prepared, with exact-text fallback");
    }
}

// component-v3.schema bounds: 64 counts, each family at most 64 bytes.
const MANIFEST_FAMILIES: usize = 64;
const MANIFEST_NAME: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789ab";
const MANIFEST_FAMILY: KeptFamily = KeptFamily {
    retired: &[],
    current: "family fixture 1
versioned record Row { n: u8 }
",
    types: &[],
    frames: None,
    name: MANIFEST_NAME,
    map: KeptMap::Store,
    keys: &[],
};
const MANIFEST_SET: OwnerSet =
    OwnerSet { jobs: &[], owner: Owner::Application, name: "manifest-bounds", families: &[], level: None };

#[test]
fn manifest_bounds_accept_the_count_and_name_ceilings() {
    let sets = OwnerSets { sets: &[OwnerSet { families: &[MANIFEST_FAMILY; MANIFEST_FAMILIES], ..MANIFEST_SET }] };
    admission::bounds(&sets, &[]).expect("manifest family count and name ceilings fit");
}

#[test]
fn manifest_bounds_refuse_a_count_or_name_past_the_ceiling() {
    const HALF: usize = MANIFEST_FAMILIES / 2;
    const PAST_HALF: usize = HALF + 1;
    for sets in [
        OwnerSets {
            sets: &[
                OwnerSet { families: &[MANIFEST_FAMILY; HALF], ..MANIFEST_SET },
                OwnerSet { owner: Owner::Jig, families: &[MANIFEST_FAMILY; PAST_HALF], ..MANIFEST_SET },
            ],
        },
        OwnerSets {
            sets: &[OwnerSet {
                families: &[KeptFamily {
                    name: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abc",
                    ..MANIFEST_FAMILY
                }],
                ..MANIFEST_SET
            }],
        },
    ] {
        admission::bounds(&sets, &[]).expect_err("invalid sets");
    }
}

#[test]
fn two_families_holding_each_other_refuse_admission() {
    const A: &str = "family cycle-a 1\nversioned record A { b: bytes 64 holds cycle-b B }\n";
    const B: &str = "family cycle-b 2\nversioned record B { a: bytes 64 holds cycle-a A }\n";
    let sets = OwnerSets {
        sets: &[OwnerSet {
            jobs: &[],
            owner: Owner::Application,
            name: "cycle",
            level: Some(1),
            families: &[
                KeptFamily {
                    retired: &[],
                    current: A,
                    types: &[],
                    frames: None,
                    map: KeptMap::Store,
                    name: "cycle-a",
                    keys: &[],
                },
                KeptFamily {
                    retired: &[],
                    current: B,
                    types: &[],
                    frames: None,
                    map: KeptMap::Store,
                    name: "cycle-b",
                    keys: &[],
                },
            ],
        }],
    };
    assert_eq!(admission::bounds(&sets, &[]).expect_err("hold cycle"), Owner::Application);
}
