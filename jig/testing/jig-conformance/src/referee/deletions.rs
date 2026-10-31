//! Project deletion ends whole (jig/docs/design/protocol/store.md, 11.1).
use super::{Promise, Snapshot, Violation, require};
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
