//! Immutable routing survives a subject's retirement; current offers do not.
use super::{Failed, chip_project, settings_history};
use crate::jig::{
    call::{Call, Value, number, value},
    render, rows,
};
use jig_client::contract as c;
use skein_lib::List;

pub(super) enum Scope {
    Project(u32),
    Task(u64),
}
fn expected(call: &Call) -> c::ObjectKey {
    let task = number(call, 0).unwrap_or(0);
    match call.tool {
        rows::DECIDE_AT => {
            let Some(Value::Proposal(key)) = value(call, 0) else { unreachable!("proposal argument") };
            c::ObjectKey::Proposal(*key)
        }
        rows::TASK_DECIDE_AT => c::ObjectKey::Escalation(task),
        rows::ANSWER_AT => c::ObjectKey::Question { task, message: number(call, 1).expect("question") },
        _ => c::ObjectKey::Task(task),
    }
}
pub(super) fn scope(call: &Call, document: &c::ReadValue) -> Result<Scope, Failed> {
    if let c::ReadValue::Historical(history) = document {
        if !history.matches(expected(call)) {
            return Err(Failed::Fixed(b"unexpected act subject"));
        }
        if settings_history(call, document) {
            return Ok(Scope::Project(0));
        }
        return Ok(match &history.value {
            c::OwnerHistoricalValue::Task { document, .. }
            | c::OwnerHistoricalValue::Escalation { task: document, .. } => Scope::Project(document.project),
            c::OwnerHistoricalValue::Question { question, .. } => chip_scope(&question.task),
            c::OwnerHistoricalValue::Proposal(proposal) => Scope::Task(proposal.proposer),
            c::OwnerHistoricalValue::PersonProposal { proposal, .. } => Scope::Project(proposal.project),
            c::OwnerHistoricalValue::PersonTask { .. } => return Err(Failed::Fixed(b"unexpected act subject")),
        });
    }
    if let Some((task, _)) = render::read_task(document)
        && expected(call) == c::ObjectKey::Task(task.chip.task)
    {
        return Ok(Scope::Project(task.project));
    }
    if let c::ReadValue::Question(question) = document
        && expected(call) == (c::ObjectKey::Question { task: question.task.task, message: question.message })
    {
        return Ok(chip_scope(&question.task));
    }
    if let c::ReadValue::OwnerEscalation(escalation) = document
        && expected(call) == c::ObjectKey::Escalation(escalation.task.chip.task)
    {
        return Ok(Scope::Project(escalation.task.project));
    }
    if let c::ReadValue::Escalation(escalation) = document
        && expected(call) == c::ObjectKey::Escalation(escalation.task.task)
    {
        return Ok(chip_scope(&escalation.task));
    }
    if let c::ReadValue::OwnerProposal(proposal) = document
        && expected(call) == c::ObjectKey::Proposal(proposal.key)
    {
        return Ok(Scope::Project(proposal.project));
    }
    if let c::ReadValue::Proposal(proposal) = document
        && expected(call) == c::ObjectKey::Proposal(proposal.key)
    {
        return Ok(Scope::Project(proposal.project));
    }
    Err(Failed::Fixed(b"tools' bug; unexpected act document; report it as a finding"))
}
fn chip_scope(chip: &c::Chip) -> Scope {
    match chip_project(chip) {
        Some(project) => Scope::Project(project),
        None => Scope::Task(chip.task),
    }
}
pub(super) fn offers(document: &c::ReadValue) -> Option<&List<c::Offered>> {
    if let c::ReadValue::Task(task) = document {
        return Some(&task.acts);
    }
    if let c::ReadValue::Question(question) = document {
        return Some(&question.acts);
    }
    if let c::ReadValue::OwnerEscalation(escalation) = document {
        return Some(&escalation.acts);
    }
    if let c::ReadValue::Escalation(escalation) = document {
        return Some(&escalation.acts);
    }
    if let c::ReadValue::OwnerProposal(proposal) = document {
        return Some(&proposal.acts);
    }
    if let c::ReadValue::Proposal(proposal) = document {
        return Some(&proposal.acts);
    }
    None
}

pub(super) fn ended(document: &c::ReadValue) -> Option<c::Phase> {
    let (task, result) = render::read_task(document)?;
    if !render::terminal_phase(task.chip.phase) {
        return None;
    }
    if task.chip.phase != c::Phase::Ended {
        return Some(task.chip.phase);
    }
    if let Some(result) = result {
        return Some(match &result.ending {
            c::Ending::Task(ending) => owner_ending(ending),
            c::Ending::Done => c::Phase::Done,
            c::Ending::Cancelled { .. } => c::Phase::Cancelled,
            c::Ending::Failed { .. } => c::Phase::Failed,
            c::Ending::Withdrawn { .. } | c::Ending::Decided { .. } | c::Ending::Deleted => c::Phase::Ended,
        });
    }
    if let c::ChipPhase::Owner { details, .. } = task.chip.details.as_ref()
        && let Some(ending) = &details.ending
    {
        return Some(owner_ending(ending));
    }
    Some(c::Phase::Ended)
}
fn owner_ending(ending: &c::OwnerChipEnding) -> c::Phase {
    match ending {
        c::OwnerChipEnding::Done(_) => c::Phase::Done,
        c::OwnerChipEnding::Failed(_) => c::Phase::Failed,
        c::OwnerChipEnding::Cancelled { .. } => c::Phase::Cancelled,
    }
}

pub(super) fn routes_through(call: &Call, document: &c::ReadValue, task: u64) -> bool {
    match scope(call, document) {
        Ok(Scope::Task(number)) => number == task,
        Ok(Scope::Project(_)) | Err(_) => false,
    }
}
