//! Read pages and the owner-keyed store transport (`client/domain/contract/README.md`, section 4).
//! Cursors name a query and an owner key. The client only echoes them.

use crate::{Core, Record};
pub use jig_domain_core_vocabulary::reads::vocabulary::*;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

pub(crate) fn row_owner(row: &Row) -> Option<&tasks::TaskRecord> {
    match &row.value {
        Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task)) => Some(task),
        Record::Tasks(
            tasks::Stored::Activity(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::PlacedHold(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::EndedByParty(_)
            | tasks::Stored::EndedByProject(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::DecidedProposal(_),
        )
        | Record::Core(_)
        | Record::People(_)
        | Record::Notes(_)
        | Record::Accounts(_) => row.owner.as_deref(),
    }
}

pub(super) fn read_row_visible(core: &Core, party: u64, row: &Row, run_project: Option<u32>) -> bool {
    if let Some(project) = run_project {
        return proposal_row_visible(project, row);
    }
    if let Some(owner) = row_owner(row)
        && !core.open_project(owner.project)
    {
        return false;
    }
    row_visible(core, party, row)
}

// A run reads settings proposals in its project without a party's visibility (domain/engine/executors.md, 3).
pub(super) fn proposal_row_visible(project: u32, row: &Row) -> bool {
    match &row.value {
        Record::Tasks(tasks::Stored::History(history)) => {
            let Some(owner) = row_owner(row) else { return false };
            let Some(proposal) = &history.proposal else { return false };
            owner.project == project
                && match proposal.action {
                    tasks::ProposalAction::Settings { .. } => true,
                    tasks::ProposalAction::Batch(_)
                    | tasks::ProposalAction::Effect { .. }
                    | tasks::ProposalAction::ProcedureEffect { .. }
                    | tasks::ProposalAction::Amend { .. }
                    | tasks::ProposalAction::Widen { .. }
                    | tasks::ProposalAction::Release { .. }
                    | tasks::ProposalAction::ReleasePlaced { .. } => false,
                }
        }
        Record::Tasks(tasks::Stored::DecidedProposal(proposal)) => proposal.action.is_settings(),
        Record::Tasks(_) | Record::Core(_) | Record::People(_) | Record::Notes(_) | Record::Accounts(_) => false,
    }
}

pub(crate) fn row_visible(core: &Core, party: u64, row: &Row) -> bool {
    match &row.value {
        Record::Tasks(tasks::Stored::PersonProposal(proposal) | tasks::Stored::DecidedProposal(proposal)) => {
            match proposal.action {
                tasks::PersonAction::Settings { .. } => {
                    party == proposal.proposer || core.people.role(party, people::DEPLOYMENT).is_some()
                }
                tasks::PersonAction::Batch(_) | tasks::PersonAction::Amend { .. } => {
                    core.people.role(party, proposal.project).is_some()
                }
            }
        }
        Record::Tasks(tasks::Stored::Ledger(ledger) | tasks::Stored::ClosedLedger(ledger)) => match ledger.funder {
            tasks::Funder::Period { project, .. } => {
                core.people.role(party, project).is_some() || core.people.role(party, people::DEPLOYMENT).is_some()
            }
            tasks::Funder::Pool { project, person, .. } => {
                person == party && core.people.role(party, project).is_some()
            }
            tasks::Funder::Task(_) | tasks::Funder::Recurring { .. } => false,
        },
        Record::Notes(notes::Record::Entry(entry)) => match &entry.scope {
            notes::Scope::Party { party: owner } => super::party::scope_visible(core, party, *owner),
            notes::Scope::Deployment => core.people.role(party, people::DEPLOYMENT).is_some(),
            notes::Scope::Project { project }
            | notes::Scope::Goal { project, .. }
            | notes::Scope::Resources { project, .. } => core.people.role(party, *project).is_some(),
        },
        Record::Core(_)
        | Record::Tasks(
            tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::PlacedHold(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::EndedByParty(_)
            | tasks::Stored::EndedByProject(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Pool(_),
        ) => match row_owner(row) {
            Some(owner) => core.people.role(party, owner.project).is_some(),
            None => false,
        },
        Record::People(_) | Record::Notes(notes::Record::Line(_)) | Record::Accounts(_) => false,
    }
}
