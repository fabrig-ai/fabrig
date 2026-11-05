//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, Wall, c, chip_fields, offered_piece, piece_problem, result_fields,
    specification_document_fields, task_contract_document_fields, task_link_fields, too_large, w,
};

pub(crate) fn shown_result_fields(value: &w::ShownResult, limits: &Limits) -> Result<c::ShownResult, w::Problem> {
    let opt0 = match value.result() {
        Some(item) => Some(Box::new(result_fields(item, limits)?)),
        None => None,
    };
    Ok(c::ShownResult { task: task_link_fields(value.task(), limits)?, result: opt0 })
}

pub(crate) fn shown_result_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ShownResult>, w::Problem> {
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
        let row = w::ShownResult::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = shown_result_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn person_task_fields(value: &w::PersonTask, limits: &Limits) -> Result<c::PersonTask, w::Problem> {
    let opt0 = match value.taken_by() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.since() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match value.stall() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.deadline() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::PersonTask {
        task: chip_fields(value.task(), limits)?,
        revision: crate::translate::answers::revision(value.revision()),
        requester: crate::translate::facts::requester(value.requester()),
        spec: specification_document_fields(value.spec(), limits)?,
        address: crate::translate::facts::person_address(value.address()),
        taken_by: opt0,
        since: opt1,
        stall: opt2,
        deadline: opt3,
        contract: task_contract_document_fields(value.contract(), limits)?,
        results: shown_result_piece(value.results(), limits, limits.client().rows)?,
        cut: value.cut(),
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
    })
}
