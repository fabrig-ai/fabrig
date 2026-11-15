//! Value-type declarations are refused before any open writes (protocol/store/layouts.md, 2).
use super::*;
use crate::{Converts, Job, KeptFamily, KeptMap, KeyRange, Keys, Owner, PatchSet, PatchSets, ValueType};

const TEXT: &str = "family fixture-values 1\nrecord Plain { n: u8 }\nenum Value { left: Plain, right }\nversioned record Row { value: Value }\n";
const OLDER: &str = "family fixture-values 0\nrecord Plain { n: u8 }\n";
const NEWER: &str = "family fixture-values 2\nrecord Plain { n: u8 }\n";
const CHANGED: &str = "family fixture-values 1\nrecord Plain { n: u16 }\nenum Value { left: Plain, right }\nversioned record Row { value: Value }\n";

fn ranges(rows: &[(&'static [u8], &'static [u8])]) -> &'static [KeyRange] {
    Box::leak(rows.iter().map(|&(from, to)| KeyRange { from, to }).collect::<Vec<_>>().into_boxed_slice())
}
fn types(rows: Vec<ValueType>) -> &'static [ValueType] {
    Box::leak(rows.into_boxed_slice())
}
fn family(owner: Owner) -> KeptFamily {
    let keys = if owner == Owner::Application {
        Keys { from: b"l", to: b"m", root: "Row" }
    } else {
        Keys { from: jig_test_patches_jig::keys::NOTES_FROM, to: jig_test_patches_jig::keys::NOTES_TO, root: "Row" }
    };
    KeptFamily {
        name: "fixture-values",
        texts: &[TEXT],
        current: TEXT,
        types: types(vec![ValueType {
            number: 0,
            root: "Row",
            variant: Some("left"),
            keys: ranges(&[(keys.from, keys.to)]),
        }]),
        keys: Box::leak(Box::new([keys])),
        map: KeptMap::Store,
        flights: &[],
        frames: None,
    }
}
fn set(owner: Owner, family: KeptFamily, jobs: &'static [Job]) -> PatchSet {
    PatchSet {
        owner,
        name: if owner == Owner::Application { "fixture" } else { "jig-fixture" },
        families: Box::leak(Box::new([family])),
        level: Some(0),
        patches: &[],
        fresh: None,
        jobs,
    }
}
fn opened(disk: &mut Disk, set: PatchSet) -> Box<[Event]> {
    let mut store = disk.store_with_chunk(CHUNK);
    store.application.version = 0;
    store.patch_sets(PatchSets { sets: Box::leak(Box::new([set])) });
    disk.open(&mut store)
}
fn check(label: &str, set: PatchSet, accepted: bool) {
    let mut empty = Disk::new(&[]);
    // A valid declaration seeds an equal-level store; refusal must not depend
    // on whether the open is a first start or already at the binary's levels.
    let valid = self::set(set.owner, family(set.owner), &[]);
    assert_opened(&opened(&mut empty, valid), 0);
    let items = empty.items();
    for mut disk in [Disk::new(&[]), Disk::new(&items)] {
        let events = opened(&mut disk, set);
        if accepted {
            assert_opened(&events, 0);
        } else {
            let [Event::OpenRefused { why: OpenRefusal::Set { owner, patch: 0 }, .. }] = events.as_ref() else {
                panic!("{label}: {events:?}");
            };
            assert_eq!(*owner, set.owner, "{label}");
        }
    }
}

#[test]
fn value_type_admission_checks_metadata_on_first_and_equal_level_opens() {
    for owner in [Owner::Application, Owner::Jig] {
        let base = family(owner);
        let ty = base.types[0];
        let mid: &'static [u8] = Box::leak([ty.keys[0].from, b"/"].concat().into_boxed_slice());
        let a = ValueType { keys: ranges(&[(ty.keys[0].from, mid)]), ..ty };
        let b = ValueType { number: 1, variant: Some("right"), keys: ranges(&[(mid, ty.keys[0].to)]), ..ty };
        for (label, family, accepted) in [
            ("unregistered", KeptFamily { types: &[], ..base }, true),
            ("distinct numbers", KeptFamily { types: types(vec![a, b]), ..base }, true),
            ("duplicate number", KeptFamily { types: types(vec![a, ValueType { number: 0, ..b }]), ..base }, false),
            ("missing root", KeptFamily { types: types(vec![ValueType { root: "Absent", ..ty }]), ..base }, false),
            (
                "plain root",
                KeptFamily { types: types(vec![ValueType { root: "Plain", variant: None, ..ty }]), ..base },
                true,
            ),
            (
                "variant on plain root",
                KeptFamily { types: types(vec![ValueType { root: "Plain", ..ty }]), ..base },
                false,
            ),
            ("whole broad record", KeptFamily { types: types(vec![ValueType { variant: None, ..ty }]), ..base }, true),
            (
                "whole enumeration",
                KeptFamily { types: types(vec![ValueType { root: "Value", variant: None, ..ty }]), ..base },
                true,
            ),
            ("enumeration variant", KeptFamily { types: types(vec![ValueType { root: "Value", ..ty }]), ..base }, true),
            (
                "absent variant",
                KeptFamily { types: types(vec![ValueType { variant: Some("absent"), ..ty }]), ..base },
                false,
            ),
            ("secrets map", KeptFamily { map: KeptMap::Secrets, ..base }, false),
            ("malformed current", KeptFamily { current: "not a schema", ..base }, false),
            ("newer version", KeptFamily { current: NEWER, ..base }, false),
            ("older version", KeptFamily { current: OLDER, ..base }, false),
            ("same version changed text", KeptFamily { current: CHANGED, ..base }, true),
            ("no latest text", KeptFamily { texts: &[], ..base }, false),
        ] {
            check(label, set(owner, family, &[]), accepted);
        }
    }
}

#[test]
fn value_type_ranges_are_ordered_disjoint_and_cover_exactly_the_family() {
    for owner in [Owner::Application, Owner::Jig] {
        let base = family(owner);
        let ty = base.types[0];
        let from = ty.keys[0].from;
        let to = ty.keys[0].to;
        let mid: &'static [u8] = Box::leak([from, b"/"].concat().into_boxed_slice());
        let later: &'static [u8] = Box::leak([from, b"x"].concat().into_boxed_slice());
        let a = ValueType { keys: ranges(&[(from, mid)]), ..ty };
        let b = ValueType { number: 1, keys: ranges(&[(mid, to)]), ..ty };
        let split = Box::leak(Box::new([Keys { from, to: mid, root: "Row" }, Keys { from: mid, to, root: "Row" }]));
        let gapped = Box::leak(Box::new([Keys { from, to: mid, root: "Row" }, Keys { from: later, to, root: "Row" }]));
        for (label, family, accepted) in [
            ("adjacent types", KeptFamily { types: types(vec![a, b]), ..base }, true),
            ("one type crosses adjacent keys", KeptFamily { keys: split, ..base }, true),
            (
                "one type several ranges",
                KeptFamily { types: types(vec![ValueType { keys: ranges(&[(from, mid), (mid, to)]), ..ty }]), ..base },
                true,
            ),
            (
                "interleaved types",
                KeptFamily {
                    types: types(vec![
                        ValueType { keys: ranges(&[(from, mid), (later, to)]), ..ty },
                        ValueType { keys: ranges(&[(mid, later)]), ..b },
                    ]),
                    ..base
                },
                true,
            ),
            (
                "exact gapped keys",
                KeptFamily {
                    keys: gapped,
                    types: types(vec![ValueType { keys: ranges(&[(from, mid), (later, to)]), ..ty }]),
                    ..base
                },
                true,
            ),
            ("empty ranges", KeptFamily { types: types(vec![ValueType { keys: &[], ..ty }]), ..base }, false),
            (
                "empty interval",
                KeptFamily { types: types(vec![ValueType { keys: ranges(&[(from, from)]), ..ty }]), ..base },
                false,
            ),
            (
                "reversed interval",
                KeptFamily { types: types(vec![ValueType { keys: ranges(&[(to, from)]), ..ty }]), ..base },
                false,
            ),
            ("unordered types", KeptFamily { types: types(vec![b, a]), ..base }, false),
            (
                "unordered ranges",
                KeptFamily { types: types(vec![ValueType { keys: ranges(&[(mid, to), (from, mid)]), ..ty }]), ..base },
                false,
            ),
            (
                "duplicate starts",
                KeptFamily { types: types(vec![a, ValueType { keys: ranges(&[(from, to)]), ..b }]), ..base },
                false,
            ),
            (
                "overlap",
                KeptFamily { types: types(vec![ValueType { keys: ranges(&[(from, later)]), ..a }, b]), ..base },
                false,
            ),
            (
                "internal overlap",
                KeptFamily {
                    types: types(vec![ValueType { keys: ranges(&[(from, later), (mid, to)]), ..ty }]),
                    ..base
                },
                false,
            ),
            (
                "gap",
                KeptFamily { types: types(vec![a, ValueType { keys: ranges(&[(later, to)]), ..b }]), ..base },
                false,
            ),
            ("missing prefix", KeptFamily { types: types(vec![b]), ..base }, false),
            ("missing suffix", KeptFamily { types: types(vec![a]), ..base }, false),
            (
                "outside family end",
                KeptFamily { types: types(vec![ValueType { keys: ranges(&[(from, b"z")]), ..ty }]), ..base },
                false,
            ),
            (
                "outside family start",
                KeptFamily { types: types(vec![ValueType { keys: ranges(&[(b"", to)]), ..ty }]), ..base },
                false,
            ),
            ("fills family gap", KeptFamily { keys: gapped, ..base }, false),
            ("no family keys", KeptFamily { keys: &[], ..base }, false),
        ] {
            check(label, set(owner, family, &[]), accepted);
        }
    }
}

#[test]
fn written_jobs_name_a_registered_type_and_its_ranges() {
    for owner in [Owner::Application, Owner::Jig] {
        let base = family(owner);
        let job = Job {
            counts: &[],
            name: "convert",
            keys: base.types[0].keys,
            converts: Some(Converts { family: base.name, value_type: 0, from: [0; 32] }),
        };
        let wrong = Job { converts: Some(Converts { value_type: 1, ..job.converts.unwrap() }), ..job };
        let mid: &'static [u8] = Box::leak([job.keys[0].from, b"/"].concat().into_boxed_slice());
        let subset = Job { keys: ranges(&[(job.keys[0].from, mid)]), ..job };
        for (label, family, job, accepted) in [
            ("registered number", base, job, true),
            ("undeclared number", base, wrong, false),
            ("unregistered family", KeptFamily { types: &[], ..base }, job, false),
            ("other ranges", base, subset, false),
        ] {
            check(label, set(owner, family, Box::leak(Box::new([job]))), accepted);
        }
    }
}
