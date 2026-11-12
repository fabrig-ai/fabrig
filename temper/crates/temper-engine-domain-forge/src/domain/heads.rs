//! Recording branch heads at their bound (temper/docs/design/domain/forge/keeping-up.md, 1).
use super::{BranchHead, Domain, Env, Limits, Name, Queue, Request};

#[cfg(test)]
mod tests;

/// Record or replace a head; a full table refuses only a new name.
pub(crate) fn record(
    d: &mut Domain,
    _env: &Env<Limits>,
    head: BranchHead,
    concerning: Option<u64>,
    out: &mut Queue<Request>,
) -> bool {
    match d.heads.insert(head.name.clone(), head) {
        Ok(_) => true,
        Err((name, _)) => {
            super::emit(out, Request::HeadRefused {
                task: concerning,
                resource: name,
                configured: u64::from(d.heads.capacity()),
                needed: u64::from(d.heads.len()).saturating_add(1),
            });
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
