//! Rows loaded for erasure by key (jig/docs/design/protocol/store.md, 11.1).
use crate::{CoreKey, Key};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

/// Rows the core loads to erase them by key (protocol/store.md, 11.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sweep {
    /// The tasks family's `Ended` rows numbered above `after`, by number.
    Ended { after: u64 },
    /// Task-first history, milestones, activity, ended read positions,
    /// terminals and escalation decisions. Turns are the transcript's.
    Task(u64),
    /// Index lines in the project's project, goal and resource scopes.
    Notes { project: u32 },
}

impl Sweep {
    /// Predicate checked against the store's exact intervals (store.md, 4.3).
    #[must_use]
    pub fn admits(&self, key: &Key) -> bool {
        match self {
            Self::Ended { after } => match key {
                Key::Tasks(tasks::Key::Ended(task)) => task > after,
                Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
            },
            Self::Task(task) => match key {
                Key::Tasks(
                    tasks::Key::History { task: owner, .. }
                    | tasks::Key::Milestone { task: owner, .. }
                    | tasks::Key::Activity { task: owner, .. },
                )
                | Key::People(people::Key::EndedRead { task: owner, .. })
                | Key::Core(CoreKey::Terminal { task: owner, .. } | CoreKey::EscalationDecision { task: owner, .. }) => {
                    task == owner
                }
                Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
            },
            Self::Notes { project } => match key {
                Key::Notes(notes::Key::Line { scope, .. }) => match scope {
                    notes::Scope::Project { project: owner }
                    | notes::Scope::Goal { project: owner, .. }
                    | notes::Scope::Resources { project: owner, .. } => project == owner,
                    notes::Scope::Deployment => false,
                },
                Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweeps_admit_only_their_numbered_families() {
        const TASK: u64 = 8;
        const PROJECT: u32 = 3;
        let task_keys = [
            Key::Core(CoreKey::Terminal { task: TASK, attempt: 1 }),
            Key::Core(CoreKey::EscalationDecision { task: TASK, revision: 1 }),
            Key::Tasks(tasks::Key::History { task: TASK, revision: 1 }),
            Key::Tasks(tasks::Key::Milestone { task: TASK, position: 1 }),
            Key::Tasks(tasks::Key::Activity { task: TASK, position: 1 }),
            Key::People(people::Key::EndedRead { task: TASK, party: 1 }),
        ];
        for key in task_keys {
            assert!(Sweep::Task(TASK).admits(&key), "{key:?}");
            assert!(!Sweep::Task(9).admits(&key), "{key:?}");
            assert!(!Sweep::Ended { after: 0 }.admits(&key));
            assert!(!Sweep::Notes { project: PROJECT }.admits(&key));
        }
        for key in [
            Key::Tasks(tasks::Key::Ended(TASK)),
            Key::Tasks(tasks::Key::Live(TASK)),
            Key::Tasks(tasks::Key::Stub(TASK)),
            Key::Tasks(tasks::Key::Delegate { requester: TASK, task: 9 }),
            Key::People(people::Key::ReadConversation { party: 1, task: TASK }),
            Key::Core(CoreKey::Turn { task: TASK, attempt: 1, turn: 1 }),
            Key::Core(CoreKey::RunProof(TASK)),
            Key::Accounts(crate::accounts::Key::Account { number: 1 }),
            Key::Notes(notes::Key::Entry { name: TASK }),
        ] {
            assert!(!Sweep::Task(TASK).admits(&key), "{key:?}");
        }
        for (after, task, admitted) in [(7, 8, true), (8, 8, false), (9, 8, false), (u64::MAX, u64::MAX, false)] {
            assert_eq!(Sweep::Ended { after }.admits(&Key::Tasks(tasks::Key::Ended(task))), admitted);
        }
        for scope in [
            notes::Scope::Project { project: PROJECT },
            notes::Scope::Goal { project: PROJECT, goal: TASK },
            notes::Scope::Resources {
                project: PROJECT,
                connector: 1,
                pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
            },
        ] {
            let key = Key::Notes(notes::Key::Line { scope, name: 1 });
            assert!(Sweep::Notes { project: PROJECT }.admits(&key));
            assert!(!Sweep::Notes { project: 4 }.admits(&key));
        }
        assert!(
            !Sweep::Notes { project: 0 }
                .admits(&Key::Notes(notes::Key::Line { scope: notes::Scope::Deployment, name: 1 }))
        );
    }
}
