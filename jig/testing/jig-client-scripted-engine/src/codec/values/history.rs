//! Wire projections of owner history facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{
    wire_address_change, wire_amendment_state, wire_authority_document, wire_document_text, wire_message_proposal_kind,
    wire_part, wire_proposed_task, wire_requester, wire_retained_action_settings, wire_specification_document,
    wire_wake_policy,
};
pub(in crate::codec) fn wire_proposal_history(
    value: &c::ProposalHistory,
    limits: &w::Limits,
) -> Result<w::ProposalHistory, w::Problem> {
    w::ProposalHistory::new(
        limits,
        w::ProposalHistoryParts {
            action: wire_retained_action(&value.action, limits)?,
            number: value.number,
            proposer: value.proposer,
            as_holder: value.as_holder,
            state: wire_proposal_history_state(&value.state, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_retained_action(
    value: &c::RetainedAction,
    limits: &w::Limits,
) -> Result<w::RetainedAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RetainedAction::Settings(value) => {
            w::RetainedAction::Settings(wire_retained_action_settings(value, limits)?)
        }
        c::RetainedAction::Batch(value) => w::RetainedAction::Batch(w::RetainedActionBatch::new(
            limits,
            w::RetainedActionBatchParts {
                value: pieces(
                    (value).iter().map(|item| wire_proposed_task(item, limits)),
                    w::ProposedTask::measure,
                    w::ProposedTask::encode,
                )?,
            },
        )?),
        c::RetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::RetainedAction::Effect(w::RetainedActionEffect::new(
                limits,
                w::RetainedActionEffectParts {
                    part: wire_part(part, limits)?,
                    authority: wire_authority_document(authority, limits)?,
                    attempt: *(attempt),
                    completion: *(completion),
                    position: *(position),
                    purpose: *(purpose),
                },
            )?)
        }
        c::RetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            w::RetainedAction::ProcedureEffect(w::RetainedActionProcedureEffect::new(
                limits,
                w::RetainedActionProcedureEffectParts {
                    part: wire_part(part, limits)?,
                    authority: wire_authority_document(authority, limits)?,
                    step: *(step),
                    purpose: *(purpose),
                },
            )?)
        }
        c::RetainedAction::Amend { task, change } => w::RetainedAction::Amend(Box::new(w::RetainedActionAmend::new(
            limits,
            w::RetainedActionAmendParts { task: *(task), change: wire_retained_amendment(change, limits)? },
        )?)),
        c::RetainedAction::Widen { task, authority } => w::RetainedAction::Widen(w::RetainedActionWiden::new(
            limits,
            w::RetainedActionWidenParts { task: *(task), authority: wire_authority_document(authority, limits)? },
        )?),
        c::RetainedAction::Release { task } => w::RetainedAction::Release(w::RetainedActionRelease::new(
            limits,
            w::RetainedActionReleaseParts { task: *(task) },
        )?),
    })
}

pub(in crate::codec) fn wire_retained_amendment(
    value: &c::RetainedAmendment,
    limits: &w::Limits,
) -> Result<w::RetainedAmendment, w::Problem> {
    w::RetainedAmendment::new(
        limits,
        w::RetainedAmendmentParts {
            addressed: value.addressed.as_ref().map(|item| wire_address_change(item, limits)).transpose()?,
            instructions: value.instructions.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            release: value.release.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            spec: value.spec.as_ref().map(|item| wire_specification_document(item, limits)).transpose()?,
            authority: value.authority.as_ref().map(|item| wire_authority_document(item, limits)).transpose()?,
            wake: value.wake.as_ref().map(|item| wire_wake_policy(item, limits)).transpose()?,
            dependencies: value
                .dependencies
                .as_ref()
                .map(|item| list((item).iter().map(|item| Ok::<_, w::Problem>(*(item)))))
                .transpose()?,
            reason: wire_document_text(&value.reason, limits)?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_proposal_history_state(
    value: &c::ProposalHistoryState,
    limits: &w::Limits,
) -> Result<w::ProposalHistoryState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProposalHistoryState::Pending { holder, since } => {
            w::ProposalHistoryState::Pending(w::ProposalHistoryStatePending::new(
                limits,
                w::ProposalHistoryStatePendingParts {
                    holder: wire_history_holder(holder, limits)?,
                    since: (since).as_nanos(),
                },
            )?)
        }
        c::ProposalHistoryState::Accepted { by } => {
            w::ProposalHistoryState::Accepted(w::ProposalHistoryStateAccepted::new(
                limits,
                w::ProposalHistoryStateAcceptedParts { by: wire_requester(by, limits)? },
            )?)
        }
        c::ProposalHistoryState::Rejected { by, reason } => {
            w::ProposalHistoryState::Rejected(w::ProposalHistoryStateRejected::new(
                limits,
                w::ProposalHistoryStateRejectedParts {
                    by: wire_requester(by, limits)?,
                    reason: wire_document_text(reason, limits)?,
                },
            )?)
        }
        c::ProposalHistoryState::Withdrawn => w::ProposalHistoryState::Withdrawn,
    })
}

pub(in crate::codec) fn wire_history_holder(
    value: &c::HistoryHolder,
    limits: &w::Limits,
) -> Result<w::HistoryHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HistoryHolder::Task(value) => {
            w::HistoryHolder::Task(w::HistoryHolderTask::new(limits, w::HistoryHolderTaskParts { value: *(value) })?)
        }
        c::HistoryHolder::Person(value) => w::HistoryHolder::Person(w::HistoryHolderPerson::new(
            limits,
            w::HistoryHolderPersonParts { value: *(value) },
        )?),
        c::HistoryHolder::Policy { project, kind } => w::HistoryHolder::Policy(w::HistoryHolderPolicy::new(
            limits,
            w::HistoryHolderPolicyParts { project: *(project), kind: wire_message_proposal_kind(kind, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_history_line(
    value: &c::HistoryLine,
    limits: &w::Limits,
) -> Result<w::HistoryLine, w::Problem> {
    w::HistoryLine::new(
        limits,
        w::HistoryLineParts {
            task: value.task,
            revision: value.revision,
            by: wire_requester(&value.by, limits)?,
            change: wire_history_change(&value.change, limits)?,
            reason: wire_document_text(&value.reason, limits)?,
            proposal: value.proposal.as_ref().map(|item| wire_proposal_history(item, limits)).transpose()?,
            amendment: value
                .amendment
                .as_ref()
                .map(|item| {
                    w::HistoryAmendment::new(
                        limits,
                        w::HistoryAmendmentParts {
                            before: wire_amendment_state(&(item).0, limits)?,
                            after: wire_amendment_state(&(item).1, limits)?,
                        },
                    )
                })
                .transpose()?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_history_change(
    value: &c::HistoryChange,
    _limits: &w::Limits,
) -> Result<w::HistoryChange, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HistoryChange::Cancelled => w::HistoryChange::Cancelled,
        c::HistoryChange::Released => w::HistoryChange::Released,
        c::HistoryChange::Amended => w::HistoryChange::Amended,
        c::HistoryChange::Moved => w::HistoryChange::Moved,
        c::HistoryChange::Proposed => w::HistoryChange::Proposed,
        c::HistoryChange::ProposalPassed => w::HistoryChange::ProposalPassed,
        c::HistoryChange::ProposalAccepted => w::HistoryChange::ProposalAccepted,
        c::HistoryChange::ProposalRejected => w::HistoryChange::ProposalRejected,
        c::HistoryChange::ProposalWithdrawn => w::HistoryChange::ProposalWithdrawn,
    })
}
