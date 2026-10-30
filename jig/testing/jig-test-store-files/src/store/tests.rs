use super::{Fault, Store};
use jig_domain_core as core;
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up};
use std::collections::BTreeMap;

// Nine captured first pages exercise more than eight concurrent routes.
const FLIGHTS: u16 = 9;
// The route asks for two rows per page, so three rows require a continuation.
const ROWS: u64 = 3;
const PAGE_DELAY: u32 = 2;
// The last first page arrives at FLIGHTS; its continuation needs PAGE_DELAY.
const TICKS: u32 = 11;
const CRASH_SEED: u64 = 18;

fn result(number: u16, task: u64) -> root::Record {
    root::Record::Connector { number, record: jig_test_connector::Record::Result { task, code: number } }
}

fn snapshot_store() -> Store {
    let limits = jig_core_world::world::limits();
    let mut store = Store::new(&limits, &[]);
    let header = core::Deployment {
        id: [0; 16],
        tasks: ROWS,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: u64::from(FLIGHTS) * ROWS,
        projects: 0,
        applications: 0,
        commits: 1,
    };
    let mut writes =
        vec![root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))))];
    writes.push(root::Write::Wires);
    for number in 1..=FLIGHTS {
        for task in 1..=ROWS {
            writes.push(root::Write::Save(result(number, task)));
        }
    }
    store.submit(1, writes.into_boxed_slice(), 0);
    let _committed = store.tick(false).expect("fixture commit");
    store.fault(Fault::SlowPages { by: PAGE_DELAY });
    store.pause_pages(true);
    store
}

fn ask_snapshots(store: &mut Store) {
    for connector in 1..=FLIGHTS {
        store.ask(StoreOut::Restart { step: core::RestartStep::RestoreConnector { connector }, census: None });
    }
}

fn reverse_delays(store: &mut Store) {
    let mut delay = u32::from(FLIGHTS);
    for page in &mut store.delayed {
        page.delay = delay;
        delay -= 1;
    }
}

fn finish_snapshots(store: &mut Store) {
    store.pause_pages(false);
    let mut restored = BTreeMap::<u16, Vec<root::Record>>::new();
    let mut finished = BTreeMap::<u16, u32>::new();
    let mut first_pages = Vec::new();
    for _ in 0..TICKS {
        let _committed = store.tick(false).expect("read-only traversal");
        for answer in store.answers() {
            match answer {
                Up::Restore(row @ root::Record::Connector { number, .. }) => {
                    assert!(!finished.contains_key(&number), "all rows precede the terminal");
                    let rows = restored.entry(number).or_default();
                    if rows.is_empty() {
                        first_pages.push(number);
                    }
                    rows.push(row);
                }
                Up::Root(root::Event::RestartDone(core::RestartStep::RestoreConnector { connector })) => {
                    *finished.entry(connector).or_default() += 1;
                }
                Up::Restore(_) | Up::Root(_) | Up::Ask(_) => panic!("only connector rows and terminals"),
            }
        }
    }
    assert!(!store.loading());
    assert_eq!(first_pages, [9, 8, 7, 6, 5, 4, 3, 2, 1]);
    assert_eq!(restored.len(), usize::from(FLIGHTS));
    assert_eq!(finished.len(), usize::from(FLIGHTS));
    for number in 1..=FLIGHTS {
        assert_eq!(
            restored.get(&number).expect("each snapshot"),
            &[result(number, 1), result(number, 2), result(number, 3)]
        );
        assert_eq!(finished.get(&number), Some(&1));
    }
}

#[test]
fn delayed_pages_arrive_in_reverse_and_across_a_crash() {
    let mut store = snapshot_store();
    ask_snapshots(&mut store);
    reverse_delays(&mut store);
    // Pausing preserves both the flights and their countdowns.
    let _committed = store.tick(false).expect("paused traversal");
    assert!(store.answers().is_empty());
    assert_eq!(store.delayed.len(), usize::from(FLIGHTS));
    let mut delays = Vec::new();
    for page in &store.delayed {
        delays.push(page.delay);
    }
    assert_eq!(delays, [9, 8, 7, 6, 5, 4, 3, 2, 1]);
    finish_snapshots(&mut store);

    // A crash discards captured pages and their owners; fresh snapshots still
    // restore every durable row exactly once, including continuation pages.
    store.pause_pages(true);
    ask_snapshots(&mut store);
    store.crash(CRASH_SEED);
    assert!(!store.loading());
    assert!(store.answers().is_empty());
    ask_snapshots(&mut store);
    reverse_delays(&mut store);
    finish_snapshots(&mut store);
}

#[test]
fn readback_discovers_actual_inventory_without_changing_the_mirror() {
    let mut store = snapshot_store();
    let expected = store.rows.clone();
    let before = store.retained();
    assert_eq!(store.readback(), expected);
    assert_eq!(store.rows, expected);
    assert_eq!(store.retained(), before, "readers write and erase nothing");
    assert_eq!(store.applied, 1);
    assert!(
        before.contains_key(&jig_protocol_store::keys::installation(store.component_limits.store.key).expect("key"))
    );
    for own in [
        jig_protocol_store::keys::Own::Format,
        jig_protocol_store::keys::Own::Layouts,
        jig_protocol_store::keys::Own::Wires,
    ] {
        assert!(
            before.contains_key(
                &jig_protocol_store::keys::encode_own(&own, store.component_limits.store.key).expect("key")
            )
        );
    }
}

#[test]
#[should_panic(expected = "quiescent fixture boundary")]
fn readback_refuses_pending_writes_without_draining_them() {
    let mut store = snapshot_store();
    store.submit(2, Box::new([]), 0);
    let _found = store.readback();
}

fn component_commit(store: &mut Store, number: u64, writes: Box<[jig_protocol_store::Write]>) {
    let terminal = store.readback_query(jig_protocol_store::Request::Commit { number, writes });
    assert!(matches!(terminal, jig_protocol_store::Event::Committed { number: found, .. } if found == number));
}

#[test]
fn readback_exposes_extra_missing_and_altered_actual_rows() {
    use jig_protocol_store::Write;
    let mut store = snapshot_store();
    let expected = store.rows.clone();
    let root::Record::Connector { number, record } = result(1, 1) else { panic!("connector fixture") };
    let key = jig_test_store::keys::encode(number, &record.key());
    let root::Record::Connector { record: extra, .. } = result(1, ROWS + 1) else { panic!("extra fixture") };
    let extra_key = jig_test_store::keys::encode(number, &extra.key());
    component_commit(
        &mut store,
        2,
        Box::new([Write::Put { key: extra_key.clone(), value: jig_test_store::records::encode(&extra) }]),
    );
    assert_ne!(store.readback(), expected, "extra valid row is discovered");
    component_commit(&mut store, 3, Box::new([Write::Remove { key: extra_key }, Write::Remove { key: key.clone() }]));
    assert_ne!(store.readback(), expected, "missing expected row is exposed");
    let altered = jig_test_connector::Record::Result { task: 1, code: 99 };
    component_commit(&mut store, 4, Box::new([Write::Put { key, value: jig_test_store::records::encode(&altered) }]));
    assert_ne!(store.readback(), expected, "altered value is read through Get");
    assert_eq!(store.rows, expected, "all changes were component-only");
}

#[test]
#[should_panic(expected = "recognized application key")]
fn readback_rejects_unknown_actual_key() {
    let mut store = snapshot_store();
    component_commit(
        &mut store,
        2,
        Box::new([jig_protocol_store::Write::Put { key: Box::new([255]), value: Box::new([]) }]),
    );
    let _found = store.readback();
}

#[test]
#[should_panic(expected = "application record decodes")]
fn readback_rejects_undecodable_actual_value() {
    let mut store = snapshot_store();
    component_commit(
        &mut store,
        2,
        Box::new([jig_protocol_store::Write::Put {
            key: jig_test_store::keys::encode(1, &jig_test_connector::RecordKey::Result(1)),
            value: Box::new([]),
        }]),
    );
    let _found = store.readback();
}

fn transcript_store() -> (Store, Vec<core::TurnRecord>) {
    let mut store = snapshot_store();
    let turns: Vec<_> = (1..=3)
        .map(|turn| core::TurnRecord {
            task: 1,
            attempt: if turn == 3 { 2 } else { 1 },
            turn: if turn == 3 { 1 } else { turn },
            opens_window: turn != 2,
            spent: 0,
            read: None,
            at: skein_lib::Wall::EPOCH,
            transcript: Box::from(b"fixture turn".as_slice()),
        })
        .collect();
    for (index, turn) in turns.iter().enumerate() {
        let number = u64::try_from(index).expect("three turns") + 2;
        store.submit(
            number,
            Box::new([root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn.clone()))))]),
            0,
        );
        assert_eq!(store.tick(false), Ok(Some(number)));
    }
    (store, turns)
}

#[test]
fn readback_transcripts_compare_both_frame_orders_with_independent_turns() {
    let (mut store, expected) = transcript_store();
    let mirror = store.rows.clone();
    let before = store.retained();
    assert_eq!(store.transcript(1, false), expected);
    assert_eq!(store.transcript(1, true), expected.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(store.readback(), mirror);
    assert_eq!(store.rows, mirror);
    assert_eq!(store.retained(), before);
}

#[test]
#[should_panic(expected = "complete transcript requires successful Turns")]
fn readback_rejects_unread_transcript() {
    let (mut store, _) = transcript_store();
    store.unread_transcript(1);
    let _found = store.transcript(1, false);
}

#[test]
#[should_panic(expected = "complete transcript requires successful Turns")]
fn readback_rejects_unread_transcript_after_a_successful_prefix() {
    use skein_io::file;
    use skein_lib::Token;
    let (mut store, _) = transcript_store();
    let event = store.readback_query(jig_protocol_store::Request::Transcript {
        owner: Token::new(0),
        read: jig_protocol_store::TranscriptRead::Page { task: 1, backwards: false, after: None },
        rows: 2,
        bytes: u64::from(store.component_limits.store.page.bytes),
    });
    let jig_protocol_store::Event::Turns { turns, next: Some(offset), .. } = event else { panic!("prefix page") };
    assert_eq!(turns.len(), 2);
    let root = store.files.root();
    let opened = store.file(
        2,
        file::Request::OpenWrite {
            owner: Token::new(800),
            root: root.fd,
            name: Box::from(b"0000000000000000/0000000000000001".as_slice()),
        },
    );
    let file::Event::Opened { file, .. } = opened.event else { panic!("actual transcript") };
    let cut = store.file(2, file::Request::Cut { owner: Token::new(800), file, length: offset });
    assert!(matches!(cut.event, file::Event::Cut { .. }));
    let _closed = store.file(2, file::Request::Close { owner: Token::new(800), file });
    // The first two real frames still read successfully; the committed third
    // frame is now missing, so a complete collector must reject its prefix.
    let _found = store.transcript(1, false);
}
