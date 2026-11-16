//! Each text parsed, layout computed and text's held texts found once in planning,
//! then dropped when the plans are made (jig/docs/design/protocol/store/layouts.md, 7 and 8).
use super::{Layouts, Text, counting, texts};
use crate::tests::fixtures::{self, disk::Disk};
use crate::{KeptFamily, KeptMap, KeyRange, Keys, Owner, PatchSet, PatchSets, Store, ValueType};
use alloc::boxed::Box;
use jig_test_patches::layouts::{CURRENT, FIRST, KEPT, PATCH};

const FAMILY: KeptFamily = KeptFamily {
    name: "fixture-layouts",
    texts: &[FIRST, KEPT],
    current: CURRENT,
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
    fresh: None,
    jobs: &[],
};
const SETS: PatchSets = PatchSets { sets: &[SET] };

#[test]
fn checked_texts_stay_parsed_until_the_decoders_family_clear() {
    let mut layouts = Layouts::new();
    texts::Binary::new(&SETS, &mut layouts).expect("fixture texts check");
    let current = texts::digest(CURRENT.as_bytes());
    let kept = texts::digest(KEPT.as_bytes());
    assert_ne!(current, kept);
    assert!(layouts.texts.get(&current).expect("current text").schema.is_some());
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

fn planning_store(sets: PatchSets) -> Store {
    let mut store = fixtures::over(fixtures::stores());
    store.application.version = 0;
    store.patch_sets(sets);
    store
}

fn persist_registry(store: &mut Store) {
    let mut ops = skein_lib::List::uncapped();
    for _ in 0..store.registry.pending.len() {
        ops.push(store.registry.pending.pop().expect("pending registry write")).expect("uncapped writes");
    }
    fixtures::plant(&mut store.stores, ops.as_slice());
}

fn assert_counts(parsed: &[&str], layouts: u32) {
    let counts = counting::get();
    assert_eq!(counts.parses.len(), parsed.len());
    for text in parsed {
        assert_eq!(counts.parses.get(&texts::digest(text.as_bytes())), Some(&1), "{text}");
    }
    assert!(!counts.parses.contains_key(&texts::digest(FIRST.as_bytes())), "nothing names FIRST");
    assert_eq!(counts.layouts, layouts);
    for count in counts.held.values() {
        assert_eq!(*count, 1, "each text's held texts found once per current or kept");
    }
}

#[test]
fn seeding_and_older_rows_reuse_each_parse_and_layout() {
    let mut store = planning_store(SETS);
    fixtures::plant(
        &mut store.stores,
        &[skein_kv::Op::Put { key: Box::from(b"\x02a/0".as_slice()), value: Box::from([0, 2, 0, 42]) }],
    );
    assert_eq!(super::open(&mut store, false), Ok(true));
    assert_counts(&[KEPT, CURRENT], 2);
    assert_schemas_dropped(&store.registry);
    persist_registry(&mut store);

    // The old row is still held by its values, beside the binary's stored row.
    assert_eq!(super::open(&mut store, false), Ok(false));
    assert_counts(&[KEPT, CURRENT], 2);
    assert_eq!(store.registry.rows.len(), 2);
    assert_schemas_dropped(&store.registry);
}

#[test]
fn seeded_reopen_computes_one_binary_layout_for_many_rows() {
    let mut store = planning_store(SETS);
    assert_eq!(super::open(&mut store, true), Ok(true));
    assert_counts(&[KEPT, CURRENT], 1);
    persist_registry(&mut store);
    let value = store.registry.rows.get(&(Box::from(FAMILY.name), 0, 2)).expect("binary row").value.clone();
    let mut writer = skein_lib::Writer::new(skein_lib::usize_of(value.measure()));
    value.encode(&mut writer).expect("fixture row encodes");
    let key = crate::keys::encode_own(
        &crate::keys::Own::Layout { family: Box::from(FAMILY.name), number: 0, id: 9 },
        store.limits.store.key,
    )
    .expect("fixture row key");
    fixtures::plant(
        &mut store.stores,
        &[
            skein_kv::Op::Put { key, value: writer.finish() },
            skein_kv::Op::Put { key: Box::from(b"\x02a/0".as_slice()), value: Box::from([0, 9, 0, 42, 7]) },
            skein_kv::Op::Put { key: Box::from(b"\x02a/1".as_slice()), value: Box::from([0, 2, 0, 42, 7]) },
        ],
    );
    assert_eq!(super::open(&mut store, false), Ok(false));
    assert_counts(&[KEPT, CURRENT], 1);
    assert_eq!(store.registry.rows.len(), 2);
    for (_, row) in &store.registry.rows {
        assert!(!row.retire, "both stored ids have values and need their checks");
    }
    assert_schemas_dropped(&store.registry);
}

#[test]
fn seeding_families_share_held_parses_and_keep_distinct_layout_inputs() {
    use jig_test_patches::layouts::{CHILD, CHILD_CURRENT, HOLDER};
    const OTHER: &str =
        "family fixture-other 1\nversioned record Other { child: bytes 32 holds fixture-child Child }\n";
    const HOLDER_FAMILY: KeptFamily = KeptFamily {
        name: "fixture-holder",
        texts: &[HOLDER],
        current: HOLDER,
        keys: &[Keys { from: b"h", to: b"i", root: "Row" }, Keys { from: b"j", to: b"k", root: "Row" }],
        types: &[
            ValueType { number: 0, root: "Row", variant: Some("changed"), keys: &[KeyRange { from: b"h", to: b"i" }] },
            ValueType { number: 1, root: "Row", variant: Some("stable"), keys: &[KeyRange { from: b"j", to: b"k" }] },
        ],
        ..FAMILY
    };
    const SETS: PatchSets = PatchSets {
        sets: &[PatchSet {
            families: &[
                FAMILY,
                HOLDER_FAMILY,
                KeptFamily {
                    name: "fixture-child",
                    texts: &[CHILD],
                    current: CHILD_CURRENT,
                    keys: &[],
                    types: &[],
                    ..FAMILY
                },
                KeptFamily {
                    name: "fixture-other",
                    texts: &[OTHER],
                    current: OTHER,
                    keys: &[Keys { from: b"o", to: b"p", root: "Other" }],
                    types: &[ValueType {
                        number: 0,
                        root: "Other",
                        variant: None,
                        keys: &[KeyRange { from: b"o", to: b"p" }],
                    }],
                    ..FAMILY
                },
            ],
            ..SET
        }],
    };
    let mut store = planning_store(SETS);
    assert_eq!(super::open(&mut store, false), Ok(true));
    assert_counts(&[KEPT, CURRENT, HOLDER, CHILD, CHILD_CURRENT, OTHER], 8);
    assert_eq!(counting::get().held.len(), 6);
    assert_schemas_dropped(&store.registry);
}

#[test]
fn refused_planning_keeps_no_unfinished_registry_in_the_store() {
    let mut store = planning_store(SETS);
    let text = crate::registry_v1::Text::new(
        &crate::registry_v1::CEILINGS,
        crate::registry_v1::TextParts { text: Box::from(FIRST.as_bytes()) },
    )
    .expect("fixture text fits");
    let mut writer = skein_lib::Writer::new(skein_lib::usize_of(text.measure()));
    text.encode(&mut writer).expect("fixture text encodes");
    let key =
        crate::keys::encode_own(&crate::keys::Own::Text(texts::digest(CURRENT.as_bytes())), store.limits.store.key)
            .expect("fixture text key");
    fixtures::plant(&mut store.stores, &[skein_kv::Op::Put { key, value: writer.finish() }]);
    let last = store.stores.store.last();
    assert_eq!(super::open(&mut store, true), Err(crate::OpenRefusal::Set { owner: Owner::Application, patch: 0 }));
    assert!(store.registry.texts.is_empty());
    assert!(store.registry.rows.is_empty());
    assert!(store.registry.types.is_empty());
    assert!(store.registry.ranges.is_empty());
    assert!(store.registry.pending.is_empty());
    assert_eq!(store.stores.store.last(), last, "a refused planning writes nothing");
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
