//! The documents and outcomes emitted by the engine translator (protocol/client.md, 5–7).
use jig_client_domain::contract::{
    Activity, Chip, ChipPhase, Delivery, EndedGroup, Outcome, OwnerDeployment, OwnerHistorical, OwnerHistoricalValue,
    OwnerInboxEntry, OwnerPersonTask, OwnerProject, OwnerProposal, OwnerProposalAction, OwnerTaskLink, ReadValue,
    Result, Task, TaskHold, TaskResource, TaskResult,
};

pub(crate) fn outcome(value: &Outcome) -> bool {
    matches!(
        value,
        Outcome::Owner(_) | Outcome::Made { .. } | Outcome::SettingsChanged { .. } | Outcome::MovedToNewest { .. }
    )
}
pub(crate) fn read(value: &ReadValue) -> bool {
    match value {
        ReadValue::Task(value) => task(value),
        ReadValue::OwnerEscalation(value) => task(&value.task),
        ReadValue::OwnerPersonTask(value) => person(value),
        ReadValue::Question(value) => chip(&value.task),
        ReadValue::Result(value) => result(value),
        ReadValue::Historical(value) => historical(value),
        ReadValue::Tasks(value) => {
            value.live.iter().all(|row| chip(&row.chip))
                && match &value.ended {
                    EndedGroup::None => true,
                    EndedGroup::Rows { rows, .. } => rows.iter().all(|row| chip(&row.chip)),
                }
        }
        ReadValue::OwnerEnded(value) => value.rows.iter().all(|row| chip(&row.chip)),
        ReadValue::OwnerInbox(value) => value.rows.iter().all(inbox_row),
        ReadValue::Activity(value) => value.rows.iter().all(|row| matches!(row.event, Activity::Owner { .. })),
        ReadValue::OwnerProject(value) => project(value),
        ReadValue::OwnerDeployment(value) => deployment(value),
        ReadValue::OwnerProposal(value) => proposal(value),
        ReadValue::Me(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::OwnerSpend(_)
        | ReadValue::Notes(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::Revisions(_) => true,
        ReadValue::Proposal(_)
        | ReadValue::Escalation(_)
        | ReadValue::PersonTask(_)
        | ReadValue::Ended(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Project(_)
        | ReadValue::Deployment(_) => false,
    }
}
pub(crate) fn delivery(value: &Delivery) -> bool {
    match value {
        Delivery::Task(value) => task(value),
        Delivery::OwnerTree(value) => {
            chip(&value.root) && value.rows.iter().all(|row| chip(&row.chip) && row.dependencies.iter().all(link))
        }
        Delivery::Chips(value) => value.iter().all(chip),
        Delivery::OwnerInbox(value) => value.needs.iter().all(inbox_row) && value.updates.iter().all(inbox_row),
        Delivery::Tasks(value) => value.rows.iter().all(|row| chip(&row.chip)),
        Delivery::OwnerProject(value) => project(value),
        Delivery::OwnerProposal(value) => proposal(value),
        Delivery::OwnerDeployment(value) => deployment(value),
        Delivery::Me(_) | Delivery::RunState(_) | Delivery::Settings(_) | Delivery::OwnerAccounts(_) => true,
        Delivery::Tree(_) | Delivery::Inbox(_) | Delivery::Deployment(_) | Delivery::Run(_) => false,
    }
}
fn chip(value: &Chip) -> bool {
    matches!(value.details.as_ref(), ChipPhase::Owner { .. })
}
fn link(value: &OwnerTaskLink) -> bool {
    match value {
        OwnerTaskLink::Live(value) | OwnerTaskLink::Retained { chip: value, .. } => chip(value),
        OwnerTaskLink::Stored { .. } => true,
    }
}
fn result(value: &Result) -> bool {
    chip(&value.task)
}
fn task(value: &Task) -> bool {
    chip(&value.chip)
        && value.hold.as_ref().is_none_or(|value| matches!(value, TaskHold::Owner(_)))
        && value.resources.iter().all(|value| matches!(value, TaskResource::Owner(_)))
        && value.delegates.iter().all(|row| chip(&row.chip))
        && value.inputs.iter().all(link)
        && value.dependencies.iter().all(link)
        && value.lineage_chips.iter().all(chip)
        && value.result.as_ref().is_none_or(|value| match value {
            TaskResult::Document(value) => result(value),
            TaskResult::Ending(_) => true,
        })
}
fn person(value: &OwnerPersonTask) -> bool {
    chip(&value.task)
        && value
            .results
            .iter()
            .all(|value| link(&value.task) && value.result.as_ref().is_none_or(|value| result(value)))
}
fn inbox_row(value: &OwnerInboxEntry) -> bool {
    link(&value.task) && value.reply.as_ref().is_none_or(|row| chip(&row.chip))
}
fn project(value: &OwnerProject) -> bool {
    value.spend.iter().all(|row| chip(&row.chip)) && value.recurring.iter().all(|row| chip(&row.chip))
}
fn proposal(value: &OwnerProposal) -> bool {
    match &value.action {
        OwnerProposalAction::Amend { task, .. }
        | OwnerProposalAction::Widen { task, .. }
        | OwnerProposalAction::Release { task, .. } => link(task),
        OwnerProposalAction::Settings(_)
        | OwnerProposalAction::Batch(_)
        | OwnerProposalAction::Effect { .. }
        | OwnerProposalAction::ProcedureEffect { .. } => true,
    }
}
fn deployment(value: &OwnerDeployment) -> bool {
    value.hosts.iter().all(|row| row.runs.iter().all(|row| link(&row.task)))
        && value.attempts.iter().all(|row| link(&row.task))
        && value.accounts.iter().all(|row| row.runs.iter().all(chip) && row.waiting.iter().all(chip))
}
fn historical(value: &OwnerHistorical) -> bool {
    match &value.value {
        OwnerHistoricalValue::Task { document, result: ended } => task(document) && result(ended),
        OwnerHistoricalValue::Escalation { task: value, .. } => task(value),
        OwnerHistoricalValue::Question { question, ending, .. } => {
            chip(&question.task) && ending.as_ref().is_none_or(|value| result(value))
        }
        OwnerHistoricalValue::PersonTask { document, result: ended } => person(document) && result(ended),
        OwnerHistoricalValue::Proposal(_) | OwnerHistoricalValue::PersonProposal { .. } => true,
    }
}
