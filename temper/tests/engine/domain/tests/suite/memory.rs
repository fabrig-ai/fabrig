use skein_lib::{Queue, Token, Wall};
use skein_world::domain::heap::Meter;
use temper_engine_domain::{
    self as root, Counters, Decision, Delivery, Journal, JournalOutput as Output, Record, TurnRecord, Write,
};
use temper_engine_domain_world::store::{HEADER, LIMITS};

// Measured peak: 1,480 bytes; 25% headroom, rounded up to two significant digits.
const PARTIAL_LOAD_HEAP_CEILING: u64 = 1_900;

#[test]
fn retired_and_abandoned_load_slots_and_a_partial_page_fit_without_payload_clones() {
    use root::{Range, loads};
    let slot = u32::try_from(core::mem::size_of::<Record>()).expect("small slot");
    let limits = loads::Limits {
        read_bytes: 65_536,
        loads: 2,
        rows: 4,
        bytes: slot * 2 + 8,
        reply_bytes: 4 * (slot + 4),
        transcript_bytes: 4,
    };
    let mut out = Queue::<loads::Request>::with_capacity(1);
    let meter = Meter::new();
    let mut domain = loads::Loads::new(&limits);
    let incoming: Box<[Record]> = (1..=4)
        .map(|turn| {
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(TurnRecord {
                task: 1,
                attempt: 1,
                turn,
                spent: 0,
                read: None,
                at: Wall::EPOCH,
                opens_window: false,
                transcript: vec![b'x'; 4].into_boxed_slice(),
            })))
        })
        .collect();
    let mut addresses = [core::ptr::null(); 4];
    for (address, row) in addresses.iter_mut().zip(incoming.iter()) {
        let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(row))) = row else {
            panic!("turn");
        };
        *address = row.transcript.as_ptr();
    }
    let first = loads::begin(
        &mut domain,
        Token::new(1),
        Range::Core(temper_engine_domain::CoreRange::Turns { task: 1, attempt: 1 }),
        None,
        4,
        &mut out,
    )
    .expect("first slot");
    drop(out.pop().expect("first IO"));
    let second = loads::begin(
        &mut domain,
        Token::new(2),
        Range::Core(temper_engine_domain::CoreRange::Deployment),
        None,
        1,
        &mut out,
    )
    .expect("second slot");
    drop(out.pop().expect("second IO"));
    loads::abandon(&mut domain, second);
    meter.start();
    loads::loaded(&mut domain, first, incoming, None, &mut out);
    let measured = meter.end();
    let loads::Request::Loaded { rows, cut, .. } = out.pop().expect("partial page") else {
        panic!("loaded");
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(cut.expect("cut").rows, 2);
    for (row, original) in rows.iter().zip(addresses) {
        let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(row))) = row else {
            panic!("turn");
        };
        assert_eq!(row.transcript.as_ptr(), original, "owned bytes moved directly");
    }
    drop(rows);
    meter.check(measured, PARTIAL_LOAD_HEAP_CEILING, &limits);
    assert!(
        loads::begin(
            &mut domain,
            Token::new(3),
            Range::Core(temper_engine_domain::CoreRange::Deployment),
            None,
            1,
            &mut out
        )
        .is_none(),
        "retired and abandoned both retain slots"
    );
    loads::reclaim(&mut domain);
    assert!(
        loads::begin(
            &mut domain,
            Token::new(3),
            Range::Core(temper_engine_domain::CoreRange::Deployment),
            None,
            1,
            &mut out
        )
        .is_some()
    );
    drop(out.pop().expect("reclaimed first slot"));
    loads::unloaded(&mut domain, second, &mut out);
    assert!(out.is_empty(), "abandoned terminal does not wake its old waiter");
}

// Measured peak: 733,184 bytes; 25% headroom, rounded up to two significant digits.
const PARTIAL_DELIVERY_HEAP_CEILING: u64 = 920_000;

#[test]
fn partial_delivery_transfer_never_allocates_a_second_shrinking_container() {
    let limits = root::JournalLimits {
        commits: 1,
        held: 1000,
        writes: 1,
        deliveries: 1000,
        transcript_bytes: 0,
        result_bytes: 0,
        run_bytes: 1,
    };
    let mut out = Queue::<Output>::with_capacity(1);
    let meter = Meter::new();
    let mut journal = Journal::uncapped(1, HEADER.commits);
    let mut counters = Counters::new(HEADER);
    let mut decision = Decision::decide(&mut journal, &limits);
    for task in 1_u64..1000 {
        decision
            .deliver(
                &limits,
                Delivery::Link {
                    channel: Token::new(7),
                    down: jig_domain_host::LinkDown::Acknowledge { run: Token::new(task), attempt: Token::new(1) },
                },
            )
            .expect("partial delivery bound");
    }
    meter.start();
    root::accept_routed(&mut journal, &mut counters, &limits, decision, true).expect("opened whole decision");
    if let Some(output) = root::commit(&mut journal, &limits) {
        out.push(output);
    }
    let measured = meter.end();
    assert!(out.is_empty(), "clean header and no writes");
    meter.check(measured, PARTIAL_DELIVERY_HEAP_CEILING, &limits);
    assert!(!journal.idle());
}

// Measured peak: 79,584 bytes; 25% headroom, rounded up to two significant digits.
const PARTIAL_WRITE_HEAP_CEILING: u64 = 100_000;

#[test]
fn partial_write_transfer_moves_values_without_shrinking_the_source() {
    let limits = root::JournalLimits {
        commits: 1,
        held: 1,
        writes: 128,
        deliveries: 1,
        transcript_bytes: 0,
        result_bytes: 0,
        run_bytes: 1,
    };
    let mut out = Queue::<Output>::with_capacity(1);
    let meter = Meter::new();
    let mut journal = Journal::uncapped(1, HEADER.commits);
    let mut counters = Counters::new(HEADER);
    let mut decision = Decision::decide(&mut journal, &limits);
    for turn in 1_u32..127 {
        decision
            .write(
                &limits,
                Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(
                    TurnRecord {
                        task: 1,
                        attempt: 1,
                        turn,
                        spent: 0,
                        read: None,
                        at: Wall::EPOCH,
                        opens_window: false,
                        transcript: Box::new([]),
                    },
                )))),
            )
            .expect("partial write bound");
    }
    meter.start();
    root::accept_routed(&mut journal, &mut counters, &limits, decision, true).expect("opened whole decision");
    if let Some(output) = root::commit(&mut journal, &limits) {
        out.push(output);
    }
    let measured = meter.end();
    let Output::Commit { writes, .. } = out.pop().expect("one commit") else {
        panic!("commit");
    };
    assert_eq!(writes.len(), 127);
    drop(writes);
    meter.check(measured, PARTIAL_WRITE_HEAP_CEILING, &limits);
}

// Measured peak: 10,456 bytes; 25% headroom, rounded up to two significant digits.
const FULL_DECISIONS_HEAP_CEILING: u64 = 14_000;

#[test]
fn full_decisions_and_maximum_held_results_fit_the_declared_root_journal_bound() {
    let mut out = Queue::<Output>::with_capacity(1);
    let meter = Meter::new();
    let mut journal = Journal::uncapped(1, HEADER.commits);
    let mut counters = Counters::new(HEADER);
    for commit in 1_u64..=u64::from(LIMITS.commits) {
        meter.start();
        let mut decision = Decision::decide(&mut journal, &LIMITS);
        for turn in 1..LIMITS.writes {
            decision
                .write(
                    &LIMITS,
                    Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(
                        TurnRecord {
                            task: commit,
                            attempt: 1,
                            turn,
                            spent: 0,
                            read: None,
                            at: Wall::EPOCH,
                            opens_window: false,
                            transcript: vec![b'x'; usize::try_from(LIMITS.transcript_bytes).expect("small transcript")]
                                .into_boxed_slice(),
                        },
                    )))),
                )
                .expect("maximum admitted turn");
        }
        // Replace a fully allocated value while preserving the decision's
        // key position; the incoming value and old value are both live first.
        decision
            .write(
                &LIMITS,
                Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(
                    TurnRecord {
                        task: commit,
                        attempt: 1,
                        turn: 1,
                        spent: 1,
                        read: None,
                        at: Wall::EPOCH,
                        opens_window: false,
                        transcript: vec![b'y'; usize::try_from(LIMITS.transcript_bytes).expect("small transcript")]
                            .into_boxed_slice(),
                    },
                )))),
            )
            .expect("bounded replacement");
        for task in 1_u64..=u64::from(LIMITS.deliveries) {
            decision
                .deliver(
                    &LIMITS,
                    Delivery::Result {
                        person: 1,
                        task,
                        words: vec![b'x'; usize::try_from(LIMITS.result_bytes).expect("small result")]
                            .into_boxed_slice(),
                    },
                )
                .expect("maximum admitted result");
        }
        root::accept_routed(&mut journal, &mut counters, &LIMITS, decision, true).expect("whole decision opened");
        if let Some(output) = root::commit(&mut journal, &LIMITS) {
            out.push(output);
        }
        let measured = meter.end();
        drop(out.pop().expect("one commit"));
        meter.check(measured, FULL_DECISIONS_HEAP_CEILING, &LIMITS);
    }
    assert!(
        journal.backlog() >= u64::from(root::default_watermark(&LIMITS)),
        "retained work reaches the root's watermark"
    );
    root::committed(&mut journal, u64::from(LIMITS.commits));
    for _ in 0..LIMITS.held {
        meter.start();
        root::resume(&mut journal, &mut out);
        let measured = meter.end();
        drop(out.pop().expect("one bounded ready result"));
        meter.check(measured, FULL_DECISIONS_HEAP_CEILING, &LIMITS);
    }
    assert!(journal.idle());
    assert_eq!(journal.backlog(), 0, "durable deliveries drain the backlog below the watermark");
}

// Measured peak: 261,085 bytes; 25% headroom, rounded up to two significant digits.
const WALKING_HEAP_CEILING: u64 = 330_000;

#[test]
fn real_root_state_and_complete_walking_handoffs_fit_the_declared_counted_bound() {
    use temper_engine_domain_world::walking::{Settings, World, limits};
    let limits = limits();
    let bound = WALKING_HEAP_CEILING;
    for settings in [Settings::calm(73), Settings::terminal(74)] {
        let meter = Meter::new();
        meter.start();
        let mut world = World::new(settings);
        world.run();
        let measured = meter.end();
        // The meter also sees the independent fake's durable map and script
        // observations. Their tiny footprint only makes this stricter: root
        // scratch queues and every real restored child are exercised together.
        meter.check(measured, bound, &limits);
    }
}

// Measured peak: 365,893 bytes; 25% headroom, rounded up to two significant digits.
const ESCALATION_HEAP_CEILING: u64 = 460_000;

#[test]
fn held_reasons_history_queries_and_real_escalation_handoffs_fit_counted_root_memory() {
    use temper_engine_domain_world::{escalation, escalation_referee::Story};
    let limits = escalation::limits();
    let bound = ESCALATION_HEAP_CEILING;
    for story in [Story::Release, Story::PassRelease, Story::RaceReject] {
        let meter = Meter::new();
        meter.start();
        let mut world = escalation::World::new(escalation::Settings {
            cut: escalation::Cut::Decision,
            commit_delay: 1,
            page_delay: 1,
            ..escalation::Settings::calm(9201, story)
        });
        world.run();
        let measured = meter.end();
        // Includes the fake's durable history and independent observations as
        // well as actual root query/context/journal and restored child copies.
        meter.check(measured, bound, &limits);
    }
}

// Measured peak: 1,349,074 bytes with numbered catalogue records and resolved
// charters in recovery snapshots; 25% headroom, rounded up to two significant digits.
const ROLE_RECOVERY_HEAP_CEILING: u64 = 1_700_000;

#[test]
fn candidate_rosters_waiting_snapshots_keyed_history_and_recovery_fit_counted_memory() {
    use temper_engine_domain_world::roles::{Base, Settings, limits, reroute_replayed};
    let limits = limits();
    let bound = ROLE_RECOVERY_HEAP_CEILING;
    let meter = Meter::new();
    meter.start();
    let world = reroute_replayed(
        Settings { cut: true, commit_delay: 1, page_delay: 1, ..Settings::calm(9311, Base::Requester) },
        1,
    );
    assert_eq!(world.referee.replacements, 1);
    let measured = meter.end();
    // Includes the fake store, immutable before-observers and exact replay
    // transcript in addition to real children and every root transient copy.
    meter.check(measured, bound, &limits);
}

// Measured peaks: 130,053 bytes at 26 watches; 506,965 at 208, including the
// driver and fake store. 25% headroom, rounded up to two significant digits.
const ONE_STREAM_WATCH_HEAP_CEILING: u64 = 170_000;
const EIGHT_STREAM_WATCH_HEAP_CEILING: u64 = 640_000;
const WATCH_ROOM_SEED: u64 = 91; // The direct driver's configuration.
const WATCH_ROOM_PROJECT: u32 = 1; // Walking's seeded project.
const FIRST_ROOM_WATCH: u64 = 10_000;
const SHIPPED_JOURNAL_DELIVERIES: u32 = 512; // temper-service's small profile.

fn watch_room_root_heap(streams: u32) -> u64 {
    let limits = root::Limits { streams, ..temper_engine_domain_world::walking::limits() };
    let meter = Meter::new();
    let root = root::Domain::new(temper_engine_domain_world::walking::config(WATCH_ROOM_SEED), &limits)
        .expect("watch-room configuration");
    let held = meter.held();
    drop(root);
    held
}

fn watch_room_query(label: u32) -> jig_domain_core::documents::Watched {
    use jig_domain_core::documents::{QueryOrder, TaskQuery, Watched};
    Watched::Query {
        query: TaskQuery {
            project: Some(WATCH_ROOM_PROJECT),
            requester: None,
            root: None,
            parent: None,
            label: Some(label),
            phase: None,
            executor: None,
            tracked: None,
            order: QueryOrder::Number,
        },
    }
}

#[test]
fn a_full_watch_room_fits_its_heap_ceiling() {
    assert_eq!(watch_room_root_heap(1), watch_room_root_heap(8), "unused watch room allocates nothing");
    for (streams, room, ceiling) in [(1, 26, ONE_STREAM_WATCH_HEAP_CEILING), (8, 208, EIGHT_STREAM_WATCH_HEAP_CEILING)]
    {
        for distinct in [false, true] {
            full_watch_room_peak(streams, room, distinct, ceiling);
        }
    }
}

fn full_watch_room_peak(streams: u32, room: u32, distinct: bool, ceiling: u64) {
    use jig_domain_core::documents::Window;
    use temper_engine_domain_world::walking;
    let mut limits = root::Limits { streams, ..walking::limits() };
    limits.journal.deliveries = SHIPPED_JOURNAL_DELIVERIES;
    limits.watermark = root::default_watermark(&limits.journal);
    let meter = Meter::new();
    let mut driver = super::watch_room::driver_with_limits(&limits);
    let session = driver.session();
    driver.delivered = Vec::new();
    driver.transactions = Vec::new();
    driver.immediate = Vec::new();
    for offset in 0..room {
        let watcher = Token::new(FIRST_ROOM_WATCH + u64::from(offset));
        driver.send(root::Event::Party(root::PartyInput::Watch {
            watcher,
            sign_in: session,
            subject: watch_room_query(if distinct { offset } else { 0 }),
            window: Window { rows: 1, after: None },
        }));
        driver.settle();
        driver.send(root::Event::Party(root::PartyInput::Delivered { watcher }));
        driver.settle();
        driver.delivered.clear();
    }
    driver.delivered = Vec::new();
    meter.start();
    driver.send(root::Event::Party(root::PartyInput::Period {
        project: WATCH_ROOM_PROJECT,
        period: 2,
        budget: 1000,
        next: None,
    }));
    // The eight-stream room has more deliveries than one direct-driver settlement batch.
    driver.pump();
    driver.settle();
    let measured = meter.end();
    let documents = driver.delivered.iter().filter(|value| matches!(value, root::Delivery::Document { .. })).count();
    assert_eq!(
        documents,
        usize::try_from(room).expect("watch room fits usize"),
        "a full room marked in one commit: {streams}, {distinct}"
    );
    assert_eq!(driver.transactions.len(), 1, "one commit marks the whole room");
    assert!(room <= limits.journal.deliveries, "the full room fits the journal's door");
    meter.check(measured, ceiling, &(streams, distinct));
}
