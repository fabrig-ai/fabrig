//! The single index serving a task query's ended group (protocol/store/loads.md, 3).

use crate::{
    Core, Limits,
    documents::{self, Missing, TaskQuery},
    reads::{PageRow, Row, vocabulary::row_owner},
};
pub use jig_domain_core_vocabulary::reads::ended::*;
use jig_domain_tasks as tasks;
/// Apply keys beyond the prefix to the joined ended owner.
pub(super) fn line(
    core: &Core,
    limits: &Limits,
    party: u64,
    query: &TaskQuery,
    row: &Row,
) -> Result<Option<PageRow>, Missing> {
    let Some(owner) = row_owner(row) else {
        return summary_line(core, limits, party, query, row);
    };
    let tasks::Phase::Ended(_) = owner.phase else { return Ok(None) };
    if !documents::task_query::may_touch(query, owner) {
        return Ok(None);
    }
    if query.requester.is_some() && query.requester != Some(owner.requester)
        || query.tracked.is_some() && query.tracked != Some(owner.tracked.is_some())
        || query.root.is_some() && query.root != Some(row.ancestors.last().copied().unwrap_or(owner.number))
    {
        return Ok(None);
    }
    if let Some(parent) = query.parent
        && owner.requester != tasks::Party::Task(parent)
    {
        return Ok(None);
    }
    Ok(Some(PageRow::Line(documents::task_line(core, limits, party, owner, row.ended_read)?)))
}

/// Summary-only rows keep query keys, but join no ancestry or conversation
/// (protocol/store/retention.md, 7).
fn summary_line(
    core: &Core,
    limits: &Limits,
    party: u64,
    query: &TaskQuery,
    row: &Row,
) -> Result<Option<PageRow>, Missing> {
    let Some(summary) = &row.summary else { return Ok(None) };
    let line = documents::task_line_from_summary(core, limits, party, summary)?;
    if query.project.is_some() && query.project != Some(summary.project)
        || query.label.is_some() && query.label != Some(summary.label)
        || query.executor.is_some() && query.executor != Some(line.chip.executor)
        || query.requester.is_some() && query.requester != Some(summary.requester)
        || query.tracked.is_some() && query.tracked != Some(summary.tracked.is_some())
        || query.root.is_some() && query.root != Some(summary.root)
        || query.phase.is_some() && query.phase != Some(documents::PhaseClass::Ended)
    {
        return Ok(None);
    }
    if let Some(parent) = query.parent
        && summary.requester != tasks::Party::Task(parent)
    {
        return Ok(None);
    }
    Ok(Some(PageRow::Line(line)))
}
