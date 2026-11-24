//! Wire projections of owner tree facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, try_pieces, w};
use super::{wire_cursor, wire_phase_counts, wire_proposal_key, wire_task_link};
pub(in crate::codec) fn wire_tree(value: &c::Tree, limits: &w::Limits) -> Result<w::Tree, w::Problem> {
    Ok(w::Tree {
        root: facts::chip(&value.root, limits)?,
        rows: try_pieces(value.rows.iter().map(|item| wire_tree_row(item, limits)), limits)?,
        folded: list(value.folded.iter().map(wire_tree_fold)),
        held_below: value.held_below,
        next: value.next.as_ref().map(wire_cursor),
    })
}

pub(in crate::codec) fn wire_tree_row(value: &c::TreeRow, limits: &w::Limits) -> Result<w::TreeRow, w::Problem> {
    Ok(w::TreeRow {
        chip: facts::chip(&value.chip, limits)?,
        parent: value.parent,
        depth: value.depth,
        proposals: list(value.proposals.iter().map(wire_proposal_key)),
        spent: value.spent,
        dependencies: try_pieces(value.dependencies.iter().map(|item| wire_task_link(item, limits)), limits)?,
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_tree_fold(value: &c::TreeFold) -> w::TreeFold {
    w::TreeFold { parent: value.parent, phases: wire_phase_counts(&value.phases), held_below: value.held_below }
}
