//! Task documents without a page harness.
use super::{FORGE, PROJECT, acts, limits};
use jig_client::contract::{self as c, DocumentText};
use skein_lib::{List, Wall};
pub const TASK: u64 = 9;
pub const ATTEMPT: u64 = 23;
pub const POSITION: u64 = 3;
pub const LABEL: u32 = 7;
const ONE: u32 = 1;
const EXCHANGES: u32 = 8;
pub fn task() -> c::Task {
    use jig_client_protocol_wire::v24 as w;
    let mut record = jig_client_test_records::task_record().into_parts();
    record.chip = jig_client_test_records::owner_waiting_chip_record();
    let record = w::Task::new(&w::CEILINGS, record).expect("checked task fixture");
    let link = jig_client_protocol_link::Limits::new(limits().client.domain, EXCHANGES).expect("link limits");
    let mut task = jig_client_protocol_link::translate::documents::task(record, &link).expect("task document");
    task.chip.task = TASK;
    task.label = LABEL;
    task.project = PROJECT;
    task.activity = POSITION;
    task.acts = List::with_capacity(ONE);
    task.acts.push(c::Offered { act: c::ActKind::Stop, admission: c::Admission::Allowed, effect: None }).expect("stop");
    phase(&mut task, c::ChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT }));
    task
}
pub fn phase(task: &mut c::Task, phase: c::ChipPhase) {
    task.chip.phase = phase.compact();
    let c::ChipPhaseFacts { phase: target, .. } = task.chip.details.as_mut();
    *target = phase;
}
pub const WAKE_SUMMARY: &[u8] =
    b"Begin the work your brief describes.\n\n2: Result from task 2:\n1d0b14e4db018fed000000000000000300000000";
pub fn delegate_call(task: u64) -> c::Call {
    let mut objects = List::with_capacity(ONE);
    objects.push(c::ObjectKey::Task(task)).expect("delegated task");
    c::Call {
        number: task,
        tool: acts::text(b"delegate"),
        input: acts::text(b"{}"),
        outcome: Some(acts::text(b"Delegated tasks: 2, 3")),
        failed: false,
        objects,
        parts: List::with_capacity(0),
    }
}
pub fn result(task: &c::Task, ending: c::Ending, value: Option<c::ResultValue>) -> c::Result {
    c::Result {
        task: task.chip.clone(),
        ending,
        value,
        at: Some(Wall::EPOCH),
        details: None,
        acts: List::with_capacity(0),
    }
}
pub fn document_text(bytes: &[u8]) -> DocumentText {
    let bound = u32::try_from(bytes.len()).expect("fixture text fits");
    DocumentText::complete(c::Text::new(Box::from(bytes), bound).expect("fixture text"))
}
pub fn change() -> c::Task {
    let mut task = task();
    task.hold = None;
    task.result = None;
    task.result_contract = c::TaskContractDocument::Change { connector: FORGE, kind: 1, words: 64 };
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.ending = None;
    task
}
pub fn change_value(words: &[u8]) -> c::ResultValue {
    c::ResultValue::Change {
        kind: 1,
        part: c::Part { connector: FORGE, kind: c::PartKind::Result, key: c::PartKey::Number(TASK) },
        words: document_text(words),
    }
}
