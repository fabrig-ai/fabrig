//! Instructions and offered kinds chosen by run reasons (jig/docs/design/domain/tasks.md, 8.6;
//! jig/docs/design/domain/engine.md, 7.1).
use crate::{Charter, Core};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Writer};

/// A preparing run keeps its selected parts while every newer word is kept by them.
pub(crate) fn preparation_current(core: &Core, context: &tasks::RunContext) -> bool {
    let Some(task) = core.tasks.task(context.task) else { return false };
    if task.phase != tasks::Phase::Active(tasks::Active::Preparing) {
        return false;
    }
    if core.tasks.run_message(context.task) == Some(context.last_message) {
        return true;
    }
    let offered = match task.executor {
        tasks::Executor::Agent { .. } => {
            offered_kinds(core.charter_of(context.task), task.instructions.is_some(), context.reasons)
        }
        tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } | tasks::Executor::Person(_) => {
            tasks::Reasons::ALL
        }
    };
    let Some(current) = core.tasks.run_context(context.task) else { return false };
    for word in &current.inbox {
        if word.number > context.last_message && tasks::offered(word, offered) {
            return false;
        }
    }
    true
}

/// Compare fresh preparation facts after excluding newer kept words and only the reasons
/// they add. All other changes still abandon the prepared snapshot before admission.
pub(crate) fn preparation_facts(core: &Core, context: &tasks::RunContext) -> Option<Box<tasks::RunContext>> {
    if !preparation_current(core, context) {
        return None;
    }
    let mut current = core.tasks.run_context(context.task)?;
    let mut inbox = List::with_capacity(u32::try_from(current.inbox.len()).expect("bounded preparation inbox"));
    let mut kept_reasons = tasks::Reasons::EMPTY;
    for word in current.inbox {
        if word.number > context.last_message {
            kept_reasons = kept_reasons.union(tasks::Reasons::of(&word.kind));
        } else {
            inbox.push(word).expect("current inbox counted");
        }
    }
    current.inbox = inbox.into_boxed();
    if current.last_message > context.last_message {
        current.last_message = context.last_message;
    }
    if current.reasons.contains(context.reasons)
        && current.reasons.union(kept_reasons) == context.reasons.union(kept_reasons)
    {
        current.reasons = context.reasons;
    }
    if current.as_ref() != context {
        return None;
    }
    Some(current)
}

/// The task's own instructions, or its charter's followed by the parts its reasons meet.
pub(crate) fn run_instructions(charter: &Charter, own: Option<&[u8]>, reasons: tasks::Reasons) -> Box<[u8]> {
    if let Some(own) = own {
        return own.into();
    }
    let mut length = charter.policy.instructions.len();
    for part in &charter.parts {
        if part.on.meets(reasons) {
            length = length
                .checked_add(2)
                .expect("checked charter joining blank line")
                .checked_add(part.instructions.len())
                .expect("checked charter instructions");
        }
    }
    let mut instructions = Writer::new(length);
    instructions.put(&charter.policy.instructions).expect("charter instructions counted");
    for part in &charter.parts {
        if part.on.meets(reasons) {
            instructions.put(b"\n\n").expect("joining blank line counted");
            instructions.put(&part.instructions).expect("part instructions counted");
        }
    }
    instructions.finish()
}

/// Kinds beside the always-offered: every kind, or the selected parts' `on` without `first`.
pub(crate) fn offered_kinds(charter: &Charter, own: bool, reasons: tasks::Reasons) -> tasks::Reasons {
    if own || charter.parts.is_empty() {
        return tasks::Reasons::ALL;
    }
    let mut offered = tasks::Reasons::EMPTY;
    for part in &charter.parts {
        if part.on.meets(reasons) {
            offered = offered.union(part.on);
        }
    }
    tasks::Reasons(offered.0 & !tasks::Reasons::FIRST.0)
}
