//! Project deletion ends whole (jig/docs/design/domain/people.md, 5.2 and 8).
use super::{Promise, Receipt, Snapshot, Violation, require};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
struct Project {
    tasks: BTreeSet<u64>,
    removed: bool,
    deletion_seen: bool,
    ended: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Deletions {
    projects: BTreeMap<u32, Project>,
}

impl Deletions {
    pub(super) fn removed(&mut self, project: u32) {
        self.projects.entry(project).or_default().removed = true;
    }

    pub(super) fn removed_project(&self, project: u32) -> bool {
        self.projects.get(&project).is_some_and(|project| project.removed)
    }

    /// A kept deletion erases ended owners' prerequisites a durable page at a time.
    pub(super) fn retire_receipts(&self, before: &Snapshot, after: &Snapshot, seen: &mut BTreeSet<Receipt>) {
        seen.retain(|receipt| {
            let task = match receipt {
                Receipt::Task(task) | Receipt::Turn(task, ..) | Receipt::Terminal(task, ..) | Receipt::Ended(task) => {
                    *task
                }
                Receipt::Claim(..)
                | Receipt::Call(..)
                | Receipt::SignIn(_)
                | Receipt::Party(..)
                | Receipt::Outbox(..)
                | Receipt::Word(..) => return true,
            };
            let erasing = before.tasks.get(&task).is_some_and(|row| {
                row.closed()
                    && (before.deleting.contains(&row.project)
                        || after.deleting.contains(&row.project)
                        || self.removed_project(row.project))
            });
            !erasing || after.receipts.contains(receipt)
        });
    }

    pub(super) fn durable(&mut self, snapshot: &Snapshot) -> Result<(), Violation> {
        for task in snapshot.tasks.values() {
            let project = self.projects.entry(task.project).or_default();
            require(
                !project.removed || project.tasks.contains(&task.number),
                Promise::Ownership,
                "a closed project took a new task",
            )?;
            project.tasks.insert(task.number);
        }
        for project in &snapshot.deleting {
            self.projects.entry(*project).or_default().deletion_seen = true;
        }
        for (number, project) in &mut self.projects {
            if !project.removed {
                continue;
            }
            project.ended |=
                !snapshot.deleting.contains(number) && !snapshot.tasks.values().any(|task| task.project == *number);
            if project.ended {
                for naming in &snapshot.naming {
                    require(
                        naming.outlives.is_some()
                            || (naming.project != Some(*number)
                                && !naming.task.is_some_and(|task| project.tasks.contains(&task))),
                        Promise::Lost,
                        format!("a deleted project's row outlived it: {}", naming.row),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn drained(&self, snapshot: &Snapshot) -> Result<(), Violation> {
        for (number, project) in &self.projects {
            require(
                !project.removed || !project.deletion_seen || !snapshot.deleting.contains(number),
                Promise::Order,
                "a deletion was cut and never resumed",
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
