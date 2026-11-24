//! Wire projections of owner history facts (client/domain/testing.md, 4).
use super::super::{c, list, try_pieces, w};
use super::{
    wire_address_change, wire_amendment_state, wire_authority_document, wire_document_text, wire_message_proposal_kind,
    wire_part, wire_proposed_task, wire_requester, wire_retained_action_settings, wire_specification_document,
    wire_wake_policy,
};
pub(in crate::codec) fn wire_proposal_history(
    value: &c::ProposalHistory,
    limits: &w::Limits,
) -> Result<w::ProposalHistory, w::Problem> {
    Ok(w::ProposalHistory {
        reason: wire_document_text(&value.reason),
        action: wire_retained_action(&value.action, limits)?,
        number: value.number,
        proposer: value.proposer,
        as_holder: value.as_holder,
        state: wire_proposal_history_state(&value.state),
    })
}

pub(in crate::codec) fn wire_retained_action(
    value: &c::RetainedAction,
    limits: &w::Limits,
) -> Result<w::RetainedAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RetainedAction::Settings(value) => w::RetainedAction::Settings(wire_retained_action_settings(value)),
        c::RetainedAction::Batch(value) => w::RetainedAction::Batch(w::RetainedActionBatch {
            value: try_pieces((value).iter().map(|item| wire_proposed_task(item, limits)), limits)?,
        }),
        c::RetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::RetainedAction::Effect(w::RetainedActionEffect {
                part: wire_part(part),
                authority: wire_authority_document(authority, limits)?,
                attempt: *(attempt),
                completion: *(completion),
                position: *(position),
                purpose: *(purpose),
            })
        }
        c::RetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            w::RetainedAction::ProcedureEffect(w::RetainedActionProcedureEffect {
                part: wire_part(part),
                authority: wire_authority_document(authority, limits)?,
                step: *(step),
                purpose: *(purpose),
            })
        }
        c::RetainedAction::Amend { task, change } => w::RetainedAction::Amend(Box::new(w::RetainedActionAmend {
            task: *(task),
            change: wire_retained_amendment(change, limits)?,
        })),
        c::RetainedAction::Widen { task, authority } => w::RetainedAction::Widen(w::RetainedActionWiden {
            task: *(task),
            authority: wire_authority_document(authority, limits)?,
        }),
        c::RetainedAction::Release { task } => w::RetainedAction::Release(w::RetainedActionRelease { task: *(task) }),
    })
}

pub(in crate::codec) fn wire_retained_amendment(
    value: &c::RetainedAmendment,
    limits: &w::Limits,
) -> Result<w::RetainedAmendment, w::Problem> {
    Ok(w::RetainedAmendment {
        addressed: value.addressed.as_ref().map(wire_address_change),
        instructions: value.instructions.as_ref().map(wire_document_text),
        release: value.release,
        spec: value.spec.as_ref().map(wire_specification_document),
        authority: value.authority.as_ref().map(|item| wire_authority_document(item, limits)).transpose()?,
        wake: value.wake.as_ref().map(wire_wake_policy),
        dependencies: value.dependencies.as_ref().map(|item| list((item).iter().copied())),
        reason: wire_document_text(&value.reason),
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_proposal_history_state(value: &c::ProposalHistoryState) -> w::ProposalHistoryState {
    match value {
        c::ProposalHistoryState::Pending { holder, since } => {
            w::ProposalHistoryState::Pending(w::ProposalHistoryStatePending {
                holder: wire_history_holder(holder),
                since: (since).as_nanos(),
            })
        }
        c::ProposalHistoryState::Accepted { by } => {
            w::ProposalHistoryState::Accepted(w::ProposalHistoryStateAccepted { by: wire_requester(by) })
        }
        c::ProposalHistoryState::Rejected { by, reason } => {
            w::ProposalHistoryState::Rejected(w::ProposalHistoryStateRejected {
                by: wire_requester(by),
                reason: wire_document_text(reason),
            })
        }
        c::ProposalHistoryState::Withdrawn => w::ProposalHistoryState::Withdrawn,
    }
}

pub(in crate::codec) fn wire_history_holder(value: &c::HistoryHolder) -> w::HistoryHolder {
    match value {
        c::HistoryHolder::Task(value) => w::HistoryHolder::Task(w::HistoryHolderTask { value: *(value) }),
        c::HistoryHolder::Person(value) => w::HistoryHolder::Person(w::HistoryHolderPerson { value: *(value) }),
        c::HistoryHolder::Policy { project, kind } => w::HistoryHolder::Policy(w::HistoryHolderPolicy {
            project: *(project),
            kind: wire_message_proposal_kind(kind),
        }),
    }
}

pub(in crate::codec) fn wire_history_line(
    value: &c::HistoryLine,
    limits: &w::Limits,
) -> Result<w::HistoryLine, w::Problem> {
    Ok(w::HistoryLine {
        task: value.task,
        revision: value.revision,
        by: wire_requester(&value.by),
        change: wire_history_change(&value.change),
        reason: wire_document_text(&value.reason),
        proposal: value.proposal.as_ref().map(|item| wire_proposal_history(item, limits)).transpose()?,
        amendment: value
            .amendment
            .as_ref()
            .map(|item| {
                Ok(w::HistoryAmendment {
                    before: wire_amendment_state(&(item).0, limits)?,
                    after: wire_amendment_state(&(item).1, limits)?,
                })
            })
            .transpose()?,
        at: value.at.as_ref().map(|item| (item).as_nanos()),
    })
}

pub(in crate::codec) fn wire_history_change(value: &c::HistoryChange) -> w::HistoryChange {
    match value {
        c::HistoryChange::Made => w::HistoryChange::Made,
        c::HistoryChange::Concluded => w::HistoryChange::Concluded,
        c::HistoryChange::Cancelled => w::HistoryChange::Cancelled,
        c::HistoryChange::Released => w::HistoryChange::Released,
        c::HistoryChange::Amended => w::HistoryChange::Amended,
        c::HistoryChange::Moved => w::HistoryChange::Moved,
        c::HistoryChange::Proposed => w::HistoryChange::Proposed,
        c::HistoryChange::ProposalPassed => w::HistoryChange::ProposalPassed,
        c::HistoryChange::ProposalAccepted => w::HistoryChange::ProposalAccepted,
        c::HistoryChange::ProposalRejected => w::HistoryChange::ProposalRejected,
        c::HistoryChange::ProposalWithdrawn => w::HistoryChange::ProposalWithdrawn,
    }
}
