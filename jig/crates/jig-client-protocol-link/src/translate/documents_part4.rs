//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{Limits, List, Reader, bytes, c, piece_problem, too_large, w};

pub(crate) fn cursor_fields(value: &w::Cursor, limits: &Limits) -> Result<c::Cursor, w::Problem> {
    match c::Cursor::new(bytes::copy_of(value.value()), limits.wire().cursor_value) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}

pub(crate) fn object_key_fields(value: &w::ObjectKey) -> c::ObjectKey {
    match value {
        w::ObjectKey::Task(row) => c::ObjectKey::Task(row.value()),
        w::ObjectKey::Proposal(row) => c::ObjectKey::Proposal(crate::translate::answers::proposal_key(row.value())),
        w::ObjectKey::Escalation(row) => c::ObjectKey::Escalation(row.value()),
        w::ObjectKey::Question(row) => c::ObjectKey::Question { task: row.task(), message: row.message() },
        w::ObjectKey::PersonTask(row) => c::ObjectKey::PersonTask(row.value()),
        w::ObjectKey::Result(row) => c::ObjectKey::Result(row.value()),
        w::ObjectKey::Note(row) => c::ObjectKey::Note { project: row.project(), note: row.note() },
    }
}

pub(crate) fn owner_phase_counts_fields(value: &w::OwnerPhaseCounts) -> c::OwnerPhaseCounts {
    c::OwnerPhaseCounts {
        waiting: value.waiting(),
        idle: value.idle(),
        starting: value.starting(),
        running: value.running(),
        backing_off: value.backing_off(),
        closing: value.closing(),
        held: value.held(),
        done: value.done(),
        failed: value.failed(),
        cancelled: value.cancelled(),
    }
}

pub(crate) fn task_line_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::TaskLine>, w::Problem> {
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
        let row = w::TaskLine::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = crate::translate::queries::line(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn tasks_fields(value: &w::Tasks, limits: &Limits) -> Result<c::Tasks, w::Problem> {
    Ok(c::Tasks { rows: task_line_piece(value.rows(), limits, limits.client().rows)?, more: value.more() })
}
