use super::{Fault, Store};
use jig_domain_core::{self as core, engine::store as engine_store};
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

fn drain_open_reports(store: &mut Store) {
    for answer in store.answers() {
        assert!(matches!(answer, Up::Root(root::Event::Store(engine_store::In::Census(_)))), "only census after open");
    }
}

fn result(number: u16, task: u64) -> root::Record {
    root::Record::Connector { number, record: jig_test_connector::Record::Result { task, code: number } }
}

fn snapshot_store() -> Store {
    snapshot(Store::new(&jig_test_profile::limits(), &[]))
}

fn snapshot(mut store: Store) -> Store {
    drain_open_reports(&mut store);
    let header = core::Deployment {
        id: [0; 16],
        restart_wait_ms: None,
        wait_floors: None,
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
        store.ask(StoreOut::ConnectorLoad { number: connector, after: None });
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
                Up::Root(root::Event::ConnectorPage { number, records, after }) => {
                    assert!(!finished.contains_key(&number), "all rows precede the terminal");
                    let rows = restored.entry(number).or_default();
                    if rows.is_empty() {
                        first_pages.push(number);
                    }
                    rows.extend(records.into_iter().map(|record| root::Record::Connector { number, record }));
                    if let Some(after) = after {
                        store.ask(StoreOut::ConnectorLoad { number, after: Some(after) });
                    } else {
                        *finished.entry(number).or_default() += 1;
                    }
                }
                Up::Root(_) | Up::Ask(_) => panic!("only connector rows and terminals"),
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
    store.crash(CRASH_SEED).expect("crash readback");
    assert!(!store.loading());
    drain_open_reports(&mut store);
    ask_snapshots(&mut store);
    reverse_delays(&mut store);
    finish_snapshots(&mut store);
}

#[test]
fn readback_discovers_actual_inventory_without_changing_the_mirror() {
    let mut store = snapshot_store();
    let expected = store.rows.clone();
    let before = store.retained().expect("complete readback");
    assert_eq!(store.readback().expect("complete readback"), expected);
    assert_eq!(store.rows, expected);
    assert_eq!(store.retained().expect("complete readback"), before, "readers write and erase nothing");
    assert_eq!(store.applied, 1);
    assert!(
        before.contains_key(&jig_protocol_store::keys::installation(store.checked.limits().store.key).expect("key"))
    );
    for own in [jig_protocol_store::keys::Own::Format, jig_protocol_store::keys::Own::Wires] {
        assert!(
            before.contains_key(
                &jig_protocol_store::keys::encode_own(&own, store.checked.limits().store.key).expect("key")
            )
        );
    }
}

#[test]
fn readback_refuses_pending_writes_without_draining_them() {
    let mut store = snapshot_store();
    let inventory = store.retained().expect("quiet inventory before submission");
    store.submit(2, Box::new([root::Write::Save(result(1, 1))]), 0);
    let pending = store.pending.clone();
    let held = store.held.clone();
    let rows = store.rows.clone();
    let applied = store.applied;
    assert!(matches!(store.readback(), Err(super::ReadbackFinding::NotQuiescent)));
    assert_eq!(store.pending, pending);
    assert!(matches!(store.retained(), Err(super::ReadbackFinding::NotQuiescent)));
    assert!(matches!(store.readback_of(&inventory), Err(super::ReadbackFinding::NotQuiescent)));
    assert!(matches!(store.transcript(1, false), Err(super::ReadbackFinding::NotQuiescent)));
    assert_eq!(store.held, held);
    assert_eq!(store.rows, rows);
    assert_eq!(store.applied, applied);
    assert!(store.inflight.is_empty());
    assert!(store.replies.is_empty());
    assert!(store.delayed.is_empty());
}

fn component_commit(store: &mut Store, number: u64, writes: Box<[jig_protocol_store::Write]>) {
    let terminal =
        store.readback_query(jig_protocol_store::Request::Commit { number, writes }).expect("component commit");
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
    assert_ne!(store.readback().expect("complete readback"), expected, "extra valid row is discovered");
    component_commit(&mut store, 3, Box::new([Write::Remove { key: extra_key }, Write::Remove { key: key.clone() }]));
    assert_ne!(store.readback().expect("complete readback"), expected, "missing expected row is exposed");
    let altered = jig_test_connector::Record::Result { task: 1, code: 99 };
    component_commit(&mut store, 4, Box::new([Write::Put { key, value: jig_test_store::records::encode(&altered) }]));
    assert_ne!(store.readback().expect("complete readback"), expected, "altered value is read through Get");
    assert_eq!(store.rows, expected, "all changes were component-only");
}

#[test]
fn history_readback_discovers_logs_and_survives_a_memory_restart() {
    let mut store = snapshot_store();
    let histories: Vec<_> = [1, 2]
        .into_iter()
        .flat_map(|task| {
            (1..=3).map(move |attempt| {
                root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Attempt(
                    jig_domain_tasks::AttemptRow {
                        sources: None,
                        task,
                        attempt,
                        claimed: Some(skein_lib::Wall::EPOCH),
                        started: None,
                        ended: None,
                        end: None,
                        phases: None,
                    },
                )))
            })
        })
        .collect();
    store.seed(histories.into_boxed_slice());
    let expected = store.rows.clone();
    let inventory = store.retained().expect("inventory");
    assert!(inventory.keys().all(|key| jig_protocol_store::keys::logged(key).is_none()));
    assert_eq!(store.readback().expect("history through view"), expected);
    store.crash(CRASH_SEED).expect("restart with logs");
    assert_eq!(store.readback().expect("restarted history"), expected);
    let number = store.applied + 1;
    component_commit(
        &mut store,
        number,
        Box::new([jig_protocol_store::Write::Erase(core::Key::Tasks(jig_domain_tasks::Key::Attempt {
            task: 1,
            attempt: 1,
        }))]),
    );
    assert_ne!(store.readback().expect("erased history"), expected, "readback never fills from the mirror");
    assert_eq!(store.rows, expected);
}

fn disk_history_store() -> Store {
    let mut store = Store::on_disk(&jig_test_profile::limits(), &[]);
    let header = core::Deployment {
        id: [0; 16],
        restart_wait_ms: None,
        wait_floors: None,
        tasks: 1,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 1,
        calls: 0,
        connector_rows: 0,
        projects: 0,
        applications: 0,
        commits: 1,
    };
    let attempt = jig_domain_tasks::AttemptRow {
        sources: None,
        task: 1,
        attempt: 1,
        claimed: Some(skein_lib::Wall::EPOCH),
        started: None,
        ended: None,
        end: None,
        phases: None,
    };
    store.submit(
        1,
        Box::new([
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))),
            root::Write::Save(root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Attempt(attempt)))),
        ]),
        0,
    );
    assert_eq!(store.tick(false), Ok(Some(1)));
    store
}

#[test]
fn a_kill_during_recovery_history_readback_can_restart_again() {
    let mut reference = disk_history_store();
    reference.crash(CRASH_SEED).expect("complete reference recovery");
    // The last file operation fetches the log for the recovered inventory.
    let last = reference.open_operation_count() - 1;
    let expected = reference.rows.clone();
    for operation in [0, last] {
        let mut store = disk_history_store();
        store.fault(Fault::Kill { from: super::KillFrom::Open, operation });
        store.crash(CRASH_SEED).expect("a killed recovery yields to its caller");
        assert!(store.reached_kill(), "cut {operation} reached");
        assert_eq!(store.open_operation_count(), operation);
        assert_eq!(store.rows, expected, "a killed readback does not replace the mirror");
        store.crash(CRASH_SEED).expect("restart after the second kill");
        assert!(!store.reached_kill());
        assert_eq!(store.readback().expect("complete recovered history"), expected);
    }
}

#[test]
fn readback_rejects_unknown_actual_key() {
    let mut store = snapshot_store();
    component_commit(
        &mut store,
        2,
        Box::new([jig_protocol_store::Write::Put { key: Box::new([255]), value: Box::new([]) }]),
    );
    let Err(super::ReadbackFinding::ApplicationKey { key }) = store.readback() else {
        panic!("unknown application key finding");
    };
    assert_eq!(key.as_ref(), &[255]);
}

#[test]
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
    let Err(super::ReadbackFinding::ApplicationValue { key }) = store.readback() else {
        panic!("undecodable application value finding");
    };
    assert_eq!(key, jig_test_store::keys::encode(1, &jig_test_connector::RecordKey::Result(1)));
}

#[test]
fn readback_rejects_an_application_value_under_another_records_key() {
    let mut store = snapshot_store();
    let key = jig_test_store::keys::encode(1, &jig_test_connector::RecordKey::Result(1));
    let record = jig_test_connector::Record::Result { task: ROWS + 1, code: 7 };
    component_commit(
        &mut store,
        2,
        Box::new([jig_protocol_store::Write::Put {
            key: key.clone(),
            value: jig_test_store::records::encode(&record),
        }]),
    );
    let before = store.retained().expect("raw inventory");
    let mirror = store.rows.clone();
    let Err(super::ReadbackFinding::Inventory { key: Some(found), reason }) = store.readback() else {
        panic!("application record key agreement finding");
    };
    assert_eq!(jig_protocol_store::keys::of_application(&found), Some(key.as_ref()));
    assert_eq!(reason, "application record key agreement");
    assert_eq!(store.retained().expect("unchanged inventory"), before);
    assert_eq!(store.rows, mirror);
}

const TRANSCRIPT_TASK: u64 = 1;

fn transcript_store() -> (Store, Vec<core::TurnRecord>) {
    let mut store = snapshot_store();
    let turns = save_transcript(&mut store);
    (store, turns)
}

fn save_transcript(store: &mut Store) -> Vec<core::TurnRecord> {
    let turns: Vec<_> = (1..=3)
        .map(|turn| core::TurnRecord {
            task: TRANSCRIPT_TASK,
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
    turns
}

fn history_and_turns(mut store: Store) -> Store {
    use jig_domain_tasks as tasks;
    let _turns = save_transcript(&mut store);
    let ended = core::Record::Tasks(tasks::Stored::Ended(crate::tests::ended_chat()));
    let attempt = core::Record::Tasks(tasks::Stored::Attempt(tasks::AttemptRow {
        sources: None,
        task: TRANSCRIPT_TASK,
        attempt: 1,
        claimed: Some(skein_lib::Wall::EPOCH),
        started: None,
        ended: Some(skein_lib::Wall::EPOCH),
        end: None,
        phases: None,
    }));
    store.seed(Box::new([root::Record::Core(ended), root::Record::Core(attempt)]));
    store
}

fn without_read_reuse() -> Store {
    history_and_turns(snapshot(Store::starting_with_read_reuse(
        &jig_test_profile::limits(),
        &[],
        super::protocol::OwnerSets::NONE,
        false,
    )))
}

fn assert_reads_not_kept(mut store: Store, stage: &str) {
    use skein_kv::{Direction, Event, LogRule, Op, Request};
    use skein_lib::{Queue, Token};
    const LOG: u64 = 1;
    const BODY: &[u8] = b"s";
    const BYTES: u32 = 1; // One byte, the whole fixture record.
    const ROWS: u32 = 1; // The record appended to the probe log.

    let mut stores = store.component.take().expect("component").into_stores();
    assert_eq!(stores.store.kept_reads(), 0, "main store {stage}");

    // The component doesn't use payload logs in secrets. Read one directly to
    // distinguish a disabled store from an enabled one that hasn't read a log.
    let env = store.env().with(store.checked.limits().secrets);
    stores.secrets.logs(LogRule { prefix: Box::from(b"test-extent/".as_slice()) });
    let frame = stores.secrets.frame(BODY).expect("secret probe record");
    let owner = Token::new(0);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    for request in [
        Request::Commit { owner, ops: Box::new([Op::Append { log: LOG, frame }]) },
        Request::ReadLog { owner, log: LOG, at: 0, direction: Direction::Forward, max: BYTES, rows: ROWS },
    ] {
        skein_kv::down(&mut stores.secrets, &env, request, &mut above, &mut below);
    }
    assert!(matches!(above.pop(), Some(Event::Committed { .. })));
    let Some(Event::LogRead { records, .. }) = above.pop() else { panic!("secret probe read") };
    assert!(records.iter().map(|record| record.record.as_deref()).eq([Some(BODY)]));
    assert_eq!(stores.secrets.kept_reads(), 0, "secrets store {stage}");
}

#[test]
fn disabling_read_reuse_reaches_every_memory_store() {
    const SEED: u64 = 3191;
    let mut default = history_and_turns(snapshot_store());
    default.readback().expect("history and transcript with defaults");
    assert!(default.component.take().expect("component").into_stores().store.kept_reads() > 0);

    let mut original = without_read_reuse();
    original.readback().expect("initial history and transcript");
    assert_reads_not_kept(original, "at first open");

    // Checking the live KV stores consumes the component, so fork each continuation.
    let mut crashed = without_read_reuse();
    crashed.readback().expect("history and transcript before crash");
    crashed.crash(SEED).expect("restart with logs");
    crashed.readback().expect("restarted history and transcript");
    let mut reconfigured = crashed.fork().expect("fork before reconfiguration");
    assert_reads_not_kept(crashed, "after crash");

    reconfigured.reconfigure(SEED, &[jig_test_profile::scripted_model()]).expect("reconfigure with logs");
    reconfigured.readback().expect("reconfigured history and transcript");
    let mut forked = reconfigured.fork().expect("fork memory store");
    assert_reads_not_kept(reconfigured, "after reconfigure");

    forked.readback().expect("forked history and transcript");
    let mut corrupted = forked.fork().expect("fork before corrupting a turn");
    assert_reads_not_kept(forked, "after fork");

    corrupted.unread_memory_turn(TRANSCRIPT_TASK, 0);
    assert!(matches!(corrupted.readback(), Err(super::ReadbackFinding::Transcript { task: TRANSCRIPT_TASK, .. })));
    assert_reads_not_kept(corrupted, "after rebuilding a corrupted turn");
}

#[test]
fn readback_agrees_with_and_without_read_reuse() {
    const SEED: u64 = 3192;
    let mut kept = history_and_turns(snapshot_store());
    let mut forgotten = without_read_reuse();
    let expected = kept.readback().expect("history and transcript with reuse");
    assert_eq!(forgotten.readback().expect("without reuse"), expected);
    for store in [&mut kept, &mut forgotten] {
        store.crash(SEED).expect("restart with logs");
        assert_eq!(store.readback().expect("after restart"), expected);
    }
}

#[test]
fn a_fork_copies_committed_storage_and_accepts_the_next_commit() {
    let (mut source, _) = transcript_store();
    source.seed(Box::new([]));
    let expected = source.readback().expect("source readback");
    let mut forked = source.fork().expect("fork image");
    forked.crash(CRASH_SEED).expect("fork cold open");
    assert_eq!(forked.applied, source.applied);
    assert_eq!(forked.readback().expect("fork readback"), expected);
    let next = source.applied + 1;
    let writes = Box::new([result(1, ROWS + 1)]);
    forked.seed(writes.clone());
    assert_eq!(forked.applied, next);
    assert_eq!(source.readback().expect("source unchanged"), expected);
    source.seed(writes);
    assert_eq!(source.applied, next);
    assert_eq!(forked.readback().expect("fork next commit"), source.readback().expect("source next commit"));

    source.unread_transcript(1);
    let mut forked = source.fork().expect("copy unread physical transcript");
    assert!(matches!(forked.readback(), Err(super::ReadbackFinding::Transcript { .. })));
}

#[test]
fn readback_of_agrees_with_readback() {
    let (mut store, _) = transcript_store();
    let inventory = store.retained().expect("complete readback");
    let expected = store.readback().expect("complete readback");
    assert_eq!(store.readback_of(&inventory).expect("complete readback"), expected);
    assert_eq!(expected, store.rows);
    assert_eq!(store.retained().expect("complete readback"), inventory, "readback preserves the supplied inventory");
}

#[test]
fn readback_transcripts_compare_both_frame_orders_with_independent_turns() {
    let (mut store, expected) = transcript_store();
    let mirror = store.rows.clone();
    let before = store.retained().expect("complete readback");
    assert_eq!(store.transcript(1, false).expect("complete readback"), expected);
    assert_eq!(store.transcript(1, true).expect("complete readback"), expected.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(store.readback().expect("complete readback"), mirror);
    assert_eq!(store.rows, mirror);
    assert_eq!(store.retained().expect("complete readback"), before);
}

#[test]
fn readback_rejects_unread_transcript() {
    let (mut store, _) = transcript_store();
    store.unread_transcript(1);
    let Err(super::ReadbackFinding::Transcript { task: 1, backwards: false, after: None, terminal, .. }) =
        store.transcript(1, false)
    else {
        panic!("unread transcript finding");
    };
    assert!(matches!(*terminal, jig_protocol_store::Event::Unread { owner } if owner == skein_lib::Token::new(0)));
}

#[test]
fn readback_rejects_unread_transcript_after_a_successful_prefix() {
    use skein_lib::Token;
    let (mut store, _) = transcript_store();
    let event = store
        .readback_query(jig_protocol_store::Request::Transcript {
            owner: Token::new(0),
            read: jig_protocol_store::TranscriptRead::Page { task: 1, backwards: false, after: None },
            rows: 2,
            bytes: u64::from(store.checked.limits().store.page.bytes),
        })
        .expect("prefix page");
    let jig_protocol_store::Event::Turns { turns, next: Some(offset), .. } = event else { panic!("prefix page") };
    assert_eq!(turns.len(), 2);
    store.unread_memory_turn(1, offset);
    // The first two records still read successfully; the committed third
    // record is now malformed, so a complete collector must reject its prefix.
    let Err(super::ReadbackFinding::Transcript { task: 1, backwards: false, after: Some(after), terminal, .. }) =
        store.transcript(1, false)
    else {
        panic!("unread terminal after the prefix");
    };
    assert_eq!(after, offset);
    assert!(matches!(*terminal, jig_protocol_store::Event::Unread { owner } if owner == Token::new(0)));
}

fn starting_snapshot() -> Store {
    let limits = jig_test_profile::limits();
    let mut store = Store::starting(&limits, &[]);
    drain_open_reports(&mut store);
    let header = core::Deployment {
        id: [0; 16],
        restart_wait_ms: None,
        wait_floors: None,
        tasks: ROWS,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: ROWS,
        projects: 0,
        applications: 0,
        commits: 1,
    };
    store.submit(
        1,
        Box::new([
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))),
            root::Write::Save(result(1, 1)),
            root::Write::Save(result(1, 2)),
            root::Write::Save(result(1, 3)),
        ]),
        0,
    );
    assert_eq!(store.tick(false), Ok(Some(1)));
    store
}
fn check_start_snapshot(store: &mut Store) {
    let mut records = Vec::new();
    let mut after = None;
    for expected_rows in [2, 1] {
        store.ask(StoreOut::ConnectorLoad { number: 1, after });
        let [Up::Root(root::Event::ConnectorPage { number: 1, records: page, after: next })]: [Up; 1] =
            store.answers().try_into().expect("one restore reply")
        else {
            panic!("typed connector page")
        };
        assert_eq!(page.len(), expected_rows);
        records.extend(page.into_iter().map(|record| root::Record::Connector { number: 1, record }));
        after = next;
    }
    assert!(after.is_none(), "terminal page");
    assert_eq!(records, [result(1, 1), result(1, 2), result(1, 3)]);
    assert!(!store.loading());
}
#[test]
fn start_mode_survives_crash_and_fork_with_complete_readback() {
    let mut store = starting_snapshot();
    let expected = store.rows.clone();
    check_start_snapshot(&mut store);
    assert_eq!(store.readback().expect("complete readback"), expected);
    store.crash(CRASH_SEED).expect("crash readback");
    drain_open_reports(&mut store);
    check_start_snapshot(&mut store);
    assert_eq!(store.readback().expect("complete readback"), expected);
    let mut forked = store.fork().expect("fork readback");
    drain_open_reports(&mut forked);
    check_start_snapshot(&mut forked);
    assert_eq!(forked.readback().expect("complete readback"), expected);
}
#[test]
fn start_switch_without_a_pending_generation_finishes_without_a_store_switch() {
    let mut store = starting_snapshot();
    assert!(!store.generation_pending);
    store.ask(StoreOut::Engine(engine_store::Out::Switch));
    assert_eq!(store.switch_requests, 0);
    assert!(matches!(store.answers().as_slice(), [Up::Root(root::Event::Store(engine_store::In::Switched))]));
    assert!(!store.loading());
}
#[test]
fn start_pages_delay_and_a_crash_discards_their_flights() {
    let mut store = starting_snapshot();
    store.fault(Fault::SlowPages { by: PAGE_DELAY });
    store.pause_pages(true);
    store.ask(StoreOut::ConnectorLoad { number: 1, after: None });
    assert!(store.answers().is_empty());
    assert!(store.loading());
    store.crash(CRASH_SEED).expect("crash readback");
    assert!(!store.loading());
    drain_open_reports(&mut store);
    store.pause_pages(false);
    for _ in 0..PAGE_DELAY {
        let _commit = store.tick(false).expect("read-only tick");
    }
    assert!(store.answers().is_empty(), "crashed pages never reappear");
    store.fault(Fault::SlowPages { by: 0 });
    check_start_snapshot(&mut store);
}
