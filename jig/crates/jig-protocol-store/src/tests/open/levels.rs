//! Levels decide before the version and layout checks, with no file operation.
use super::{assert_opened, assert_refusal, fixture_format, replacing, rows};
use crate::tests::{
    fixtures,
    owners::{BASE, SETS, levels},
};
use crate::upgrade::admission::{self, Levels};
use crate::{Event, Family, KeptFamily, OpenRefusal, Owner, OwnerSet, OwnerSets, Store, format};
use skein_lib::{Queue, Token};

const FOUR: OwnerSets = OwnerSets {
    sets: &[
        OwnerSet { owner: Owner::Skein, families: &[], level: Some(0), ..BASE.sets[1] },
        OwnerSet { owner: Owner::Smith, families: &[], level: Some(0), ..BASE.sets[1] },
        OwnerSet { level: Some(0), ..BASE.sets[1] },
        OwnerSet { level: Some(0), ..BASE.sets[0] },
    ],
};
const ZERO: Levels = Levels { skein: Some(0), smith: Some(0), jig: Some(0), application: Some(0) };

fn store(found: Levels, sets: OwnerSets) -> Store {
    let (key, mut row) = fixture_format();
    row.levels = found;
    let mut store = replacing(skein_kv::Op::Put { key, value: row.encode().expect("format fits") });
    store.owner_sets(sets);
    store.limits.store.value = 128;
    store
}
fn open(store: &mut Store) -> Box<[Event]> {
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::open::open(store, Token::new(1), &mut above, &mut below);
    assert!(below.store.is_empty() && below.secrets.is_empty() && below.transcripts.is_empty(), "no file operation");
    fixtures::events(&mut above)
}
fn refused(store: &mut Store, why: OpenRefusal) {
    let before = rows(store);
    let last = store.stores.store.last();
    let used = store.stores.store.used();
    assert_refusal(&open(store), why);
    assert_eq!(rows(store), before);
    assert_eq!(store.stores.store.last(), last);
    assert_eq!(store.stores.store.used(), used);
    assert_eq!(store.stores.secrets.last(), 0);
}

#[test]
fn each_owner_past_the_binary_refuses_before_any_file_operation() {
    for owner in [Owner::Skein, Owner::Smith, Owner::Jig, Owner::Application] {
        let mut found = ZERO;
        admission::set_level(&mut found, owner, Some(1));
        refused(&mut store(found, FOUR), OpenRefusal::Ahead { owner, found: Some(1), wanted: Some(0) });
        let mut found = format::NONE;
        admission::set_level(&mut found, owner, Some(0));
        refused(&mut store(found, OwnerSets::NONE), OpenRefusal::Ahead { owner, found: Some(0), wanted: None });
    }
}
#[test]
fn ahead_wins_over_unleveled_and_behind_in_upstream_order() {
    for (found, sets, owner, wanted) in [
        (Levels { jig: Some(1), ..format::NONE }, FOUR, Owner::Jig, Some(0)),
        (Levels { skein: Some(1), smith: Some(1), ..ZERO }, FOUR, Owner::Skein, Some(0)),
        (Levels { jig: Some(0), application: Some(3), ..format::NONE }, SETS, Owner::Application, Some(2)),
    ] {
        refused(&mut store(found, sets), OpenRefusal::Ahead { owner, found: admission::level(&found, owner), wanted });
    }
}

#[test]
fn levels_refuse_before_versions_and_layouts_are_considered() {
    let (key, mut row) = fixture_format();
    row.levels = Levels { jig: Some(3), ..format::NONE };
    row.tasks = 1;
    let mut store = replacing(skein_kv::Op::Put { key, value: row.encode().expect("format") });
    store.owner_sets(SETS);
    store.limits.store.value = 128;
    refused(&mut store, OpenRefusal::Ahead { owner: Owner::Jig, found: Some(3), wanted: Some(2) });
}
#[test]
fn a_later_component_version_refuses_ahead_without_reading_its_levels() {
    for sets in [OwnerSets::NONE, SETS] {
        let (key, _) = fixture_format();
        // Only the version prefix is readable; none of the later row is needed.
        let mut store = replacing(skein_kv::Op::Put { key, value: Box::new([0, 4]) });
        store.owner_sets(sets);
        store.limits.store.value = 128;
        refused(
            &mut store,
            OpenRefusal::Ahead { owner: Owner::Jig, found: None, wanted: admission::binary(&sets).jig },
        );
    }
}

#[test]
fn equal_levels_check_current_versions_and_unregistered_layouts() {
    for found in [format::NONE, levels(0, 0)] {
        let sets = if found == format::NONE { OwnerSets::NONE } else { BASE };
        assert_opened(&open(&mut store(found, sets)), 1);
        let (key, mut row) = fixture_format();
        row.levels = found;
        row.application = 2;
        let mut changed = replacing(skein_kv::Op::Put { key, value: row.encode().expect("format") });
        changed.owner_sets(sets);
        refused(&mut changed, OpenRefusal::Format { family: Family::Application, found: 2, wanted: 1 });
        let mut changed = store(found, sets);
        changed.layouts.core[0] ^= 1;
        assert_opened(&open(&mut changed), 1);
    }
}

#[test]
fn a_family_past_the_manifests_bounds_refuses_set_before_any_write() {
    // The manifest counts each kept family by name: at most 64, each at most 64 bytes.
    const BOUND: &str = "fixture-label-at-the-manifests-sixty-four-byte-family-name-bound";
    const PAST: &str = "fixture-label-at-the-manifests-sixty-four-byte-family-name-bounds";
    const AT_BOUND: OwnerSets = OwnerSets {
        sets: &[
            OwnerSet { families: &[KeptFamily { name: BOUND, ..SETS.sets[0].families[0] }], ..SETS.sets[0] },
            SETS.sets[1],
        ],
    };
    const LONG: OwnerSets = OwnerSets {
        sets: &[
            OwnerSet { families: &[KeptFamily { name: PAST, ..SETS.sets[0].families[0] }], ..SETS.sets[0] },
            SETS.sets[1],
        ],
    };
    // Sixty-four of the application's, so jig's one family is the 65th.
    const MANY: OwnerSets =
        OwnerSets { sets: &[OwnerSet { families: &[SETS.sets[0].families[0]; 64], ..SETS.sets[0] }, SETS.sets[1]] };
    assert_eq!((BOUND.len(), PAST.len()), (64, 65));
    for (sets, owner) in [(LONG, Owner::Application), (MANY, Owner::Jig)] {
        let why = OpenRefusal::Set { owner };
        // Equal levels: admission refuses before any file operation.
        refused(&mut store(levels(2, 2), sets), why.clone());
        // A first start: nothing is made in the empty deployment.
        let mut disk = fixtures::disk::Disk::new(&[]);
        let before = disk.contents();
        let mut first = disk.store();
        first.owner_sets(sets);
        assert_refusal(&disk.open(&mut first), why);
        assert_eq!(disk.contents(), before);
    }
    let mut disk = fixtures::disk::Disk::new(&[]);
    let mut first = disk.store();
    first.owner_sets(AT_BOUND);
    let events = disk.open(&mut first);
    let [Event::Opened { .. }, Event::Census { .. }] = &*events else {
        panic!("a first start at the bound opens: {events:?}")
    };
    let manifest = crate::codec::component::Manifest::decode(&disk.bytes(b"generations/1/manifest"));
    assert_eq!(
        manifest.expect("first manifest").counts,
        fixtures::manifest_counts(&[(BOUND, 0, 0), ("fixture-note", 0, 0)])
    );
}
