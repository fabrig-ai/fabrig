//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1).
use super::{Limits, List, Wall, application_part_piece, c, object_key_fields, text, too_large, w};
use crate::generated::client;

pub(crate) fn call_fields(value: &w::Call, limits: &Limits) -> Result<c::Call, w::Problem> {
    let opt0 = match &value.outcome {
        Some(item) => Some(text(item, limits)?),
        None => None,
    };
    if value.objects.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.objects.len());
    for item in &value.objects {
        list1.push(object_key_fields(item)).expect("reserved supplied list");
    }
    Ok(c::Call {
        number: value.number,
        tool: text(&value.tool, limits)?,
        input: text(&value.input, limits)?,
        outcome: opt0,
        failed: value.failed,
        objects: list1,
        parts: application_part_piece(&value.parts)?,
    })
}

pub(crate) fn current_call_fields(value: &w::CurrentCall, limits: &Limits) -> Result<c::CurrentCall, w::Problem> {
    let opt0 = match &value.began {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::CurrentCall {
        identity: crate::translate::facts::call_identity(&value.identity, limits)?,
        tool: text(&value.tool, limits)?,
        input: text(&value.input, limits)?,
        cut: value.cut,
        began: opt0,
    })
}

pub(crate) fn run_text_fields(value: &w::RunText, limits: &Limits) -> Result<c::RunText, w::Problem> {
    let opt0 = match &value.began {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match &value.call {
        Some(item) => Some(client::live_call_to_domain(item.clone(), limits)?),
        None => None,
    };
    Ok(c::RunText {
        writing: match &value.writing {
            Some(call) => {
                if call.through.checked_sub(call.from) != u32::try_from(call.input.len()).ok() {
                    return Err(too_large());
                }
                Some(c::WritingCall {
                    id: match c::Text::new(call.id.clone(), 256) {
                        Ok(text) => text,
                        Err(_) => return Err(too_large()),
                    },
                    tool: match c::Text::new(call.tool.clone(), 64) {
                        Ok(text) => text,
                        Err(_) => return Err(too_large()),
                    },
                    input: text(&call.input, limits)?,
                    from: call.from,
                    through: call.through,
                    cut: call.cut,
                })
            }
            None => None,
        },
        attempt: value.attempt,
        began: opt0,
        turn: client::provisional_turn_name_to_domain(value.turn.clone()),
        from: value.from,
        through: value.through,
        text: match c::Text::new(skein_lib::bytes::copy_of(&value.text), limits.client().long_text) {
            Ok(text) => text,
            Err(_) => return Err(too_large()),
        },
        cut: value.cut,
        call: opt1,
    })
}
