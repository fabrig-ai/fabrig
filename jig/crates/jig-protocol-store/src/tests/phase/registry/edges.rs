//! Serving edges and the rehearsal over the registered fixture (protocol/store/layouts.md, 4).
use super::*;
use crate::{Load, Restore};
use skein_codec::{schema, value::Value};

fn opened() -> (Disk, Store) {
    let mut disk = Disk::new(&base(false, true));
    let mut store = store(&disk, SET);
    assert_opened(&disk.open(&mut store), 4);
    switch(&mut disk, &mut store);
    (disk, store)
}
fn assert_committed(events: &[Event], number: u64) {
    let [Event::Committed { number: found, .. }] = events else { panic!("{events:?}") };
    assert_eq!(*found, number);
}
fn range() -> Box<[skein_kv::Range]> {
    Box::new([skein_kv::Range { start: b"a".as_slice().into(), end: Some(b"d".as_slice().into()) }])
}
fn decoded(root: &[u8], bytes: Box<[u8]>) {
    let schema = schema::parse(CURRENT.as_bytes()).unwrap();
    Value::with_room(16).decode(&schema, schema.find(root).unwrap(), bytes).unwrap();
}
fn assert_rows(rows: &[&Row]) {
    let [Row::Application { key: a, value: growing }, Row::Application { key: c, value: stable }] = rows else {
        panic!("two application rows: {rows:?}")
    };
    assert_eq!(a.as_ref(), b"a/0");
    assert_eq!(growing.as_ref(), &[0, 2, 0, 42, 7]);
    decoded(b"Growing", growing.clone());
    assert_eq!(c.as_ref(), b"c/0");
    assert_eq!(stable.as_ref(), &[0, 2, 1]);
    decoded(b"Stable", stable.clone());
}
fn alias(store: &mut Store, number: u16, from: u16, to: u16) -> skein_kv::Op {
    let value = row(store, number, from).unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    skein_kv::Op::Put {
        key: crate::keys::encode_own(
            &crate::keys::Own::Layout { family: FAMILY.name.into(), number, id: to },
            store.limits.store.key,
        )
        .unwrap(),
        value: writer.finish(),
    }
}

#[test]
fn new_value_types_register_at_the_seed_and_read_back() {
    const KEPT_NEW: &str = "family fixture-layouts 2\nenum Value { stable }\nversioned record Row { value: Value }\n";
    const CURRENT_NEW: &str = "family fixture-layouts 2\nrecord Payload { n: u8 }\nenum Value { stable, added: Payload }\nversioned record Row { value: Value }\nversioned record Added { n: u8 }\n";
    const NEW_FAMILY: KeptFamily = KeptFamily {
        texts: &[KEPT_NEW],
        current: CURRENT_NEW,
        keys: &[Keys { from: b"a", to: b"c", root: "Row" }, Keys { from: b"c", to: b"d", root: "Added" }],
        // New types occupy census-distinct prefixes so the seed can prove their ranges empty.
        types: &[
            ValueType { number: 0, root: "Row", variant: Some("stable"), keys: &[KeyRange { from: b"a", to: b"b" }] },
            ValueType { number: 1, root: "Row", variant: Some("added"), keys: &[KeyRange { from: b"b", to: b"c" }] },
            ValueType { number: 2, root: "Added", variant: None, keys: &[KeyRange { from: b"c", to: b"d" }] },
        ],
        ..FAMILY
    };
    const NEW_SET: PatchSet = PatchSet { families: &[NEW_FAMILY], patches: &[], ..SET };
    let mut disk = Disk::new(&[]);
    let mut before = fresh(
        &mut disk,
        PatchSet { families: &[KeptFamily { current: KEPT_NEW, types: &[], ..NEW_FAMILY }], ..NEW_SET },
    );
    raw(
        &mut disk,
        &mut before,
        vec![skein_kv::Op::Put { key: b"\x02a/2".as_slice().into(), value: Box::from([0, 2, 0]) }],
    );
    let mut disk = Disk::new(&disk.items());
    let current = schema::parse(CURRENT_NEW.as_bytes()).unwrap();
    let mut store = store(&disk, NEW_SET);
    let events = disk.open(&mut store);
    let [Event::Opened { pending: true, .. }, Event::Census { .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(*row(&mut store, 0, 2).unwrap().text(), hash(KEPT_NEW));
    for number in [1, 2] {
        let ty = NEW_FAMILY.types.iter().find(|ty| ty.number == number).unwrap();
        let registered = row(&mut store, number, 2).unwrap();
        assert_eq!(*registered.text(), hash(CURRENT_NEW), "only the binary's row, not a seed row");
        assert_eq!(registered.root(), ty.root.as_bytes());
        assert_eq!(registered.variant().as_deref(), ty.variant.map(str::as_bytes));
        let root = current.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).unwrap();
        assert_eq!(*registered.layout(), current.layout(root, &[]).unwrap().digest());
        assert!(row(&mut store, number, 3).is_none(), "no seed row consumed the family's version id");
    }
    assert_eq!(app(&mut store, b"a/2").unwrap().as_ref(), &[0, 2, 0]);
    switch(&mut disk, &mut store);
    let number = store.stores.store.last() + 1;
    let writes = Box::new([
        fixtures::header(number),
        Write::Put { key: b"b/0".as_slice().into(), value: Box::from([0, 2, 1, 42]) },
        Write::Put { key: b"c/0".as_slice().into(), value: Box::from([0, 2, 43]) },
    ]);
    assert_committed(&disk.request(&mut store, fixtures::commit(number, writes)), number);
    for (key, root, expected) in
        [(b"b/0", b"Row".as_slice(), [0, 2, 1, 42].as_slice()), (b"c/0", b"Added".as_slice(), [0, 2, 43].as_slice())]
    {
        let get = disk
            .request(&mut store, Request::Get { owner: Token::new(2), key: Get::Application(key.as_slice().into()) });
        let [Event::Got { row: Some(Row::Application { value, .. }), .. }] = get.as_ref() else { panic!("{get:?}") };
        assert_eq!(value.as_ref(), expected);
        Value::with_room(16).decode(&current, current.find(root).unwrap(), value.clone()).unwrap();
    }
}

#[test]
fn loads_gets_and_restore_pages_deliver_the_binary_layout_and_preserve_unknown_heads() {
    let (mut disk, mut store) = opened();
    let stable = alias(&mut store, 1, 2, 8);
    raw(
        &mut disk,
        &mut store,
        vec![stable, skein_kv::Op::Put { key: b"\x02c/0".as_slice().into(), value: Box::from([0, 8, 1]) }],
    );
    crate::registry::open(&mut store, false).unwrap();
    let load = disk.request(
        &mut store,
        Request::Load { owner: Token::new(2), range: Load::Application(range()), after: None, rows: 4, bytes: 100 },
    );
    let [Event::Loaded { rows, next: None, .. }] = load.as_ref() else { panic!("{load:?}") };
    assert_rows(&rows.iter().collect::<Vec<_>>());
    for (key, root, expected) in [
        (b"a/0", b"Growing".as_slice(), &[0, 2, 0, 42, 7].as_slice()),
        (b"c/0", b"Stable".as_slice(), &[0, 2, 1].as_slice()),
    ] {
        let get = disk
            .request(&mut store, Request::Get { owner: Token::new(3), key: Get::Application(key.as_slice().into()) });
        let [Event::Got { row: Some(Row::Application { value, .. }), .. }] = get.as_ref() else { panic!("{get:?}") };
        assert_eq!(value.as_ref(), *expected);
        decoded(root, value.clone());
    }
    let restore = disk.request(
        &mut store,
        Request::Restore {
            owner: Token::new(4),
            range: Restore::Application(range()),
            after: None,
            rows: 4,
            bytes: 100,
        },
    );
    let [Event::Page { items, next: None, .. }] = restore.as_ref() else { panic!("{restore:?}") };
    let rows: Vec<_> = items
        .iter()
        .map(|item| {
            let crate::restore::Item::Row(row) = item else { panic!("row") };
            row
        })
        .collect();
    assert_rows(&rows);
    assert_eq!(app(&mut store, b"a/0").unwrap().as_ref(), &[0, 2, 0, 42], "the store stays old");
    let mut stable: Box<[u8]> = Box::from([0, 8, 1].as_slice());
    let address = stable.as_ptr();
    crate::registry::read(&mut store, b"\x02c/0", &mut stable);
    assert_eq!(address, stable.as_ptr(), "a head-only read keeps its allocation");
    raw(
        &mut disk,
        &mut store,
        vec![skein_kv::Op::Put { key: b"\x02a/0".as_slice().into(), value: Box::from([0, 99, 0, 42]) }],
    );
    let get = disk
        .request(&mut store, Request::Get { owner: Token::new(5), key: Get::Application(b"a/0".as_slice().into()) });
    let [Event::Got { row: Some(Row::Application { value, .. }), .. }] = get.as_ref() else { panic!("{get:?}") };
    assert_eq!(value.as_ref(), &[0, 99, 0, 42]);
    let schema = schema::parse(CURRENT.as_bytes()).unwrap();
    let failure = Value::with_room(16).decode(&schema, schema.find(b"Growing").unwrap(), value.clone()).unwrap_err();
    let skein_codec::value::Refusal::Problem(problem) = failure else { panic!("malformed head") };
    assert_eq!(problem.reason, skein_codec::Reason::Version);
}

#[test]
fn root_puts_store_the_binary_id_count_it_and_refuse_a_nonversion_head() {
    let (mut disk, mut store) = opened();
    let number = store.stores.store.last() + 1;
    let events = disk.request(
        &mut store,
        fixtures::commit(
            number,
            Box::new([
                fixtures::header(number),
                Write::Put { key: b"a/1".as_slice().into(), value: Box::from([0, 2, 0, 43, 9]) },
            ]),
        ),
    );
    assert_committed(&events, number);
    assert_eq!(app(&mut store, b"a/1").unwrap().as_ref(), &[0, 3, 0, 43, 9]);
    let read = crate::open::get(&mut store, Token::new(0), b"\x02a/1".as_slice().into()).unwrap().unwrap();
    assert_eq!(read.as_ref(), &[0, 2, 0, 43, 9]);
    decoded(b"Growing", read);
    let space = store.stores.store.census().keyspaces().iter().find(|space| space.prefix() == b"\x02a").unwrap();
    assert_eq!(
        space.heads().iter().map(|head| (head.head, head.rows)).collect::<Vec<_>>(),
        [(Some(2), 1), (Some(3), 1)]
    );
    let events = disk.request(
        &mut store,
        fixtures::commit(
            number + 1,
            Box::new([
                fixtures::header(number + 1),
                Write::Put { key: b"a/2".as_slice().into(), value: Box::from([0, 3, 0, 44, 9]) },
            ]),
        ),
    );
    let [Event::Failed { number: failed, why: crate::CommitFailure::Check(crate::CommitCheck::Encoding) }] =
        events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(*failed, number + 1);
    assert_eq!(store.stores.store.last(), number);
    assert!(app(&mut store, b"a/2").is_none());
}

#[test]
fn a_copy_reads_the_sources_layout_and_stores_the_destinations_id() {
    use jig_domain_core_vocabulary::Key;
    use jig_domain_tasks::Key as TaskKey;
    // Route the fixture's growing type through two jig addresses so Write::Copy
    // crosses types. A kept alias gives the destination a distinct binary id.
    const FAMILY_AT_JIG: KeptFamily = KeptFamily {
        keys: &[
            Keys { from: &[1, 1, 3], to: &[1, 1, 5], root: "Growing" },
            Keys { from: b"\x02a", to: b"\x02b", root: "Growing" },
            Keys { from: b"\x02c", to: b"\x02d", root: "Stable" },
        ],
        types: &[
            ValueType {
                number: 0,
                root: "Growing",
                variant: None,
                keys: &[KeyRange { from: &[1, 1, 3], to: &[1, 1, 4] }, KeyRange { from: b"\x02a", to: b"\x02b" }],
            },
            ValueType {
                number: 2,
                root: "Growing",
                variant: None,
                keys: &[KeyRange { from: &[1, 1, 4], to: &[1, 1, 5] }],
            },
            ValueType { number: 1, root: "Stable", variant: None, keys: &[KeyRange { from: b"\x02c", to: b"\x02d" }] },
        ],
        ..FAMILY
    };
    let (mut disk, mut store) = opened();
    store.patch_sets(sets(PatchSet { owner: Owner::Jig, families: &[FAMILY_AT_JIG], ..SET }));
    crate::registry::open(&mut store, false).unwrap();
    for _ in 0_u32..10 {
        let ops = store.registry.batch(&store.limits).unwrap();
        if ops.is_empty() {
            break;
        }
        raw(&mut disk, &mut store, ops.into_vec());
    }
    let destination = alias(&mut store, 2, 2, 9);
    raw(&mut disk, &mut store, vec![destination]);
    crate::registry::open(&mut store, false).unwrap();
    let from = Key::Tasks(TaskKey::Writer(1));
    let to = Key::Tasks(TaskKey::Pool(1));
    let source = crate::keys::encode(&from, store.limits.store.key).unwrap();
    raw(&mut disk, &mut store, vec![skein_kv::Op::Put { key: source, value: Box::from([0, 2, 0, 42]) }]);
    let number = store.stores.store.last() + 1;
    let events = disk.request(
        &mut store,
        fixtures::commit(number, Box::new([fixtures::header(number), Write::Copy { from, to: to.clone() }])),
    );
    assert_committed(&events, number);
    let key = crate::keys::encode(&to, store.limits.store.key).unwrap();
    assert_eq!(fixtures::get_kv(&mut store, &key).unwrap().as_ref(), &[0, 9, 0, 42, 7]);
    assert_eq!(app(&mut store, b"a/0").unwrap().as_ref(), &[0, 2, 0, 42]);
}

#[test]
fn ended_joins_restore_the_family_version_before_its_decoder() {
    let mut disk = Disk::new(&[]);
    let expected = fixtures::ended_task(21);
    let limits = fixtures::store_limits().codec_view().unwrap();
    let mut bytes = crate::codec::encode(&expected, &limits, &[]).unwrap();
    let version = crate::registry::head(&bytes).unwrap();
    let text: &'static str = Box::leak(
        format!("family fixture-layouts {version}\nversioned record Growing {{ n: u16 }}\n").into_boxed_str(),
    );
    let family = KeptFamily {
        texts: Box::leak(Box::new([text])),
        current: text,
        keys: &[Keys { from: &[1, 1, 6], to: &[1, 1, 7], root: "Growing" }],
        types: &[ValueType {
            number: 0,
            root: "Growing",
            variant: None,
            keys: &[KeyRange { from: &[1, 1, 6], to: &[1, 1, 7] }],
        }],
        ..FAMILY
    };
    let set = PatchSet { owner: Owner::Jig, families: Box::leak(Box::new([family])), patches: &[], ..SET };
    let mut store = fresh(&mut disk, set);
    let layout = alias(&mut store, 0, version, 99);
    bytes[..2].copy_from_slice(&99_u16.to_be_bytes());
    let key = crate::keys::encode(&expected.key(), store.limits.store.key).unwrap();
    let size = u64::try_from(bytes.len()).unwrap();
    raw(&mut disk, &mut store, vec![layout, skein_kv::Op::Put { key: key.clone(), value: bytes }]);
    crate::registry::open(&mut store, false).unwrap();
    let source = crate::restore::site(&store, b"\x01\x01\x0d");
    assert_eq!(crate::restore::reader::ended(&mut store, &expected.key(), &source).unwrap(), Some((expected, size)));
    assert_eq!(crate::registry::head(&fixtures::get_kv(&mut store, &key).unwrap()), Some(99));
}

#[test]
fn secrets_pages_and_component_rows_are_not_serving_layout_values() {
    let (mut disk, mut store) = opened();
    // The same key in the secrets map must not reach the main map's registry.
    store.stores.secrets = fixtures::stores().secrets;
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.secrets,
        &crate::memory_env(store.limits.secrets),
        skein_kv::Request::Commit {
            owner: Token::new(0),
            ops: Box::new([skein_kv::Op::Put { key: b"\x02a/0".as_slice().into(), value: Box::from([0, 3, 0, 42]) }]),
        },
        &mut above,
        &mut below,
    );
    let Some(skein_kv::Event::Committed { .. }) = above.pop() else { panic!("secret put") };
    assert!(below.is_empty());
    let page = crate::pages::reader::load_map(
        &mut store,
        KeptMap::Secrets,
        skein_kv::Range { start: b"\x02a".as_slice().into(), end: Some(b"\x02b".as_slice().into()) },
    )
    .unwrap();
    assert_eq!(page.rows[0].value.as_ref(), &[0, 3, 0, 42]);
    let installation = crate::keys::installation(store.limits.store.key).unwrap();
    let before = fixtures::get_kv(&mut store, &installation).unwrap();
    let read = crate::open::get(&mut store, Token::new(0), installation).unwrap().unwrap();
    assert_eq!(read, before);
    let number = store.stores.store.last() + 1;
    let events = disk.request(&mut store, fixtures::commit(number, Box::new([fixtures::header(number), Write::Wires])));
    assert_committed(&events, number);
    let key = crate::keys::encode_own(&crate::keys::Own::Wires, store.limits.store.key).unwrap();
    let expected = crate::codec::component::wires(store.wires);
    assert_eq!(fixtures::get_kv(&mut store, &key), Some(expected.clone()));
    assert_eq!(crate::open::get(&mut store, Token::new(0), key).unwrap(), Some(expected));
}

#[test]
fn rehearsal_decodes_seeded_records_at_their_layout_text_and_names_an_unknown_head() {
    let (mut disk, mut store) = opened();
    let number = store.stores.store.last() + 1;
    let events = disk.request(
        &mut store,
        fixtures::commit(
            number,
            Box::new([
                fixtures::header(number),
                Write::Put { key: b"a/1".as_slice().into(), value: Box::from([0, 2, 0, 43, 9]) },
            ]),
        ),
    );
    assert_committed(&events, number);
    let items = disk.items();
    let mut disk = Disk::new(&items);
    let before = disk.contents();
    let mut store = self::store(&disk, SET);
    let scratch = disk.scratch_root();
    let directory = disk.root();
    let events = disk.request(
        &mut store,
        Request::Rehearse { owner: Token::new(1), directory, scratch, installation: fixtures::INSTALLATION },
    );
    assert!(
        events.iter().any(|event| if let Event::Decoded { records: 3, .. } = event { true } else { false }),
        "{events:?}"
    );
    assert!(events.iter().any(|event| if let Event::Opened { .. } = event { true } else { false }), "{events:?}");
    assert_eq!(disk.contents(), before);
    let mut disk = Disk::new(&items);
    let mut store = self::store(&disk, SET);
    assert_opened(&disk.open(&mut store), number);
    raw(
        &mut disk,
        &mut store,
        vec![skein_kv::Op::Put { key: b"\x02a/0".as_slice().into(), value: Box::from([0, 99, 0, 42]) }],
    );
    let mut disk = Disk::new(&disk.items());
    let before = disk.contents();
    let mut store = self::store(&disk, SET);
    let scratch = disk.scratch_root();
    let directory = disk.root();
    let events = disk.request(
        &mut store,
        Request::Rehearse { owner: Token::new(1), directory, scratch, installation: fixtures::INSTALLATION },
    );
    let [
        Event::Rehearsing { .. },
        Event::Undecodable { family, version: 99, at: crate::Undecoded::Record { map: KeptMap::Store, key }, .. },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(family.as_ref(), FAMILY.name);
    assert_eq!(key.as_ref(), b"\x02a/0");
    assert_eq!(disk.contents(), before);
}
