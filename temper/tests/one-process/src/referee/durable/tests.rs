use super::*;
use crate::fixtures::{self, durable as fixture};
use fixture::{POWER_LOSS_SEED, READ, SYNC, USER, WRITE};
use layouts::tests::{
    another_layout, assert_empty_preparation, assert_same_decoder, attempt, registered_attempt, registry_ops,
};
use skein_fake_machine::{How, Item, Machine};

// The KV frame and snapshot headers both store their sequence number after the magic and version.
const SEQUENCE: std::ops::Range<usize> = 6..14;

fn number_frame(frame: &mut [u8], number: u64) {
    frame[SEQUENCE].copy_from_slice(&number.to_be_bytes());
    let checksum = frame.len() - 4;
    let crc = transcripts::frame::crc32c(&frame[..checksum]);
    frame[checksum..].copy_from_slice(&crc.to_be_bytes());
}

#[test]
fn store_reuses_decoders_until_snapshot_or_generation_switch() {
    const NEXT_REVISION: u64 = 3;
    let (registered, key, value) = registered_attempt();
    let mut registry = registry_ops(&registered);
    let text_key = registry
        .iter()
        .find_map(|op| match op {
            skein_kv::Op::Put { key, .. } if matches!(keys::decode_own(key), Some(keys::Own::Text(_))) => {
                Some(key.clone())
            }
            _ => None,
        })
        .expect("registered text key");
    let attempt_value = skein_kv::Op::Put { key: key.clone(), value };
    let mut first = registry.clone();
    first.push(attempt_value.clone());
    let first = fixture::bytes_with_ops(false, first.into_boxed_slice()).frames.remove(0);
    let other = another_layout(&registered, &key);
    registry.push(other.clone());
    let second = fixture::bytes_with_ops(false, Box::new([other, attempt_value.clone()])).frames.remove(1);
    let limits = fixtures::codec();
    let bounds = fixture::bounds();
    let observe = |store: &mut Store, mut events| store.observe(&mut events, &limits, fixtures::store_names(), &bounds);
    let mut store = Store::new();
    let log = fixture::LOG.to_vec();
    let check_commit = |store: &mut Store, at, bytes, number| {
        let commits =
            observe(store, vec![Observed::Bytes { path: log.clone(), at, bytes }, Observed::Sync(log.clone())])
                .expect("commit is readable through the registry");
        let [commit] = commits.as_slice() else { panic!("one observed commit") };
        let [root::Write::Save(root::Record::Core(row))] = commit.writes.as_slice() else {
            panic!("one saved attempt")
        };
        assert_eq!(commit.number, number);
        assert_eq!(*row, attempt());
    };
    let second_offset = first.len();
    check_commit(&mut store, 0, first, 1);
    let prepared = store.layouts.clone();
    check_commit(&mut store, second_offset, second, 2);
    assert_same_decoder(&prepared, &store.layouts, &key);

    let mut pending = fixture::bytes_with_ops(
        false,
        Box::new([skein_kv::Op::Put { key: text_key, value: Box::default() }, attempt_value.clone()]),
    )
    .frames
    .remove(0);
    number_frame(&mut pending, NEXT_REVISION);
    let error = observe(&mut store, vec![Observed::Pending { path: log, bytes: pending }])
        .expect_err("a pending attempt must check its changed registry text");
    assert!(error.starts_with("registry text:"), "{error}");
    assert_same_decoder(&prepared, &store.layouts, &key);
    let row = root::Record::Core(attempt());
    assert_eq!(store.rows.get(&row.key()), Some(&row));

    // Carry exactly the committed registry, without values that would warm the new preparation.
    let mut snapshot = fixture::bytes_with_ops(true, registry.into_boxed_slice()).snapshot;
    let next = NEXT_REVISION + 1;
    snapshot[SEQUENCE].copy_from_slice(&next.to_be_bytes());
    let header_crc = transcripts::frame::crc32c(&snapshot[..SEQUENCE.end]);
    snapshot[SEQUENCE.end..SEQUENCE.end + 4].copy_from_slice(&header_crc.to_be_bytes());
    let snapshot_read = |path: &[u8]| {
        vec![Observed::Bytes { path: path.to_vec(), at: 0, bytes: snapshot.clone() }, Observed::Snapshot(path.to_vec())]
    };
    observe(&mut store, snapshot_read(b"generations/1/store/snapshot")).expect("current snapshot read");
    assert_eq!(store.applied, NEXT_REVISION);
    assert_empty_preparation(&store.layouts);

    let skein_kv::Op::Put { value, .. } = attempt_value else { unreachable!() };
    store.layouts.serving(&key, value).expect("warm preparation in the current generation");
    observe(&mut store, snapshot_read(b"generations/2/store/snapshot")).expect("next generation's snapshot read");
    observe(&mut store, vec![Observed::Bytes { path: b"active".to_vec(), at: 0, bytes: b"2\n".to_vec() }])
        .expect("generation switch");
    assert_eq!(store.active.as_deref(), Some(b"generations/2".as_slice()));
    assert_empty_preparation(&store.layouts);
}

fn generation_snapshot(old: bool, malformed: bool) -> Vec<u8> {
    let mut format = jig_protocol_store::format::current(temper_protocol_store::STORE_VERSION, [7; 16]);
    format.levels = jig_protocol_store::levels(&temper_engine_service::store::OWNER_SETS);
    let root::Record::Core(row) = fixture::row() else { unreachable!() };
    let value = if malformed {
        Box::default()
    } else {
        codec::encode(&row, &fixtures::codec(), fixtures::store_names()).expect("current row")
    };
    let mut ops = vec![
        skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Format, 64).expect("format key"),
            value: format.encode().expect("format bytes"),
        },
        skein_kv::Op::Put { key: keys::encode(&row.key(), 64).expect("row key"), value },
    ];
    for set in temper_engine_service::store::OWNER_SETS.sets {
        for job in set.jobs {
            if old && crate::upgrade::JOBS.contains(&job.name) {
                continue;
            }
            let job_row = jig_protocol_store::registry_v1::Job { ended: 1 };
            let out = job_row.encode(&jig_protocol_store::registry_v1::CEILINGS).expect("job bytes");
            ops.push(skein_kv::Op::Put {
                key: keys::encode_own(&keys::Own::Job { owner: set.owner, name: job.name.into() }, 1024)
                    .expect("job key"),
                value: out,
            });
        }
    }
    fixture::bytes_with_ops(true, ops.into_boxed_slice()).snapshot
}

#[test]
fn pending_job_at_equal_levels_keeps_rows_opaque_until_switch_and_current_rows_must_decode() {
    fn start() -> Store {
        let mut store = Store::new();
        store.recovering();
        let path = b"generations/1/store/snapshot".to_vec();
        let mut events = vec![
            Observed::Bytes { path: path.clone(), at: 0, bytes: generation_snapshot(true, true) },
            Observed::Snapshot(path),
        ];
        assert!(
            store
                .observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
                .expect("durable observation")
                .is_empty()
        );
        assert!(store.rows.is_empty(), "no typed claim about the old generation");
        assert!(!store.generations[b"generations/1".as_slice()].rows.is_empty(), "raw rows retained");
        store
    }
    fn switch(store: &mut Store, malformed: bool) -> Result<Vec<Commit>, String> {
        let path = b"generations/2/store/snapshot".to_vec();
        let log = b"generations/2/store/log-00000000000000000002".to_vec();
        let row = core::Record::Core(core::CoreRecord::Settings(core::settings::Record::Extent { oldest: 1, keys: 7 }));
        let mut frames = fixture::bytes_with_ops(
            false,
            Box::new([skein_kv::Op::Put {
                key: keys::encode(&row.key(), 64).expect("newer row key"),
                value: codec::encode(&row, &fixtures::codec(), fixtures::store_names()).expect("newer row"),
            }]),
        )
        .frames;
        let bytes = frames.pop().expect("second commit frame");
        let mut events = vec![
            Observed::Bytes { path: log.clone(), at: 0, bytes },
            Observed::Recovered(log),
            Observed::Bytes { path: path.clone(), at: 0, bytes: generation_snapshot(false, malformed) },
            Observed::Snapshot(path),
            Observed::Bytes { path: b"active.tmp".to_vec(), at: 0, bytes: b"2\n".to_vec() },
            Observed::Renamed { from: b"active.tmp".to_vec(), to: b"active".to_vec() },
            Observed::Sync(Vec::new()),
        ];
        store.observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
    }
    let mut store = start();
    switch(&mut store, false).expect("current generation switches");
    let header = root::Record::Core(core::Record::Core(core::CoreRecord::Settings(core::settings::Record::Extent {
        oldest: 1,
        keys: 7,
    })));
    let expected = BTreeMap::from([(fixture::row().key(), fixture::row()), (header.key(), header)]);
    assert_eq!(store.rows, expected);
    assert_eq!(store.restored, Some(expected));
    assert!(store.deferred.is_none());
    let mut malformed = start();
    let failed = switch(&mut malformed, true).expect_err("malformed current-generation row");
    assert_eq!(failed, "real stored row decodes");
}

#[test]
fn a_copied_log_arriving_after_new_commits_keeps_untouched_rows_and_newer_writes() {
    let row = |task, cumulative| {
        let root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(mut row))) = fixture::row() else {
            unreachable!()
        };
        row.task = task;
        row.cumulative = cumulative;
        root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(row)))
    };
    let put = |row: &root::Record| {
        let root::Record::Core(row) = row else { unreachable!() };
        skein_kv::Op::Put {
            key: keys::encode(&row.key(), 64).expect("terminal key"),
            value: codec::encode(row, &fixtures::codec(), fixtures::store_names()).expect("terminal value"),
        }
    };
    let copied = fixture::bytes_with_ops(false, Box::new([put(&row(1, 1)), put(&row(2, 1)), put(&row(3, 1))]));
    let root::Key::Core(erased) = row(2, 1).key() else { unreachable!() };
    let mut newer = fixture::bytes_with_ops(
        false,
        Box::new([put(&row(1, 2)), skein_kv::Op::Erase { key: keys::encode(&erased, 64).expect("terminal key") }]),
    )
    .frames
    .remove(0);
    number_frame(&mut newer, 3);
    let copy = b"generations/2/store/log-00000000000000000001".to_vec();
    let log = b"generations/2/store/log-00000000000000000003".to_vec();
    let mut store = Store::new();
    store.sets = Some(jig_protocol_store::OwnerSets { sets: &[] });
    let mut events = vec![
        Observed::Bytes { path: b"active".to_vec(), at: 0, bytes: b"1\n".to_vec() },
        Observed::Bytes { path: log.clone(), at: 0, bytes: newer },
        Observed::Sync(log),
        Observed::Bytes { path: copy.clone(), at: 0, bytes: copied.frames.concat() },
        Observed::Sync(copy),
        Observed::Bytes { path: b"active.tmp".to_vec(), at: 0, bytes: b"2\n".to_vec() },
        Observed::Renamed { from: b"active.tmp".to_vec(), to: b"active".to_vec() },
        Observed::Sync(Vec::new()),
    ];
    store.observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds()).expect("copy switches");
    assert_eq!(store.applied, 3);
    assert_eq!(store.rows, BTreeMap::from([(row(1, 2).key(), row(1, 2)), (row(3, 1).key(), row(3, 1))]));
}

#[test]
fn a_torn_log_write_recovers_the_durable_prefix() {
    let bytes = fixture::bytes(false);
    let first = &bytes.frames[0];
    let second = &bytes.frames[1];
    let mut machine = Machine::for_user(USER);
    let root = machine.lay(&[Item::file(b"log", b"")]);
    let file = machine.open(root, b"log", How::Write).expect("log");
    machine.write(file, 0, first).expect("first write");
    machine.sync(file);
    machine.write(file, first.len() as u64, second).expect("second write");
    machine.crash(POWER_LOSS_SEED);
    let root = machine.reopen_root(0);
    let file = machine.open(root, b"log", How::Read).expect("recovered log");
    let bytes = machine.read(file, 0, 4096).expect("recovery read");
    assert!(bytes.len() > first.len() && bytes.len() < first.len() + second.len(), "second write torn");
    let mut store = Store::new();
    let mut events =
        vec![Observed::Bytes { path: fixture::LOG.to_vec(), at: 0, bytes }, Observed::Recovered(fixture::LOG.to_vec())];
    assert_eq!(
        store
            .observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
            .expect("durable observation")
            .len(),
        1
    );
    assert_eq!(store.applied, 1);
    assert_eq!(store.rows.get(&fixture::row().key()), Some(&fixture::row()));
}

#[test]
fn recovery_counts_synced_and_new_frames_across_split_reads() {
    let fixture = fixture::bytes(false);
    let first = &fixture.frames[0];
    let second = &fixture.frames[1];
    let (head, tail) = second.split_at(second.len() - 1);
    let path = fixture::LOG.to_vec();
    let mut store = Store::new();
    let limits = fixtures::codec();
    let bounds = fixture::bounds();
    let observe = |store: &mut Store, at, bytes: &[u8], end| {
        let events = &mut vec![Observed::Bytes { path: path.clone(), at, bytes: bytes.to_vec() }, end];
        store.observe(events, &limits, fixtures::store_names(), &bounds).expect("durable observation")
    };

    let synced = observe(&mut store, 0, first, Observed::Sync(path.clone()));
    assert_eq!(synced.iter().map(|commit| commit.number).collect::<Vec<_>>(), [1]);
    assert!(store.recovered.is_empty(), "sync alone is not recovery");

    let incomplete = observe(&mut store, first.len(), head, Observed::Recovered(path.clone()));
    assert!(incomplete.is_empty(), "an incomplete frame produces no commit");
    assert_eq!(store.recovered, [1].into(), "read back the frame already checked at sync");

    let completed = observe(&mut store, first.len() + head.len(), tail, Observed::Recovered(path.clone()));
    assert_eq!(completed.iter().map(|commit| commit.number).collect::<Vec<_>>(), [2]);
    assert_eq!(store.recovered, [1, 2].into());
}

#[test]
fn a_repair_snapshot_write_yields_no_commit_frames() {
    let bytes = fixture::bytes(true).snapshot;
    let mut capture = Capture::default();
    capture.names.insert(1, b"generations/1/store/snapshot.tmp".to_vec());
    let write = Op::Write { fd: Fd::new(1), bytes: bytes.clone().into(), from: 0, at: 0 };
    capture.pending(&write);
    assert!(capture.events.is_empty(), "snapshot bytes are not pending commit frames");
    capture.complete(&Complete {
        op: WRITE,
        kind: write,
        result: Ok(Done::Count(u32::try_from(bytes.len()).expect("snapshot length"))),
    });
    capture.complete(&Complete { op: SYNC, kind: Op::Sync { fd: Fd::new(1) }, result: Ok(Done::Nothing) });
    let mut store = Store::new();
    assert!(
        store
            .observe(&mut capture.events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
            .expect("durable observation")
            .is_empty()
    );
    assert_eq!(store.applied, 0);
}

#[test]
fn a_repair_snapshot_read_restores_the_durable_rows() {
    let bytes = fixture::bytes(true).snapshot;
    let mut store = Store::new();
    store.recovering();
    let mut capture = Capture::default();
    capture.names.insert(1, b"generations/1/store/snapshot".to_vec());
    capture.complete(&Complete {
        op: READ,
        kind: Op::Read { fd: Fd::new(1), buf: bytes.clone().into(), at: 0 },
        result: Ok(Done::Count(u32::try_from(bytes.len()).expect("snapshot length"))),
    });
    assert_eq!(
        store
            .observe(&mut capture.events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
            .expect("durable observation")
            .len(),
        1
    );
    assert_eq!(store.applied, 1);
    assert_eq!(store.recovered, [1].into());
    assert_eq!(store.rows.get(&fixture::row().key()), Some(&fixture::row()));
}

#[test]
fn the_durable_observer_retains_a_settings_copy_before_its_erase() {
    use jig_domain_core::settings as s;
    let revision = s::Revision {
        head: s::Head {
            number: 1,
            against: 0,
            at: skein_lib::Wall::EPOCH,
            by: s::Decider::FirstStart,
            proposal: None,
            moved: false,
        },
        items: Box::new([]),
        applications: Box::new([]),
    };
    let held = core::Record::Core(core::CoreRecord::Settings(s::Record::Revision {
        history: false,
        revision: revision.clone(),
    }));
    let history = core::Record::Core(core::CoreRecord::Settings(s::Record::Revision { history: true, revision }));
    let limits = fixtures::codec();
    let mut store = Store::new();
    store.rows.insert(root::Key::Core(held.key()), root::Record::Core(held.clone()));
    let ops = Box::new([
        skein_kv::Op::Put {
            key: keys::encode(&history.key(), u32::MAX).expect("history key"),
            value: codec::encode(&history, &limits, fixtures::store_names())
                .expect("settings revision fits fixture bounds"),
        },
        skein_kv::Op::Erase { key: keys::encode(&held.key(), u32::MAX).expect("held key") },
    ]);
    let commit = store.commit(1, ops, &limits, fixtures::store_names(), &fixture::bounds()).expect("observed commit");
    assert_eq!(
        commit.writes,
        vec![root::Write::CopySettingsRevision(1), root::Write::Erase(root::Key::Core(held.key()))]
    );
    store.apply(&commit, &limits);
    assert!(!store.rows.contains_key(&root::Key::Core(held.key())));
    assert_eq!(store.rows.get(&root::Key::Core(history.key())), Some(&root::Record::Core(history)));
}
