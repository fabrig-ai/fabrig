//! Wire projections of owner history facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{
    wire_address_change, wire_document_text, wire_owner_amendment_state, wire_owner_authority_document,
    wire_owner_message_proposal_kind, wire_owner_part, wire_owner_requester, wire_owner_retained_action_settings,
    wire_owner_wake_policy, wire_proposed_task, wire_specification_document,
};
pub(in crate::codec) fn wire_owner_proposal_history(
    value: &c::OwnerProposalHistory,
    limits: &w::Limits,
) -> Result<w::OwnerProposalHistory, w::Problem> {
    w::OwnerProposalHistory::new(
        limits,
        w::OwnerProposalHistoryParts {
            action: wire_owner_retained_action(&value.action, limits)?,
            number: value.number,
            proposer: value.proposer,
            as_holder: value.as_holder,
            state: wire_owner_proposal_history_state(&value.state, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_retained_action(
    value: &c::OwnerRetainedAction,
    limits: &w::Limits,
) -> Result<w::OwnerRetainedAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRetainedAction::Settings(value) => {
            w::OwnerRetainedAction::Settings(wire_owner_retained_action_settings(value, limits)?)
        }
        c::OwnerRetainedAction::Batch(value) => w::OwnerRetainedAction::Batch(w::OwnerRetainedActionBatch::new(
            limits,
            w::OwnerRetainedActionBatchParts {
                value: pieces(
                    (value).iter().map(|item| wire_proposed_task(item, limits)),
                    w::ProposedTask::measure,
                    w::ProposedTask::encode,
                )?,
            },
        )?),
        c::OwnerRetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::OwnerRetainedAction::Effect(w::OwnerRetainedActionEffect::new(
                limits,
                w::OwnerRetainedActionEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    attempt: *(attempt),
                    completion: *(completion),
                    position: *(position),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerRetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            w::OwnerRetainedAction::ProcedureEffect(w::OwnerRetainedActionProcedureEffect::new(
                limits,
                w::OwnerRetainedActionProcedureEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    step: *(step),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerRetainedAction::Amend { task, change } => {
            w::OwnerRetainedAction::Amend(Box::new(w::OwnerRetainedActionAmend::new(
                limits,
                w::OwnerRetainedActionAmendParts {
                    task: *(task),
                    change: wire_owner_retained_amendment(change, limits)?,
                },
            )?))
        }
        c::OwnerRetainedAction::Widen { task, authority } => {
            w::OwnerRetainedAction::Widen(w::OwnerRetainedActionWiden::new(
                limits,
                w::OwnerRetainedActionWidenParts {
                    task: *(task),
                    authority: wire_owner_authority_document(authority, limits)?,
                },
            )?)
        }
        c::OwnerRetainedAction::Release { task } => w::OwnerRetainedAction::Release(
            w::OwnerRetainedActionRelease::new(limits, w::OwnerRetainedActionReleaseParts { task: *(task) })?,
        ),
    })
}

pub(in crate::codec) fn wire_owner_retained_amendment(
    value: &c::OwnerRetainedAmendment,
    limits: &w::Limits,
) -> Result<w::OwnerRetainedAmendment, w::Problem> {
    w::OwnerRetainedAmendment::new(
        limits,
        w::OwnerRetainedAmendmentParts {
            addressed: value.addressed.as_ref().map(|item| wire_address_change(item, limits)).transpose()?,
            instructions: value.instructions.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            release: value.release.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            spec: value.spec.as_ref().map(|item| wire_specification_document(item, limits)).transpose()?,
            authority: value.authority.as_ref().map(|item| wire_owner_authority_document(item, limits)).transpose()?,
            wake: value.wake.as_ref().map(|item| wire_owner_wake_policy(item, limits)).transpose()?,
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

pub(in crate::codec) fn wire_owner_proposal_history_state(
    value: &c::OwnerProposalHistoryState,
    limits: &w::Limits,
) -> Result<w::OwnerProposalHistoryState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProposalHistoryState::Pending { holder, since } => {
            w::OwnerProposalHistoryState::Pending(w::OwnerProposalHistoryStatePending::new(
                limits,
                w::OwnerProposalHistoryStatePendingParts {
                    holder: wire_owner_history_holder(holder, limits)?,
                    since: (since).as_nanos(),
                },
            )?)
        }
        c::OwnerProposalHistoryState::Accepted { by } => {
            w::OwnerProposalHistoryState::Accepted(w::OwnerProposalHistoryStateAccepted::new(
                limits,
                w::OwnerProposalHistoryStateAcceptedParts { by: wire_owner_requester(by, limits)? },
            )?)
        }
        c::OwnerProposalHistoryState::Rejected { by, reason } => {
            w::OwnerProposalHistoryState::Rejected(w::OwnerProposalHistoryStateRejected::new(
                limits,
                w::OwnerProposalHistoryStateRejectedParts {
                    by: wire_owner_requester(by, limits)?,
                    reason: wire_document_text(reason, limits)?,
                },
            )?)
        }
        c::OwnerProposalHistoryState::Withdrawn => w::OwnerProposalHistoryState::Withdrawn,
    })
}

pub(in crate::codec) fn wire_owner_history_holder(
    value: &c::OwnerHistoryHolder,
    limits: &w::Limits,
) -> Result<w::OwnerHistoryHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHistoryHolder::Task(value) => w::OwnerHistoryHolder::Task(w::OwnerHistoryHolderTask::new(
            limits,
            w::OwnerHistoryHolderTaskParts { value: *(value) },
        )?),
        c::OwnerHistoryHolder::Person(value) => w::OwnerHistoryHolder::Person(w::OwnerHistoryHolderPerson::new(
            limits,
            w::OwnerHistoryHolderPersonParts { value: *(value) },
        )?),
        c::OwnerHistoryHolder::Policy { project, kind } => {
            w::OwnerHistoryHolder::Policy(w::OwnerHistoryHolderPolicy::new(
                limits,
                w::OwnerHistoryHolderPolicyParts {
                    project: *(project),
                    kind: wire_owner_message_proposal_kind(kind, limits)?,
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_history_line(
    value: &c::OwnerHistoryLine,
    limits: &w::Limits,
) -> Result<w::OwnerHistoryLine, w::Problem> {
    w::OwnerHistoryLine::new(
        limits,
        w::OwnerHistoryLineParts {
            task: value.task,
            revision: value.revision,
            by: wire_owner_requester(&value.by, limits)?,
            change: wire_owner_history_change(&value.change, limits)?,
            reason: wire_document_text(&value.reason, limits)?,
            proposal: value.proposal.as_ref().map(|item| wire_owner_proposal_history(item, limits)).transpose()?,
            amendment: value
                .amendment
                .as_ref()
                .map(|item| {
                    w::OwnerHistoryAmendment::new(
                        limits,
                        w::OwnerHistoryAmendmentParts {
                            before: wire_owner_amendment_state(&(item).0, limits)?,
                            after: wire_owner_amendment_state(&(item).1, limits)?,
                        },
                    )
                })
                .transpose()?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_history_change(
    value: &c::OwnerHistoryChange,
    _limits: &w::Limits,
) -> Result<w::OwnerHistoryChange, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHistoryChange::Cancelled => w::OwnerHistoryChange::Cancelled,
        c::OwnerHistoryChange::Released => w::OwnerHistoryChange::Released,
        c::OwnerHistoryChange::Amended => w::OwnerHistoryChange::Amended,
        c::OwnerHistoryChange::Moved => w::OwnerHistoryChange::Moved,
        c::OwnerHistoryChange::Proposed => w::OwnerHistoryChange::Proposed,
        c::OwnerHistoryChange::ProposalPassed => w::OwnerHistoryChange::ProposalPassed,
        c::OwnerHistoryChange::ProposalAccepted => w::OwnerHistoryChange::ProposalAccepted,
        c::OwnerHistoryChange::ProposalRejected => w::OwnerHistoryChange::ProposalRejected,
        c::OwnerHistoryChange::ProposalWithdrawn => w::OwnerHistoryChange::ProposalWithdrawn,
    })
}
