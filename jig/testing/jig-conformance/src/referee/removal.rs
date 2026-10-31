//! Project erasure judged from independent durable evidence
//! (temper/docs/design/draft/playground-reset.md, 5 and 8).
use super::{Promise, Receipt, Snapshot, Violation, require};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(super) struct Removal {
    removed: BTreeSet<u32>,
    tasks: BTreeMap<u64, u32>,
}

impl Removal {
    pub(super) fn answered(&mut self, project: u32) {
        self.removed.insert(project);
    }

    pub(super) fn durable(&mut self, before: &Snapshot, after: &Snapshot) -> Result<(), Violation> {
        for task in before.tasks.values() {
            require(
                after.tasks.contains_key(&task.number) || task.closed() || self.removed.contains(&task.project),
                Promise::Lost,
                format!("task {} left the durable view before ending", task.number),
            )?;
        }
        for task in after.tasks.values() {
            self.tasks.insert(task.number, task.project);
        }
        Ok(())
    }

    pub(super) fn receipt(&self, receipt: &Receipt) -> bool {
        let task = match receipt {
            Receipt::Task(task)
            | Receipt::Claim(task, _)
            | Receipt::Turn(task, ..)
            | Receipt::Terminal(task, _)
            | Receipt::Call(task, ..)
            | Receipt::Word(task, _)
            | Receipt::Ended(task) => task,
            Receipt::SignIn(_) | Receipt::Party(..) | Receipt::Outbox(..) => return false,
        };
        self.tasks.get(task).is_some_and(|project| self.removed.contains(project))
    }

    pub(super) fn check(&self, snapshot: &Snapshot) -> Result<(), Violation> {
        for project in &self.removed {
            let left = snapshot.naming.get(project).copied().unwrap_or(0);
            require(
                left == 0,
                Promise::Erasure,
                format!("removed project {project} still named by {left} durable rows"),
            )?;
        }
        Ok(())
    }
}
