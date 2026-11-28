//! Instructions and offered kinds chosen by run reasons (jig/docs/design/domain/tasks/messages.md, 6;
//! jig/docs/design/domain/engine/executors.md, 1).
use crate::{
    Charter, Core, ProjectCharter,
    settings::{Overlay, overlays::instruction_join},
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Writer};

/// A preparing run keeps its parts only while no new reason or offered word arrives.
pub(crate) fn preparation_current(core: &Core, context: &tasks::RunContext) -> bool {
    let Some(task) = core.tasks.task(context.task) else { return false };
    if task.phase != tasks::Phase::Active(tasks::Active::Preparing) {
        return false;
    }
    let Some(current) = core.tasks.run_context(context.task) else { return false };
    if !context.reasons.contains(current.reasons) {
        return false;
    }
    if current.last_message == context.last_message {
        return true;
    }
    let offered = preparation_offered(core, context);
    for word in &current.inbox {
        if word.number > context.last_message && tasks::offered(word, offered) {
            return false;
        }
    }
    true
}

/// The offered kinds of the prepared snapshot, shared by freshness and sent-word routing.
pub(crate) fn preparation_offered(core: &Core, context: &tasks::RunContext) -> tasks::Reasons {
    let task = core.tasks.task(context.task).expect("preparing context names a live task");
    match task.executor {
        tasks::Executor::Agent { .. } => offered_kinds(
            core.charter_of(context.task),
            project_words(core, context.task),
            task.instructions.is_some(),
            context.reasons,
            task.flags.ended_by == tasks::EndedBy::Standing,
        ),
        tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } | tasks::Executor::Person(_) => {
            tasks::Reasons::ALL
        }
    }
}

/// Compare fresh preparation facts after excluding newer words that add no reason and
/// are kept by the prepared parts. Other changes abandon the snapshot before admission.
pub(crate) fn preparation_facts(core: &Core, context: &tasks::RunContext) -> Option<Box<tasks::RunContext>> {
    if !preparation_current(core, context) {
        return None;
    }
    let mut current = core.tasks.run_context(context.task)?;
    let mut inbox = List::with_capacity(u32::try_from(current.inbox.len()).expect("bounded preparation inbox"));
    for word in current.inbox {
        if word.number <= context.last_message {
            inbox.push(word).expect("current inbox counted");
        }
    }
    current.inbox = inbox.into_boxed();
    if current.last_message > context.last_message {
        current.last_message = context.last_message;
    }
    if current.as_ref() != context {
        return None;
    }
    Some(current)
}

/// The held project's words and this attempt's overlay, only for run preparation.
pub(crate) fn project_words(core: &Core, task: u64) -> Option<(&ProjectCharter, Option<&Overlay>)> {
    let row = core.tasks.task(task)?;
    let scope = row.scope.as_deref()?;
    let tasks::Executor::Agent { charter } = row.executor else { unreachable!("bound agent task") };
    let words = core.project_charter_words(row.project, row.charters, scope).expect("held charter binding");
    let overlay = core.overlay_at(charter, row.settings.run.unwrap_or(row.settings.task));
    Some((words, overlay))
}

/// The task's own instructions, or its charter's followed by the parts its reasons meet.
pub(crate) fn run_instructions(
    charter: &Charter,
    project: Option<(&ProjectCharter, Option<&Overlay>)>,
    own: Option<&[u8]>,
    reasons: tasks::Reasons,
    standing: bool,
) -> Box<[u8]> {
    if let Some(own) = own {
        return own.into();
    }
    let (base, added, parts, more) = match project {
        Some((project, Some(overlay))) => (
            project.instructions.as_ref(),
            overlay.instructions.as_ref(),
            project.parts.as_ref(),
            overlay.parts.as_ref(),
        ),
        Some((project, None)) => (project.instructions.as_ref(), &[][..], project.parts.as_ref(), &[][..]),
        None => (charter.policy.instructions.as_ref(), &[][..], charter.parts.as_ref(), &[][..]),
    };
    let (parts, more) = if charter.policy.resume && !standing { (&[][..], &[][..]) } else { (parts, more) };
    let join = instruction_join(base, added);
    let mut length = base.len().checked_add(join.len()).expect("joining instructions");
    length = length.checked_add(added.len()).expect("checked charter instructions");
    for part in parts.iter().chain(more) {
        if part.on.meets(reasons) {
            length = length
                .checked_add(2)
                .expect("checked charter joining blank line")
                .checked_add(part.instructions.len())
                .expect("checked charter instructions");
        }
    }
    let mut instructions = Writer::new(length);
    instructions.put(base).expect("charter instructions counted");
    instructions.put(join).expect("joining blank line counted");
    instructions.put(added).expect("overlay instructions counted");
    for part in parts.iter().chain(more) {
        if part.on.meets(reasons) {
            instructions.put(b"\n\n").expect("joining blank line counted");
            instructions.put(&part.instructions).expect("part instructions counted");
        }
    }
    instructions.finish()
}

/// Kinds beside the always-offered: every kind, or the selected parts' `on` without `first`.
pub(crate) fn offered_kinds(
    charter: &Charter,
    project: Option<(&ProjectCharter, Option<&Overlay>)>,
    own: bool,
    reasons: tasks::Reasons,
    standing: bool,
) -> tasks::Reasons {
    let (parts, more) = match project {
        Some((project, Some(overlay))) => (project.parts.as_ref(), overlay.parts.as_ref()),
        Some((project, None)) => (project.parts.as_ref(), &[][..]),
        None => (charter.parts.as_ref(), &[][..]),
    };
    if own || parts.is_empty() && more.is_empty() || charter.policy.resume && !standing {
        return tasks::Reasons::ALL;
    }
    let mut offered = tasks::Reasons::EMPTY;
    for part in parts.iter().chain(more) {
        if part.on.meets(reasons) {
            offered = offered.union(part.on);
        }
    }
    tasks::Reasons(offered.0 & !tasks::Reasons::FIRST.0)
}
