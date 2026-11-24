//! Wire projections of task hold facts (client/domain/testing.md, 4).
use super::super::{bytes, c, w};
use super::wire_hold_owner;
pub(in crate::codec) fn wire_placed_line(value: &c::PlacedHold) -> w::PlacedLine {
    w::PlacedLine {
        number: value.number,
        owner: wire_hold_owner(&value.owner),
        mode: wire_hold_mode(&value.mode),
        scope: wire_hold_scope(&value.scope),
        reason: bytes::copy_of(&value.reason),
        placed_at: value.placed_at.as_nanos(),
        revision: value.revision.value,
        lifted: value.lifted,
        may_lift: value.may_lift,
    }
}

pub(in crate::codec) fn wire_hold_mode(value: &c::HoldMode) -> w::HoldMode {
    match value {
        c::HoldMode::Stop => w::HoldMode::Stop,
        c::HoldMode::Drain => w::HoldMode::Drain,
    }
}

pub(in crate::codec) fn wire_hold_scope(value: &c::HoldScope) -> w::HoldScope {
    match value {
        c::HoldScope::Subtree { task } => w::HoldScope::Subtree(w::HoldScopeSubtree { task: *(task) }),
        c::HoldScope::Project => w::HoldScope::Project,
    }
}
