//! Wire projections of owner proposal facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{
    wire_address_change, wire_amendment_state, wire_authority_document, wire_chip_hold, wire_document_text,
    wire_funder, wire_history_holder, wire_offered, wire_part, wire_proposal_action_settings, wire_proposal_key,
    wire_proposed_task, wire_task_link,
};
pub(in crate::codec) fn wire_proposal(value: &c::Proposal, limits: &w::Limits) -> Result<w::Proposal, w::Problem> {
    w::Proposal::new(
        limits,
        w::ProposalParts {
            key: wire_proposal_key(&value.key, limits)?,
            project: value.project,
            reason: wire_document_text(&value.reason, limits)?,
            action: wire_proposal_action(&value.action, limits)?,
            as_holder: value.as_holder,
            waiting: wire_proposal_waiting(&value.waiting, limits)?,
            funding: list(value.funding.iter().map(|item| wire_funding_offer(item, limits)))?,
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_proposal_action(
    value: &c::ProposalAction,
    limits: &w::Limits,
) -> Result<w::ProposalAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProposalAction::Settings(value) => {
            w::ProposalAction::Settings(Box::new(wire_proposal_action_settings(value, limits)?))
        }
        c::ProposalAction::Batch(value) => w::ProposalAction::Batch(w::ProposalActionBatch::new(
            limits,
            w::ProposalActionBatchParts {
                value: pieces(
                    (value).iter().map(|item| wire_proposed_task(item, limits)),
                    w::ProposedTask::measure,
                    w::ProposedTask::encode,
                )?,
            },
        )?),
        c::ProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::ProposalAction::Effect(w::ProposalActionEffect::new(
                limits,
                w::ProposalActionEffectParts {
                    part: wire_part(part, limits)?,
                    authority: wire_authority_document(authority, limits)?,
                    attempt: *(attempt),
                    completion: *(completion),
                    position: *(position),
                    purpose: *(purpose),
                },
            )?)
        }
        c::ProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            w::ProposalAction::ProcedureEffect(w::ProposalActionProcedureEffect::new(
                limits,
                w::ProposalActionProcedureEffectParts {
                    part: wire_part(part, limits)?,
                    authority: wire_authority_document(authority, limits)?,
                    step: *(step),
                    purpose: *(purpose),
                },
            )?)
        }
        c::ProposalAction::Amend { task, addressed, before, after, reason, release } => {
            w::ProposalAction::Amend(Box::new(w::ProposalActionAmend::new(
                limits,
                w::ProposalActionAmendParts {
                    task: wire_task_link(task, limits)?,
                    addressed: (addressed).as_ref().map(|item| wire_address_change(item, limits)).transpose()?,
                    before: wire_amendment_state(before, limits)?,
                    after: wire_amendment_state(after, limits)?,
                    reason: wire_document_text(reason, limits)?,
                    release: (release).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                },
            )?))
        }
        c::ProposalAction::Widen { task, before, after } => w::ProposalAction::Widen(w::ProposalActionWiden::new(
            limits,
            w::ProposalActionWidenParts {
                task: wire_task_link(task, limits)?,
                before: wire_authority_document(before, limits)?,
                after: wire_authority_document(after, limits)?,
            },
        )?),
        c::ProposalAction::Release { task, hold } => w::ProposalAction::Release(w::ProposalActionRelease::new(
            limits,
            w::ProposalActionReleaseParts {
                task: wire_task_link(task, limits)?,
                hold: (hold).as_ref().map(|item| wire_chip_hold(item, limits)).transpose()?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_proposal_waiting(
    value: &c::ProposalWaiting,
    limits: &w::Limits,
) -> Result<w::ProposalWaiting, w::Problem> {
    w::ProposalWaiting::new(
        limits,
        w::ProposalWaitingParts { holder: wire_history_holder(&value.holder, limits)?, since: value.since.as_nanos() },
    )
}

pub(in crate::codec) fn wire_funding_offer(
    value: &c::FundingOffer,
    limits: &w::Limits,
) -> Result<w::FundingOffer, w::Problem> {
    w::FundingOffer::new(
        limits,
        w::FundingOfferParts {
            source: wire_funder(&value.source, limits)?,
            needed: value.needed,
            left: value.left.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            after: value.after.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}
