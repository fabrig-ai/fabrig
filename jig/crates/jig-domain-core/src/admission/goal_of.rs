//! The single nearest-goal rule shared by documents, scopes and projections.
use crate::{Core, tasks};

impl Core {
    /// The nearest goal's note scope, or a root conversation's own
    /// (`jig/docs/design/domain/engine.md`, 10).
    #[must_use]
    pub fn note_goal(&self, row: &tasks::TaskRecord) -> Option<u64> {
        if let Some(goal) = self.goal_of(row) {
            return Some(goal);
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
    /// (`domain/core.md`, 6.2).
    #[must_use]
    pub fn goal_of(&self, row: &tasks::TaskRecord) -> Option<u64> {
        let mut current = row;
        for _ in 0..=row.depth {
            if current.tracked.is_some() {
                return Some(current.number);
            }
            match current.requester {
                tasks::Party::Task(parent) => match self.tasks.task(parent) {
                    Some(above) => current = above,
                    None => return None,
                },
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return None,
            }
        }
        None
    }
}
