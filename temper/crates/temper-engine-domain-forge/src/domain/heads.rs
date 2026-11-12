//! Recording branch heads at their bound (temper/docs/design/domain/forge/keeping-up.md, 1).
use super::{BranchHead, Domain, Env, Key, Limits, List, Name, Queue, Request};
use skein_forge::api::{Repository, Write};

#[cfg(test)]
mod tests;

/// A head belongs to the adopted working set, not to the branch's lifetime.
fn needed(d: &Domain, name: &Name, watched: &[Name]) -> bool {
    let repository = Repository { forge: name.forge, repository: name.repository };
    if d.forge_ended(name.forge) || !d.repositories.contains_key(&repository) {
        return false;
    }
    if d.holds.contains_key(name) || watched.contains(name) {
        return true;
    }
    for (_, names) in &d.names {
        if names.contains(name) {
            return true;
        }
    }
    for (_, row) in &d.changes {
        if row.repository == repository
            && row.change.state != temper_engine_domain_forge_change::State::Ended
            && (crate::items::branch_matches(name, &row.branch) || crate::items::branch_matches(name, &row.base))
        {
            return true;
        }
    }
    for (_, row) in &d.tiers {
        if crate::tiers::needs_head(row, name) {
            return true;
        }
    }
    for (_, row) in &d.entries {
        if row.entry.repository == repository && writes(&row.entry.effect, name) {
            return true;
        }
    }
    for (_, pending) in &d.lost {
        if pending.name == *name {
            return true;
        }
    }
    false
}

fn writes(effect: &skein_forge_client::Effect, name: &Name) -> bool {
    if let skein_forge_client::Condition::Merge { base } = &effect.condition
        && crate::items::branch_matches(name, base)
    {
        return true;
    }
    match &effect.write {
        Write::CreateBranch { branch, .. } | Write::DeleteBranch { branch } => {
            crate::items::branch_matches(name, branch)
        }
        Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Update { .. }
        | Write::Status { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. } => false,
    }
}

/// Erase every head that no live work needs, including its stored row.
pub(crate) fn free(d: &mut Domain, _env: &Env<Limits>, out: &mut Queue<Request>) {
    let watched = crate::tiers::watched(d);
    let mut names = List::with_capacity(d.heads.capacity());
    for (name, _) in &d.heads {
        if !needed(d, name, watched.as_slice()) {
            names.push(name.clone()).expect("one name per head");
        }
    }
    for name in names.into_boxed() {
        d.heads.remove(&name);
        super::emit(out, Request::Erase { key: Key::BranchHead(name) });
    }
}

/// Record or replace a head; a full table refuses only a new name after freeing.
pub(crate) fn record(
    d: &mut Domain,
    env: &Env<Limits>,
    head: BranchHead,
    concerning: Option<u64>,
    out: &mut Queue<Request>,
) -> bool {
    if !d.heads.contains_key(&head.name) && d.heads.len() == d.heads.capacity() {
        free(d, env, out);
    }
    match d.heads.insert(head.name.clone(), head) {
        Ok(_) => true,
        Err((name, _)) => {
            super::emit(
                out,
                Request::HeadRefused {
                    task: concerning,
                    resource: name,
                    configured: u64::from(d.heads.capacity()),
                    needed: u64::from(d.heads.len()).saturating_add(1),
                },
            );
            false
        }
    }
}

/// A change's own branch takes precedence over the task performing the step.
pub(super) fn concerning(d: &Domain, name: &Name, otherwise: Option<u64>) -> Option<u64> {
    for (_, row) in &d.changes {
        if row.change.state != temper_engine_domain_forge_change::State::Ended
            && row.repository.forge == name.forge
            && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.branch)
        {
            return Some(row.task);
        }
    }
    otherwise
}
