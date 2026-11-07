//! Cross-row facts retained by their source keys, resolved once per commit
//! (`jig/docs/design/domain/testing.md`, 10).
use super::super::{acceptance, core, people, r, tasks};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Joins {
    authority: BTreeMap<core::Key, (skein_lib::Wall, u64, r::Acceptance)>,
    decisions: BTreeMap<core::Key, core::ProposalDecisionRecord>,
    amendments: BTreeMap<core::Key, u64>,
}

impl Joins {
    pub(super) fn replace(&mut self, key: &core::Key, row: Option<&core::Record>) {
        self.authority.remove(key);
        self.decisions.remove(key);
        self.amendments.remove(key);
        if let Some(core::Record::Tasks(tasks::Stored::History(row))) = row
            && let Some(accepted) = acceptance::task(row)
        {
            self.authority.insert(key.clone(), accepted);
        }
        if let Some(core::Record::Core(core::CoreRecord::ProposalDecision(row))) = row
            && row.choice == people::ProposalChoice::Accepted
        {
            self.decisions.insert(key.clone(), *row);
        }
        if let Some(core::Record::Tasks(tasks::Stored::DecidedProposal(row))) = row
            && let tasks::PersonAction::Amend { task, .. } = row.action
        {
            self.amendments.insert(key.clone(), task);
        }
    }

    pub(super) fn decisions(&self) -> Vec<core::ProposalDecisionRecord> {
        self.decisions.values().copied().collect()
    }

    pub(super) fn resolve(&self, snapshot: &mut r::Snapshot) {
        let mut authority: Vec<_> = self.authority.values().copied().collect();
        for accepted in self.decisions.values() {
            let key = core::Key::Tasks(tasks::Key::DecidedProposal(accepted.proposal));
            if let Some(task) = self.amendments.get(&key) {
                authority.push((
                    accepted.at,
                    *task,
                    r::Acceptance { proposal: accepted.proposal, by: r::Source::Person(accepted.by) },
                ));
            }
            if let Some((first, last)) = accepted.created {
                for task in first..=last {
                    if snapshot.tasks.get(&task).is_some_and(|row| {
                        row.source == match accepted.proposer {
                            people::Proposer::Task(task) => r::Source::Task(task),
                            people::Proposer::Party(person) => r::Source::Person(person),
                        }
                    }) {
                        snapshot.accepted_tasks.insert(
                            task,
                            r::Acceptance { proposal: accepted.proposal, by: r::Source::Person(accepted.by) },
                        );
                    }
                }
            }
        }
        // Stable ordering keeps a history before a party decision at the same wall.
        authority.sort_by_key(|(at, ..)| *at);
        for (_, task, accepted) in authority {
            snapshot.accepted_tasks.insert(task, accepted);
        }
    }
}
