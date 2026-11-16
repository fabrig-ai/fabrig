//! Domain rows and external limits for store fixtures.
use super::{DEPLOYMENT, KEY_LIMIT, calls, core_rows, identity, note_entry, note_line, task_history, task_record};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{CoreRecord, Record};
use skein_lib::Duration;

pub const OTHER_DEPLOYMENT: [u8; 16] = [9; 16];
pub const ALICE: u64 = 11;
pub const LIVE: u64 = 21;
pub const ENDED: u64 = 22;
pub const PROJECT: u32 = 3;

pub fn kv_limits() -> skein_kv::Limits {
    skein_kv::Limits {
        key: KEY_LIMIT,
        value: 16_777_216,
        ops: 16,
        commit: 134_217_728,
        queued: 2,
        queued_bytes: 268_435_456,
        budget: 16_384,
        segment: 268_435_456,
        snapshot_after: 134_217_728,
        chunk: 16_778_260,
        page: skein_kv::Page { rows: 2, bytes: 16_778_240 },
        record: 16_777_216,
        log: 1 << 40,
        logs: 16,
        deadline: Duration::from_nanos(1_000_000_000),
    }
}

pub fn deployment_record(deployment: [u8; 16], commits: u64) -> Record {
    let mut header = core_rows::deployment();
    header.id = deployment;
    header.commits = commits;
    Record::Core(CoreRecord::Deployment(header))
}

pub fn header_record(commits: u64) -> Record {
    deployment_record(DEPLOYMENT, commits)
}

pub fn person(number: u64) -> Record {
    Record::People(jig_domain_people::Stored::Person { number, identity: identity() })
}

pub fn live_task(number: u64) -> Record {
    let mut row = task_record();
    row.number = number;
    Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(row)))
}

pub fn cold_task(row: &jig_domain_tasks::TaskRecord) -> Record {
    Record::Tasks(jig_domain_tasks::Stored::LiveCold {
        number: row.number,
        spec: Box::new(row.spec.clone()),
        contract: row.contract.clone(),
        authority: row.authority.clone(),
        instructions: row.instructions.clone(),
    })
}

pub fn ended_task(number: u64) -> Record {
    let mut row = task_record();
    row.number = number;
    row.settings = jig_domain_tasks::SettingsRevision { task: 5, run: None };
    row.phase = jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Report {
        words: Box::from(b"result".as_slice()),
    }));
    Record::Tasks(jig_domain_tasks::Stored::Ended(Box::new(row)))
}

pub fn history(task: u64, revision: u64) -> Record {
    let mut row = task_history();
    row.task = task;
    row.revision = revision;
    Record::Tasks(jig_domain_tasks::Stored::History(row))
}

pub fn run_proof(task: u64) -> Record {
    let mut row = core_rows::run();
    row.task = task;
    Record::Core(CoreRecord::RunProof(row))
}

pub fn call(task: u64, completion: u32) -> Record {
    let CoreRecord::Call(mut row) = calls::call(jig_domain_core_vocabulary::CallPart::Controlled) else {
        panic!("call fixture");
    };
    row.key.task = task;
    row.key.completion = completion;
    Record::Core(CoreRecord::Call(row))
}

pub fn notes_entry(name: u64) -> Record {
    let mut row = note_entry();
    row.name = name;
    Record::Notes(jig_domain_notes::Record::Entry(row))
}

pub fn result_index(party: u64, position: u64, task: u64) -> Record {
    Record::Tasks(jig_domain_tasks::Stored::Result(jig_domain_tasks::ResultRow { party, position, task }))
}

pub fn notes_line(scope: &jig_domain_notes::Scope, name: u64) -> Record {
    let mut row = note_line();
    row.scope = scope.clone();
    row.name = name;
    Record::Notes(jig_domain_notes::Record::Line(row))
}

pub fn turn(task: u64, attempt: u64, number: u32, bytes: &[u8]) -> jig_domain_core_vocabulary::TurnRecord {
    let mut row = core_rows::turn();
    row.task = task;
    row.attempt = attempt;
    row.turn = number;
    row.transcript = Box::from(bytes);
    row.opens_window = false;
    row
}

pub fn window_turn(task: u64, attempt: u64, number: u32, bytes: &[u8]) -> jig_domain_core_vocabulary::TurnRecord {
    let mut row = turn(task, attempt, number, bytes);
    row.opens_window = true;
    row
}
