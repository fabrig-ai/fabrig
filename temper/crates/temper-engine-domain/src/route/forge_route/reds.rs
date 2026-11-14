//! A check's goal-local snapshot (temper/docs/design/domain/forge/reds.md, 5.1).
use super::{Domain, List, branch_what, forge, tasks};
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunRed {
    pub key: Box<[u8]>,
    pub head: Box<[u8]>,
}

/// Read only the reds applying to this live, fenced attempt.
#[must_use]
pub fn reds_for_run(domain: &Domain, task: u64, attempt: u64) -> Box<[RunRed]> {
    let mut rows = List::with_capacity(domain.limits.forge.branch_reds);
    if !domain.core.current_proof(task, attempt) {
        return rows.into_boxed();
    }
    if let Some(goal) = goal_for_run(domain, task) {
        for (_, red) in domain.forge.reds(goal) {
            rows.push(RunRed { key: red.key.clone(), head: red.head.clone() }).expect("bounded goal reds");
        }
    }
    rows.into_boxed()
}

fn goal_for_run(domain: &Domain, task: u64) -> Option<u64> {
    let mut current = domain.core.tasks.task(task)?;
    if domain.forge.holds_goal_branch(task) {
        return Some(task);
    }
    for _ in 0..=current.depth {
        if let Some(change) = domain.forge.change(current.number) {
            let name = forge::Name {
                forge: change.repository.forge,
                repository: change.repository.repository,
                what: branch_what(&change.base, domain.limits.forge.name_bytes)?,
            };
            return domain.forge.goal_branch_holder(&name);
        }
        let tasks::Party::Task(parent) = current.requester else { break };
        current = domain.core.tasks.task(parent)?;
    }
    None
}
