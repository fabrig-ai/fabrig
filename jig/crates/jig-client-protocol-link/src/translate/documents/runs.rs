//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{Limits, List, Wall, application_part_piece, c, object_key_fields, text, too_large, w};

pub(crate) fn provisional_turn_name_fields(value: &w::ProvisionalTurnName) -> c::ProvisionalTurnName {
    c::ProvisionalTurnName { attempt: value.attempt(), number: value.number(), retry: value.retry() }
}

pub(crate) fn call_fields(value: &w::Call, limits: &Limits) -> Result<c::Call, w::Problem> {
    let opt0 = match value.outcome() {
        Some(item) => Some(text(item, limits)?),
        None => None,
    };
    if value.objects().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.objects().len());
    for item in value.objects() {
        list1.push(object_key_fields(item)).expect("reserved supplied list");
    }
    Ok(c::Call {
        number: value.number(),
        tool: text(value.tool(), limits)?,
        input: text(value.input(), limits)?,
        outcome: opt0,
        failed: value.failed(),
        objects: list1,
        parts: application_part_piece(value.parts(), limits, limits.client().rows)?,
    })
}

pub(crate) fn current_call_fields(value: &w::CurrentCall, limits: &Limits) -> Result<c::CurrentCall, w::Problem> {
    let opt0 = match value.began() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::CurrentCall {
        identity: crate::translate::facts::call_identity(value.identity(), limits)?,
        tool: text(value.tool(), limits)?,
        input: text(value.input(), limits)?,
        cut: value.cut(),
        began: opt0,
    })
}

pub(crate) fn live_call_fields(value: &w::LiveCall, limits: &Limits) -> Result<c::LiveCall, w::Problem> {
    Ok(match value {
        w::LiveCall::Decoded(row) => c::LiveCall::Decoded(call_fields(row.value(), limits)?),
        w::LiveCall::Raw(row) => c::LiveCall::Raw(current_call_fields(row.value(), limits)?),
    })
}

pub(crate) fn run_text_fields(value: &w::RunText, limits: &Limits) -> Result<c::RunText, w::Problem> {
    let opt0 = match value.began() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.call() {
        Some(item) => Some(live_call_fields(item, limits)?),
        None => None,
    };
    Ok(c::RunText {
        writing: match value.writing() {
            Some(call) => {
                if call.through().checked_sub(call.from()) != u32::try_from(call.input().len()).ok() {
                    return Err(too_large());
                }
                Some(c::WritingCall {
                    id: match c::Text::new(call.id().into(), 256) {
                        Ok(text) => text,
                        Err(_) => return Err(too_large()),
                    },
                    tool: match c::Text::new(call.tool().into(), 64) {
                        Ok(text) => text,
                        Err(_) => return Err(too_large()),
                    },
                    input: text(call.input(), limits)?,
                    from: call.from(),
                    through: call.through(),
                    cut: call.cut(),
                })
            }
            None => None,
        },
        attempt: value.attempt(),
        began: opt0,
        turn: provisional_turn_name_fields(value.turn()),
        from: value.from(),
        through: value.through(),
        text: match c::Text::new(skein_lib::bytes::copy_of(value.text()), limits.client().long_text) {
            Ok(text) => text,
            Err(_) => return Err(too_large()),
        },
        cut: value.cut(),
        call: opt1,
    })
}

pub(crate) fn run_state_fields(value: &w::RunState, limits: &Limits) -> Result<c::RunState, w::Problem> {
    let opt0 = match value.attempt() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.writing() {
        Some(item) => Some(run_text_fields(item, limits)?),
        None => None,
    };
    let opt2 = match value.call() {
        Some(item) => Some(current_call_fields(item, limits)?),
        None => None,
    };
    Ok(c::RunState {
        task: value.task(),
        attempt: opt0,
        committed_turn: value.committed_turn(),
        writing: opt1,
        call: opt2,
    })
}
