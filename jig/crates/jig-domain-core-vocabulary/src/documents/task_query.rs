//! Fixed query keys and one bounded live scan per document (domain/engine/views.md, 2).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

use super::{Cursor, DocumentLimits, ExecutorKind, TaskLine};
use jig_domain_tasks as tasks;
use skein_lib::List;

/// Lifecycle class selected by a task query.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PhaseClass {
    Working,
    Waiting,
    Held,
    Closing,
    Ended,
}
/// Direction is fixed; every tie goes by ascending task number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum QueryOrder {
    Number,
    Changed,
    Activity,
    Priority,
    Ended,
}
/// Fixed conjunctive keys; an absent key matches every value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TaskQuery {
    pub project: Option<u32>,
    pub requester: Option<tasks::Party>,
    pub root: Option<u64>,
    pub parent: Option<u64>,
    pub label: Option<u32>,
    pub phase: Option<PhaseClass>,
    pub executor: Option<ExecutorKind>,
    pub tracked: Option<bool>,
    pub order: QueryOrder,
}
/// A bounded window over a query's live group.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Lines {
    pub query: TaskQuery,
    pub lines: List<TaskLine>,
    pub next: Option<Cursor>,
}

#[must_use]
pub fn document_bound(limits: &DocumentLimits) -> Option<u64> {
    List::<TaskLine>::price(limits.rows)?.checked_add(
        u64::from(limits.rows)
            .checked_mul(super::chip_heap_bound(limits)?.checked_add(List::<u64>::price(limits.lines)?)?)?,
    )
}
