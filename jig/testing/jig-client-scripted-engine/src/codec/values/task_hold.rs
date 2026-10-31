//! Wire projections of task hold facts (client/domain/testing.md, 4).
use super::super::{bytes, c, w};
use super::wire_hold_owner;
pub(in crate::codec) fn wire_placed_line(
    value: &c::PlacedHold,
    limits: &w::Limits,
) -> Result<w::PlacedLine, w::Problem> {
    w::PlacedLine::new(
        limits,
        w::PlacedLineParts {
            number: value.number,
            owner: wire_hold_owner(&value.owner, limits)?,
            mode: wire_hold_mode(&value.mode, limits)?,
            scope: wire_hold_scope(&value.scope, limits)?,
            reason: bytes::copy_of(&value.reason),
            placed_at: value.placed_at.as_nanos(),
            revision: value.revision.value,
            lifted: value.lifted,
            may_lift: value.may_lift,
        },
    )
}

pub(in crate::codec) fn wire_hold_mode(value: &c::HoldMode, _limits: &w::Limits) -> Result<w::HoldMode, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldMode::Stop => w::HoldMode::Stop,
        c::HoldMode::Drain => w::HoldMode::Drain,
    })
}

pub(in crate::codec) fn wire_hold_scope(value: &c::HoldScope, limits: &w::Limits) -> Result<w::HoldScope, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldScope::Subtree { task } => {
            w::HoldScope::Subtree(w::HoldScopeSubtree::new(limits, w::HoldScopeSubtreeParts { task: *(task) })?)
        }
        c::HoldScope::Project => w::HoldScope::Project,
    })
}
