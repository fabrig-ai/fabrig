//! Whole single-read owner documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::super::{owner, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
pub fn retained_amendment(
    models: &[crate::Model],
    value: &d::AmendmentChange,
    limits: &w::Limits,
) -> Result<w::RetainedAmendment, w::Problem> {
    mapped::retained_amendment_to_record(value, &(models, limits))
}
pub fn retained_action(
    models: &[crate::Model],
    value: &d::RetainedAction,
    limits: &w::Limits,
) -> Result<w::RetainedAction, w::Problem> {
    Ok(match value {
        d::RetainedAction::Settings { against, items, made, projects } => {
            w::RetainedAction::Settings(w::RetainedActionSettings {
                projects: projects.clone(),
                against: *against,
                items: super::super::settings::item_heads(items),
                made: *made,
            })
        }
        d::RetainedAction::Batch(rows) => {
            let mut copied = List::with_capacity(rows.len());
            for row in rows {
                copied.push(super::proposals::proposed(models, row, limits)?).expect("supplied rows fit");
            }
            let copied = super::proposals::proposed_task_pieces(
                &copied,
                limits.retained_action_batch_value,
                w::paths::RETAINED_ACTION_BATCH_VALUE,
                limits,
            )?;
            w::RetainedAction::Batch(w::RetainedActionBatch { value: copied })
        }
        d::RetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::RetainedAction::Effect(w::RetainedActionEffect {
                part: owner::part(part),
                authority: owner::authority(models, authority, limits)?,
                attempt: *attempt,
                completion: *completion,
                position: *position,
                purpose: *purpose,
            })
        }
        d::RetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            w::RetainedAction::ProcedureEffect(w::RetainedActionProcedureEffect {
                part: owner::part(part),
                authority: owner::authority(models, authority, limits)?,
                step: *step,
                purpose: *purpose,
            })
        }
        d::RetainedAction::Amend { task, change } => w::RetainedAction::Amend(Box::new(w::RetainedActionAmend {
            task: *task,
            change: retained_amendment(models, change, limits)?,
        })),
        d::RetainedAction::Widen { task, authority } => w::RetainedAction::Widen(w::RetainedActionWiden {
            task: *task,
            authority: owner::authority(models, authority, limits)?,
        }),
        d::RetainedAction::Release { task } => w::RetainedAction::Release(w::RetainedActionRelease { task: *task }),
    })
}
pub fn proposal_history(
    models: &[crate::Model],
    value: &d::ProposalHistory,
    limits: &w::Limits,
) -> Result<w::ProposalHistory, w::Problem> {
    mapped::proposal_history_to_record(value, &(models, limits))
}
/// Retained lifecycle facts (client/domain/contract/reads.md, 1).
pub fn attempt(value: &d::AttemptLine) -> Result<w::AttemptLine, w::Problem> {
    mapped::attempt_line_to_record(value)
}
pub fn hold(value: &d::HoldLine, limits: &w::Limits) -> Result<w::HoldLine, w::Problem> {
    mapped::hold_line_to_record(value, &(&[], limits))
}
pub fn escalation_decision(
    value: &jig_domain_core_vocabulary::EscalationDecisionRecord,
) -> w::EscalationDecisionRecord {
    mapped::escalation_decision_record_to_record(value)
}
pub fn proposal_decision(value: jig_domain_core_vocabulary::ProposalDecisionRecord) -> w::ProposalDecision {
    mapped::proposal_decision_to_record(&value)
}
pub fn person_proposal(
    models: &[crate::Model],
    value: &t::PersonProposal,
    limits: &w::Limits,
) -> Result<w::PersonProposal, w::Problem> {
    let [goal] = value.batch() else { return Err(super::super::problem(skein_codec::Reason::Malformed)) };
    Ok(w::PersonProposal {
        number: value.number,
        proposer: value.proposer,
        project: value.project,
        goal: super::goal::goal(models, goal, limits)?,
        state: match &value.state {
            t::PersonProposalState::Withdrawn { .. } => {
                return Err(super::super::problem(skein_codec::Reason::Malformed));
            }
            t::PersonProposalState::Pending { since } => {
                w::PersonProposalState::Pending(w::PersonProposalStatePending { since: since.as_nanos() })
            }
            t::PersonProposalState::Accepted { by } | t::PersonProposalState::SettingsAccepted { by, .. } => {
                w::PersonProposalState::Accepted(w::PersonProposalStateAccepted { by: owner::requester(*by) })
            }
            t::PersonProposalState::Rejected { by, reason, message, at } => {
                w::PersonProposalState::Rejected(w::PersonProposalStateRejected {
                    by: owner::requester(*by),
                    reason: bytes::copy_of(reason),
                    message: *message,
                    at: at.as_nanos(),
                })
            }
        },
    })
}

pub(crate) fn settings_history(
    models: &[crate::Model],
    proposal: &t::PersonProposal,
    against: u32,
    items: &[t::SettingHead],
    limits: &w::Limits,
) -> Result<w::ProposalHistory, w::Problem> {
    let (state, made) = match &proposal.state {
        t::PersonProposalState::Pending { since } => (
            d::ProposalHistoryState::Pending(d::ProposalWaiting {
                holder: t::ProposalHolder::Policy { project: 0, kind: t::ProposalKind::Settings },
                since: *since,
            }),
            None,
        ),
        t::PersonProposalState::SettingsAccepted { by, revision } => {
            (d::ProposalHistoryState::Accepted { by: *by }, Some(*revision))
        }
        t::PersonProposalState::Accepted { .. } => unreachable!("settings acceptance records its revision"),
        t::PersonProposalState::Rejected { by, reason, .. } => {
            (d::ProposalHistoryState::Rejected { by: *by, reason: d::Text { bytes: reason.clone(), cut: false } }, None)
        }
        t::PersonProposalState::Withdrawn { .. } => (d::ProposalHistoryState::Withdrawn, None),
    };
    proposal_history(
        models,
        &d::ProposalHistory {
            reason: d::Text { bytes: proposal.reason.clone(), cut: false },
            number: proposal.number,
            proposer: proposal.proposer,
            as_holder: false,
            action: d::RetainedAction::Settings {
                against,
                items: d::settings_heads(items),
                made,
                projects: List::with_capacity(0),
            },
            state,
        },
        limits,
    )
}
