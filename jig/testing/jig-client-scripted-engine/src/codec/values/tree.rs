//! Wire projections of owner tree facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, pieces, w};
use super::{wire_cursor, wire_phase_counts, wire_proposal_key, wire_task_link};
pub(in crate::codec) fn wire_tree(value: &c::Tree, limits: &w::Limits) -> Result<w::Tree, w::Problem> {
    w::Tree::new(
        limits,
        w::TreeParts {
            root: facts::chip(&value.root, limits)?,
            rows: pieces(
                value.rows.iter().map(|item| wire_tree_row(item, limits)),
                w::TreeRow::measure,
                w::TreeRow::encode,
            )?,
            folded: list(value.folded.iter().map(|item| wire_tree_fold(item, limits)))?,
            held_below: value.held_below,
            next: value.next.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_tree_row(value: &c::TreeRow, limits: &w::Limits) -> Result<w::TreeRow, w::Problem> {
    w::TreeRow::new(
        limits,
        w::TreeRowParts {
            chip: facts::chip(&value.chip, limits)?,
            parent: value.parent,
            depth: value.depth,
            proposals: list(value.proposals.iter().map(|item| wire_proposal_key(item, limits)))?,
            spent: value.spent,
            dependencies: pieces(
                value.dependencies.iter().map(|item| wire_task_link(item, limits)),
                w::TaskLink::measure,
                w::TaskLink::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_tree_fold(value: &c::TreeFold, limits: &w::Limits) -> Result<w::TreeFold, w::Problem> {
    w::TreeFold::new(
        limits,
        w::TreeFoldParts {
            parent: value.parent,
            phases: wire_phase_counts(&value.phases, limits)?,
            held_below: value.held_below,
        },
    )
}
