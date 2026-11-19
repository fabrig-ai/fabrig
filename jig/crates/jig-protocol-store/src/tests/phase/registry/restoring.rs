//! A new build restores the source's format and registry (restoring.md, 4.3 and 8).
use super::*;
use crate::codec::component::Format;

const BEFORE: OwnerSet = OwnerSet { families: &[KeptFamily { current: KEPT, ..FAMILY }], ..SET };
const AFTER: OwnerSet = OwnerSet { jobs: &[crate::Job { keys: FAMILY.types[0].keys, ..UPGRADE_JOB }], ..SET };
const OWNER: Token = Token::new(73);
// Header, then a record before C; registry, job and seal; two serving commits.
const SOURCE: u64 = 2;
const UPGRADED: u64 = 5;
const FIRST_SERVING: u64 = 6;
const LATER: u64 = 7;
const LATER_KEY: &[u8] = b"a/later";
const LATER_VALUE: &[u8] = &[0, 2, 0, 43, 7]; // CURRENT's family head and Growing fields.

fn format(store: &mut Store) -> Format {
    let key = crate::keys::encode_own(&crate::keys::Own::Format, fixtures::KEY_LIMIT).unwrap();
    Format::decode(&fixtures::get_kv(store, &key).unwrap()).unwrap()
}
fn registry(store: &mut Store) -> Vec<skein_kv::Row> {
    // Layouts, texts and completed jobs are the registry (records.md, 3).
    let ranges = [skein_kv::Range { start: Box::from([0, 6]), end: Some(Box::from([0, 9])) }];
    let mut rows = Vec::new();
    let mut after = None;
    loop {
        let page =
            crate::pages::reader::walk_stored(store, &ranges, after.as_ref(), store.limits.store.page.rows).unwrap();
        rows.extend(page.rows.into_iter().map(|row| row.row));
        after = page.next;
        if after.is_none() {
            return rows;
        }
    }
}
fn commit(disk: &mut Disk, store: &mut Store, number: u64, writes: Box<[Write]>) {
    let events = disk.request(store, fixtures::commit(number, writes));
    let [Event::Committed { number: found, .. }] = events.as_ref() else { panic!("commit: {events:?}") };
    assert_eq!(*found, number);
}
fn recover_map(disk: &mut Disk) -> Store {
    // Read the restored rows without a build's open planning or registering layouts.
    let arrived = disk.file(
        0,
        file::Request::OpenDirectory {
            owner: OWNER,
            root: disk.root().fd,
            name: Box::from(b"generations/3/store".as_slice()),
            no_follow: true,
        },
    );
    let root = crate::directory::opened(arrived).unwrap();
    let mut store = disk.store_with_chunk(CHUNK);
    let env = fixtures::env().with(store.limits.store);
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    skein_kv::down(
        &mut store.stores.store,
        &env,
        skein_kv::Request::Open { owner: OWNER, root: root.fd },
        &mut above,
        &mut below.store,
    );
    disk.drive(&mut below, |io, below| {
        let Io::Store(arrived) = io else { panic!("main map lane") };
        skein_kv::up(&mut store.stores.store, &env, arrived.event, &mut above, &mut below.store);
    });
    let events: Vec<_> = core::iter::from_fn(|| above.pop()).collect();
    let [skein_kv::Event::Opened { owner: OWNER, last: SOURCE }] = events.as_slice() else {
        panic!("restored map: {events:?}")
    };
    store
}

#[test]
fn a_new_build_restores_the_previous_builds_registry_and_state() {
    let mut disk = Disk::new(&[]);
    let mut previous = fresh(&mut disk, BEFORE);
    commit(
        &mut disk,
        &mut previous,
        SOURCE,
        Box::new([fixtures::header(SOURCE), Write::Put { key: Box::from(KEY), value: Box::from(VALUE) }]),
    );
    let source_format = format(&mut previous);
    let source_registry = registry(&mut previous);
    let events = disk.request(&mut previous, Request::Checkpoint { owner: OWNER, draw: [1; 16] });
    let [Event::Checkpointed { checkpoint, .. }] = events.as_ref() else { panic!("source: {events:?}") };
    let source = checkpoint.name;

    let mut disk = Disk::new(&disk.items());
    let mut newer = store(&disk, AFTER);
    let events = disk.open(&mut newer);
    let [Event::Job { owner, page, .. }] = events.as_ref() else { panic!("pending job: {events:?}") };
    assert!(page.end && page.rows.is_empty());
    let events = disk.request(
        &mut newer,
        Request::Job {
            owner: *owner,
            answer: Ok(crate::JobAnswer { writes: Box::new([]), counts: Box::new([]), through: None }),
        },
    );
    let [Event::Ran { .. }, Event::Opened { pending: true, last: UPGRADED, .. }, Event::Census { .. }] =
        events.as_ref()
    else {
        panic!("upgraded: {events:?}")
    };
    switch(&mut disk, &mut newer);
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
    assert!(row(&mut newer, 0, 3).is_some());
    commit(&mut disk, &mut newer, FIRST_SERVING, Box::new([fixtures::header(FIRST_SERVING), Write::Wires]));
    commit(
        &mut disk,
        &mut newer,
        LATER,
        Box::new([fixtures::header(LATER), Write::Put { key: Box::from(LATER_KEY), value: Box::from(LATER_VALUE) }]),
    );

    // B can read N's values, but must register its retired layout again.
    let items = disk.items();
    let mut contrast = Disk::new(&items);
    let mut previous = store(&contrast, BEFORE);
    let events = contrast.open(&mut previous);
    let [Event::Opened { pending: true, .. }, Event::Census { .. }] = events.as_ref() else {
        panic!("B before restore: {events:?}")
    };

    let mut disk = Disk::new(&items);
    let mut restorer = store(&disk, AFTER);
    let events = disk.restore(&mut restorer, OWNER, source, [2; 16]);
    let [Event::Restored { restored, .. }] = events.as_ref() else { panic!("restore: {events:?}") };
    assert_eq!((restored.name, restored.generation), (source, 3));
    assert_eq!(disk.bytes(b"active").as_ref(), b"3\n");
    let mut map = recover_map(&mut disk);
    assert_eq!(format(&mut map), source_format);
    assert_eq!(registry(&mut map), source_registry);
    assert!(row(&mut map, 0, 3).is_none());

    let mut disk = Disk::new(&disk.items());
    let mut previous = store(&disk, BEFORE);
    let events = disk.open(&mut previous);
    let [Event::Opened { last: SOURCE, pending: false, .. }, Event::Census { .. }] = events.as_ref() else {
        panic!("B after restore: {events:?}")
    };
    assert_eq!(disk.bytes(b"active").as_ref(), b"3\n");
    assert!(!disk.tree().iter().any(|(path, ..)| path.starts_with(b"generations/4")));
    read_row(&mut disk, &mut previous);
    let events =
        disk.request(&mut previous, Request::Get { owner: OWNER, key: Get::Application(Box::from(LATER_KEY)) });
    let [Event::Got { row: None, .. }] = events.as_ref() else { panic!("record after C: {events:?}") };
}
