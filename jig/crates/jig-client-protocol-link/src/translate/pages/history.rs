//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{
    Box, Limits, List, Reader, Wall, authority_document_fields, c, cursor_fields, proposed_task_piece,
    specification_document_fields, too_large, w,
};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn address_change_fields(value: &w::AddressChange) -> c::AddressChange {
    client::address_change_to_domain(value.clone())
}

pub(crate) fn retained_amendment_fields(
    value: &w::RetainedAmendment,
    limits: &Limits,
) -> Result<c::RetainedAmendment, w::Problem> {
    let opt0 = match &value.addressed {
        Some(item) => Some(address_change_fields(item)),
        None => None,
    };
    let opt1 = match &value.instructions {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    let opt2 = match &value.release {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match &value.spec {
        Some(item) => Some(specification_document_fields(item, limits)?),
        None => None,
    };
    let opt4 = match &value.authority {
        Some(item) => Some(authority_document_fields(item, limits)?),
        None => None,
    };
    let opt5 = match &value.wake {
        Some(item) => Some(crate::translate::facts::wake_policy(item)),
        None => None,
    };
    let opt6 = match &value.dependencies {
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
    Ok(c::RetainedAmendment {
        addressed: opt0,
        instructions: opt1,
        release: opt2,
        spec: opt3,
        authority: opt4,
        wake: opt5,
        dependencies: opt6,
        reason: crate::translate::facts::document_text(&value.reason, limits)?,
        cut: value.cut,
    })
}

pub(crate) fn retained_action_fields(
    value: &w::RetainedAction,
    limits: &Limits,
) -> Result<c::RetainedAction, w::Problem> {
    Ok(match value {
        w::RetainedAction::Settings(row) => c::RetainedAction::Settings(
            crate::translate::settings_proposals::restore_owner_retained_action_settings(row, limits)?,
        ),
        w::RetainedAction::Batch(row) => {
            c::RetainedAction::Batch(proposed_task_piece(&row.value, limits, limits.client().rows)?)
        }
        w::RetainedAction::Effect(row) => c::RetainedAction::Effect {
            part: crate::translate::facts::part(&row.part),
            authority: authority_document_fields(&row.authority, limits)?,
            attempt: row.attempt,
            completion: row.completion,
            position: row.position,
            purpose: row.purpose,
        },
        w::RetainedAction::ProcedureEffect(row) => c::RetainedAction::ProcedureEffect {
            part: crate::translate::facts::part(&row.part),
            authority: authority_document_fields(&row.authority, limits)?,
            step: row.step,
            purpose: row.purpose,
        },
        w::RetainedAction::Amend(row) => c::RetainedAction::Amend {
            task: row.task,
            change: Box::new(retained_amendment_fields(&row.change, limits)?),
        },
        w::RetainedAction::Widen(row) => {
            c::RetainedAction::Widen { task: row.task, authority: authority_document_fields(&row.authority, limits)? }
        }
        w::RetainedAction::Release(row) => c::RetainedAction::Release { task: row.task },
    })
}

#[cfg(test)]
pub(crate) fn proposal_history_fields(
    value: &w::ProposalHistory,
    limits: &Limits,
) -> Result<c::ProposalHistory, w::Problem> {
    client::proposal_history_to_domain(value.clone(), limits)
}

pub(crate) fn amendment_state_fields(
    value: &w::AmendmentState,
    limits: &Limits,
) -> Result<c::AmendmentState, w::Problem> {
    let opt0 = match &value.addressed {
        Some(item) => Some(crate::translate::facts::person_address(item)),
        None => None,
    };
    let opt1 = match &value.taken_by {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match &value.instructions {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    if value.dependencies.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.dependencies.len());
    for item in &value.dependencies {
        list3.push(*item).expect("reserved supplied list");
    }
    Ok(c::AmendmentState {
        addressed: opt0,
        taken_by: opt1,
        instructions: opt2,
        spec: specification_document_fields(&value.spec, limits)?,
        authority: authority_document_fields(&value.authority, limits)?,
        wake: crate::translate::facts::wake_policy(&value.wake),
        dependencies: list3,
        cut: value.cut,
    })
}

pub(crate) fn history_line_fields(value: w::HistoryLine, limits: &Limits) -> Result<c::HistoryLine, w::Problem> {
    let opt0 = match value.proposal {
        Some(item) => Some(client::proposal_history_to_domain(item, limits)?),
        None => None,
    };
    let opt1 = match &value.amendment {
        Some(item) => Some((
            Box::new(amendment_state_fields(&item.before, limits)?),
            Box::new(amendment_state_fields(&item.after, limits)?),
        )),
        None => None,
    };
    let opt2 = match &value.at {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::HistoryLine {
        task: value.task,
        revision: value.revision,
        by: crate::translate::facts::requester(&value.by),
        change: client::history_change_to_domain(value.change.clone()),
        reason: crate::translate::facts::document_text(&value.reason, limits)?,
        proposal: opt0,
        amendment: opt1,
        at: opt2,
    })
}

#[cfg(test)]
pub(crate) fn attempt_line_fields(value: &w::AttemptLine) -> c::AttemptLine {
    client::attempt_line_to_domain(value.clone()).expect("valid attempt fixture")
}

pub(crate) fn history_row_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::HistoryRow>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::HistoryRow::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = client::history_row_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn history_page_fields(value: &w::HistoryPage, limits: &Limits) -> Result<c::HistoryPage, w::Problem> {
    let opt0 = match &value.before {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match &value.after {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::HistoryPage {
        cut: value.cut,
        rows: history_row_piece(&value.rows, limits, limits.client().rows)?,
        before: opt0,
        after: opt1,
    })
}
