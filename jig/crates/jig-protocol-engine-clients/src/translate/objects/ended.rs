//! Whole single-read owner documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::super::{chips, owner, task, w};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
pub fn retained_amendment(
    models: &[crate::Model],
    value: &d::AmendmentChange,
    limits: &w::Limits,
) -> Result<w::RetainedAmendment, w::Problem> {
    let dependencies = match &value.dependencies {
        Some(rows) => {
            let mut copied = List::with_capacity(rows.len());
            for row in rows {
                copied.push(*row).expect("supplied rows fit");
            }
            Some(copied)
        }
        None => None,
    };
    w::RetainedAmendment::new(
        limits,
        w::RetainedAmendmentParts {
            addressed: match value.addressed {
                Some(item) => Some(w::AddressChange::new(
                    limits,
                    w::AddressChangeParts {
                        revision: w::Revision::new(limits, w::RevisionParts { value: item.revision })?,
                        address: chips::address(item.address, limits)?,
                    },
                )?),
                None => None,
            },
            instructions: match &value.instructions {
                Some(item) => Some(owner::text(item, limits)?),
                None => None,
            },
            release: value.release,
            spec: match &value.spec {
                Some(item) => Some(owner::specification(item, limits)?),
                None => None,
            },
            authority: match &value.authority {
                Some(item) => Some(owner::authority(models, item, limits)?),
                None => None,
            },
            wake: match value.wake {
                Some(item) => Some(owner::wake(item, limits)?),
                None => None,
            },
            dependencies,
            reason: owner::text(&value.reason, limits)?,
            cut: value.cut,
        },
    )
}
pub fn retained_action(
    models: &[crate::Model],
    value: &d::RetainedAction,
    limits: &w::Limits,
) -> Result<w::RetainedAction, w::Problem> {
    Ok(match value {
        d::RetainedAction::Settings { against, items, made, projects } => {
            w::RetainedAction::Settings(w::RetainedActionSettings::new(
                limits,
                w::RetainedActionSettingsParts {
                    projects: projects.clone(),
                    against: *against,
                    items: super::super::settings::item_heads(items, limits)?,
                    made: *made,
                },
            )?)
        }
        d::RetainedAction::Batch(rows) => {
            let mut copied = List::with_capacity(rows.len());
            for row in rows {
                copied.push(super::proposals::proposed(models, row, limits)?).expect("supplied rows fit");
            }
            let copied = super::proposals::proposed_task_pieces(
                &copied,
                limits.retained_action_batch_value,
                w::Path::RetainedActionBatchValue,
            )?;
            w::RetainedAction::Batch(w::RetainedActionBatch::new(
                limits,
                w::RetainedActionBatchParts { value: copied },
            )?)
        }
        d::RetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::RetainedAction::Effect(w::RetainedActionEffect::new(
                limits,
                w::RetainedActionEffectParts {
                    part: owner::part(part, limits)?,
                    authority: owner::authority(models, authority, limits)?,
                    attempt: *attempt,
                    completion: *completion,
                    position: *position,
                    purpose: *purpose,
                },
            )?)
        }
        d::RetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            w::RetainedAction::ProcedureEffect(w::RetainedActionProcedureEffect::new(
                limits,
                w::RetainedActionProcedureEffectParts {
                    part: owner::part(part, limits)?,
                    authority: owner::authority(models, authority, limits)?,
                    step: *step,
                    purpose: *purpose,
                },
            )?)
        }
        d::RetainedAction::Amend { task, change } => w::RetainedAction::Amend(Box::new(w::RetainedActionAmend::new(
            limits,
            w::RetainedActionAmendParts { task: *task, change: retained_amendment(models, change, limits)? },
        )?)),
        d::RetainedAction::Widen { task, authority } => w::RetainedAction::Widen(w::RetainedActionWiden::new(
            limits,
            w::RetainedActionWidenParts { task: *task, authority: owner::authority(models, authority, limits)? },
        )?),
        d::RetainedAction::Release { task } => w::RetainedAction::Release(w::RetainedActionRelease::new(
            limits,
            w::RetainedActionReleaseParts { task: *task },
        )?),
    })
}
pub fn proposal_history(
    models: &[crate::Model],
    value: &d::ProposalHistory,
    limits: &w::Limits,
) -> Result<w::ProposalHistory, w::Problem> {
    w::ProposalHistory::new(
        limits,
        w::ProposalHistoryParts {
            reason: owner::text(&value.reason, limits)?,
            action: retained_action(models, &value.action, limits)?,
            number: value.number,
            proposer: value.proposer,
            as_holder: value.as_holder,
            state: match &value.state {
                d::ProposalHistoryState::Pending(value) => {
                    w::ProposalHistoryState::Pending(w::ProposalHistoryStatePending::new(
                        limits,
                        w::ProposalHistoryStatePendingParts {
                            holder: super::proposals::holder(value.holder, limits)?,
                            since: value.since.as_nanos(),
                        },
                    )?)
                }
                d::ProposalHistoryState::Accepted { by } => {
                    w::ProposalHistoryState::Accepted(w::ProposalHistoryStateAccepted::new(
                        limits,
                        w::ProposalHistoryStateAcceptedParts { by: owner::requester(*by, limits)? },
                    )?)
                }
                d::ProposalHistoryState::Rejected { by, reason } => {
                    w::ProposalHistoryState::Rejected(w::ProposalHistoryStateRejected::new(
                        limits,
                        w::ProposalHistoryStateRejectedParts {
                            by: owner::requester(*by, limits)?,
                            reason: owner::text(reason, limits)?,
                        },
                    )?)
                }
                d::ProposalHistoryState::Withdrawn => w::ProposalHistoryState::Withdrawn,
            },
        },
    )
}
pub fn milestone(value: &d::MilestoneLine, limits: &w::Limits) -> Result<w::MilestoneLine, w::Problem> {
    w::MilestoneLine::new(
        limits,
        w::MilestoneLineParts {
            task: value.task,
            position: value.position,
            at: match value.at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            phase: match &value.phase {
                d::MilestonePhase::Waiting => w::MilestonePhase::Waiting,
                d::MilestonePhase::Active(value) => w::MilestonePhase::Active(w::MilestonePhaseActive::new(
                    limits,
                    w::MilestonePhaseActiveParts { value: chips::active(*value, limits)? },
                )?),
                d::MilestonePhase::Closing { stage, ending } => {
                    w::MilestonePhase::Closing(w::MilestonePhaseClosing::new(
                        limits,
                        w::MilestonePhaseClosingParts {
                            stage: chips::stage(*stage, limits)?,
                            ending: chips::ending(ending, limits)?,
                        },
                    )?)
                }
                d::MilestonePhase::Held { before, why, source, ending } => {
                    w::MilestonePhase::Held(w::MilestonePhaseHeld::new(
                        limits,
                        w::MilestonePhaseHeldParts {
                            before: chips::prior(*before, limits)?,
                            why: chips::hold(*why, limits)?,
                            source: match source {
                                Some(item) => Some(owner::part(item, limits)?),
                                None => None,
                            },
                            ending: match ending {
                                Some(item) => Some(chips::ending(item, limits)?),
                                None => None,
                            },
                        },
                    )?)
                }
                d::MilestonePhase::Ended(value) => w::MilestonePhase::Ended(w::MilestonePhaseEnded::new(
                    limits,
                    w::MilestonePhaseEndedParts { value: chips::ending(value, limits)? },
                )?),
            },
        },
    )
}
pub fn escalation_decision(
    value: &jig_domain_core_vocabulary::EscalationDecisionRecord,
    limits: &w::Limits,
) -> Result<w::EscalationDecisionRecord, w::Problem> {
    w::EscalationDecisionRecord::new(
        limits,
        w::EscalationDecisionRecordParts {
            project: value.project,
            requester: value.requester,
            task: value.task,
            revision: value.revision,
            by: value.by,
            at: value.at.as_nanos(),
            decision: match &value.decision {
                p::EscalationDecision::Release => w::EscalationDecision::Release,
                p::EscalationDecision::Pass => w::EscalationDecision::Pass,
                p::EscalationDecision::Reject { reason } => {
                    w::EscalationDecision::Reject(w::EscalationDecisionReject::new(
                        limits,
                        w::EscalationDecisionRejectParts { reason: bytes::copy_of(reason) },
                    )?)
                }
            },
        },
    )
}
pub fn proposal_decision(
    value: jig_domain_core_vocabulary::ProposalDecisionRecord,
    limits: &w::Limits,
) -> Result<w::ProposalDecision, w::Problem> {
    w::ProposalDecision::new(
        limits,
        w::ProposalDecisionParts {
            project: value.project,
            proposer: task::proposal(&d::ProposalRef { proposer: value.proposer, number: value.proposal }, limits)?
                .proposer()
                .clone(),
            proposal: value.proposal,
            kind: super::proposals::kind(value.kind),
            by: value.by,
            at: value.at.as_nanos(),
            choice: match value.choice {
                p::ProposalChoice::Accepted => w::ProposalChoice::Accepted,
                p::ProposalChoice::Rejected => w::ProposalChoice::Rejected,
                p::ProposalChoice::Passed => w::ProposalChoice::Passed,
                p::ProposalChoice::Withdrawn => w::ProposalChoice::Withdrawn,
                p::ProposalChoice::Stale => w::ProposalChoice::Stale,
            },
            created: match value.created {
                Some(item) => Some(w::ProposalDecisionCreated::new(
                    limits,
                    w::ProposalDecisionCreatedParts { first: item.0, last: item.1 },
                )?),
                None => None,
            },
        },
    )
}
pub fn person_proposal(
    models: &[crate::Model],
    value: &t::PersonProposal,
    limits: &w::Limits,
) -> Result<w::PersonProposal, w::Problem> {
    let [goal] = value.batch() else { return Err(super::super::problem(skein_codec::Reason::Malformed)) };
    w::PersonProposal::new(
        limits,
        w::PersonProposalParts {
            number: value.number,
            proposer: value.proposer,
            project: value.project,
            goal: super::goal::goal(models, goal, limits)?,
            state: match &value.state {
                t::PersonProposalState::Withdrawn { .. } => {
                    return Err(super::super::problem(skein_codec::Reason::Malformed));
                }
                t::PersonProposalState::Pending { since } => {
                    w::PersonProposalState::Pending(w::PersonProposalStatePending::new(
                        limits,
                        w::PersonProposalStatePendingParts { since: since.as_nanos() },
                    )?)
                }
                t::PersonProposalState::Accepted { by } | t::PersonProposalState::SettingsAccepted { by, .. } => {
                    w::PersonProposalState::Accepted(w::PersonProposalStateAccepted::new(
                        limits,
                        w::PersonProposalStateAcceptedParts { by: owner::requester(*by, limits)? },
                    )?)
                }
                t::PersonProposalState::Rejected { by, reason, message, at } => {
                    w::PersonProposalState::Rejected(w::PersonProposalStateRejected::new(
                        limits,
                        w::PersonProposalStateRejectedParts {
                            by: owner::requester(*by, limits)?,
                            reason: bytes::copy_of(reason),
                            message: *message,
                            at: at.as_nanos(),
                        },
                    )?)
                }
            },
        },
    )
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
