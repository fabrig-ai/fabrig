use super::{Fault, Store};
use jig_domain_core as core;
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up};
use std::collections::BTreeMap;

// Nine captured first pages exceed the route carrier's eight entries.
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
