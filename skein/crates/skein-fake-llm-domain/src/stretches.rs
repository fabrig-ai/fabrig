//! Stretches span logical turns but occupy one script position
//! (skein/docs/design/testing/fakes/drawn-agent.md, 5).

use alloc::boxed::Box;
use skein_lib::usize_of;

use crate::api::{Finish, Line, Turn};

pub(crate) fn valid(turn: &Turn, rounds: u32, tools: &[Box<[u8]>]) -> bool {
    if turn.lines.len() != 1
        || turn.finish != Finish::ToolCalls
        || turn.tokens != 0
        || !(1..=64).contains(&rounds)
        || !(1..=8).contains(&tools.len())
    {
        return false;
    }
    for (index, tool) in tools.iter().enumerate() {
        let earlier = tools.get(..index).expect("preceding names");
        if !(1..=64).contains(&tool.len()) || earlier.contains(tool) {
            return false;
        }
    }
    true
}

pub(crate) fn drawn(turn: &Turn) -> Option<(u32, &[Box<[u8]>])> {
    match turn.lines.first() {
        Some(Line::Drawn { rounds, tools }) => Some((*rounds, tools)),
        Some(
            Line::Cut { .. }
            | Line::Take { .. }
            | Line::Fail { .. }
            | Line::Pieces { .. }
            | Line::Text { .. }
            | Line::Opaque { .. }
            | Line::Served { .. }
            | Line::Call { .. },
        )
        | None => None,
    }
}

fn width(turn: &Turn) -> usize {
    match drawn(turn) {
        Some((rounds, _)) => usize_of(rounds),
        None => 1,
    }
}

/// Convert the assistant message count to a physical turn, or an index past the script.
pub(crate) fn position(turns: &[Turn], mut remaining: usize) -> usize {
    for (index, turn) in turns.iter().enumerate() {
        let Some(rest) = remaining.checked_sub(width(turn)) else { return index };
        remaining = rest;
    }
    turns.len().checked_add(remaining).expect("no more than the initial count past the last turn")
}

pub(crate) fn count(turns: &[Turn]) -> u64 {
    let mut total = 0_u64;
    for turn in turns {
        total = total.checked_add(skein_lib::len_u64(width(turn))).expect("admitted script bytes bound the turn count");
    }
    total
}

/// An empty list leaves ordinary random answers unrestricted.
pub(crate) fn includes(tools: &[Box<[u8]>], name: &[u8]) -> bool {
    if tools.is_empty() {
        return true;
    }
    for tool in tools {
        if tool.as_ref() == name {
            return true;
        }
    }
    false
}
