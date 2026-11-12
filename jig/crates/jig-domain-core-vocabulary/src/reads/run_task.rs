//! Agent task read values (jig/docs/design/domain/engine/executors.md, 3).
use super::{TaskText, TaskTextField};
use crate::Limits;
use alloc::boxed::Box;
use core::mem::{size_of, size_of_val};
use jig_domain_tasks as tasks;
use skein_lib::length::len_u64;

/// The application-named task text and answer room (domain/engine/executors.md, 3).
#[derive(Clone, Copy, Hash, Debug)]
pub struct Read {
    pub task: u64,
    pub field: TaskTextField,
    pub from: u64,
    pub room: u32,
    /// The specification's one retry has requested requester-chain evidence.
    pub joined: bool,
}
/// A task's lifecycle without result payloads or live-run secrets.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskState {
    /// Waiting for prerequisites or holdings.
    Waiting,
    /// An executor's current activation.
    Active(tasks::Active),
    /// Closing through its obligations.
    Closing(tasks::Stage),
    /// Held pending a decision.
    Held,
    /// Retained after ending.
    Ended(tasks::Status),
}
/// Current independent hold revision and the connector's bounded account.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskHold {
    pub revision: u64,
    pub reason: tasks::Hold,
    pub words: Option<Box<[u8]>>,
}
/// Bounded source facts needed to reconstruct or amend related work.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskRead {
    pub task: u64,
    pub state: TaskState,
    pub spec_revision: u64,
    pub dependencies: Box<[u64]>,
    /// Durable unanswered credits, bounded by `tasks.inbox_messages`.
    pub questions: Box<[tasks::QuestionCredit]>,
    pub words: TaskText,
    /// Verdict selected by a result read; offsets still count only reason bytes.
    pub verdict_code: Option<u32>,
    /// A result read of a live task, answered with its specification (domain/engine/executors.md, 3).
    pub result_pending: bool,
    pub hold: Option<TaskHold>,
}
/// Heap retained by one successful read, excluding its inline carrier.
#[must_use]
pub fn owned_bytes(read: &TaskRead) -> u64 {
    let mut bytes = len_u64(read.words.bytes.len());
    bytes = bytes.saturating_add(u64::try_from(size_of_val(read.dependencies.as_ref())).expect("bounded dependencies"));
    bytes = bytes.saturating_add(u64::try_from(size_of_val(read.questions.as_ref())).expect("bounded questions"));
    if let Some(TaskHold { words: Some(words), .. }) = &read.hold {
        bytes = bytes.saturating_add(len_u64(words.len()));
    }
    bytes
}
/// The most a task read's carrier and metadata take, without its words.
#[must_use]
pub fn answer_carrier_bound(limits: &Limits) -> Option<u64> {
    u64::try_from(size_of::<Result<TaskRead, tasks::Refusal>>())
        .ok()?
        .checked_add(u64::from(limits.tasks.dependencies).checked_mul(u64::try_from(size_of::<u64>()).ok()?)?)?
        .checked_add(
            u64::from(limits.tasks.inbox_messages)
                .checked_mul(u64::try_from(size_of::<tasks::QuestionCredit>()).ok()?)?,
        )?
        .checked_add(u64::from(tasks::HOLD_ACCOUNT_BYTES))
}
/// Maximum task-read carrier and payload under the admitted limits.
#[must_use]
pub fn answer_bound(limits: &Limits) -> Option<u64> {
    answer_carrier_bound(limits)?.checked_add(u64::from(limits.call_answer_bytes))
}
