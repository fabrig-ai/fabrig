//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1).
use super::{Limits, List, Reader, bytes, c, too_large, w};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn cursor_fields(value: &w::Cursor, limits: &Limits) -> Result<c::Cursor, w::Problem> {
    match c::Cursor::new(bytes::copy_of(&value.value), limits.wire().cursor_value) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}

pub(crate) fn object_key_fields(value: &w::ObjectKey) -> c::ObjectKey {
    client::object_key_to_domain(value.clone())
}

pub(crate) fn phase_counts_fields(value: &w::PhaseCounts) -> c::PhaseCounts {
    client::phase_counts_to_domain(value.clone())
}

pub(crate) fn task_line_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::TaskLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::TaskLine::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = crate::translate::queries::line_from_record(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn tasks_fields(value: &w::Tasks, limits: &Limits) -> Result<c::Tasks, w::Problem> {
    Ok(c::Tasks {
        ended_mark: value.ended_mark,
        rows: task_line_piece(&value.rows, limits, limits.client().rows)?,
        more: value.more,
    })
}
