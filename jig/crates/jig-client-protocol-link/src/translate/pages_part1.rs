//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, archived_turn_fields, c, cursor_fields, owner_activity_line_fields,
    owner_authority_document_fields, piece_problem, proposed_task_piece, specification_document_fields, too_large, w,
};

pub(crate) fn owner_history_change_fields(value: &w::OwnerHistoryChange) -> c::OwnerHistoryChange {
    match value {
        w::OwnerHistoryChange::Cancelled => c::OwnerHistoryChange::Cancelled,
        w::OwnerHistoryChange::Released => c::OwnerHistoryChange::Released,
        w::OwnerHistoryChange::Amended => c::OwnerHistoryChange::Amended,
        w::OwnerHistoryChange::Moved => c::OwnerHistoryChange::Moved,
        w::OwnerHistoryChange::Proposed => c::OwnerHistoryChange::Proposed,
        w::OwnerHistoryChange::ProposalPassed => c::OwnerHistoryChange::ProposalPassed,
        w::OwnerHistoryChange::ProposalAccepted => c::OwnerHistoryChange::ProposalAccepted,
        w::OwnerHistoryChange::ProposalRejected => c::OwnerHistoryChange::ProposalRejected,
        w::OwnerHistoryChange::ProposalWithdrawn => c::OwnerHistoryChange::ProposalWithdrawn,
    }
}

pub(crate) fn address_change_fields(value: &w::AddressChange) -> c::AddressChange {
    c::AddressChange {
        revision: crate::translate::answers::revision(value.revision()),
        address: crate::translate::facts::person_address(value.address()),
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_retained_amendment_fields(
    value: &w::OwnerRetainedAmendment,
    limits: &Limits,
) -> Result<c::OwnerRetainedAmendment, w::Problem> {
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
        Some(item) => Some(owner_authority_document_fields(item, limits)?),
        None => None,
    };
    let opt5 = match value.wake() {
        Some(item) => Some(crate::translate::facts::owner_wake_policy(item)),
        None => None,
    };
    let opt6 = match value.dependencies() {
        Some(item) => {
            if (item).len() > limits.client.rows {
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
    Ok(c::OwnerRetainedAmendment {
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

pub(crate) fn owner_retained_action_fields(
    value: &w::OwnerRetainedAction,
    limits: &Limits,
) -> Result<c::OwnerRetainedAction, w::Problem> {
    Ok(match value {
        w::OwnerRetainedAction::Settings(row) => c::OwnerRetainedAction::Settings(
            crate::translate::settings_proposals::restore_owner_retained_action_settings(row, limits)?,
        ),
        w::OwnerRetainedAction::Batch(row) => {
            c::OwnerRetainedAction::Batch(proposed_task_piece(row.value(), limits, limits.client.rows)?)
        }
        w::OwnerRetainedAction::Effect(row) => c::OwnerRetainedAction::Effect {
            part: crate::translate::facts::owner_part(row.part()),
            authority: owner_authority_document_fields(row.authority(), limits)?,
            attempt: row.attempt(),
            completion: row.completion(),
            position: row.position(),
            purpose: row.purpose(),
        },
        w::OwnerRetainedAction::ProcedureEffect(row) => c::OwnerRetainedAction::ProcedureEffect {
            part: crate::translate::facts::owner_part(row.part()),
            authority: owner_authority_document_fields(row.authority(), limits)?,
            step: row.step(),
            purpose: row.purpose(),
        },
        w::OwnerRetainedAction::Amend(row) => c::OwnerRetainedAction::Amend {
            task: row.task(),
            change: Box::new(owner_retained_amendment_fields(row.change(), limits)?),
        },
        w::OwnerRetainedAction::Widen(row) => c::OwnerRetainedAction::Widen {
            task: row.task(),
            authority: owner_authority_document_fields(row.authority(), limits)?,
        },
        w::OwnerRetainedAction::Release(row) => c::OwnerRetainedAction::Release { task: row.task() },
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
        rows: activity_line_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn activity_line_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ActivityLine>, w::Problem> {
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
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::ActivityLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = activity_line_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn activity_line_fields(value: &w::ActivityLine, limits: &Limits) -> Result<c::ActivityLine, w::Problem> {
    let decoded = match value.decoded() {
        Some(item) => Some(archived_turn_fields(item, limits)),
        None => None,
    };
    Ok(c::ActivityLine { source: Box::new(owner_activity_line_fields(value.source(), limits)?), decoded })
}
