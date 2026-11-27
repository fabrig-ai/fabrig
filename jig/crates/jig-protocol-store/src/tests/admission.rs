//! Family admission, independent of job scheduling (protocol/store/opening-owners.md, 1).
use super::{frame_layouts, owners::SETS};
use crate::upgrade::admission;
use crate::{JIG_STORE_SET, KeptFamily, KeptMap, Owner, OwnerSet, OwnerSets, SKEIN_SET, SMITH_SET};
use alloc::collections::BTreeSet;
use jig_test_layouts::{layouts, records};
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
    for (sets, refusal) in [(ACCEPT, None), (CHANGED, Some(Owner::Application)), (REFUSE, Some(Owner::Application))] {
        assert_eq!(admission::bounds(&sets, &[]).err(), refusal, "self-parsed");
        assert_eq!(admission::bounds(&sets, &prepared).err(), refusal, "prepared, with exact-text fallback");
    }
}

#[test]
fn prepare_keeps_only_current_texts_in_first_seen_order() {
    const OWNERS: OwnerSets =
        OwnerSets { sets: &[SETS.sets[0], frame_layouts::SETS.sets[0], frame_layouts::SETS.sets[1]] };
    const EXPECTED: [&str; 5] = [
        records::LABEL_CURRENT,
        records::SHELF_CURRENT,
        records::TOKEN_CURRENT,
        layouts::FRAME_BODY,
        layouts::FRAME_TURN,
    ];
    let prepared = OWNERS.prepare();
    assert_eq!(prepared.len(), EXPECTED.len());
    for (schema, text) in prepared.iter().zip(EXPECTED) {
        assert_eq!(schema.text(), text.as_bytes());
    }
}

#[test]
fn prepare_omits_a_malformed_current_and_keeps_the_other_texts() {
    const APP: OwnerSet = SETS.sets[0];
    const BROKEN: KeptFamily =
        KeptFamily { current: "family fixture-shelf 2\nversioned record Shelf {", ..APP.families[1] };
    const OWNERS: OwnerSets =
        OwnerSets { sets: &[OwnerSet { families: &[APP.families[0], BROKEN, APP.families[2]], ..APP }] };
    let prepared = OWNERS.prepare();
    let expected = [records::LABEL_CURRENT, records::TOKEN_CURRENT];
    assert_eq!(prepared.len(), expected.len());
    for (schema, text) in prepared.iter().zip(expected) {
        assert_eq!(schema.text(), text.as_bytes());
    }
}

#[test]
fn prepare_no_owner_sets_is_empty() {
    assert!(OwnerSets::NONE.prepare().is_empty());
}

#[test]
fn prepare_binary_sets_keeps_each_text_once_including_core() {
    const BINARY: OwnerSets = OwnerSets { sets: &[SKEIN_SET, SMITH_SET, JIG_STORE_SET] };
    let mut expected = BTreeSet::new();
    let mut core_text = None;
    for set in BINARY.sets {
        for family in set.families {
            expected.insert(family.current.as_bytes());
            if set.owner == Owner::Jig && family.frames.is_some() {
                core_text = Some(family.current.as_bytes());
            }
        }
    }

    let prepared = BINARY.prepare();
    let mut texts = BTreeSet::new();
    for schema in &*prepared {
        texts.insert(schema.text());
    }
    assert_eq!(texts, expected);
    assert_eq!(prepared.len(), texts.len(), "one schema per distinct text");
    assert!(texts.contains(core_text.expect("core frames family")));
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
