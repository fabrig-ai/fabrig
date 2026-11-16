//! The open's parses (protocol/store/layouts.md, 7 and 8).
use super::{Layouts, Text, texts};
use crate::tests::fixtures::{self, disk::Disk};
use crate::{KeptFamily, KeptMap, KeyRange, Keys, Owner, PatchSet, PatchSets, Store, ValueType};
use alloc::boxed::Box;
use jig_test_patches::layouts::{CURRENT, FIRST, KEPT, PATCH};

const FAMILY: KeptFamily = KeptFamily {
    name: "fixture-layouts",
    texts: &[FIRST, KEPT],
    current: CURRENT,
    retired: &[],
    map: KeptMap::Store,
    frames: None,
    flights: &[],
    keys: &[Keys { from: b"a", to: b"b", root: "Growing" }],
    types: &[ValueType { number: 0, root: "Growing", variant: None, keys: &[KeyRange { from: b"a", to: b"b" }] }],
};
const SET: PatchSet = PatchSet {
    owner: Owner::Application,
    name: "fixture-layouts",
    families: &[FAMILY],
    patches: &[PATCH],
    level: Some(1),
    fresh: None,
    jobs: &[],
};
const SETS: PatchSets = PatchSets { sets: &[SET] };

#[test]
fn checked_current_survives_family_clear_while_kept_schema_drops() {
    let mut layouts = Layouts::new();
    texts::Binary::new(&SETS, &mut layouts).expect("fixture texts check");
    let current = texts::digest(CURRENT.as_bytes());
    let kept = texts::digest(KEPT.as_bytes());
    assert_ne!(current, kept);
    assert!(layouts.texts.get(&current).expect("current text").schema.is_some());
    assert!(layouts.texts.get(&kept).expect("kept text").schema.is_none());
    texts::parse(&mut layouts, kept).expect("seed parses kept text");
    assert!(layouts.texts.get(&kept).expect("kept text").schema.is_some());
    texts::clear(&mut layouts);
    assert!(layouts.texts.get(&current).expect("current text").schema.is_some());
    assert!(layouts.texts.get(&kept).expect("kept text").schema.is_none());
}

#[test]
fn current_equal_to_kept_keeps_its_checked_schema() {
    const SETS: PatchSets =
        PatchSets { sets: &[PatchSet { families: &[KeptFamily { current: KEPT, ..FAMILY }], ..SET }] };
    let mut layouts = Layouts::new();
    texts::Binary::new(&SETS, &mut layouts).expect("fixture texts check");
    assert_eq!(layouts.texts.len(), 1);
    let digest = texts::digest(KEPT.as_bytes());
    assert!(layouts.texts.get(&digest).expect("current and kept text").schema.is_some());
    texts::clear(&mut layouts);
    assert!(layouts.texts.get(&digest).expect("current and kept text").schema.is_some());
}

#[test]
fn stored_current_waits_for_first_parse_then_survives_family_clear() {
    let mut layouts = Layouts::new();
    let digest = texts::digest(CURRENT.as_bytes());
    layouts
        .texts
        .insert(digest, Text { bytes: Box::from(CURRENT.as_bytes()), schema: None, stored: true, current: false })
        .expect("uncapped texts");
    texts::Binary::new(&SETS, &mut layouts).expect("fixture texts check");
    let text = layouts.texts.get(&digest).expect("stored current");
    assert!(text.stored);
    assert!(text.schema.is_none());
    texts::parse(&mut layouts, digest).expect("first use checks the stored digest and parses");
    texts::clear(&mut layouts);
    assert!(layouts.texts.get(&digest).expect("stored current").schema.is_some());
}

fn opened(disk: &mut Disk) -> Store {
    let mut store = disk.store_with_chunk(4096);
    store.application.version = 0;
    store.patch_sets(SETS);
    let events = disk.open(&mut store);
    let [crate::Event::Opened { last: 0, .. }, crate::Event::Census { .. }] = events.as_ref() else {
        unreachable!("empty fixture disk opens: {events:?}");
    };
    store
}

fn assert_schemas_dropped(layouts: &Layouts) {
    assert!(!layouts.texts.is_empty(), "inspect the open before settlement drops its texts");
    for (_, text) in &layouts.texts {
        assert!(text.schema.is_none(), "plans release every schema");
    }
}

#[test]
fn empty_registry_open_releases_schemas_after_planning() {
    let mut disk = Disk::new(&[]);
    let mut store = opened(&mut disk);
    assert_eq!(super::open(&mut store, true), Ok(true));
    assert_schemas_dropped(&store.registry);
}

#[test]
fn seeded_registry_open_releases_schemas_after_planning() {
    let mut disk = Disk::new(&[]);
    let mut store = opened(&mut disk);
    let events = disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [crate::Event::Committed { number: 1, .. }] = events.as_ref() else {
        unreachable!("first commit registers fixture layouts: {events:?}");
    };
    assert_eq!(super::open(&mut store, false), Ok(false));
    assert!(!store.registry.rows.is_empty());
    for (_, row) in &store.registry.rows {
        assert!(row.stored, "reopen loads the durable registry");
    }
    assert_schemas_dropped(&store.registry);
}
