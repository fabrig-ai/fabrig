//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, activity_line_fields, archived_turn_fields, authority_document_fields, c, cursor_fields,
    piece_problem, proposed_task_piece, specification_document_fields, too_large, w,
};

pub(crate) fn history_change_fields(value: &w::HistoryChange) -> c::HistoryChange {
    match value {
        w::HistoryChange::Cancelled => c::HistoryChange::Cancelled,
        w::HistoryChange::Released => c::HistoryChange::Released,
        w::HistoryChange::Amended => c::HistoryChange::Amended,
        w::HistoryChange::Moved => c::HistoryChange::Moved,
        w::HistoryChange::Proposed => c::HistoryChange::Proposed,
        w::HistoryChange::ProposalPassed => c::HistoryChange::ProposalPassed,
        w::HistoryChange::ProposalAccepted => c::HistoryChange::ProposalAccepted,
        w::HistoryChange::ProposalRejected => c::HistoryChange::ProposalRejected,
        w::HistoryChange::ProposalWithdrawn => c::HistoryChange::ProposalWithdrawn,
    }
}

pub(crate) fn address_change_fields(value: &w::AddressChange) -> c::AddressChange {
    c::AddressChange {
        revision: crate::translate::answers::revision(value.revision()),
        address: crate::translate::facts::person_address(value.address()),
    }
}

pub(crate) fn retained_amendment_fields(
    value: &w::RetainedAmendment,
    limits: &Limits,
) -> Result<c::RetainedAmendment, w::Problem> {
    let opt0 = match value.addressed() {
        Some(item) => Some(address_change_fields(item)),
        None => None,
    };
    let opt1 = match value.instructions() {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    let opt2 = match value.release() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.spec() {
        Some(item) => Some(specification_document_fields(item, limits)?),
        None => None,
    };
    let opt4 = match value.authority() {
        Some(item) => Some(authority_document_fields(item, limits)?),
        None => None,
    };
    let opt5 = match value.wake() {
        Some(item) => Some(crate::translate::facts::wake_policy(item)),
        None => None,
    };
    let opt6 = match value.dependencies() {
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
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        cut: value.cut(),
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
            c::RetainedAction::Batch(proposed_task_piece(row.value(), limits, limits.client().rows)?)
        }
        w::RetainedAction::Effect(row) => c::RetainedAction::Effect {
            part: crate::translate::facts::part(row.part()),
            authority: authority_document_fields(row.authority(), limits)?,
            attempt: row.attempt(),
            completion: row.completion(),
            position: row.position(),
            purpose: row.purpose(),
        },
        w::RetainedAction::ProcedureEffect(row) => c::RetainedAction::ProcedureEffect {
            part: crate::translate::facts::part(row.part()),
            authority: authority_document_fields(row.authority(), limits)?,
            step: row.step(),
            purpose: row.purpose(),
        },
        w::RetainedAction::Amend(row) => c::RetainedAction::Amend {
            task: row.task(),
            change: Box::new(retained_amendment_fields(row.change(), limits)?),
        },
        w::RetainedAction::Widen(row) => c::RetainedAction::Widen {
            task: row.task(),
            authority: authority_document_fields(row.authority(), limits)?,
        },
        w::RetainedAction::Release(row) => c::RetainedAction::Release { task: row.task() },
    })
}

pub(crate) fn activity_page_fields(value: &w::ActivityPage, limits: &Limits) -> Result<c::ActivityPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::ActivityPage {
        cut: value.cut(),
        rows: activity_line_piece(value.rows(), limits, limits.client().rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn activity_line_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::DecodedActivityLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::DecodedActivityLine::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = decoded_activity_line_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn decoded_activity_line_fields(
    value: &w::DecodedActivityLine,
    limits: &Limits,
) -> Result<c::DecodedActivityLine, w::Problem> {
    let decoded = match value.decoded() {
        Some(item) => Some(archived_turn_fields(item, limits)),
        None => None,
    };
    Ok(c::DecodedActivityLine { source: Box::new(activity_line_fields(value.source(), limits)?), decoded })
}
