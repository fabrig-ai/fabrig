//! Agent task reads use task relationships, never a person's visibility
//! (jig/docs/design/domain/engine.md, 7.3).
use super::{Destination, Loaded, Pending, Range, Scope, TaskText, TaskTextField};
use crate::{CallKey, Core, Held, Limits, Now, Request};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Id, Queue, ReplyTo};

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
    pub words: TaskText,
    pub hold: Option<TaskHold>,
}

fn related(core: &Core, caller: u64, row: &tasks::TaskRecord) -> bool {
    let Some(owner) = core.tasks.task(caller) else { return false };
    if owner.project != row.project {
        return false;
    }
    if row.requester == tasks::Party::Task(caller) {
        return true;
    }
    match row.phase {
        tasks::Phase::Ended(_) => return false,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {}
    }
    caller == row.number
        || owner.requester == tasks::Party::Task(row.number)
        || owner.delegates.contains(&row.number)
        || owner.references.contains(&row.number)
        || match row.escalation {
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(task), .. } => task == caller,
            tasks::Escalation::Waiting { .. }
            | tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Rejected { .. } => false,
        }
}

fn value(
    core: &Core,
    limits: &Limits,
    caller: u64,
    row: &tasks::TaskRecord,
    from: u64,
) -> Result<TaskRead, tasks::Refusal> {
    if !related(core, caller, row) {
        return Err(tasks::Refusal::Reference);
    }
    let words = super::task_text::chunk(
        row.number,
        TaskTextField::Specification,
        &row.spec.words,
        from,
        limits.documents.text_bytes,
    )
    .or(Err(tasks::Refusal::Unknown))?;
    let state = task_state(row);
    let hold = task_hold(limits, row)?;
    Ok(TaskRead {
        task: row.number,
        state,
        spec_revision: row.revisions.spec,
        dependencies: row.dependencies.clone(),
        words,
        hold,
    })
}

fn task_state(row: &tasks::TaskRecord) -> TaskState {
    match &row.phase {
        tasks::Phase::Waiting => TaskState::Waiting,
        tasks::Phase::Active(active) => TaskState::Active(*active),
        tasks::Phase::Closing(stage) => TaskState::Closing(stage.stage),
        tasks::Phase::Held { .. } => TaskState::Held,
        tasks::Phase::Ended(ending) => TaskState::Ended(match ending {
            tasks::Ending::Done(_) => tasks::Status::Done,
            tasks::Ending::Failed { .. } => tasks::Status::Failed,
            tasks::Ending::Cancelled { .. } => tasks::Status::Cancelled,
        }),
    }
}

fn task_hold(limits: &Limits, row: &tasks::TaskRecord) -> Result<Option<TaskHold>, tasks::Refusal> {
    Ok(match crate::documents::task_hold(limits, row) {
        Ok(Some(hold)) => Some(TaskHold {
            revision: hold.revision,
            reason: hold.reason,
            words: match hold.words {
                Some(text) => Some(text.bytes),
                None => None,
            },
        }),
        Ok(None) => None,
        Err(_) => return Err(tasks::Refusal::Busy),
    })
}

fn answer(to: ReplyTo, key: CallKey, value: Result<TaskRead, tasks::Refusal>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::TaskRead { to, key, answer: Box::new(value) })));
    out
}

pub(crate) fn run(
    core: &mut Core,
    env: &Env<Limits>,
    to: ReplyTo,
    key: CallKey,
    task: u64,
    from: u64,
) -> Queue<Request> {
    if let Some(part) = core.authorize_tool(key, crate::ToolKind::Read) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(Now::EffectAnswer { to, key, part })));
        return out;
    }
    if let Some(row) = core.tasks.task(task) {
        return answer(to, key, value(core, &env.limits, key.task, row, from));
    }
    load(core, env, to, key, task, from)
}

fn load(core: &mut Core, env: &Env<Limits>, to: ReplyTo, key: CallKey, task: u64, from: u64) -> Queue<Request> {
    if core.reads.len() >= env.limits.load_slots {
        return answer(to, key, Err(tasks::Refusal::Busy));
    }
    let range = Range {
        party: 0,
        backwards: false,
        scope: Scope::TaskText { task, field: TaskTextField::Specification, from },
        after: None,
        rows: 1,
        bytes: env.limits.load_bytes,
    };
    let pending = Pending {
        destination: Destination::RunTask { to, key, task, from },
        sign_in: 0,
        party: 0,
        range: range.clone(),
        tasks: None,
    };
    let load = core.reads.insert(pending).expect("read room checked").token();
    let mut out = Queue::uncapped();
    out.push(Request::Held(Box::new(Held::ReadLoad { load, range: Box::new(range) })));
    out
}

pub(super) fn loaded(core: &mut Core, env: &Env<Limits>, name: Id<Pending>, result: Loaded) -> Queue<Request> {
    let pending = core.reads.get_mut(name).expect("pending task read");
    let Destination::RunTask { to, key, task, from } = core::mem::replace(&mut pending.destination, Destination::Done)
    else {
        unreachable!("task read")
    };
    core.reads.retire(name);
    if !core.current_proof(key.task, key.attempt) {
        return Queue::uncapped();
    }
    if let Some(part) = core.authorize_tool(key, crate::ToolKind::Read) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(Now::EffectAnswer { to, key, part })));
        return out;
    }
    match result {
        Loaded::Page { rows, .. } => {
            for row in rows {
                if let crate::Record::Tasks(tasks::Stored::Ended(owner)) = row.value
                    && owner.number == task
                {
                    return answer(to, key, value(core, &env.limits, key.task, &owner, from));
                }
            }
            answer(to, key, Err(tasks::Refusal::Reference))
        }
        Loaded::Busy | Loaded::TooLinked { .. } => answer(to, key, Err(tasks::Refusal::Busy)),
        Loaded::Gone | Loaded::TurnProblem { .. } => answer(to, key, Err(tasks::Refusal::Unknown)),
    }
}

/// Heap retained by one successful read, excluding its inline carrier.
#[must_use]
pub fn owned_bytes(read: &TaskRead) -> u64 {
    let mut bytes = u64::try_from(read.words.bytes.len()).expect("bounded words");
    bytes = bytes.saturating_add(u64::try_from(size_of_val(read.dependencies.as_ref())).expect("bounded dependencies"));
    if let Some(TaskHold { words: Some(words), .. }) = &read.hold {
        bytes = bytes.saturating_add(u64::try_from(words.len()).expect("bounded hold words"));
    }
    bytes
}

/// Maximum task-read carrier and payload under the admitted task and document limits.
#[must_use]
pub fn answer_bound(limits: &Limits) -> Option<u64> {
    u64::try_from(size_of::<Result<TaskRead, tasks::Refusal>>())
        .ok()?
        .checked_add(u64::from(limits.tasks.dependencies).checked_mul(u64::try_from(size_of::<u64>()).ok()?)?)?
        .checked_add(u64::from(limits.documents.text_bytes))?
        .checked_add(u64::from(tasks::HOLD_ACCOUNT_BYTES))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_reads_preserve_connector_words_and_require_a_relationship() {
        let (core, mut limits) = crate::tests::make_tree();
        limits.documents.text_bytes = 4;
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.spec.words = b"a\xc3\xa9bcdef".as_slice().into();
        row.phase = tasks::Phase::Held {
            was: tasks::Was::Waiting,
            why: tasks::Hold::Procedure,
            source: Some(Box::new(tasks::HoldSource::Account {
                connector: 2,
                entry: None,
                words: b"missing grant; amend authority and release".as_slice().into(),
            })),
        };
        row.revisions.hold = 7;
        let first = value(&core, &limits, 1, &row, 0).expect("direct delegate");
        assert_eq!(first.words.bytes.as_ref(), b"a\xc3\xa9b");
        assert_eq!(first.words.next, Some(4));
        let hold = first.hold.expect("connector hold");
        assert_eq!(hold.revision, 7);
        assert_eq!(hold.words.as_deref(), Some(b"missing grant; amend authority and release".as_slice()));
        assert!(value(&core, &limits, 1, &row, 2).is_err(), "continuation cannot split UTF-8");
        assert_eq!(value(&core, &limits, 3, &row, 0), Err(tasks::Refusal::Reference));
        row.escalation = tasks::Escalation::Waiting {
            revision: 7,
            holder: tasks::EscalationHolder::Task(3),
            entry: 1,
            since: skein_lib::Wall::EPOCH,
        };
        assert!(value(&core, &limits, 3, &row, 4).is_ok(), "escalation holder can read its subject");
        row.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: b"ended".as_slice().into() });
        assert!(value(&core, &limits, 1, &row, 4).is_ok(), "retained requester remains authorized");
        assert_eq!(value(&core, &limits, 3, &row, 4), Err(tasks::Refusal::Reference));
    }
}
