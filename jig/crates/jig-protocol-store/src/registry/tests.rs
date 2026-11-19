//! Each text parsed, layout computed and text's held texts found once in planning,
//! then dropped when the plans are made (jig/docs/design/protocol/store/layouts.md, 7 and 8).
use super::{Layouts, Text, counting, texts};
use crate::tests::fixtures::{self, disk::Disk};
use crate::{KeptFamily, KeptMap, KeyRange, Keys, Owner, OwnerSet, OwnerSets, Store, ValueType};
use alloc::boxed::Box;
use jig_test_layouts::layouts::{BUFFER_CURRENT, BUFFER_KEPT, CURRENT, FIRST, KEPT};

const FAMILY: KeptFamily = KeptFamily {
    name: "fixture-layouts",
    current: CURRENT,
    retired: &[],
    map: KeptMap::Store,
    frames: None,
    keys: &[Keys { from: b"a", to: b"b", root: "Growing" }],
    types: &[ValueType { number: 0, root: "Growing", variant: None, keys: &[KeyRange { from: b"a", to: b"b" }] }],
};
const SET: OwnerSet =
    OwnerSet { owner: Owner::Application, name: "fixture-layouts", families: &[FAMILY], level: Some(1), jobs: &[] };
const SETS: OwnerSets = OwnerSets { sets: &[SET] };

#[test]
fn checked_texts_stay_parsed_until_the_decoders_family_clear() {
    let mut layouts = Layouts::new();
    let kept = texts::digest(KEPT.as_bytes());
    layouts
        .texts
        .insert(kept, Text { bytes: Box::from(KEPT.as_bytes()), schema: None, stored: true, current: false })
        .expect("uncapped texts");
    texts::Binary::new(&SETS, &mut layouts).expect("fixture texts check");
    texts::parse(&mut layouts, kept).expect("stored text parses");
    let current = texts::digest(CURRENT.as_bytes());
    assert_ne!(current, kept);
    assert!(layouts.texts.get(&current).expect("current text").schema.is_some());
    assert!(layouts.texts.get(&kept).expect("kept text").schema.is_some());
    texts::clear(&mut layouts);
    assert!(layouts.texts.get(&current).expect("current text").schema.is_some());
    assert!(layouts.texts.get(&kept).expect("kept text").schema.is_none());
}

#[test]
fn current_equal_to_kept_keeps_its_checked_schema() {
    const SETS: OwnerSets =
        OwnerSets { sets: &[OwnerSet { families: &[KeptFamily { current: KEPT, ..FAMILY }], ..SET }] };
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
    store.owner_sets(SETS);
    let events = disk.open(&mut store);
    let [crate::Event::Opened { last: 0, .. }, crate::Event::Census { .. }] = events.as_ref() else {
        unreachable!("empty fixture disk opens: {events:?}");
    };
    store
}

fn planning_store(sets: OwnerSets) -> Store {
    let mut store = fixtures::over(fixtures::stores());
    store.application.version = 0;
    store.owner_sets(sets);
    let events = fixtures::run(
        &mut store,
        crate::Request::Open {
            purpose: crate::Purpose::Serve,
            owner: skein_lib::Token::new(1),
            root: fixtures::ROOT,
            installation: fixtures::INSTALLATION,
        },
    );
    let [crate::Event::Opened { last: 0, .. }, crate::Event::Census { .. }] = events.as_ref() else {
        panic!("{events:?}")
    };
    fixtures::committed(&mut store, Box::new([fixtures::header(1)]));
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
fn held_rows_reuse_each_parse_and_layout() {
    let mut store =
        planning_store(OwnerSets { sets: &[OwnerSet { families: &[KeptFamily { current: KEPT, ..FAMILY }], ..SET }] });
    store.owner_sets(SETS);
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
    assert_eq!(super::open(&mut store, false), Ok(false));
    assert_counts(&[CURRENT], 1);
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
    assert_counts(&[CURRENT], 1);
    assert_eq!(store.registry.rows.len(), 2);
    for (_, row) in &store.registry.rows {
        assert!(!row.retire, "both stored ids have values and need their checks");
    }
    assert_schemas_dropped(&store.registry);
}

#[test]
fn registered_families_share_held_parses_and_keep_distinct_layout_inputs() {
    use jig_test_layouts::layouts::{CHILD_CURRENT, HOLDER};
    const OTHER: &str =
        "family fixture-other 1\nversioned record Other { child: bytes 32 holds fixture-child Child }\n";
    const HOLDER_FAMILY: KeptFamily = KeptFamily {
        name: "fixture-holder",
        current: HOLDER,
        keys: &[Keys { from: b"h", to: b"i", root: "Row" }, Keys { from: b"j", to: b"k", root: "Row" }],
        types: &[
            ValueType { number: 0, root: "Row", variant: Some("changed"), keys: &[KeyRange { from: b"h", to: b"i" }] },
            ValueType { number: 1, root: "Row", variant: Some("stable"), keys: &[KeyRange { from: b"j", to: b"k" }] },
        ],
        ..FAMILY
    };
    const SETS: OwnerSets = OwnerSets {
        sets: &[OwnerSet {
            families: &[
                FAMILY,
                HOLDER_FAMILY,
                KeptFamily { name: "fixture-child", current: CHILD_CURRENT, keys: &[], types: &[], ..FAMILY },
                KeptFamily {
                    name: "fixture-other",
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
    assert_eq!(super::open(&mut store, false), Ok(false));
    assert_counts(&[CURRENT, HOLDER, CHILD_CURRENT, OTHER], 4);
    assert_eq!(counting::get().held.len(), 3);
    assert_schemas_dropped(&store.registry);
}

#[test]
fn refused_planning_keeps_no_unfinished_registry_in_the_store() {
    let mut store = planning_store(SETS);
    // A new open starts with no serving edge, over the committed registry.
    *store.registry = Layouts::new();
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
    assert_eq!(super::open(&mut store, false), Err(crate::OpenRefusal::Set { owner: Owner::Application }));
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

fn buffer_value(payload: u32, items: u32, converted: bool) -> Box<[u8]> {
    let item_bytes = items.checked_mul(if converted { 9 } else { 1 }).expect("fixture items fit");
    let length = payload.checked_add(item_bytes).expect("fixture body fits").checked_add(10).expect("heads fit");
    let mut value = skein_lib::Writer::new(skein_lib::usize_of(length));
    value.put(&2_u16.to_be_bytes()).expect("version fits");
    value.put(&payload.to_be_bytes()).expect("payload length fits");
    for _ in 0..payload {
        value.put(&[42]).expect("payload fits");
    }
    value.put(&items.to_be_bytes()).expect("item count fits");
    for _ in 0..items {
        value.put(&[42]).expect("item fits");
        if converted {
            value.put(&7_u64.to_be_bytes()).expect("default fits");
        }
    }
    value.finish()
}

fn converting_store(bound: u32) -> Store {
    let mut store = planning_store(OwnerSets {
        sets: &[OwnerSet { families: &[KeptFamily { current: BUFFER_KEPT, ..FAMILY }], ..SET }],
    });
    fixtures::plant(
        &mut store.stores,
        &[skein_kv::Op::Put { key: Box::from(b"\x02a/0".as_slice()), value: buffer_value(0, 1, false) }],
    );
    store.owner_sets(OwnerSets {
        sets: &[OwnerSet { families: &[KeptFamily { current: BUFFER_CURRENT, ..FAMILY }], ..SET }],
    });
    store.limits.store.value = bound;
    assert_eq!(super::open(&mut store, false), Ok(true));
    assert!(!store.registry.resolve(b"\x02a/0").expect("registered type").plans.is_empty());
    store
}

#[test]
fn converting_open_defers_buffer_until_first_converted_read() {
    let mut store = converting_store(1_000_000);
    assert_eq!(store.layout_work.buffer.capacity(), 0);
    let mut value = buffer_value(2990, 0, false);
    assert_eq!(value.len(), 3000);
    // The deployment header uses the same lazy conversion path before activation.
    super::header(&mut store, b"\x02a/0", &mut value);
    assert_eq!(store.layout_work.buffer.capacity(), 8192);
    assert_eq!(value, buffer_value(2990, 0, true));
    let held = value;
    let buffer = store.layout_work.buffer.as_slice().as_ptr();
    store.registry.activate();
    let mut value = buffer_value(0, 1, false);
    super::read(&mut store, b"\x02a/0", &mut value);
    assert_eq!(value, buffer_value(0, 1, true));
    assert_eq!(store.layout_work.buffer.capacity(), 8192, "a smaller read keeps the room");
    assert_eq!(store.layout_work.buffer.as_slice().as_ptr(), buffer);
    assert_eq!(held, buffer_value(2990, 0, true), "converted rows own their bytes across reads");
}

#[test]
fn converted_values_grow_by_doubling_then_clamp_to_the_bound() {
    let mut store = converting_store(50_000);
    store.registry.activate();
    for (items, capacity) in [(2990, 32_768), (5000, 50_000)] {
        let mut value = buffer_value(0, items, false);
        super::read(&mut store, b"\x02a/0", &mut value);
        assert_eq!(store.layout_work.buffer.capacity(), capacity);
        assert_eq!(value, buffer_value(0, items, true));
    }
}

#[test]
fn conversion_that_cannot_fit_at_the_bound_leaves_the_owned_row_untouched() {
    let mut store = converting_store(20_000);
    store.registry.activate();
    let original = buffer_value(0, 2990, false);
    let mut value = original.clone();
    let pointer = value.as_ptr();
    super::read(&mut store, b"\x02a/0", &mut value);
    assert_eq!(store.layout_work.buffer.capacity(), 20_000);
    assert_eq!(value.as_ptr(), pointer);
    assert_eq!(value, original);
    let mut value = buffer_value(0, 1, false);
    super::read(&mut store, b"\x02a/0", &mut value);
    assert_eq!(value, buffer_value(0, 1, true), "a failed pass does not poison the workspace");
}

#[test]
fn initial_conversion_room_has_a_floor_and_never_exceeds_the_bound() {
    for bound in [1000, 5000, u32::MAX] {
        let mut store = converting_store(bound);
        store.registry.activate();
        let mut value = buffer_value(0, 1, false);
        super::read(&mut store, b"\x02a/0", &mut value);
        assert_eq!(store.layout_work.buffer.capacity(), bound.min(4096));
        assert_eq!(value, buffer_value(0, 1, true));
    }
}

#[test]
fn kept_and_binary_reads_rehead_in_place_without_a_buffer() {
    let mut store = planning_store(SETS);
    assert_eq!(super::open(&mut store, false), Ok(false));
    let row = store.registry.rows.get(&(Box::from(FAMILY.name), 0, 2)).expect("binary row").value.clone();
    let mut writer = skein_lib::Writer::new(skein_lib::usize_of(row.measure()));
    row.encode(&mut writer).expect("fixture row encodes");
    let key = crate::keys::encode_own(
        &crate::keys::Own::Layout { family: Box::from(FAMILY.name), number: 0, id: 8 },
        store.limits.store.key,
    )
    .expect("alias row key");
    fixtures::plant(
        &mut store.stores,
        &[
            skein_kv::Op::Put { key, value: writer.finish() },
            skein_kv::Op::Put { key: Box::from(b"\x02a/0".as_slice()), value: Box::from([0, 8, 0, 42, 7]) },
            skein_kv::Op::Put { key: Box::from(b"\x02a/1".as_slice()), value: Box::from([0, 2, 0, 42, 7]) },
        ],
    );
    assert_eq!(super::open(&mut store, false), Ok(false));
    assert!(store.registry.resolve(b"\x02a/0").expect("registered type").keeping.as_slice().contains(&2));
    for id in [2, 8] {
        let mut value: Box<[u8]> = Box::from([0, id, 0, 42, 7].as_slice());
        let pointer = value.as_ptr();
        super::read(&mut store, b"\x02a/0", &mut value);
        assert_eq!(value.as_ref(), &[0, 2, 0, 42, 7]);
        assert_eq!(value.as_ptr(), pointer);
        assert_eq!(store.layout_work.buffer.capacity(), 0);
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
