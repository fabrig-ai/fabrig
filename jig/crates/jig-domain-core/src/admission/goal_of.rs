//! The single nearest-goal rule shared by documents, scopes and projections.
use crate::{Core, tasks};

impl Core {
    /// The nearest goal's note scope, or a root conversation's or standing task's own
    /// (`jig/docs/design/domain/engine/briefs.md`, 2; `domain/tasks/standing.md`, 6).
    #[must_use]
    pub fn note_goal(&self, row: &tasks::TaskRecord) -> Option<u64> {
        if let Some(goal) = self.goal_of(row) {
            return Some(goal);
        }
        if row.flags.ended_by == tasks::EndedBy::Standing {
            return match row.requester {
                tasks::Party::Person(_) => Some(row.number),
                tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
            };
        }
        if !row.flags.conversation {
            return None;
        }
        match row.requester {
            tasks::Party::Task(_) => None,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => Some(row.number),
        }
    }

    /// The nearest tracked task at or above this task, itself included
    /// (`domain/model.md`, 6.2).
    #[must_use]
    pub fn goal_of(&self, row: &tasks::TaskRecord) -> Option<u64> {
        let mut current = row;
        for _ in 0..=row.depth {
            if current.tracked.is_some() {
                return Some(current.number);
            }
            match current.requester {
                tasks::Party::Task(parent) => current = self.tasks.task(parent)?,
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return None,
            }
        }
        None
    }
}
