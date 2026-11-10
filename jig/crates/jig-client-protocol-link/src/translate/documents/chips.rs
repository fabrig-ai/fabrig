//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{Limits, List, Reader, c, piece_problem, too_large, w};

pub(crate) fn chip_fields(value: &w::Chip, limits: &Limits) -> Result<c::Chip, w::Problem> {
    match crate::translate::facts::decode(value.rows(), limits)? {
        crate::translate::facts::Node::Chip(value) => Ok(*value),
        crate::translate::facts::Node::ChipPhaseFacts(_)
        | crate::translate::facts::Node::Closing(_)
        | crate::translate::facts::Node::Ending(_)
        | crate::translate::facts::Node::HeldRuntime(_)
        | crate::translate::facts::Node::HoldReason(_)
        | crate::translate::facts::Node::ChipDetails(_)
        | crate::translate::facts::Node::ChipEnding(_)
        | crate::translate::facts::Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn chip_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Chip>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::Chip::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = chip_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
