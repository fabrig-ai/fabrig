//! Admit the composed changes and train as batches at the holds bound (temper/docs/design/testing/engine-checks.md, 5.2).
use super::{super::scripts::ROOM_CUE, World};
use crate::composed::{people, tasks};
use std::collections::BTreeMap;
use temper_engine_domain_forge as forge;
use temper_engine_forge_world::open::BRANCH_HOLDS_CAPACITY;
use temper_workload::draw::Goal;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Load {
    Waiting,
    Admitted,
}

impl World {
    pub(in crate::composed::workload) fn waits_for_branch_room(&self, task: u64) -> bool {
        self.workload_faults.as_ref().and_then(|f| f.branch_holds.as_ref()).and_then(|a| a.get(&task))
            == Some(&Load::Waiting)
    }

    /// The coordinator's first script starts its changes in one call, including the whole train.
    pub(in crate::composed::workload) fn defer_branch_load(&mut self, task: u64) -> bool {
        let Some(admission) = self.workload_faults.as_ref().and_then(|f| f.branch_holds.as_ref()) else {
            return false;
        };
        let Some(goal) = self.workload_names().logical(task).and_then(|name| self.workload().goal(name)) else {
            return false;
        };
        let defer = !self.branch_load_fits(admission, goal);
        if defer {
            eprintln!("{}: deferring the change batch for branch holds at {:?}", goal.name, self.env.now);
        }
        let f = self.workload_faults.as_mut().expect("faulted workload");
        let admission = f.branch_holds.as_mut().expect("planned branch holds");
        admission.insert(task, if defer { Load::Waiting } else { Load::Admitted });
        if defer {
            f.ledger.leave_out(BRANCH_HOLDS_CAPACITY.scenario, BRANCH_HOLDS_CAPACITY.fault, self.env.now);
        }
        defer
    }

    pub(in crate::composed::workload) fn release_branch_load(&mut self) {
        let Some(admission) = self.workload_faults.as_ref().and_then(|f| f.branch_holds.as_ref()) else { return };
        // Restored task rows can precede restored holds. Read both only once
        // the root has opened.
        if !self.opened {
            return;
        }
        let story = self.workload.as_ref().expect("workload");
        let ready = self.workload_tasks().find_map(|row| {
            let tasks::Party::Task(owner) = row.requester else { return None };
            if !row.spec.words.starts_with(ROOM_CUE.as_bytes()) || story.answered.contains(&row.number) {
                return None;
            }
            let goal = self.workload().goal(self.workload_names().logical(owner)?)?;
            match admission.get(&owner)? {
                Load::Waiting if self.branch_load_fits(admission, goal) => Some((owner, goal.party, None)),
                Load::Admitted if goal.changes.iter().all(|change| story.names.task(&change.name).is_some()) => {
                    Some((owner, goal.party, Some((row.number, row.revisions.spec))))
                }
                _ => None,
            }
        });
        let Some((owner, party, report)) = ready else { return };
        if let Some((task, revision)) = report {
            self.workload.as_mut().expect("workload").answered.insert(task);
            self.workload_enqueue(
                party,
                people::Ask::AnswerPerson {
                    project: 1,
                    task,
                    revision,
                    result: people::PersonResult::Report {
                        words: b"The batch of changes has started.".as_slice().into(),
                    },
                },
            );
        } else {
            eprintln!("goal {owner}: admitting the deferred change batch at {:?}", self.env.now);
            let admission = self.workload_faults.as_mut().and_then(|f| f.branch_holds.as_mut()).expect("planned holds");
            admission.insert(owner, Load::Admitted);
            // Words wake the coordinator. Answering its only delegate before
            // the batch exists would make the closer instead (jig/docs/design/domain/tasks/lifecycle-endings.md, 3).
            self.workload_enqueue(
                party,
                people::Ask::Say {
                    project: 1,
                    task: owner,
                    words: b"There is room for the changes.".as_slice().into(),
                },
            );
        }
    }

    fn branch_load_fits(&self, admission: &BTreeMap<u64, Load>, new: &Goal) -> bool {
        let forge = self.root.forge();
        let holds = forge.inspection_holds();
        let changes = forge.inspection_changes();
        let pending = changes
            .iter()
            .filter(|(task, _)| !self.workload_task_ended(**task))
            .map(|(task, row)| {
                let branch = u32::from(!branch_held(forge, &row.branch));
                let prepared =
                    forge.landing_of(*task).and_then(|landing| forge.landing(landing)).is_some_and(|landing| {
                        landing.landing.members.iter().any(|member| {
                            member.change == *task && (member.candidate.is_some() || branch_held(forge, &member.branch))
                        })
                    });
                branch + u32::from(!prepared)
            })
            .sum::<u32>();
        let reserved = admission
            .iter()
            .filter(|(task, load)| **load == Load::Admitted && !self.workload_task_ended(**task))
            .map(|(task, _)| {
                let name = self.workload_names().logical(*task).expect("admitted goal named");
                let goal = self.workload().goal(name).expect("admitted goal drawn");
                self.reserved_branch_load(*task, goal)
            })
            .sum::<u32>();
        let added = 2 * u32::try_from(new.changes.len()).expect("bounded batch");
        holds.len() + pending + reserved + added <= holds.capacity()
    }

    fn reserved_branch_load(&self, task: u64, goal: &Goal) -> u32 {
        let forge = self.root.forge();
        let changes = forge.inspection_changes();
        // An admitted batch reserves its branches and prepare prefixes until
        // its rows enter the adapter. A closer reuses its goal's branch and
        // needs just a prefix after the goal's ordinary changes have landed.
        let mut waiting = 0;
        let mut live = false;
        for change in &goal.changes {
            match self.workload_names().task(&change.name) {
                Some(task) if self.workload_task_ended(task) => {}
                Some(task) if changes.contains_key(&task) => live = true,
                _ => waiting += 2,
            }
        }
        if waiting == 0
            && !live
            && !goal.train
            && !changes.iter().any(|(change, row)| row.goal == Some(task) && !self.workload_task_ended(*change))
        {
            waiting = 1;
        }
        waiting
    }
}

fn branch_held(forge: &forge::Domain, branch: &[u8]) -> bool {
    forge.inspection_holds().iter().any(|(name, _)| match &name.what {
        forge::What::Branch(parts) => parts.iter().map(AsRef::as_ref).eq(branch.split(|byte| *byte == b'/')),
        _ => false,
    })
}
