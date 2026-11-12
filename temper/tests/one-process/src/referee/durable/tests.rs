use super::*;
use crate::fixtures::{self, durable as fixture};
use fixture::{POWER_LOSS_SEED, READ, SYNC, USER, WRITE};
use skein_fake_machine::{How, Item, Machine};

fn generation_snapshot(old: bool, malformed: bool) -> Vec<u8> {
    let mut format = jig_protocol_store::format::current(temper_protocol_store::STORE_VERSION, [7; 16]);
    format.levels = jig_protocol_store::levels(&temper_service::store::PATCH_SETS);
    if old {
        format.tasks = 9;
        format.settings = 1;
        format.levels.jig = Some(0);
        format.levels.application = Some(0);
    }
    let root::Record::Core(row) = fixture::row() else { unreachable!() };
    let value = if malformed {
        Box::default()
    } else {
        codec::encode(&row, &fixtures::codec(), fixtures::store_names()).expect("current row")
    };
    fixture::bytes_with_ops(
        true,
        Box::new([
            skein_kv::Op::Put {
                key: keys::encode_own(&keys::Own::Format, 64).expect("format key"),
                value: format.encode().expect("format bytes"),
            },
            skein_kv::Op::Put { key: keys::encode(&row.key(), 64).expect("row key"), value },
        ]),
    )
    .snapshot
}

#[test]
fn old_generation_rows_are_opaque_until_switch_and_current_rows_must_decode() {
    fn start() -> Store {
        let mut store = Store::new();
        store.recovering();
        let path = b"generations/1/store/snapshot".to_vec();
        let mut events = vec![
            Observed::Bytes { path: path.clone(), at: 0, bytes: generation_snapshot(true, true) },
            Observed::Snapshot(path),
        ];
        assert!(store.observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds()).is_empty());
        assert!(store.rows.is_empty(), "no typed claim about the old generation");
        assert!(!store.generations[b"generations/1".as_slice()].rows.is_empty(), "raw rows retained");
        store
    }
    fn switch(store: &mut Store, malformed: bool) {
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
        store.observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds());
    }
    let mut store = start();
    switch(&mut store, false);
    let header = root::Record::Core(core::Record::Core(core::CoreRecord::Settings(core::settings::Record::Extent {
        oldest: 1,
        keys: 7,
    })));
    let expected = BTreeMap::from([(fixture::row().key(), fixture::row()), (header.key(), header)]);
    assert_eq!(store.rows, expected);
    assert_eq!(store.restored, Some(expected));
    assert!(store.deferred.is_none());
    let mut malformed = start();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| switch(&mut malformed, true)));
    assert!(failed.is_err(), "a malformed current-generation row is still fatal");
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
    assert_eq!(store.observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds()).len(), 1);
    assert_eq!(store.applied, 1);
    assert_eq!(store.rows.get(&fixture::row().key()), Some(&fixture::row()));
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
        store.observe(&mut capture.events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds()).is_empty()
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
        store.observe(&mut capture.events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds()).len(),
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
    let commit = store.commit(1, ops, &limits, fixtures::store_names(), &fixture::bounds());
    assert_eq!(
        commit.writes,
        vec![root::Write::CopySettingsRevision(1), root::Write::Erase(root::Key::Core(held.key()))]
    );
    store.apply(&commit, &limits);
    assert!(!store.rows.contains_key(&root::Key::Core(held.key())));
    assert_eq!(store.rows.get(&root::Key::Core(history.key())), Some(&root::Record::Core(history)));
}
