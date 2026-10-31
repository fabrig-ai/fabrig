//! Wire projections of owner proposal facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{
    wire_address_change, wire_document_text, wire_funder, wire_offered, wire_owner_amendment_state,
    wire_owner_authority_document, wire_owner_chip_hold, wire_owner_history_holder, wire_owner_part,
    wire_owner_proposal_action_settings, wire_owner_task_link, wire_proposal_key, wire_proposed_task,
};
pub(in crate::codec) fn wire_owner_proposal(
    value: &c::OwnerProposal,
    limits: &w::Limits,
) -> Result<w::OwnerProposal, w::Problem> {
    w::OwnerProposal::new(
        limits,
        w::OwnerProposalParts {
            key: wire_proposal_key(&value.key, limits)?,
            project: value.project,
            reason: wire_document_text(&value.reason, limits)?,
            action: wire_owner_proposal_action(&value.action, limits)?,
            as_holder: value.as_holder,
            waiting: wire_owner_proposal_waiting(&value.waiting, limits)?,
            funding: list(value.funding.iter().map(|item| wire_owner_funding_offer(item, limits)))?,
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_owner_proposal_action(
    value: &c::OwnerProposalAction,
    limits: &w::Limits,
) -> Result<w::OwnerProposalAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProposalAction::Settings(value) => {
            w::OwnerProposalAction::Settings(Box::new(wire_owner_proposal_action_settings(value, limits)?))
        }
        c::OwnerProposalAction::Batch(value) => w::OwnerProposalAction::Batch(w::OwnerProposalActionBatch::new(
            limits,
            w::OwnerProposalActionBatchParts {
                value: pieces(
                    (value).iter().map(|item| wire_proposed_task(item, limits)),
                    w::ProposedTask::measure,
                    w::ProposedTask::encode,
                )?,
            },
        )?),
        c::OwnerProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::OwnerProposalAction::Effect(w::OwnerProposalActionEffect::new(
                limits,
                w::OwnerProposalActionEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    attempt: *(attempt),
                    completion: *(completion),
                    position: *(position),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            w::OwnerProposalAction::ProcedureEffect(w::OwnerProposalActionProcedureEffect::new(
                limits,
                w::OwnerProposalActionProcedureEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    step: *(step),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerProposalAction::Amend { task, addressed, before, after, reason, release } => {
            w::OwnerProposalAction::Amend(Box::new(w::OwnerProposalActionAmend::new(
                limits,
                w::OwnerProposalActionAmendParts {
                    task: wire_owner_task_link(task, limits)?,
                    addressed: (addressed).as_ref().map(|item| wire_address_change(item, limits)).transpose()?,
                    before: wire_owner_amendment_state(before, limits)?,
                    after: wire_owner_amendment_state(after, limits)?,
                    reason: wire_document_text(reason, limits)?,
                    release: (release).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                },
            )?))
        }
        c::OwnerProposalAction::Widen { task, before, after } => {
            w::OwnerProposalAction::Widen(w::OwnerProposalActionWiden::new(
                limits,
                w::OwnerProposalActionWidenParts {
                    task: wire_owner_task_link(task, limits)?,
                    before: wire_owner_authority_document(before, limits)?,
                    after: wire_owner_authority_document(after, limits)?,
                },
            )?)
        }
        c::OwnerProposalAction::Release { task, hold } => {
            w::OwnerProposalAction::Release(w::OwnerProposalActionRelease::new(
                limits,
                w::OwnerProposalActionReleaseParts {
                    task: wire_owner_task_link(task, limits)?,
                    hold: (hold).as_ref().map(|item| wire_owner_chip_hold(item, limits)).transpose()?,
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_proposal_waiting(
    value: &c::OwnerProposalWaiting,
    limits: &w::Limits,
) -> Result<w::OwnerProposalWaiting, w::Problem> {
    w::OwnerProposalWaiting::new(
        limits,
        w::OwnerProposalWaitingParts {
            holder: wire_owner_history_holder(&value.holder, limits)?,
            since: value.since.as_nanos(),
        },
    )
}

pub(in crate::codec) fn wire_owner_funding_offer(
    value: &c::OwnerFundingOffer,
    limits: &w::Limits,
) -> Result<w::OwnerFundingOffer, w::Problem> {
    w::OwnerFundingOffer::new(
        limits,
        w::OwnerFundingOfferParts {
            source: wire_funder(&value.source, limits)?,
            needed: value.needed,
            left: value.left.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            after: value.after.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}
