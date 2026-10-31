//! Whole subtree facts and supplied folds (client/domain/contract.md, 4 and 7;
//! pages.md, 5.2). Parentage, counts and continuation belong to the source.
use super::{Chip, Cursor, PhaseCounts, ProposalKey, TaskLink};
use skein_lib::List;
/// One supplied descendant, with independent dependency loss.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TreeRow {
    pub chip: Chip,
    pub parent: u64,
    pub depth: u32,
    pub proposals: List<ProposalKey>,
    pub spent: u64,
    pub dependencies: List<TaskLink>,
    pub cut: bool,
}
/// Counts supplied for one parent independently of visible descendants.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TreeFold {
    pub parent: u64,
    pub phases: PhaseCounts,
    pub held_below: u32,
}
/// A whole root, descendant window, per-parent folds and opaque continuation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tree {
    pub root: Chip,
    pub rows: List<TreeRow>,
    pub folded: List<TreeFold>,
    pub held_below: u32,
    pub next: Option<Cursor>,
}
