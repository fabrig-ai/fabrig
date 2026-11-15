//! Turn parts retain their byte positions and carry an exact continuation call.
//! Conversation text excludes tool payloads while retaining archived part positions.
use super::{Text, text_end};
use crate::jig::{Limits, call::Call, call::number};
use alloc::boxed::Box;
use jig_client::contract as c;
pub(super) fn name(name: c::TurnName, out: &mut Text) {
    out.number(name.attempt);
    out.put(b".");
    out.number(u64::from(name.number));
}
pub(super) fn sized(task: u64, turn: c::TurnName, bytes: u64, past: bool, out: &mut Text) {
    out.put(b"turn ");
    name(turn, out);
    out.put(b" \xc2\xb7 ");
    out.number(bytes.div_ceil(1024));
    out.put(b" KB \xc2\xb7 ");
    if past {
        out.put(b"past the limit; report it\n");
    } else {
        out.put(b"read with: task turn ");
        out.number(task);
        out.put(b" ");
        name(turn, out);
        out.put(b"\n");
    }
}
pub(super) fn text(
    task: u64,
    turn_name: c::TurnName,
    turn: &c::TurnRead,
    call: &Call,
    limits: &Limits,
    out: &mut Text,
) {
    match turn {
        c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes } } => {
            sized(task, turn_name, *bytes, call.tool == 6, out);
        }
        c::TurnRead::Undecodable { why: c::TurnProblem::Version { .. } | c::TurnProblem::Malformed } => {
            out.put(b"turn ");
            name(turn_name, out);
            out.put(b" \xc2\xb7 unreadable; report it\n");
        }
        c::TurnRead::Decoded(turn) => {
            let mut final_part = false;
            if call.tool == 6
                && let Some(part) = number(call, 2)
                && let Some(bytes) = part_bytes(turn, part)
            {
                let room = out.room();
                let (start, end, left) = cut(bytes, number(call, 3).unwrap_or(0), room, limits.part);
                final_part = start > 0 && left == 0;
                if start > 0 || left > 0 {
                    out.put(b"part ");
                    out.number(part);
                    out.put(b" ");
                    let bound = room.min(usize::try_from(limits.part).expect("part bound fits"));
                    super::task_source::page(
                        u64::try_from(start).expect("part offset fits"),
                        u64::try_from(end).expect("part length fits"),
                        u64::try_from(left).expect("part length fits"),
                        u64::try_from(bound).expect("part bound fits"),
                        out,
                    );
                    if left > 0 {
                        out.number(u64::try_from(left).expect("part length fits"));
                        out.put(b" bytes left; read on with: ");
                        continuation(
                            task,
                            turn_name,
                            part,
                            u64::try_from(start.checked_add(end).expect("bounded offset")).expect("part offset fits"),
                            out,
                        );
                        out.put(b"\n");
                    } else {
                        out.put(b"no more\n");
                    }
                }
            }
            out.put(b"turn ");
            name(turn_name, out);
            out.put(b"\n");
            let mut part = 0u64;
            for message in &turn.messages {
                for block in &message.blocks {
                    block_text(task, turn_name, &mut part, block, call, limits, out);
                }
            }
            if final_part {
                out.put(b"no more\n");
            }
        }
    }
}
/// Only the role's text, retaining part offsets for the ordinary continuation calls.
pub(super) fn reply(task: u64, name: c::TurnName, value: &c::TurnRead, limits: &Limits, out: &mut Text) {
    let call = Call { tool: crate::jig::rows::CHAT_AT, form: 0, values: Box::default(), key: None };
    let c::TurnRead::Decoded(turn) = value else {
        text(task, name, value, &call, limits, out);
        return;
    };
    let mut part = 0u64;
    for message in &turn.messages {
        for block in &message.blocks {
            match block {
                c::Block::Text(bytes) => words(task, name, &mut part, bytes.bytes(), &call, limits, out),
                c::Block::Call(called) => {
                    part = part.checked_add(if called.outcome.is_some() { 2 } else { 1 }).expect("bounded parts");
                }
                c::Block::Result { .. } => part = part.checked_add(1).expect("bounded parts"),
                c::Block::Opaque { .. } => {}
            }
        }
    }
}
fn part_bytes(turn: &c::Turn, selected: u64) -> Option<&[u8]> {
    let mut part = 0u64;
    for message in &turn.messages {
        for block in &message.blocks {
            let (first, second) = match block {
                c::Block::Text(text) => (Some(text), None),
                c::Block::Call(called) => (Some(&called.input), called.outcome.as_ref()),
                c::Block::Result { summary, .. } => (Some(summary), None),
                c::Block::Opaque { .. } => (None, None),
            };
            for text in [first, second].into_iter().flatten() {
                part = part.checked_add(1).expect("bounded parts");
                if part == selected {
                    return Some(text.bytes());
                }
            }
        }
    }
    None
}

fn block_text(
    task: u64,
    turn: c::TurnName,
    part: &mut u64,
    block: &c::Block,
    call: &Call,
    limits: &Limits,
    out: &mut Text,
) {
    match block {
        c::Block::Text(text) => words(task, turn, part, text.bytes(), call, limits, out),
        c::Block::Call(called) => {
            out.put(b"call ");
            out.number(called.number);
            out.put(b" \xc2\xb7 ");
            out.put(called.tool.bytes());
            out.put(b"\n");
            payload(task, turn, part, called.input.bytes(), call, limits, out);
            if let Some(outcome) = &called.outcome {
                payload(task, turn, part, outcome.bytes(), call, limits, out);
            }
            if call.tool != crate::jig::rows::CHAT_AT {
                for reference in &called.parts {
                    reference_text(reference, out);
                }
            }
        }
        c::Block::Result { call: called, summary, failed } => {
            out.put(b"result ");
            out.number(*called);
            if *failed {
                out.put(b" \xc2\xb7 failed");
            }
            out.put(b"\n");
            payload(task, turn, part, summary.bytes(), call, limits, out);
        }
        c::Block::Opaque { dialect, size } => {
            out.put(b"opaque ");
            out.put(dialect.bytes());
            out.put(b" \xc2\xb7 ");
            out.number(*size);
            out.put(b" bytes\n");
        }
    }
}
fn payload(task: u64, turn: c::TurnName, part: &mut u64, bytes: &[u8], call: &Call, limits: &Limits, out: &mut Text) {
    if call.tool == crate::jig::rows::CHAT_AT {
        *part = part.checked_add(1).expect("bounded parts");
    } else {
        words(task, turn, part, bytes, call, limits, out);
    }
}
fn words(task: u64, turn: c::TurnName, part: &mut u64, bytes: &[u8], call: &Call, limits: &Limits, out: &mut Text) {
    *part = part.checked_add(1).expect("bounded parts");
    if call.tool == 6
        && let Some(selected) = number(call, 2)
        && selected != *part
    {
        return;
    }
    let from = if call.tool == 6 { number(call, 3).unwrap_or(0) } else { 0 };
    let (from, end, left) = cut(bytes, from, out.room(), limits.part);
    let bytes = bytes.get(from..).expect("offset within part");
    out.put(bytes.get(..end).expect("part cut"));
    out.reserve(end);
    out.put(b"\n");
    if left > 0 {
        out.number(u64::try_from(left).expect("bounded part"));
        out.put(b" bytes left; read on with: ");
        continuation(
            task,
            turn,
            *part,
            u64::try_from(from.checked_add(end).expect("bounded offset")).expect("bounded offset"),
            out,
        );
        out.put(b"\n");
    }
}

fn cut(bytes: &[u8], from: u64, room: usize, part: u32) -> (usize, usize, usize) {
    let start = text_end(bytes, usize::try_from(from).unwrap_or(usize::MAX));
    let bytes = bytes.get(start..).expect("offset within part");
    let end = text_end(bytes, room.min(usize::try_from(part).expect("part bound fits")));
    let left = bytes.len().checked_sub(end).expect("part cut");
    (start, end, left)
}

fn continuation(task: u64, turn: c::TurnName, part: u64, from: u64, out: &mut Text) {
    out.put(b"task turn ");
    out.number(task);
    out.put(b" ");
    name(turn, out);
    out.put(b" --part ");
    out.number(part);
    out.put(b" --from ");
    out.number(from);
}
pub(super) fn read_on(task: u64, turn: c::TurnName, part: u64, from: u64) -> Box<[u8]> {
    let mut measure = Text::measure();
    continuation(task, turn, part, from, &mut measure);
    let mut out = Text::writing_exact(measure);
    continuation(task, turn, part, from, &mut out);
    out.finish()
}
pub(super) fn reference_text(part: &c::ApplicationPart, out: &mut Text) {
    out.put(b"part: ");
    match part {
        c::ApplicationPart::Resource { connector, path } => {
            out.number(u64::from(*connector));
            out.put(b":");
            for (at, segment) in path.iter().enumerate() {
                if at > 0 {
                    out.put(b"/");
                }
                out.put(segment);
            }
        }
        c::ApplicationPart::Connector { .. }
        | c::ApplicationPart::Kind { .. }
        | c::ApplicationPart::Panel { .. }
        | c::ApplicationPart::Result { .. }
        | c::ApplicationPart::Effect { .. }
        | c::ApplicationPart::OwnerEffect { .. }
        | c::ApplicationPart::Outbox { .. }
        | c::ApplicationPart::Hold { .. }
        | c::ApplicationPart::Project { .. }
        | c::ApplicationPart::Deployment { .. } => super::document::part(part, out),
    }
    out.put(b"\n");
}
