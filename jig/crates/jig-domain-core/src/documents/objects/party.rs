//! Party and task amendment offers share the same before/after edit document.
use super::{Box, Core, Limits, List, Missing, ProposalAction, tasks};

pub(super) fn amendment(
    core: &Core,
    limits: &Limits,
    party: u64,
    task: u64,
    change: &tasks::Amendment,
) -> Result<ProposalAction, Missing> {
    use super::super::{small, task_tree};
    let row = small::visible(core, party, task)?;
    Ok(ProposalAction::Amend {
        task: task_tree::link(core, limits, party, task)?,
        addressed: change.addressed,
        before: Box::new(super::amendment(core, limits, row, None)?),
        after: Box::new(super::amendment(core, limits, row, Some(change))?),
        reason: small::text(&change.reason, limits.documents.text_bytes),
        release: change.release,
    })
}

pub(crate) fn retained(
    limits: &Limits,
    proposal: &tasks::PersonProposal,
) -> Result<Option<super::ProposalHistory>, Missing> {
    use super::{ProposalHistory, ProposalHistoryState, ProposalWaiting, RetainedAction};
    let (action, kind) = match &proposal.action {
        tasks::PersonAction::Settings { .. } => return Ok(None),
        tasks::PersonAction::Batch(members) => {
            let withdrawn = match proposal.state {
                tasks::PersonProposalState::Withdrawn { .. } => true,
                tasks::PersonProposalState::Pending { .. }
                | tasks::PersonProposalState::Accepted { .. }
                | tasks::PersonProposalState::SettingsAccepted { .. }
                | tasks::PersonProposalState::Rejected { .. } => false,
            };
            if members.len() == 1 && !withdrawn {
                return Ok(None);
            }
            let mut batch = List::with_capacity(limits.documents.lines);
            for member in members {
                batch.push(super::proposed(limits, member)?).or(Err(Missing::TooLarge))?;
            }
            (RetainedAction::Batch(batch), tasks::ProposalKind::Batch)
        }
        tasks::PersonAction::Amend { task, amendment, .. } => (
            RetainedAction::Amend { task: *task, change: Box::new(super::retained_amendment(limits, amendment)?) },
            tasks::ProposalKind::Amend,
        ),
    };
    let state = match &proposal.state {
        tasks::PersonProposalState::Pending { since } => ProposalHistoryState::Pending(ProposalWaiting {
            holder: tasks::ProposalHolder::Policy { project: proposal.project, kind },
            since: *since,
        }),
        tasks::PersonProposalState::Accepted { by } | tasks::PersonProposalState::SettingsAccepted { by, .. } => {
            ProposalHistoryState::Accepted { by: *by }
        }
        tasks::PersonProposalState::Rejected { by, reason, .. } => ProposalHistoryState::Rejected {
            by: *by,
            reason: super::super::small::text(reason, limits.documents.text_bytes),
        },
        tasks::PersonProposalState::Withdrawn { .. } => ProposalHistoryState::Withdrawn,
    };
    Ok(Some(ProposalHistory {
        proposer: proposal.proposer,
        number: proposal.number,
        reason: super::super::small::text(&proposal.reason, limits.documents.text_bytes),
        as_holder: true,
        action,
        state,
    }))
}

pub(super) fn action(
    core: &Core,
    limits: &Limits,
    party: u64,
    action: &tasks::PersonAction,
) -> Result<(ProposalAction, tasks::ProposalKind), Missing> {
    match action {
        tasks::PersonAction::Settings { against, items } => Ok((
            ProposalAction::Settings {
                against: *against,
                items: crate::documents::settings_heads(items),
                review: None,
                made: None,
            },
            tasks::ProposalKind::Settings,
        )),
        tasks::PersonAction::Batch(members) => {
            let mut batch = List::with_capacity(limits.documents.lines);
            for member in members {
                if batch.push(super::proposed(limits, member)?).is_err() {
                    return Err(Missing::TooLarge);
                }
            }
            Ok((ProposalAction::Batch(batch), tasks::ProposalKind::Batch))
        }
        tasks::PersonAction::Amend { task, amendment: change, .. } => {
            Ok((amendment(core, limits, party, *task, change)?, tasks::ProposalKind::Amend))
        }
    }
}
