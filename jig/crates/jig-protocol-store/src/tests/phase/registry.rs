//! Registry planning over a simulated disk (protocol/store/layouts.md, 8).
#![expect(clippy::arithmetic_side_effects, reason = "ordinary Rust disk step tests")]
use super::*;
use crate::{KeptFamily, KeptMap, KeyRange, Keys, Owner, PatchSet, PatchSets, Unplanned, ValueType};
use jig_test_patches::layouts::{CURRENT, FIRST, KEPT, PATCH, WRITTEN};
use skein_lib::{Buffer, Reader};
mod added_types;
mod edges;
mod retired;
mod settling;
mod turns;

const FAMILY: KeptFamily = KeptFamily {
    retired: &[],
    name: "fixture-layouts",
    texts: &[FIRST, KEPT],
    current: CURRENT,
    map: KeptMap::Store,
    frames: None,
    flights: &[],
    keys: &[Keys { from: b"a", to: b"b", root: "Growing" }, Keys { from: b"c", to: b"d", root: "Stable" }],
    types: &[
        ValueType { number: 0, root: "Growing", variant: None, keys: &[KeyRange { from: b"a", to: b"b" }] },
        ValueType { number: 1, root: "Stable", variant: None, keys: &[KeyRange { from: b"c", to: b"d" }] },
    ],
};
const SET: PatchSet = PatchSet {
    owner: Owner::Application,
    name: "fixture-layouts",
    families: &[FAMILY],
    level: Some(1),
    patches: &[PATCH],
    fresh: None,
    jobs: &[],
};
fn sets(set: PatchSet) -> PatchSets {
    PatchSets { sets: Box::leak(Box::new([set])) }
}
fn store(disk: &Disk, set: PatchSet) -> Store {
    let mut store = disk.store_with_chunk(CHUNK);
    store.application.version = 0;
    store.patch_sets(sets(set));
    store
}
fn base(first: bool, values: bool) -> Vec<Item> {
    let mut disk = Disk::new(&[]);
    let family = KeptFamily {
        texts: if first { &[FIRST] } else { &[FIRST, KEPT] },
        current: if first { FIRST } else { KEPT },
        types: &[],
        ..FAMILY
    };
    let set = PatchSet {
        families: Box::leak(Box::new([family])),
        level: Some(u32::from(!first)),
        patches: if first { &[] } else { &[PATCH] },
        ..SET
    };
    let mut store = store(&disk, set);
    assert_opened(&disk.open(&mut store), 0);
    let mut writes = vec![fixtures::header(1)];
    if values {
        writes.push(Write::Put {
            key: Box::from(b"a/0".as_slice()),
            value: if first { Box::from([0, 1, 42]) } else { Box::from([0, 2, 0, 42]) },
        });
        writes.push(Write::Put {
            key: Box::from(b"c/0".as_slice()),
            value: if first { Box::from([0, 1, 1]) } else { Box::from([0, 2, 1]) },
        });
    }
    assert!({
        if let [Event::Committed { .. }] =
            disk.request(&mut store, fixtures::commit(1, writes.into_boxed_slice())).as_ref()
        {
            true
        } else {
            false
        }
    });
    disk.items()
}
fn app(store: &mut Store, key: &[u8]) -> Option<Box<[u8]>> {
    fixtures::get_kv(store, &crate::keys::application(key, store.limits.store.key).unwrap())
}
fn row(store: &mut Store, number: u16, id: u16) -> Option<crate::registry_v1::Layout> {
    let key = crate::keys::Own::Layout { family: FAMILY.name.into(), number, id };
    let bytes = fixtures::get_kv(store, &crate::keys::encode_own(&key, store.limits.store.key).unwrap())?;
    Some(crate::registry_v1::Layout::decode(&crate::registry_v1::CEILINGS, &mut Reader::new(&bytes)).unwrap())
}
fn text(store: &mut Store, digest: [u8; 32]) -> Option<Box<[u8]>> {
    let key = crate::keys::encode_own(&crate::keys::Own::Text(digest), store.limits.store.key).unwrap();
    let bytes = fixtures::get_kv(store, &key)?;
    Some(Box::from(
        crate::registry_v1::Text::decode(&crate::registry_v1::CEILINGS, &mut Reader::new(&bytes)).unwrap().text(),
    ))
}
fn hash(text: &str) -> [u8; 32] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(text.as_bytes());
    hash.finish()
}
fn switch(disk: &mut Disk, store: &mut Store) {
    assert!({
        if let [Event::Switched { .. }] = disk.request(store, Request::Switch { owner: Token::new(1) }).as_ref() {
            true
        } else {
            false
        }
    });
}
type Contents = Vec<(Box<[u8]>, Kind, u32, Box<[u8]>)>;
fn snapshot(disk: &mut Disk) -> Contents {
    disk.contents()
        .into_iter()
        .filter(|(path, kind, _, bytes)| {
            path.starts_with(b"generations/1/") && (*kind != Kind::File || !bytes.is_empty())
        })
        .collect()
}
#[test]
fn a_new_value_type_has_no_seed_layout_and_registers_its_current_variant() {
    let current = Box::leak(
        format!("{CURRENT}enum Value {{ later: Stable }}\nversioned record Row {{ value: Value }}\n").into_boxed_str(),
    );
    let family = KeptFamily {
        current,
        keys: &[
            Keys { from: b"a", to: b"b", root: "Growing" },
            Keys { from: b"c", to: b"d", root: "Stable" },
            Keys { from: b"e", to: b"f", root: "Row" },
        ],
        types: &[
            ValueType { number: 0, root: "Growing", variant: None, keys: &[KeyRange { from: b"a", to: b"b" }] },
            ValueType { number: 1, root: "Stable", variant: None, keys: &[KeyRange { from: b"c", to: b"d" }] },
            ValueType { number: 2, root: "Row", variant: Some("later"), keys: &[KeyRange { from: b"e", to: b"f" }] },
        ],
        ..FAMILY
    };
    let set = PatchSet { families: Box::leak(Box::new([family])), ..SET };
    let mut disk = Disk::new(&base(false, true));
    let mut store = store(&disk, set);
    assert_opened(&disk.open(&mut store), 4);
    let layout = row(&mut store, 2, 2).unwrap();
    assert_eq!(layout.root(), b"Row");
    assert_eq!(layout.variant().as_deref(), Some(b"later".as_slice()));
    assert_eq!(*layout.text(), hash(current));
}

#[test]
fn seed_and_binary_registration_never_rewrite_values_and_reopen_in_place() {
    let mut disk = Disk::new(&base(false, true));
    let before = snapshot(&mut disk);
    let mut store = store(&disk, SET);
    let events = disk.open(&mut store);
    assert_opened(&events, 4); // five rows in two bounded registry commits, then the seal
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    assert_eq!(snapshot(&mut disk), before);
    for (number, root, id) in [(0, "Growing", 2), (1, "Stable", 2), (0, "Growing", 3)] {
        let row = row(&mut store, number, id).unwrap();
        let schema = skein_codec::schema::parse(if id == 3 { CURRENT.as_bytes() } else { KEPT.as_bytes() }).unwrap();
        let expected = schema.layout(schema.root(root.as_bytes(), None).unwrap(), &[]).unwrap().digest();
        assert_eq!(*row.layout(), expected);
        assert_eq!(row.root(), root.as_bytes());
        assert!(row.held().is_empty());
        assert_eq!(text(&mut store, *row.text()).unwrap().as_ref(), schema.text());
        assert_eq!(*row.text(), hash(if id == 3 { CURRENT } else { KEPT }));
        let key = crate::keys::Own::Layout { family: FAMILY.name.into(), number, id };
        let encoded = crate::keys::encode_own(&key, store.limits.store.key).unwrap();
        assert_eq!(
            encoded.as_ref(),
            [&[0, 6], FAMILY.name.as_bytes(), &[0, 0], &number.to_be_bytes(), &id.to_be_bytes()].concat()
        );
        assert_eq!(crate::keys::decode_own(&encoded), Some(key));
    }
    assert!(row(&mut store, 1, 3).is_none());
    assert_eq!(app(&mut store, b"a/0").unwrap().as_ref(), &[0, 2, 0, 42]);
    assert_eq!(app(&mut store, b"c/0").unwrap().as_ref(), &[0, 2, 1]);
    for space in store.stores.store.census().keyspaces() {
        if [b"\x02a".as_slice(), b"\x02c"].contains(&space.prefix()) {
            let bytes = if space.prefix() == b"\x02a" { 8 } else { 7 };
            assert_eq!((space.rows, space.puts, space.erases, space.written), (1, 1, 0, bytes));
            assert_eq!(space.heads()[0].head, Some(2));
        }
    }
    switch(&mut disk, &mut store);
    let mut again = self::store(&disk, SET);
    let events = disk.open(&mut again);
    assert_opened(&events, 4);
    assert!({ if let Some(Event::Opened { pending: false, .. }) = events.first() { true } else { false } });
    assert!(!disk.tree().iter().any(|(p, ..)| p.as_ref() == b"generations/3"));
}
#[test]
fn patches_finish_before_the_seed_uses_the_binarys_kept_version() {
    let mut disk = Disk::new(&base(true, true));
    let mut store = store(&disk, SET);
    let events = disk.open(&mut store);
    assert!({ if let Some(Event::Patched { patch: 1, .. }) = events.first() { true } else { false } }, "{events:?}");
    assert!(
        { if let Some(Event::Opened { last: 6, pending: true, .. }) = events.get(1) { true } else { false } },
        "{events:?}"
    );
    assert!(row(&mut store, 0, 1).is_none());
    assert_eq!(*row(&mut store, 0, 2).unwrap().text(), hash(KEPT));
    assert_eq!(app(&mut store, b"a/0").unwrap().as_ref(), &[0, 2, 0, 42]);
}
#[test]
fn empty_first_commit_registers_only_the_binary_layouts_at_the_familys_version() {
    let mut disk = Disk::new(&[]);
    let mut store = store(&disk, SET);
    assert_opened(&disk.open(&mut store), 0);
    assert!(row(&mut store, 0, 2).is_none());
    let events = disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
    assert!({ if let [Event::Committed { number: 1, .. }] = events.as_ref() { true } else { false } }, "{events:?}");
    assert_eq!(*row(&mut store, 0, 2).unwrap().text(), hash(CURRENT));
    assert_eq!(*row(&mut store, 1, 2).unwrap().text(), hash(CURRENT));
    assert!(row(&mut store, 0, 3).is_none());
    assert_eq!(text(&mut store, hash(KEPT)), None);
}
#[test]
fn held_changes_move_only_the_broad_roots_reached_variant_and_jobs_get_held_texts() {
    use jig_test_patches::layouts::{CHILD, CHILD_CURRENT, CHILD_WRITTEN, HOLDER};
    let holder = KeptFamily {
        retired: &[],
        name: "fixture-holder",
        texts: &[HOLDER],
        current: HOLDER,
        map: KeptMap::Store,
        frames: None,
        flights: &[],
        keys: &[Keys { from: b"h", to: b"i", root: "Row" }, Keys { from: b"j", to: b"k", root: "Row" }],
        types: &[
            ValueType { number: 0, root: "Row", variant: Some("changed"), keys: &[KeyRange { from: b"h", to: b"i" }] },
            ValueType { number: 1, root: "Row", variant: Some("stable"), keys: &[KeyRange { from: b"j", to: b"k" }] },
        ],
    };
    let child = KeptFamily { name: "fixture-child", texts: &[CHILD], current: CHILD, keys: &[], types: &[], ..holder };
    let set = PatchSet {
        families: Box::leak(Box::new([KeptFamily { types: &[], ..holder }, child])),
        level: Some(0),
        patches: &[],
        ..SET
    };
    let mut disk = Disk::new(&[]);
    let mut store = fresh(&mut disk, set);
    raw(
        &mut disk,
        &mut store,
        vec![
            skein_kv::Op::Put { key: b"\x02h/0".as_slice().into(), value: Box::from([0, 1, 0, 0, 0, 0, 3, 0, 1, 42]) },
            skein_kv::Op::Put { key: b"\x02j/0".as_slice().into(), value: Box::from([0, 1, 1]) },
        ],
    );
    let current =
        PatchSet { families: Box::leak(Box::new([holder, KeptFamily { current: CHILD_CURRENT, ..child }])), ..set };
    let mut disk = Disk::new(&disk.items());
    let mut store = self::store(&disk, current);
    let events = disk.open(&mut store);
    assert!(
        { if let Some(Event::Opened { pending: true, .. }) = events.first() { true } else { false } },
        "{events:?}"
    );
    store.registry.assert_settled();
    let mut value = [0, 1, 0, 0, 0, 0, 3, 0, 1, 42];
    let mut out = Buffer::with_capacity(32);
    assert_eq!(
        store.registry.read(b"\x02h/0", &mut value, &mut out, &mut store.layout_work.workspace),
        &[0, 1, 0, 0, 0, 0, 4, 0, 1, 42, 7]
    );
    let mut value = [0, 1, 1];
    store.registry.write(b"\x02j/0", &mut value).unwrap();
    assert_eq!(value, [0, 1, 1], "a sibling's held change leaves this variant's id alone");
    let mut changed = [0, 1, 0];
    store.registry.write(b"\x02h/0", &mut changed).unwrap();
    assert_eq!(changed, [0, 2, 0]);
    switch(&mut disk, &mut store);
    let old = skein_codec::schema::parse(HOLDER.as_bytes()).unwrap();
    let held = skein_codec::schema::parse(CHILD.as_bytes()).unwrap();
    let from = old.layout(old.root(b"Row", Some(b"changed")).unwrap(), &[&held]).unwrap().digest();
    let job = crate::Job {
        settles: false,
        counts: &[],
        name: "held-written",
        keys: holder.types[0].keys,
        converts: Some(crate::Converts { family: holder.name, value_type: 0, from }),
    };
    let set = PatchSet {
        families: Box::leak(Box::new([holder, KeptFamily { current: CHILD_WRITTEN, ..child }])),
        jobs: Box::leak(Box::new([job])),
        ..set
    };
    let mut disk = Disk::new(&disk.items());
    let mut store = self::store(&disk, set);
    let events = disk.open(&mut store);
    let Some(Event::Job { page, .. }) = events.first() else { panic!("{events:?}") };
    let layout = page.layout.as_ref().unwrap();
    assert_eq!(layout.root.as_ref(), "Row");
    assert_eq!(layout.variant.as_deref(), Some("changed"));
    assert_eq!(layout.text.as_ref(), HOLDER.as_bytes());
    assert_eq!(layout.held.as_ref(), &[Box::from(CHILD.as_bytes())]);
    assert_eq!(page.rows[0].value.as_ref(), &[0, 1, 0, 0, 0, 0, 3, 0, 1, 42]);
}
fn written_set() -> PatchSet {
    PatchSet { families: Box::leak(Box::new([KeptFamily { current: WRITTEN, ..FAMILY }])), ..SET }
}
#[test]
fn an_underived_layout_refuses_before_the_copy_with_its_census_count() {
    let mut disk = Disk::new(&base(false, true));
    let before = snapshot(&mut disk);
    let mut store = store(&disk, written_set());
    let events = disk.open(&mut store);
    let [
        Event::OpenRefused {
            why: OpenRefusal::Layout { family, value_type: 0, id: 2, values: Some(1), why: Unplanned::Underived(why) },
            ..
        },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(family.as_ref(), FAMILY.name);
    assert_eq!(why.why, skein_codec::layout::Why::NoDefault);
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    assert_eq!(snapshot(&mut disk), before);
    assert!(!disk.tree().iter().any(|(p, ..)| p.as_ref() == b"generations/2"));
}
#[test]
fn a_wrong_tasks_version_refuses_before_an_underived_layout_without_writing() {
    let mut disk = Disk::new(&base(false, true));
    let family = KeptFamily { current: KEPT, types: &[], ..FAMILY };
    let mut old = store(&disk, PatchSet { families: Box::leak(Box::new([family])), ..SET });
    assert_opened(&disk.open(&mut old), 1);
    let key = crate::keys::encode_own(&crate::keys::Own::Format, old.limits.store.key).expect("format key fits");
    let bytes = fixtures::get_kv(&mut old, &key).expect("format row exists");
    let mut format = crate::codec::component::Format::decode(&bytes).expect("format fixture");
    let wanted = format.tasks;
    format.tasks = wanted.checked_add(1).expect("next tasks version fits");
    raw(&mut disk, &mut old, vec![skein_kv::Op::Put { key, value: format.encode().expect("format fits") }]);
    let mut disk = Disk::new(&disk.items());
    let before = snapshot(&mut disk);
    let mut store = store(&disk, written_set());
    let events = disk.open(&mut store);
    let [Event::OpenRefused { why, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(
        *why,
        OpenRefusal::Format { family: crate::Family::Tasks, found: u32::from(format.tasks), wanted: u32::from(wanted) }
    );
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    assert_eq!(snapshot(&mut disk), before);
    assert!(!disk.tree().iter().any(|(p, ..)| p.as_ref() == b"generations/2"));
}
#[test]
fn mappings_resolve_keys_rehead_kept_plans_convert_and_preserve_unreadable_values() {
    let mut disk = Disk::new(&base(false, true));
    let mut store = store(&disk, SET);
    assert_opened(&disk.open(&mut store), 4);
    // The open has ended and shed its rows and texts; the default-adding
    // plan at the old id below must still convert without either of them.
    store.registry.assert_settled();
    let key = b"\x02a/0";
    let mut out = Buffer::with_capacity(64);
    for (key, input, expected) in [
        (key.as_slice(), vec![0, 3, 0, 42, 8], vec![0, 2, 0, 42, 8]),
        (key.as_slice(), vec![0, 2, 0, 42], vec![0, 2, 0, 42, 7]),
        (key.as_slice(), vec![0, 99, 0, 42], vec![0, 99, 0, 42]),
        (key.as_slice(), vec![0], vec![0]),
        (key.as_slice(), vec![0, 2], vec![0, 2]),
        (b"\x02b/0", vec![0, 3, 0, 42], vec![0, 3, 0, 42]),
    ] {
        let mut input = input;
        assert_eq!(store.registry.read(key, &mut input, &mut out, &mut store.layout_work.workspace), expected);
    }
    let mut value = [0, 2, 0, 42, 7];
    assert_eq!(store.registry.write(key, &mut value), Ok(()));
    assert_eq!(value, [0, 3, 0, 42, 7]);
    assert_eq!(store.registry.write(key, &mut value), Err(()));
    assert_eq!(store.registry.write(key, &mut [0]), Err(()));
    assert_eq!(store.registry.write(b"\x02b", &mut [0]), Ok(()));
    // A binary registered by a no-op layout change can use a different head.
    let kept_key = crate::keys::Own::Layout { family: FAMILY.name.into(), number: 1, id: 8 };
    let encoded = crate::keys::encode_own(&kept_key, store.limits.store.key).unwrap();
    let row = row(&mut store, 1, 2).unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    let encoded7 = crate::keys::encode_own(
        &crate::keys::Own::Layout { family: FAMILY.name.into(), number: 1, id: 7 },
        store.limits.store.key,
    )
    .unwrap();
    let value = writer.finish();
    raw(
        &mut disk,
        &mut store,
        vec![
            skein_kv::Op::Put { key: encoded7, value: value.clone() },
            skein_kv::Op::Put { key: encoded, value },
            skein_kv::Op::Put { key: b"\x02c/0".as_slice().into(), value: Box::from([0, 7, 2]) },
        ],
    );
    crate::registry::open(&mut store, false).unwrap();
    let mut value = [0, 7, 2];
    assert_eq!(store.registry.read(b"\x02c/0", &mut value, &mut out, &mut store.layout_work.workspace), &[0, 2, 2]);
}
pub(super) fn raw(disk: &mut Disk, store: &mut Store, mut ops: Vec<skein_kv::Op>) {
    let number = store.stores.store.last() + 1;
    let Write::Save(record) = fixtures::header(number) else { panic!() };
    let key = crate::keys::encode(&record.key(), store.limits.store.key).unwrap();
    let value = crate::codec::encode(&record, &store.codec_limits, &store.names).unwrap();
    ops.push(skein_kv::Op::Put { key, value });
    let mut above = Queue::uncapped();
    let mut below =
        crate::Below { store: Queue::uncapped(), secrets: Queue::uncapped(), transcripts: Queue::uncapped() };
    let env = crate::memory_env(store.limits.store);
    skein_kv::down(
        &mut store.stores.store,
        &env,
        skein_kv::Request::Commit { owner: Token::new(number), ops: ops.into_boxed_slice() },
        &mut above,
        &mut below.store,
    );
    disk.drive(&mut below, |io, below| {
        let Io::Store(arrived) = io else { panic!("public map write") };
        skein_kv::up(&mut store.stores.store, &env, arrived.event, &mut above, &mut below.store);
    });
    assert!({
        if let Some(skein_kv::Event::Committed { number: found, .. }) = above.pop() { found == number } else { false }
    });
    store.phase = Phase::Ready { next: number + 1, format_due: false };
}
fn fresh(disk: &mut Disk, set: PatchSet) -> Store {
    let mut store = store(disk, set);
    assert_opened(&disk.open(&mut store), 0);
    assert!({
        if let [Event::Committed { .. }] =
            disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)]))).as_ref()
        {
            true
        } else {
            false
        }
    });
    store
}
fn old_row(id: u16, family: &str, number: u16) -> skein_kv::Op {
    let schema = skein_codec::schema::parse(KEPT.as_bytes()).unwrap();
    let layout = schema.layout(schema.root(b"Growing", None).unwrap(), &[]).unwrap().digest();
    let value = crate::registry_v1::Layout::new(
        &crate::registry_v1::CEILINGS,
        crate::registry_v1::LayoutParts {
            root: b"Growing".as_slice().into(),
            variant: None,
            layout,
            text: hash(KEPT),
            held: skein_lib::List::with_capacity(0),
        },
    )
    .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    skein_kv::Op::Put {
        key: crate::keys::encode_own(&crate::keys::Own::Layout { family: family.into(), number, id }, 1024).unwrap(),
        value: writer.finish(),
    }
}
fn old_text() -> skein_kv::Op {
    let value = crate::registry_v1::Text::new(
        &crate::registry_v1::CEILINGS,
        crate::registry_v1::TextParts { text: KEPT.as_bytes().into() },
    )
    .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    skein_kv::Op::Put {
        key: crate::keys::encode_own(&crate::keys::Own::Text(hash(KEPT)), 1024).unwrap(),
        value: writer.finish(),
    }
}
#[test]
fn missing_text_and_undeclared_stored_families_are_layout_refusals_without_writes() {
    for (name, expected) in [(FAMILY.name, Unplanned::Text), ("absent-family", Unplanned::Undeclared)] {
        let mut disk = Disk::new(&[]);
        let mut store = fresh(&mut disk, SET);
        raw(
            &mut disk,
            &mut store,
            vec![
                old_row(9, name, 0),
                skein_kv::Op::Put { key: b"\x02a/0".as_slice().into(), value: Box::from([0, 9, 0, 42]) },
            ],
        );
        let items = disk.items();
        let mut disk = Disk::new(&items);
        let before = snapshot(&mut disk);
        let mut store = self::store(&disk, SET);
        let events = disk.open(&mut store);
        let [Event::OpenRefused { why: OpenRefusal::Layout { family, id: 9, value_type: 0, why, .. }, .. }] =
            events.as_ref()
        else {
            panic!("{events:?}")
        };
        assert_eq!(family.as_ref(), name);
        assert_eq!(*why, expected);
        assert_eq!(snapshot(&mut disk), before);
        assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    }
}
#[test]
fn a_shared_keyspace_or_folded_heads_cannot_clear_an_unplanned_layout() {
    for shared in [true, false] {
        let mut disk = Disk::new(&[]);
        let base = written_set().families[0];
        let family = if shared {
            KeptFamily {
                keys: &[
                    Keys { from: b"a/", to: b"a0", root: "Growing" },
                    Keys { from: b"c", to: b"d", root: "Stable" },
                ],
                types: Box::leak(Box::new([
                    ValueType { keys: &[KeyRange { from: b"a/", to: b"a0" }], ..base.types[0] },
                    base.types[1],
                ])),
                ..base
            }
        } else {
            base
        };
        let set = PatchSet { families: Box::leak(Box::new([family])), ..written_set() };
        let mut store = fresh(&mut disk, set);
        let mut ops = vec![old_row(9, FAMILY.name, 0), old_text()];
        for head in if shared { 2_u16..3 } else { 10_u16..19 } {
            let value: Box<[u8]> = [head.to_be_bytes().as_slice(), &[0, 42]].concat().into_boxed_slice();
            let key: Box<[u8]> = [b"\x02a/".as_slice(), &head.to_be_bytes()].concat().into_boxed_slice();
            ops.push(skein_kv::Op::Put { key, value });
        }
        raw(&mut disk, &mut store, ops);
        let mut disk = Disk::new(&disk.items());
        let mut store = self::store(&disk, set);
        let events = disk.open(&mut store);
        assert!(
            {
                if let [
                    Event::OpenRefused {
                        why: OpenRefusal::Layout { id: 9, values: None, why: Unplanned::Underived(_), .. },
                        ..
                    },
                ] = events.as_ref()
                {
                    true
                } else {
                    false
                }
            },
            "{events:?}"
        );
        assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    }
}
const NOOP: crate::Job = crate::Job {
    settles: false,
    counts: &[],
    name: "noop",
    keys: &[KeyRange { from: b"a", to: b"b" }],
    converts: None,
};
fn drive_empty(disk: &mut Disk, store: &mut Store, events: Box<[Event]>) -> Vec<Event> {
    let mut pending = alloc::collections::VecDeque::from(events.into_vec());
    let mut terminal = Vec::new();
    for _ in 0_u32..100 {
        let Some(event) = pending.pop_front() else { return terminal };
        if let Event::Job { owner, .. } = event {
            pending.extend(disk.request(
                store,
                Request::Job {
                    owner,
                    answer: Ok(crate::JobAnswer { counts: Box::new([]), writes: Box::new([]), through: None }),
                },
            ));
        } else {
            terminal.push(event);
        }
    }
    panic!("jobs bounded")
}
#[test]
fn zero_count_unplanned_layouts_wait_for_a_writing_open_to_retire_with_unshared_texts() {
    let mut disk = Disk::new(&[]);
    let mut store = fresh(&mut disk, written_set());
    raw(&mut disk, &mut store, vec![old_row(9, FAMILY.name, 0), old_text()]);
    let items = disk.items();
    let mut disk = Disk::new(&items);
    let mut store = self::store(&disk, written_set());
    let events = disk.open(&mut store);
    assert_opened(&events, 2);
    assert!({ if let Some(Event::Opened { pending: false, .. }) = events.first() { true } else { false } });
    assert!(row(&mut store, 0, 9).is_some());
    assert!(text(&mut store, hash(KEPT)).is_some());
    let mut disk = Disk::new(&disk.items());
    let mut store = self::store(&disk, PatchSet { jobs: &[NOOP], ..written_set() });
    let events = disk.open(&mut store);
    let events = drive_empty(&mut disk, &mut store, events);
    assert!(
        events.iter().any(|event| { if let Event::Opened { pending: true, .. } = event { true } else { false } }),
        "{events:?}"
    );
    assert!(row(&mut store, 0, 9).is_none());
    assert_eq!(text(&mut store, hash(KEPT)), None);
    assert!(row(&mut store, 0, 2).is_some());
}
fn history_set(current: &'static str) -> PatchSet {
    let family = KeptFamily { name: "jig-store-tasks", current, ..FAMILY };
    PatchSet { families: Box::leak(Box::new([family])), ..SET }
}
fn history_layout_key() -> Box<[u8]> {
    crate::keys::encode_own(&crate::keys::Own::Layout { family: "jig-store-tasks".into(), number: 0, id: 9 }, 1024)
        .unwrap()
}
#[test]
fn a_log_held_layout_without_map_values_survives_a_writing_open() {
    let set = history_set(CURRENT);
    let mut disk = Disk::new(&[]);
    let mut store = fresh(&mut disk, set);
    raw(&mut disk, &mut store, vec![old_row(9, "jig-store-tasks", 0), old_text()]);
    let mut disk = Disk::new(&disk.items());
    let mut store = self::store(&disk, PatchSet { jobs: &[NOOP], ..set });
    let events = disk.open(&mut store);
    let events = drive_empty(&mut disk, &mut store, events);
    assert!(
        events.iter().any(|event| if let Event::Opened { pending: true, .. } = event { true } else { false }),
        "{events:?}"
    );
    store.registry.assert_settled();
    assert!(fixtures::get_kv(&mut store, &history_layout_key()).is_some());
    assert!(text(&mut store, hash(KEPT)).is_some());
}
#[test]
fn a_log_held_layout_without_a_plan_refuses_even_without_map_values() {
    let set = history_set(WRITTEN);
    let mut disk = Disk::new(&[]);
    let mut store = fresh(&mut disk, set);
    raw(&mut disk, &mut store, vec![old_row(9, "jig-store-tasks", 0), old_text()]);
    let mut disk = Disk::new(&disk.items());
    let mut store = self::store(&disk, set);
    let events = disk.open(&mut store);
    let [
        Event::OpenRefused {
            why: OpenRefusal::Layout { family, value_type: 0, id: 9, values: None, why: Unplanned::Underived(_), .. },
            ..
        },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(family.as_ref(), "jig-store-tasks");
}

#[test]
fn a_seeded_store_refuses_a_pending_patch_of_a_registered_family() {
    let mut disk = Disk::new(&[]);
    let family = KeptFamily { texts: &[FIRST], current: FIRST, ..FAMILY };
    let store =
        fresh(&mut disk, PatchSet { families: Box::leak(Box::new([family])), level: Some(0), patches: &[], ..SET });
    assert_eq!(store.stores.store.last(), 1);
    let mut disk = Disk::new(&disk.items());
    let before = snapshot(&mut disk);
    let mut store = self::store(&disk, SET);
    let events = disk.open(&mut store);
    assert!(
        {
            if let [Event::OpenRefused { why: OpenRefusal::Set { owner: Owner::Application, patch: 1 }, .. }] =
                events.as_ref()
            {
                true
            } else {
                false
            }
        },
        "{events:?}"
    );
    assert_eq!(snapshot(&mut disk), before);
}
#[test]
fn written_jobs_filter_stored_heads_carry_the_layout_once_and_pass_pending_or_done() {
    let mut disk = Disk::new(&base(false, true));
    let mut store = store(&disk, SET);
    assert_opened(&disk.open(&mut store), 4);
    switch(&mut disk, &mut store);
    raw(
        &mut disk,
        &mut store,
        vec![
            skein_kv::Op::Put { key: b"\x02a/1".as_slice().into(), value: Box::from([0, 3, 0, 43, 8]) },
            skein_kv::Op::Put { key: b"\x02a/2".as_slice().into(), value: Box::from([0, 99, 0, 44]) },
        ],
    );
    let schema = skein_codec::schema::parse(KEPT.as_bytes()).unwrap();
    let from = schema.layout(schema.root(b"Growing", None).unwrap(), &[]).unwrap().digest();
    let job = crate::Job {
        settles: false,
        counts: &[],
        name: "written",
        keys: FAMILY.types[0].keys,
        converts: Some(crate::Converts { family: FAMILY.name, value_type: 0, from }),
    };
    let set = PatchSet { jobs: Box::leak(Box::new([job])), ..written_set() };
    let mut disk = Disk::new(&disk.items());
    let mut store = self::store(&disk, set);
    let mut events = disk.open(&mut store);
    let mut pages = 0_u32;
    for _ in 0_u32..10 {
        let Some(Event::Job { owner, page, .. }) = events.first() else { break };
        if pages == 0 {
            assert_eq!(page.rows.len(), 1);
            assert_eq!(page.rows[0].key.as_ref(), b"a/0");
            assert_eq!(page.rows[0].value.as_ref(), &[0, 2, 0, 42]);
            let layout = page.layout.as_ref().expect("first page layout");
            assert_eq!(layout.root.as_ref(), "Growing");
            assert_eq!(layout.variant, None);
            assert_eq!(layout.text.as_ref(), KEPT.as_bytes());
            assert!(layout.held.is_empty());
            assert!(!page.end);
        } else {
            assert!(page.layout.is_none());
            assert!(page.rows.is_empty());
            assert!(page.end);
        }
        pages += 1;
        // This fixture intentionally leaves the old value: even a done job is
        // the owner's promise to convert its from-layout, not a new plan.
        events = disk.request(
            &mut store,
            Request::Job {
                owner: *owner,
                answer: Ok(crate::JobAnswer { counts: Box::new([]), writes: Box::new([]), through: None }),
            },
        );
    }
    assert_eq!(pages, 2);
    assert!(
        events.iter().any(|event| { if let Event::Opened { pending: true, .. } = event { true } else { false } }),
        "{events:?}"
    );
    switch(&mut disk, &mut store);
    let last = store.stores.store.last();
    let mut disk = Disk::new(&disk.items());
    let mut store = self::store(&disk, set);
    assert_opened(&disk.open(&mut store), last);
}
