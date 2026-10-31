//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, Wall, c, chip_fields, ending_fields, offered_piece, piece_problem,
    proposal_action_fields, proposal_waiting_fields, result_fields, specification_document_fields,
    task_contract_document_fields, task_fields, task_link_fields, too_large, w,
};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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
