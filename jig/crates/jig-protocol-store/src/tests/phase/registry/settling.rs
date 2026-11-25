//! Serving rewrites and registry retirement (protocol/store/settling.md, 6).
use super::*;
use jig_test_layouts::layouts::FIRST;

const BINARY: u16 = 2;
const OLDER: u16 = 5;
const MANY: u8 = 37;
const PAGE_ROWS: u32 = 40;
const UNTOUCHED: &[u8] = b"x/root";

fn put(key: &[u8], value: &[u8]) -> Write {
    Write::Put { key: Box::from(key), value: Box::from(value) }
}

fn raw_put(key: &[u8], value: &[u8]) -> skein_kv::Op {
    skein_kv::Op::Put { key: crate::keys::application(key, fixtures::KEY_LIMIT).unwrap(), value: Box::from(value) }
}

fn commit(disk: &mut Disk, store: &mut Store, extra: impl IntoIterator<Item = Write>) {
    let number = store.stores.store.last() + 1;
    let writes = [fixtures::header(number)].into_iter().chain(extra).collect::<Vec<_>>().into_boxed_slice();
    let events = disk.request(store, fixtures::commit(number, writes));
    let [Event::Committed { number: found, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(*found, number);
}

fn read(disk: &mut Disk, store: &mut Store, key: &[u8]) -> Box<[u8]> {
    let events = disk.request(store, Request::Get { owner: Token::new(1), key: Get::Application(Box::from(key)) });
    let [Event::Got { row: Some(Row::Application { value, .. }), .. }] = events.as_ref() else { panic!("{events:?}") };
    value.clone()
}

fn heads(store: &Store) -> Vec<(Option<u16>, u64)> {
    store
        .stores
        .store
        .census()
        .keyspaces()
        .iter()
        .find(|space| space.prefix() == b"\x02a")
        .expect("fixture keyspace")
        .heads()
        .iter()
        .filter(|head| head.rows != 0)
        .map(|head| (head.head, head.rows))
        .collect()
}

fn seed(set: OwnerSet, ops: Vec<skein_kv::Op>, limits: &crate::Limits) -> Disk {
    let mut disk = Disk::new(&[]);
    let mut source = with_limits(&disk, set, limits);
    assert_opened(&disk.open(&mut source), 0);
    commit(&mut disk, &mut source, []);
    raw(&mut disk, &mut source, ops);
    Disk::new(&disk.items())
}

fn with_limits(disk: &Disk, set: OwnerSet, limits: &crate::Limits) -> Store {
    let mut opened = disk.store_with_limits(limits);
    opened.application.version = 0;
    opened.owner_sets(sets(set));
    opened
}

#[test]
fn settles_two_older_layouts_without_changing_reads_and_retires_their_rows_and_texts() {
    let limits = fixtures::store_limits();
    let ops = vec![
        layout_row(FIRST, 1, FAMILY.name, 0),
        layout_text(FIRST),
        old_row(OLDER, FAMILY.name, 0),
        old_text(),
        raw_put(b"a/0", &[0, 1, 42]),
        raw_put(b"a/1", &[0, 5, 0, 63]),
        raw_put(b"a/2", &[0, 1, 12]),
    ];
    let mut disk = seed(SET, ops, &limits);
    let mut serving = store(&disk, SET);
    assert_opened(&disk.open(&mut serving), 2);
    let keys = [b"a/0", b"a/1", b"a/2"];
    let before = keys.map(|key| read(&mut disk, &mut serving, key));
    for (value, n) in before.iter().zip([42, 63, 12]) {
        assert_eq!(value.as_ref(), &[0, 2, 0, n, 7]);
    }
    commit(&mut disk, &mut serving, [put(UNTOUCHED, b"before the mark")]);
    assert_eq!(serving.registry.walk_reads, 0);
    assert_eq!(heads(&serving), [(Some(1), 2), (Some(OLDER), 1)]);
    commit(&mut disk, &mut serving, [Write::Wires]);
    assert_eq!(serving.registry.walk_reads, 0);
    for bytes in [b"one", b"two", b"end"] {
        commit(&mut disk, &mut serving, [put(UNTOUCHED, bytes)]);
    }
    assert_eq!(serving.registry.walk_reads, 2, "three rows need two stored pages");
    assert_eq!(heads(&serving), [(Some(BINARY), 3)]);
    for (key, expected) in keys.iter().zip(&before) {
        assert_eq!(read(&mut disk, &mut serving, *key), *expected);
        assert_eq!(app(&mut serving, *key), Some(expected.clone()), "binary stored head and body");
    }
    assert!(row(&mut serving, 0, BINARY).is_some());
    for (old, schema) in [(1, FIRST), (OLDER, KEPT)] {
        assert!(row(&mut serving, 0, old).is_none());
        assert_eq!(text(&mut serving, hash(schema)), None);
    }
    assert!(text(&mut serving, hash(CURRENT)).is_some());

    let mut disk = Disk::new(&disk.items());
    let mut reopened = store(&disk, SET);
    assert_opened(&disk.open(&mut reopened), 7);
    commit(&mut disk, &mut reopened, [Write::Wires]);
    commit(&mut disk, &mut reopened, []);
    assert_eq!(reopened.registry.walk_reads, 0, "a settled store needs no walk");
}

fn many() -> (Disk, crate::Limits) {
    let mut limits = fixtures::store_limits();
    limits.store.ops = 48;
    limits.store.page.rows = PAGE_ROWS;
    let mut ops = vec![layout_row(CURRENT, 1, FAMILY.name, 0)]; // a keeps plan shares the binary's Text
    for n in 0..MANY {
        ops.push(raw_put(slot(n).as_ref(), &[0, 1, 0, n, 7]));
    }
    (seed(SET, ops, &limits), limits)
}

fn slot(n: u8) -> Box<[u8]> {
    Box::from([b'a', b'/', n])
}

fn root_frame() -> u32 {
    let skein_kv::Op::Put { key, value } = fixtures::header_put(fixtures::DEPLOYMENT, 0) else { panic!("header put") };
    skein_kv::FRAME_BYTES + skein_kv::put_bytes(key.len().try_into().unwrap(), value.len().try_into().unwrap()).unwrap()
}

#[test]
fn bounds_settled_puts_by_the_root_frame_and_operation_room_and_by_32() {
    let (mut disk, limits) = many();
    let mut serving = with_limits(&disk, SET, &limits);
    assert_opened(&disk.open(&mut serving), 2);
    commit(&mut disk, &mut serving, [Write::Wires]);
    let old = [(Some(1), u64::from(MANY))];
    // Exact root-only edges. Neither turns the extra work into a commit refusal.
    serving.limits.store.ops = 1;
    commit(&mut disk, &mut serving, []);
    assert_eq!(heads(&serving), old);
    serving.limits.store.ops = limits.store.ops;
    serving.limits.store.commit = root_frame();
    commit(&mut disk, &mut serving, []);
    assert_eq!(heads(&serving), old);
    serving.limits.store.commit = limits.store.commit;
    serving.limits.store.queued_bytes = u64::from(root_frame());
    commit(&mut disk, &mut serving, []);
    assert_eq!(heads(&serving), old);
    serving.limits.store.queued_bytes = limits.store.queued_bytes;
    serving.limits.store.commit = root_frame();
    // One put's bytes fit exactly, not two.
    let one = skein_kv::put_bytes(4, 5).unwrap();
    serving.limits.store.commit += one;
    commit(&mut disk, &mut serving, []);
    assert_eq!(heads(&serving), [(Some(1), u64::from(MANY - 1)), (Some(BINARY), 1)]);
    serving.limits.store.commit = limits.store.commit;
    serving.limits.store.ops = 2;
    commit(&mut disk, &mut serving, []);
    assert_eq!(heads(&serving), [(Some(1), u64::from(MANY - 2)), (Some(BINARY), 2)]);
    serving.limits.store.ops = limits.store.ops;
    commit(&mut disk, &mut serving, []);
    assert_eq!(heads(&serving), [(Some(1), u64::from(MANY - 34)), (Some(BINARY), 34)]);
    assert_eq!(serving.registry.walk_reads, 1);
    assert!(row(&mut serving, 0, 1).is_some(), "a not-yet-consumed page holds its layout");

    let mut disk = Disk::new(&disk.items());
    let mut restarted = with_limits(&disk, SET, &limits);
    assert_opened(&disk.open(&mut restarted), 9);
    assert_eq!(restarted.registry.walk_reads, 0);
    commit(&mut disk, &mut restarted, [Write::Wires]);
    let before = writes(&restarted);
    commit(&mut disk, &mut restarted, []);
    assert_eq!(restarted.registry.walk_reads, 1);
    assert_eq!(writes(&restarted) - before, u64::from(MANY - 34), "restart does not put binary rows again");
    assert_eq!(heads(&restarted), [(Some(BINARY), u64::from(MANY))]);
    commit(&mut disk, &mut restarted, []);
    assert!(row(&mut restarted, 0, 1).is_none());
    assert!(text(&mut restarted, hash(CURRENT)).is_some(), "the active Layout still names the shared Text");
}

fn writes(store: &Store) -> u64 {
    store.stores.store.census().keyspaces().iter().find(|space| space.prefix() == b"\x02a").unwrap().puts
}

#[test]
fn the_page_never_overwrites_a_root_put_or_resurrects_a_root_erase() {
    let turn = fixtures::turn(fixtures::LIVE, 1, 1, b"a file is synced before the map commit");
    let limits = fixtures::store_limits();
    for waiting in [false, true] {
        let mut ops = vec![old_row(OLDER, FAMILY.name, 0), old_text()];
        for n in 0..5 {
            ops.push(raw_put(slot(n).as_ref(), &[0, 5, 0, n]));
        }
        let place =
            crate::keys::encode_own(&crate::keys::Own::Transcript { task: turn.task }, fixtures::KEY_LIMIT).unwrap();
        ops.push(skein_kv::Op::Put {
            key: place,
            value: crate::codec::component::Transcript { length: 0, window: 0 }.encode().unwrap(),
        });
        let mut disk = seed(SET, ops, &limits);
        let mut serving = store(&disk, SET);
        assert_opened(&disk.open(&mut serving), 2);
        commit(&mut disk, &mut serving, [Write::Wires]);
        // Hold both K and the erase key in this page, behind the first put.
        serving.limits.store.page.rows = 8;
        serving.limits.store.ops = 2;
        commit(&mut disk, &mut serving, []);
        assert_eq!(serving.registry.walk_reads, 1);
        assert_eq!(heads(&serving), [(Some(BINARY), 1), (Some(OLDER), 4)]);
        let replacement = [0, 2, 0, 99, 12];
        let mut changes = vec![put(&slot(1), &replacement), Write::Remove { key: slot(2) }];
        if waiting {
            changes.push(fixtures::save(jig_domain_core_vocabulary::Record::Core(
                jig_domain_core_vocabulary::CoreRecord::Turn(turn.clone()),
            )));
        }
        let number = 5;
        if waiting {
            serving.limits.store.ops = 5; // n's places use all of the room; n+1 tries the held page.
            let mut above = Queue::uncapped();
            let mut below = fixtures::below();
            let writes = [fixtures::header(number)].into_iter().chain(changes).collect::<Vec<_>>();
            for (n, writes) in [(number, writes), (number + 1, vec![fixtures::header(number + 1)])] {
                crate::down(
                    &mut serving,
                    &fixtures::env(),
                    fixtures::commit(n, writes.into_boxed_slice()),
                    &mut above,
                    &mut below,
                );
            }
            assert!(!below.transcripts.is_empty() && below.store.is_empty());
            assert_eq!(serving.transcripts.queue.len(), 1, "a root change waits before skein-kv");
            disk.drive(&mut below, |io, below| crate::up(&mut serving, &fixtures::env(), io, &mut above, below));
            let events = fixtures::events(&mut above);
            let [Event::Committed { number: 5, .. }, Event::Committed { number: 6, .. }] = events.as_ref() else {
                panic!("{events:?}")
            };
        } else {
            serving.limits.store.ops = 3;
            commit(&mut disk, &mut serving, changes);
            serving.limits.store.ops = limits.store.ops;
            commit(&mut disk, &mut serving, []);
        }
        assert_eq!(serving.registry.walk_reads, 1, "both commits used the same stored page");
        assert_eq!(app(&mut serving, &slot(1)).as_deref(), Some(replacement.as_slice()));
        assert_eq!(app(&mut serving, &slot(2)), None);
        assert_eq!(heads(&serving), [(Some(BINARY), 4)]);
        commit(&mut disk, &mut serving, []);
        assert!(row(&mut serving, 0, OLDER).is_none());
    }
}

#[test]
fn refusal_keeps_the_stored_value_and_layout_but_does_not_stop_other_values() {
    let bad = [0, 5, 0]; // the old u16 is short, so its plan cannot add the default
    let limits = fixtures::store_limits();
    let ops = vec![old_row(OLDER, FAMILY.name, 0), old_text(), raw_put(b"a/0", &bad), raw_put(b"a/1", &[0, 5, 0, 9])];
    let mut disk = seed(SET, ops, &limits);
    let mut serving = store(&disk, SET);
    assert_opened(&disk.open(&mut serving), 2);
    commit(&mut disk, &mut serving, [Write::Wires]);
    assert_eq!(app(&mut serving, b"a/1").as_deref(), Some([0, 5, 0, 9].as_slice()), "idle after the mark");
    assert_eq!(serving.registry.walk_reads, 0);
    commit(&mut disk, &mut serving, []);
    commit(&mut disk, &mut serving, []);
    assert_eq!(app(&mut serving, b"a/0").as_deref(), Some(bad.as_slice()));
    assert_eq!(app(&mut serving, b"a/1").as_deref(), Some([0, 2, 0, 9, 7].as_slice()));
    assert!(row(&mut serving, 0, OLDER).is_some());
    assert!(text(&mut serving, hash(KEPT)).is_some());
}

#[test]
fn log_held_layouts_do_not_join_the_walk() {
    let mut disk = Disk::new(&[]);
    let mut source = jig_store(&disk, crate::JIG_STORE_SET);
    assert_opened(&disk.open(&mut source), 0);
    commit(&mut disk, &mut source, []);
    // An older copy of a log-held row can still be in the map. Give it the
    // same layout at a different id so it could otherwise be reheaded.
    let layout_key = |id| {
        crate::keys::encode_own(
            &crate::keys::Own::Layout { family: "jig-store-tasks".into(), number: 9, id },
            fixtures::KEY_LIMIT,
        )
        .unwrap()
    };
    let layout = fixtures::get_kv(&mut source, &layout_key(13)).unwrap();
    let record = fixtures::history(fixtures::LIVE, 1);
    let key = crate::keys::encode(&record.key(), fixtures::KEY_LIMIT).unwrap();
    assert_eq!(crate::keys::logged(&key), Some(fixtures::LIVE));
    let mut old = crate::codec::encode(&record, &source.codec_limits, &source.names).unwrap();
    old[..2].copy_from_slice(&1_u16.to_be_bytes());
    raw(
        &mut disk,
        &mut source,
        vec![
            skein_kv::Op::Put { key: layout_key(1), value: layout },
            skein_kv::Op::Put { key: key.clone(), value: old.clone() },
        ],
    );
    let mut disk = Disk::new(&disk.items());
    let mut serving = jig_store(&disk, crate::JIG_STORE_SET);
    assert_opened(&disk.open(&mut serving), 2);
    commit(&mut disk, &mut serving, [Write::Wires]);
    commit(&mut disk, &mut serving, []);
    commit(&mut disk, &mut serving, []);
    assert_eq!(serving.registry.walk_reads, 0);
    assert_eq!(fixtures::get_kv(&mut serving, &key), Some(old));
    assert!(fixtures::get_kv(&mut serving, &layout_key(1)).is_some());
}

#[test]
fn written_job_layouts_do_not_join_the_walk() {
    let schema = skein_codec::schema::parse(KEPT.as_bytes()).unwrap();
    let from = schema.layout(schema.root(b"Growing", None).unwrap(), &[]).unwrap().digest();
    let job = crate::Job {
        name: "converts-one",
        keys: FAMILY.types[0].keys,
        converts: Some(crate::Converts { family: FAMILY.name, value_type: 0, from: crate::Layouts::One(from) }),
        counts: &[],
    };
    let old = [0, 5, 0, 42];
    let ops = vec![old_row(OLDER, FAMILY.name, 0), old_text(), raw_put(b"a/0", &old)];
    let mut disk = seed(SET, ops, &fixtures::store_limits());
    let set = OwnerSet { jobs: Box::leak(Box::new([job])), ..SET };
    let mut serving = store(&disk, set);
    let events = disk.open(&mut serving);
    // The job's fixture deliberately leaves its old row as stored.
    let events = drive_empty(&mut disk, &mut serving, events);
    let [Event::Ran { .. }, Event::Opened { pending: true, .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("{events:?}")
    };
    switch(&mut disk, &mut serving);
    commit(&mut disk, &mut serving, [Write::Wires]);
    commit(&mut disk, &mut serving, []);
    commit(&mut disk, &mut serving, []);
    assert_eq!(serving.registry.walk_reads, 0);
    assert_eq!(app(&mut serving, b"a/0").as_deref(), Some(old.as_slice()));
    assert!(text(&mut serving, hash(KEPT)).is_some());
}
