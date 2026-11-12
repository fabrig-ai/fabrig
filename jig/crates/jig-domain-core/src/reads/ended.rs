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
    let Some(owner) = row_owner(row) else { return Ok(None) };
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
