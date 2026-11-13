//! Required forge links assert before the restore terminal and switch.
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
    let (mut running, _) = Driver::chat();
    let mut store: Store = running.take_store();
    store.seed(rows.into_vec().into_iter().map(|row| Record::Forge { row: Box::new(row) }).collect(), Box::new([]));
    store.crash(9761);
    Driver::cold(store, walking::config(91), &walking::limits())
}
fn asserts_before_switch(row: forge::Stored, expected: forge::MissingLink) {
    let key = engine::Key::Forge(forge::stored_key(&row));
    let mut driver = restart_with(row.clone());
    driver.broken("fixture removes a required forge link and catches the startup assertion");
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for _ in 0..STEPS {
            driver.advance(true);
        }
    }))
    .expect_err("missing required link is a writer bug");
    assert_eq!(driver.root.forge_missing_link(), Some(expected));
    assert_eq!(driver.root.start_stop(), None, "no start refusal recorded");
    assert!(driver.switches.is_empty(), "no switch asked");
    assert!(!driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)), "no mark");
    assert_eq!(driver.root.start_phase(), Some(jig_domain_core::start::Phase::Link), "no adoption stage");
    assert!(!driver.root.ready(), "no decisions");
    assert!(driver.delivered.is_empty(), "no run adopted or delivered");
    assert!(driver.immediate.is_empty(), "no external requests");
    while let Some(request) = driver.out.pop() {
        assert!(
            !matches!(
                request,
                engine::Request::Forge { .. } | engine::Request::Connect { .. } | engine::Request::Worker(_)
            ),
            "no external wires before link: {request:?}"
        );
    }
    assert_eq!(driver.store.rows.get(&key), Some(&Record::Forge { row: Box::new(row) }), "row retained");
}
#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn a_missing_task_asserts_before_the_switch() {
    asserts_before_switch(
        forge::Stored::Call {
            key: forge::NamedCall { task: ABSENT_TASK, attempt: 1, completion: 1, position: 1 },
            answer: forge::CallPayload::Refused(skein_forge::api::Error::Refused),
        },
        forge::MissingLink {
            kind: forge::RecordKind::Call,
            key: forge::Key::Call(forge::NamedCall { task: ABSENT_TASK, attempt: 1, completion: 1, position: 1 }),
            need: forge::Need::Task { task: ABSENT_TASK, retired: None },
        },
    );
}
#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn a_missing_repository_asserts_before_the_switch() {
    asserts_before_switch(
        forge::Stored::Ci(forge::CiState {
            repository: ABSENT_REPOSITORY,
            head: [3; 32],
            status: skein_forge::api::Ci::Passed,
        }),
        forge::MissingLink {
            kind: forge::RecordKind::Ci,
            key: forge::Key::Ci { repository: ABSENT_REPOSITORY, head: [3; 32] },
            need: forge::Need::Repository(ABSENT_REPOSITORY),
        },
    );
}
#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn a_missing_flight_entry_asserts_before_the_switch() {
    asserts_before_switch(
        forge::Stored::Flight { task: ABSENT_TASK, entry: ABSENT_ENTRY },
        forge::MissingLink {
            kind: forge::RecordKind::Flight,
            key: forge::Key::Flight { task: ABSENT_TASK, entry: ABSENT_ENTRY },
            need: forge::Need::Entry { task: ABSENT_TASK, entry: ABSENT_ENTRY },
        },
    );
}
#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn a_missing_configured_forge_asserts_before_the_switch() {
    asserts_before_switch(
        forge::Stored::Writer(skein_forge_client::Writer { forge: 99, author: 7 }),
        forge::MissingLink {
            kind: forge::RecordKind::Writer,
            key: forge::Key::Writer(99),
            need: forge::Need::Forge(99),
        },
    );
}
#[test]
fn landing_history_outlives_its_task_and_the_start_goes_on() {
    let mut driver = restart_with(forge::Stored::Landed { commit: [9; 32], task: ABSENT_TASK });
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert!(!driver.stopped, "historical task need not be live or retained");
    assert_eq!(driver.root.start_stop(), None);
    assert!(driver.switches.is_empty(), "no pending generation needs a store switch");
    assert!(driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)), "mark committed");
    assert!(driver.root.start_phase().is_none(), "adoption stage completed");
    assert!(driver.root.ready(), "start continues into decisions");
}
