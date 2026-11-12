//! Fixed query keys and one bounded live scan per document (domain/engine/views.md, 2).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

pub use crate::below::documents::task_query::*;
use crate::{
    Core, Limits,
    documents::{Cursor, ExecutorKind, Missing, Window},
};
use core::cmp::Reverse;
use jig_domain_tasks as tasks;
use skein_lib::{List, Map, Wall};

pub(crate) fn admission(core: &Core, party: u64, query: &TaskQuery) -> Result<(), Missing> {
    if let Some(tasks::Party::Deployment { .. }) = query.requester {
        return Err(Missing::Unknown);
    }
    if let Some(project) = query.project {
        if !core.open_project(project) || core.people.role(party, project).is_none() {
            return Err(Missing::NotAllowed);
        }
    } else if query.requester.is_some() && query.requester != Some(tasks::Party::Person(party)) {
        return Err(Missing::NotAllowed);
    }
    Ok(())
}
fn executor(row: &tasks::TaskRecord) -> ExecutorKind {
    match row.executor {
        tasks::Executor::Agent { .. } => ExecutorKind::Agent,
        tasks::Executor::Procedure { .. } => ExecutorKind::Procedure,
        tasks::Executor::Person(_) => ExecutorKind::Person,
        tasks::Executor::Job { .. } => ExecutorKind::Job,
    }
}
fn phase(row: &tasks::TaskRecord) -> PhaseClass {
    match row.phase {
        tasks::Phase::Active(_) => PhaseClass::Working,
        tasks::Phase::Waiting => PhaseClass::Waiting,
        tasks::Phase::Held { .. } => PhaseClass::Held,
        tasks::Phase::Closing(_) => PhaseClass::Closing,
        tasks::Phase::Ended(_) => PhaseClass::Ended,
    }
}
/// Conservatively mark entering, leaving and moving rows without retaining old keys.
#[must_use]
pub fn may_touch(query: &TaskQuery, row: &tasks::TaskRecord) -> bool {
    if let Some(project) = query.project
        && row.project != project
    {
        return false;
    }
    if let Some(label) = query.label
        && row.label != label
    {
        return false;
    }
    if let Some(kind) = query.executor
        && executor(row) != kind
    {
        return false;
    }
    true
}
fn matches(query: &TaskQuery, row: &tasks::TaskRecord) -> bool {
    if !may_touch(query, row) {
        return false;
    }
    if let Some(requester) = query.requester
        && row.requester != requester
    {
        return false;
    }
    if let Some(root) = query.root
        && row.root != root
    {
        return false;
    }
    if let Some(parent) = query.parent
        && row.requester != tasks::Party::Task(parent)
    {
        return false;
    }
    if let Some(selected) = query.phase
        && phase(row) != selected
    {
        return false;
    }
    if let Some(tracked) = query.tracked
        && row.tracked.is_some() != tracked
    {
        return false;
    }
    true
}
type SortKey = (bool, Reverse<Wall>, u32, u64);
fn sort_key(row: &tasks::TaskRecord, order: QueryOrder) -> SortKey {
    match order {
        QueryOrder::Number | QueryOrder::Ended => (false, Reverse(Wall::EPOCH), 0, row.number),
        QueryOrder::Changed => (false, Reverse(row.changed), 0, row.number),
        QueryOrder::Activity => (false, Reverse(row.last_activity), 0, row.number),
        QueryOrder::Priority => (row.tracked.is_none(), Reverse(Wall::EPOCH), row.tracked.unwrap_or(0), row.number),
    }
}
/// Admit and scan live tasks once, then construct only the requested window.
pub fn lines(core: &Core, limits: &Limits, party: u64, query: &TaskQuery, window: &Window) -> Result<Lines, Missing> {
    admission(core, party, query)?;
    if window.rows > limits.documents.rows {
        return Err(Missing::TooLarge);
    }
    let mut order = Map::with_capacity(limits.tasks.tasks);
    for candidate in core.tasks.view_tasks().as_ref() {
        let row = core.tasks.task(candidate.number).expect("live task view");
        if core.open_project(row.project)
            && (query.project.is_some() || row.requester == tasks::Party::Person(party))
            && matches(query, row)
        {
            order.insert(sort_key(row, query.order), row.number).expect("bounded live order");
        }
    }
    let mut lines = List::with_capacity(window.rows);
    let mut taking = window.after.is_none();
    let mut last = None;
    let mut next = None;
    for (_, number) in &order {
        if !taking {
            if window.after == Some(Cursor::Number { number: *number }) {
                taking = true;
            }
            continue;
        }
        if lines.room() == 0 {
            next = last;
            break;
        }
        let row = core.tasks.task(*number).expect("ordered live task");
        lines.push(super::task_lines::task_line(core, limits, party, row, None)?).expect("bounded query window");
        last = Some(Cursor::Number { number: *number });
    }
    if !taking {
        return Err(Missing::Unknown);
    }
    Ok(Lines { query: *query, lines, next })
}
