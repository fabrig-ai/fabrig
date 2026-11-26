//! Whether a change is waiting for its own pending prepare rebase.
use super::Domain;
use skein_forge::api::{Commit, UpdateStyle, Write};
use skein_forge_client::Condition;
use temper_engine_domain_forge_change as change;

pub(super) fn pending(d: &Domain, task: u64) -> Option<(u64, Commit, Commit)> {
    let row = d.changes.get(&task)?;
    if row.frozen() {
        return None;
    }
    let change::State::Preparing { from, main, .. } = row.change.state else { return None };
    let number = row.pending?;
    let entry = d.entries.get(&number)?;
    if entry.held || entry.entry.task != task || entry.entry.repository != row.repository {
        return None;
    }
    match entry.entry.effect.write {
        Write::Update { style: UpdateStyle::Rebase, .. } => {}
        Write::Update { style: UpdateStyle::Merge, .. }
        | Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Status { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. }
        | Write::CreateBranch { .. }
        | Write::DeleteBranch { .. } => return None,
    }
    if entry.entry.effect.condition != (Condition::Update { head: from, base: main }) {
        return None;
    }
    Some((number, from, main))
}
