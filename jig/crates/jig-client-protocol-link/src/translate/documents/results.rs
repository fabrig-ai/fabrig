//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{
    Box, Limits, Wall, c, chip_fields, offered_piece, piece_problem, task_contract_document_fields, task_link_piece, w,
};

pub(crate) fn result_value_fields(value: &w::ResultValue, limits: &Limits) -> Result<c::ResultValue, w::Problem> {
    match crate::translate::facts::decode(value.rows(), limits)? {
        crate::translate::facts::Node::ResultValue(value) => Ok(*value),
        crate::translate::facts::Node::Chip(_)
        | crate::translate::facts::Node::ChipPhaseFacts(_)
        | crate::translate::facts::Node::Closing(_)
        | crate::translate::facts::Node::Ending(_)
        | crate::translate::facts::Node::HeldRuntime(_)
        | crate::translate::facts::Node::HoldReason(_)
        | crate::translate::facts::Node::ChipDetails(_)
        | crate::translate::facts::Node::ChipEnding(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn spend_line_fields(value: &w::SpendLine) -> c::SpendLine {
    let opt0 = match value.attempt() {
        Some(item) => Some(*item),
        None => None,
    };
    c::SpendLine { task: value.task(), attempt: opt0, spent: value.spent() }
}

pub(crate) fn ending_fields(value: &w::Ending, limits: &Limits) -> Result<c::Ending, w::Problem> {
    match crate::translate::facts::decode(value.rows(), limits)? {
        crate::translate::facts::Node::Ending(value) => Ok(*value),
        crate::translate::facts::Node::Chip(_)
        | crate::translate::facts::Node::ChipPhaseFacts(_)
        | crate::translate::facts::Node::Closing(_)
        | crate::translate::facts::Node::HeldRuntime(_)
        | crate::translate::facts::Node::HoldReason(_)
        | crate::translate::facts::Node::ChipDetails(_)
        | crate::translate::facts::Node::ChipEnding(_)
        | crate::translate::facts::Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn result_details_fields(value: &w::ResultDetails, limits: &Limits) -> Result<c::ResultDetails, w::Problem> {
    let opt0 = match value.chosen_by() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.followups() {
        Some(item) => Some(task_link_piece(item, limits, limits.client().rows)?),
        None => None,
    };
    let opt2 = match value.proposal() {
        Some(item) => Some(crate::translate::answers::proposal_key(item)),
        None => None,
    };
    Ok(c::ResultDetails {
        contract: task_contract_document_fields(value.contract(), limits)?,
        requester: crate::translate::facts::requester(value.requester()),
        executor: crate::translate::facts::executor(value.executor()),
        chosen_by: opt0,
        delegates: task_link_piece(value.delegates(), limits, limits.client().rows)?,
        followups: opt1,
        proposal: opt2,
        cut: value.cut(),
    })
}

pub(crate) fn result_fields(value: &w::ResultDocument, limits: &Limits) -> Result<c::Result, w::Problem> {
    let opt0 = match value.value() {
        Some(item) => Some(result_value_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match value.details() {
        Some(item) => Some(Box::new(result_details_fields(item, limits)?)),
        None => None,
    };
    Ok(c::Result {
        task: chip_fields(value.task(), limits)?,
        ending: ending_fields(value.ending(), limits)?,
        value: opt0,
        at: opt1,
        details: opt2,
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
    })
}

pub(crate) fn chip_ending_fields(value: &w::ChipEnding, limits: &Limits) -> Result<c::ChipEnding, w::Problem> {
    match crate::translate::facts::decode(value.rows(), limits)? {
        crate::translate::facts::Node::ChipEnding(value) => Ok(*value),
        crate::translate::facts::Node::Chip(_)
        | crate::translate::facts::Node::ChipPhaseFacts(_)
        | crate::translate::facts::Node::Closing(_)
        | crate::translate::facts::Node::Ending(_)
        | crate::translate::facts::Node::HeldRuntime(_)
        | crate::translate::facts::Node::HoldReason(_)
        | crate::translate::facts::Node::ChipDetails(_)
        | crate::translate::facts::Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn task_result_fields(value: &w::TaskResult, limits: &Limits) -> Result<c::TaskResult, w::Problem> {
    Ok(match value {
        w::TaskResult::Document(row) => c::TaskResult::Document(Box::new(result_fields(row.value(), limits)?)),
        w::TaskResult::Ending(row) => c::TaskResult::Ending(Box::new(chip_ending_fields(row.value(), limits)?)),
    })
}

pub(crate) fn effect_fields(value: &w::Effect) -> c::Effect {
    match value {
        w::Effect::Release(value) => c::Effect::Release(match value.what() {
            w::ReleaseEffect::Judged => c::ReleaseEffect::Judged,
            w::ReleaseEffect::RunAgain => c::ReleaseEffect::RunAgain,
            w::ReleaseEffect::Uncertain => c::ReleaseEffect::Uncertain,
        }),
        w::Effect::Cancel(value) => c::Effect::Cancel(super::impact_fields(value)),
        w::Effect::MoveTask(value) => c::Effect::Move { reservation: value.reservation() },
    }
}
