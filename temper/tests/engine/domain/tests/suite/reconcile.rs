//! Raw rows exercise a plain restart: temper has no fixture patch set that
//! makes a pending generation yet. The old run stays unadopted on refusal.
use temper_engine_domain::{self as engine, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

const ABSENT_TASK: u64 = 999;
const ABSENT_ENTRY: u64 = 777;
const ABSENT_REPOSITORY: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 404 };
// Same full-restart horizon as switch.rs.
const STEPS: u32 = 256;

fn restart_with(row: forge::Stored) -> Driver {
    restart_with_rows(Box::new([row]))
}
fn restart_with_rows(rows: Box<[forge::Stored]>) -> Driver {
    let (running, _) = Driver::chat();
    let mut store: Store = running.store;
    store.seed(rows.into_vec().into_iter().map(|row| Record::Forge { row: Box::new(row) }).collect(), Box::new([]));
    store.crash(9761);
    let mut driver = Driver::cold(store, walking::config(91), &walking::limits());
    for _ in 0..STEPS {
        driver.advance(true);
    }
    driver
}
fn refuses(row: forge::Stored, first: forge::RecordKind) {
    let key = engine::Key::Forge(forge::stored_key(&row));
    let driver = restart_with(row.clone());
    assert!(driver.stopped, "unresolved row refuses the start");
    assert_eq!(driver.root.forge_restore_failure(), Some(forge::Unresolved { records: 1, first }));
    assert!(driver.switches.is_empty(), "no switch asked");
    assert!(!driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)), "no mark");
    assert!(!driver.root.connectors_restored(), "no adoption stage");
    assert!(driver.delivered.is_empty(), "no run adopted or delivered");
    assert_eq!(driver.store.rows.get(&key), Some(&Record::Forge { row: Box::new(row) }), "bad row retained");
}
#[test]
fn a_forge_record_naming_a_task_the_restart_lacks_refuses_before_the_switch() {
    refuses(forge::Stored::Names { task: ABSENT_TASK, resources: Box::new([]) }, forge::RecordKind::Names);
}
#[test]
fn a_forge_record_naming_a_repository_the_forge_lacks_refuses_before_the_switch() {
    refuses(
        forge::Stored::Ci(forge::CiState {
            repository: ABSENT_REPOSITORY,
            head: [3; 32],
            status: skein_forge::api::Ci::Passed,
        }),
        forge::RecordKind::Ci,
    );
}
#[test]
fn a_flight_whose_entry_the_forge_lacks_refuses_before_the_switch() {
    refuses(forge::Stored::Flight { task: ABSENT_TASK, entry: ABSENT_ENTRY }, forge::RecordKind::Flight);
}
#[test]
fn landing_history_outlives_its_task_and_the_start_goes_on() {
    let driver = restart_with(forge::Stored::Landed { commit: [9; 32], task: ABSENT_TASK });
    assert!(!driver.stopped, "historical task need not be live or retained");
    assert_eq!(driver.root.forge_restore_failure(), None);
    assert_eq!(driver.switches.len(), 1, "switch asked");
    assert!(driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)), "mark committed");
    assert!(driver.root.connectors_restored(), "adoption stage completed");
    assert!(driver.root.ready(), "start continues into decisions");
}

#[test]
fn unresolved_rows_count_once_and_keep_the_first_family() {
    let driver = restart_with_rows(Box::new([
        forge::Stored::Names {
            task: ABSENT_TASK,
            resources: Box::new([forge::Name {
                forge: ABSENT_REPOSITORY.forge,
                repository: ABSENT_REPOSITORY.repository,
                what: forge::What::Repository,
            }]),
        },
        forge::Stored::Flight { task: ABSENT_TASK, entry: ABSENT_ENTRY },
    ]));
    // Names fails both task and repository; it is still one unresolved row.
    assert_eq!(
        driver.root.forge_restore_failure(),
        Some(forge::Unresolved { records: 2, first: forge::RecordKind::Names })
    );
    assert!(driver.stopped);
    assert!(driver.switches.is_empty());
}
