//! Owned payload caps checked at admission.
//!
//! Contract: docs/design/testing/fakes/llm.md, sections 2–5; programming-model.md, sections 4.4 and 6.3.

use alloc::boxed::Box;
use core::mem::size_of;
use skein_lib::len_u64;

use crate::api::{
    Answer, Capture, Drawn, Failure, Filled, Line, Message, Part, Piece, Query, Report, Rule, Rules, Script, ToolSpec,
    Turn,
};

pub(crate) fn fits(bytes: Option<u64>, cap: u32) -> bool {
    match bytes {
        Some(bytes) => bytes <= u64::from(cap),
        None => false,
    }
}

fn array(len: usize, item_bytes: usize) -> Option<u64> {
    u64::try_from(len).ok()?.checked_mul(u64::try_from(item_bytes).ok()?)
}

#[expect(clippy::unnecessary_wraps, reason = "owned-byte measurements share an optional checked-accounting signature")]
fn bytes(items: &[u8]) -> Option<u64> {
    Some(len_u64(items.len()))
}

fn parts(items: &[Part]) -> Option<u64> {
    let mut total = array(items.len(), size_of::<Part>())?;
    for part in items {
        let owned = match part {
            Part::Text { text } => bytes(text)?,
            Part::Opaque { bytes: value } => bytes(value)?,
            Part::ToolCall { id, name, arguments } => {
                bytes(id)?.checked_add(bytes(name)?)?.checked_add(bytes(arguments)?)?
            }
            Part::ToolOutput { id, output, is_error: _ } => bytes(id)?.checked_add(bytes(output)?)?,
        };
        total = total.checked_add(owned)?;
    }
    Some(total)
}

pub(crate) fn query(query: &Query) -> Option<u64> {
    let mut total = bytes(&query.model)?.checked_add(bytes(&query.system)?)?;
    total = total
        .checked_add(array(query.tools.len(), size_of::<ToolSpec>())?)?
        .checked_add(array(query.messages.len(), size_of::<Message>())?)?;
    match &query.choice {
        crate::api::ToolChoice::Auto | crate::api::ToolChoice::None => {}
        crate::api::ToolChoice::Only(names) => {
            total = total.checked_add(array(names.len(), size_of::<Box<[u8]>>())?)?;
            for name in names {
                total = total.checked_add(bytes(name)?)?;
            }
        }
    }
    match &query.caching {
        crate::api::Caching::Scope(_) | crate::api::Caching::Unscoped => {}
        crate::api::Caching::Marks(marks) => {
            total = total.checked_add(array(marks.len(), size_of::<crate::api::Mark>())?)?;
        }
    }
    for tool in &query.tools {
        total = total
            .checked_add(bytes(&tool.name)?)?
            .checked_add(bytes(&tool.description)?)?
            .checked_add(bytes(&tool.parameters)?)?;
    }
    for message in &query.messages {
        total = total.checked_add(parts(&message.parts)?)?;
    }
    Some(total)
}

pub(crate) fn rules(rules: &Rules) -> Option<u64> {
    let mut total = 0_u64;
    for group in [&*rules.cued, &*rules.asked] {
        total = total.checked_add(array(group.len(), size_of::<Box<[Rule]>>())?)?;
        for rules in group {
            total = total.checked_add(array(rules.len(), size_of::<Rule>())?)?;
            for rule in rules {
                total = total.checked_add(bytes(&rule.cue)?)?;
            }
        }
    }
    Some(total)
}

pub(crate) fn scripts(scripts: &[Script]) -> Option<u64> {
    let mut total = array(scripts.len(), size_of::<Script>())?;
    for script in scripts {
        total = total.checked_add(bytes(&script.cue)?)?.checked_add(array(script.turns.len(), size_of::<Turn>())?)?;
        for turn in &script.turns {
            total = total.checked_add(array(turn.lines.len(), size_of::<Line>())?)?;
            for line in &turn.lines {
                let owned = match line {
                    Line::Cut { .. } => u64::try_from(size_of::<crate::respond::Cuts>()).ok()?,
                    Line::Pieces { pieces } => {
                        let mut owned = array(pieces.len(), size_of::<Piece>())?;
                        for piece in pieces {
                            owned = owned.checked_add(bytes(&piece.text)?)?;
                            if let Some(name) = &piece.hold {
                                owned = owned.checked_add(bytes(name)?)?;
                            }
                        }
                        owned
                    }
                    Line::Fail { error, report, credentials } => {
                        let resets = if let crate::api::Error::Exhausted { retry_after: Some(_) } = error {
                            array(credentials.len().max(1), size_of::<crate::respond::Reset>())?
                        } else {
                            0
                        };
                        report_bytes(report)?
                            .checked_add(array(credentials.len(), size_of::<u32>())?)?
                            .checked_add(resets)?
                    }
                    Line::Take { name, before, after } => {
                        bytes(name)?.checked_add(bytes(before)?)?.checked_add(bytes(after)?)?
                    }
                    Line::Text { text } => bytes(text)?,
                    Line::Opaque { bytes: value } => bytes(value)?,
                    Line::Call { name, arguments } => bytes(name)?.checked_add(bytes(arguments)?)?,
                };
                total = total.checked_add(owned)?;
            }
        }
    }
    Some(total)
}

fn report_bytes(report: &Report) -> Option<u64> {
    bytes(&report.code)?.checked_add(bytes(&report.request)?)?.checked_add(bytes(&report.field)?)
}

pub(crate) fn failure(failure: &Failure) -> Option<u64> {
    parts(&failure.parts)?.checked_add(report_bytes(&failure.report)?)?.checked_add(bytes(&failure.message)?)
}

pub(crate) fn answer(answer: &Answer) -> Option<u64> {
    parts(&answer.parts)
}

/// The bytes of a scripted answer before any payload is copied.
///
/// Contract: docs/design/testing/fakes/llm.md, sections 2–5; programming-model.md, section 4.4.
pub(crate) fn scripted_answer(turn: &Turn) -> Option<u64> {
    let mut total = len_u64(turn.lines.len()).checked_mul(u64::try_from(size_of::<Part>()).ok()?)?;
    for line in &turn.lines {
        let owned = match line {
            Line::Pieces { pieces } => {
                let mut owned: u64 = 0;
                for piece in pieces {
                    owned = owned.checked_add(bytes(&piece.text)?)?;
                }
                owned
            }
            Line::Fail { report, credentials, .. } => {
                report_bytes(report)?.checked_add(array(credentials.len(), size_of::<u32>())?)?
            }
            Line::Take { .. } | Line::Cut { .. } => {
                total = total.checked_sub(u64::try_from(size_of::<Part>()).ok()?)?;
                0
            }
            Line::Text { text } => bytes(text)?,
            Line::Opaque { bytes: value } => bytes(value)?,
            Line::Call { name, arguments } => bytes(name)?.checked_add(bytes(arguments)?)?.checked_add(21)?,
        };
        total = total.checked_add(owned)?;
    }
    Some(total)
}

/// Owned storage after converting a drawn reply into its capture, without copying any part.
pub(crate) fn drawn_parts(parts: &[Part]) -> Option<u64> {
    let mut total = array(1, size_of::<Drawn>())?.checked_add(array(parts.len(), size_of::<Line>())?)?;
    for part in parts {
        let owned = match part {
            Part::Text { text } => bytes(text)?,
            Part::ToolCall { name, arguments, .. } => bytes(name)?.checked_add(bytes(arguments)?)?,
            Part::Opaque { .. } | Part::ToolOutput { .. } => return None,
        };
        total = total.checked_add(owned)?;
    }
    Some(total)
}

/// Owned replay bytes, including wrappers and slot evidence.
pub(crate) fn replay(capture: &Capture) -> Option<u64> {
    let mut total = array(capture.answers.len(), size_of::<Drawn>())?;
    for drawn in &capture.answers {
        total = total.checked_add(array(drawn.answer.lines.len(), size_of::<Line>())?)?;
        for line in &drawn.answer.lines {
            let owned = match line {
                Line::Text { text } => bytes(text)?,
                Line::Call { name, arguments } => bytes(name)?.checked_add(bytes(arguments)?)?,
                Line::Cut { .. }
                | Line::Take { .. }
                | Line::Fail { .. }
                | Line::Pieces { .. }
                | Line::Opaque { .. } => {
                    return None;
                }
            };
            total = total.checked_add(owned)?;
        }
        total = total.checked_add(array(drawn.slots.len(), size_of::<Filled>())?)?;
        for slot in &drawn.slots {
            total = total
                .checked_add(bytes(&slot.id)?)?
                .checked_add(bytes(&slot.value)?)?
                .checked_add(bytes(&slot.class)?)?;
        }
    }
    Some(total)
}

/// Measures regular and invalid inputs before retaining them. Slot declarations have their own refusal.
pub(crate) fn menu_inputs(menu: &crate::api::Menu) -> Option<u64> {
    let mut total = array(menu.arguments.len(), size_of::<Box<[u8]>>())?
        .checked_add(array(menu.invalid.len(), size_of::<crate::api::InvalidInput>())?)?;
    for arguments in &menu.arguments {
        total = total.checked_add(bytes(arguments)?)?;
    }
    for input in &menu.invalid {
        total = total.checked_add(bytes(&input.arguments)?)?;
        if let Some(name) = &input.name {
            total = total.checked_add(bytes(name)?)?;
        }
    }
    Some(total)
}

pub(crate) fn slots(slots: &[crate::api::Slot]) -> Option<u64> {
    let mut total = array(slots.len(), size_of::<crate::api::Slot>())?;
    for slot in slots {
        total = total
            .checked_add(bytes(&slot.name)?)?
            .checked_add(bytes(&slot.before)?)?
            .checked_add(bytes(&slot.after)?)?
            .checked_add(bytes(&slot.unissued)?)?;
    }
    Some(total)
}

/// Bounds a random batch from the largest name and input, with slots at their never-issued values.
pub(crate) fn random_answer(query: &Query, menu: &crate::api::Menu, count: u32) -> Option<u64> {
    let mut name = 0;
    let mut arguments = 0;
    for tool in &query.tools {
        name = name.max(bytes(&tool.name)?);
    }
    let unissued = crate::slots::unissued(&menu.slots);
    for input in &menu.arguments {
        let size = if menu.slots.is_empty() {
            bytes(input)?
        } else {
            crate::slots::measure_bytes(input, &menu.slots, &unissued)?
        };
        arguments = arguments.max(size);
    }
    for input in &menu.invalid {
        arguments = arguments.max(bytes(&input.arguments)?);
        if let Some(replacement) = &input.name {
            name = name.max(bytes(replacement)?);
        }
    }
    u64::from(count).checked_mul(random_call(name, arguments)?)
}

pub(crate) fn random_call(name: u64, arguments: u64) -> Option<u64> {
    // Every minted call ID has the same 21-byte spelling.
    u64::try_from(size_of::<Part>()).ok()?.checked_add(name)?.checked_add(arguments)?.checked_add(21)
}
