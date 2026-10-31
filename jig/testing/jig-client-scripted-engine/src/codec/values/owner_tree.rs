//! Wire projections of owner tree facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, pieces, w};
use super::{wire_cursor, wire_owner_phase_counts, wire_owner_task_link, wire_proposal_key};
pub(in crate::codec) fn wire_owner_tree(value: &c::OwnerTree, limits: &w::Limits) -> Result<w::OwnerTree, w::Problem> {
    w::OwnerTree::new(
        limits,
        w::OwnerTreeParts {
            root: facts::chip(&value.root, limits)?,
            rows: pieces(
                value.rows.iter().map(|item| wire_owner_tree_row(item, limits)),
                w::OwnerTreeRow::measure,
                w::OwnerTreeRow::encode,
            )?,
            folded: list(value.folded.iter().map(|item| wire_owner_tree_fold(item, limits)))?,
            held_below: value.held_below,
            next: value.next.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_tree_row(
    value: &c::OwnerTreeRow,
    limits: &w::Limits,
) -> Result<w::OwnerTreeRow, w::Problem> {
    w::OwnerTreeRow::new(
        limits,
        w::OwnerTreeRowParts {
            chip: facts::chip(&value.chip, limits)?,
            parent: value.parent,
            depth: value.depth,
            proposals: list(value.proposals.iter().map(|item| wire_proposal_key(item, limits)))?,
            spent: value.spent,
            dependencies: pieces(
                value.dependencies.iter().map(|item| wire_owner_task_link(item, limits)),
                w::OwnerTaskLink::measure,
                w::OwnerTaskLink::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_owner_tree_fold(
    value: &c::OwnerTreeFold,
    limits: &w::Limits,
) -> Result<w::OwnerTreeFold, w::Problem> {
    w::OwnerTreeFold::new(
        limits,
        w::OwnerTreeFoldParts {
            parent: value.parent,
            phases: wire_owner_phase_counts(&value.phases, limits)?,
            held_below: value.held_below,
        },
    )
}
