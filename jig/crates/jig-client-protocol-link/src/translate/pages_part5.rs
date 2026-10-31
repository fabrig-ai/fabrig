//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Wall, address_change_fields, amendment_state_fields, authority_document_fields,
    authority_fields, c, cursor_fields, history_holder_fields, period_times_fields, proposed_task_piece,
    task_link_fields, text, too_large, w,
};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn funding_record_fields(value: &w::FundingRecord) -> c::FundingRecord {
    let opt0 = match value.period() {
        Some(item) => Some(period_times_fields(item)),
        None => None,
    };
    let opt1 = match value.parent() {
        Some(item) => Some(crate::translate::facts::funder(item)),
        None => None,
    };
    c::FundingRecord {
        period: opt0,
        funder: crate::translate::facts::funder(value.funder()),
        parent: opt1,
        numbers: crate::translate::facts::numbers(value.numbers()),
        made: value.made(),
        closed: value.closed(),
    }
}

pub(crate) fn spend_page_fields(value: &w::SpendPage, limits: &Limits) -> Result<c::SpendPage, w::Problem> {
    if value.rows().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.rows().len());
    for item in value.rows() {
        list0.push(funding_record_fields(item)).expect("reserved supplied list");
    }
    let opt1 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt2 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::SpendPage { rows: list0, before: opt1, after: opt2, cut: value.cut() })
}

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

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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
