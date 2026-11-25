//! Wire projections of owner proposal facts (client/domain/testing.md, 4).
use super::super::{c, list, try_pieces, w};
use super::{
    wire_address_change, wire_amendment_state, wire_authority_document, wire_chip_hold, wire_document_text,
    wire_funder, wire_history_holder, wire_offered, wire_part, wire_proposal_action_settings, wire_proposal_key,
    wire_proposed_task, wire_task_link,
};
pub(in crate::codec) fn wire_proposal(value: &c::Proposal, limits: &w::Limits) -> Result<w::Proposal, w::Problem> {
    Ok(w::Proposal {
        projects: value.projects.clone(),
        key: wire_proposal_key(&value.key),
        project: value.project,
        reason: wire_document_text(&value.reason),
        action: wire_proposal_action(&value.action, limits)?,
        as_holder: value.as_holder,
        waiting: wire_proposal_waiting(&value.waiting),
        funding: list(value.funding.iter().map(wire_funding_offer)),
        acts: try_pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
    })
}

pub(in crate::codec) fn wire_proposal_action(
    value: &c::ProposalAction,
    limits: &w::Limits,
) -> Result<w::ProposalAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProposalAction::Settings(value) => {
            w::ProposalAction::Settings(Box::new(wire_proposal_action_settings(value, limits)?))
        }
        c::ProposalAction::Batch(value) => w::ProposalAction::Batch(w::ProposalActionBatch {
            value: try_pieces((value).iter().map(|item| wire_proposed_task(item, limits)), limits)?,
        }),
        c::ProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::ProposalAction::Effect(w::ProposalActionEffect {
                part: wire_part(part),
                authority: wire_authority_document(authority, limits)?,
                attempt: *(attempt),
                completion: *(completion),
                position: *(position),
                purpose: *(purpose),
            })
        }
        c::ProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            w::ProposalAction::ProcedureEffect(w::ProposalActionProcedureEffect {
                part: wire_part(part),
                authority: wire_authority_document(authority, limits)?,
                step: *(step),
                purpose: *(purpose),
            })
        }
        c::ProposalAction::Amend { task, addressed, before, after, reason, release } => {
            w::ProposalAction::Amend(Box::new(w::ProposalActionAmend {
                task: wire_task_link(task, limits)?,
                addressed: (addressed).as_ref().map(wire_address_change),
                before: wire_amendment_state(before, limits)?,
                after: wire_amendment_state(after, limits)?,
                reason: wire_document_text(reason),
                release: *release,
            }))
        }
        c::ProposalAction::Widen { task, before, after } => w::ProposalAction::Widen(w::ProposalActionWiden {
            task: wire_task_link(task, limits)?,
            before: wire_authority_document(before, limits)?,
            after: wire_authority_document(after, limits)?,
        }),
        c::ProposalAction::Release { task, hold } => w::ProposalAction::Release(w::ProposalActionRelease {
            task: wire_task_link(task, limits)?,
            hold: (hold).as_deref().map(wire_chip_hold),
        }),
    })
}

pub(in crate::codec) fn wire_proposal_waiting(value: &c::ProposalWaiting) -> w::ProposalWaiting {
    w::ProposalWaiting { holder: wire_history_holder(&value.holder), since: value.since.as_nanos() }
}

pub(in crate::codec) fn wire_funding_offer(value: &c::FundingOffer) -> w::FundingOffer {
    w::FundingOffer { source: wire_funder(&value.source), needed: value.needed, left: value.left, after: value.after }
}
