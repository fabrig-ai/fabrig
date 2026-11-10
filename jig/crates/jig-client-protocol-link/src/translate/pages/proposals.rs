//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{
    Box, Limits, List, Wall, address_change_fields, amendment_state_fields, authority_document_fields,
    authority_fields, c, chip_fields, ending_fields, history_holder_fields, offered_piece, proposed_task_piece,
    task_fields, task_link_fields, text, too_large, w,
};

pub(crate) fn parameter_fields(value: &w::Parameter, limits: &Limits) -> Result<c::Parameter, w::Problem> {
    Ok(match value {
        w::Parameter::Number(row) => c::Parameter::Number { name: row.name(), value: row.value() },
        w::Parameter::Bytes(row) => c::Parameter::Bytes { name: row.name(), value: text(row.value(), limits)? },
        w::Parameter::Resource(row) => {
            c::Parameter::Resource { name: row.name(), connector: row.connector(), resource: row.resource() }
        }
    })
}

pub(crate) fn specification_fields(value: &w::Specification, limits: &Limits) -> Result<c::Specification, w::Problem> {
    if value.parameters().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.parameters().len());
    for item in value.parameters() {
        list0.push(parameter_fields(item, limits)?).expect("reserved supplied list");
    }
    if value.inputs().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.inputs().len());
    for item in value.inputs() {
        list1.push(*item).expect("reserved supplied list");
    }
    Ok(c::Specification { asks: value.asks(), words: text(value.words(), limits)?, parameters: list0, inputs: list1 })
}

pub(crate) fn amendment_fields(value: &w::Amendment, limits: &Limits) -> Result<c::Amendment, w::Problem> {
    let opt0 = match value.addressed() {
        Some(item) => Some(address_change_fields(item)),
        None => None,
    };
    let opt1 = match value.spec() {
        Some(item) => Some(specification_fields(item, limits)?),
        None => None,
    };
    let opt2 = match value.wake() {
        Some(item) => Some(crate::translate::facts::wake_policy(item)),
        None => None,
    };
    let opt3 = match value.dependencies() {
        Some(item) => {
            if (item).len() > limits.client().rows {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt4 = match value.authority() {
        Some(item) => Some(authority_fields(item, limits)?),
        None => None,
    };
    let opt5 = match value.instructions() {
        Some(item) => Some(text(item, limits)?),
        None => None,
    };
    Ok(c::Amendment {
        addressed: opt0,
        spec: opt1,
        wake: opt2,
        dependencies: opt3,
        authority: opt4,
        instructions: opt5,
        tracked: match value.tracked() {
            Some(value) => Some(crate::translate::operations::restore_tracking(value)),
            None => None,
        },
        llm: crate::translate::llm::choice(value.llm().as_ref(), limits)?,
    })
}

pub(crate) fn proposal_action_fields(
    value: &w::ProposalAction,
    limits: &Limits,
) -> Result<c::ProposalAction, w::Problem> {
    Ok(match value {
        w::ProposalAction::Settings(row) => c::ProposalAction::Settings(Box::new(
            crate::translate::settings_proposals::restore_owner_proposal_action_settings(row, limits)?,
        )),
        w::ProposalAction::Batch(row) => {
            c::ProposalAction::Batch(proposed_task_piece(row.value(), limits, limits.client().rows)?)
        }
        w::ProposalAction::Effect(row) => c::ProposalAction::Effect {
            part: crate::translate::facts::part(row.part()),
            authority: authority_document_fields(row.authority(), limits)?,
            attempt: row.attempt(),
            completion: row.completion(),
            position: row.position(),
            purpose: row.purpose(),
        },
        w::ProposalAction::ProcedureEffect(row) => c::ProposalAction::ProcedureEffect {
            part: crate::translate::facts::part(row.part()),
            authority: authority_document_fields(row.authority(), limits)?,
            step: row.step(),
            purpose: row.purpose(),
        },
        w::ProposalAction::Amend(row) => {
            let opt0 = match row.addressed() {
                Some(item) => Some(address_change_fields(item)),
                None => None,
            };
            let opt1 = match row.release() {
                Some(item) => Some(*item),
                None => None,
            };
            c::ProposalAction::Amend {
                task: task_link_fields(row.task(), limits)?,
                addressed: opt0,
                before: Box::new(amendment_state_fields(row.before(), limits)?),
                after: Box::new(amendment_state_fields(row.after(), limits)?),
                reason: crate::translate::facts::document_text(row.reason(), limits)?,
                release: opt1,
            }
        }
        w::ProposalAction::Widen(row) => c::ProposalAction::Widen {
            task: task_link_fields(row.task(), limits)?,
            before: authority_document_fields(row.before(), limits)?,
            after: authority_document_fields(row.after(), limits)?,
        },
        w::ProposalAction::Release(row) => {
            let opt0 = match row.hold() {
                Some(item) => Some(Box::new(crate::translate::facts::chip_hold(item, limits)?)),
                None => None,
            };
            c::ProposalAction::Release { task: task_link_fields(row.task(), limits)?, hold: opt0 }
        }
    })
}

pub(crate) fn proposal_waiting_fields(value: &w::ProposalWaiting) -> c::ProposalWaiting {
    c::ProposalWaiting { holder: history_holder_fields(value.holder()), since: Wall::from_nanos(value.since()) }
}

pub(crate) fn funding_offer_fields(value: &w::FundingOffer) -> c::FundingOffer {
    let opt0 = match value.left() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(*item),
        None => None,
    };
    c::FundingOffer {
        source: crate::translate::facts::funder(value.source()),
        needed: value.needed(),
        left: opt0,
        after: opt1,
    }
}

pub(crate) fn proposal_fields(value: &w::Proposal, limits: &Limits) -> Result<c::Proposal, w::Problem> {
    if value.funding().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.funding().len());
    for item in value.funding() {
        list0.push(funding_offer_fields(item)).expect("reserved supplied list");
    }
    Ok(c::Proposal {
        projects: value.projects().clone(),
        key: crate::translate::answers::proposal_key(value.key()),
        project: value.project(),
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        action: proposal_action_fields(value.action(), limits)?,
        as_holder: value.as_holder(),
        waiting: proposal_waiting_fields(value.waiting()),
        funding: list0,
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
    })
}

pub(crate) fn escalation_fields(value: &w::Escalation, limits: &Limits) -> Result<c::Escalation, w::Problem> {
    Ok(c::Escalation {
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        task: Box::new(task_fields(value.task(), limits)?),
    })
}

pub(crate) fn question_fields(value: &w::Question, limits: &Limits) -> Result<c::Question, w::Problem> {
    let opt0 = match value.from() {
        Some(item) => Some(crate::translate::facts::requester(item)),
        None => None,
    };
    let opt1 = match value.ending() {
        Some(item) => Some(ending_fields(item, limits)?),
        None => None,
    };
    Ok(c::Question {
        task: chip_fields(value.task(), limits)?,
        from: opt0,
        message: value.message(),
        words: crate::translate::facts::document_text(value.words(), limits)?,
        since: Wall::from_nanos(value.since()),
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        ending: opt1,
    })
}
