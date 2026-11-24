//! Funds are drawn independently of tasks and parties, checked at admission
//! (jig/docs/design/domain/testing-workloads.md, 5).
use super::{
    Driver, Kind, Party, Stream, Streams, Struck, Sweep, TaskKind, Workload, World, core, fixtures, open, person,
    requests, root, tasks,
};

pub(super) struct Draw {
    pub kind: Kind,
    pub target: usize,
    budget: u64,
    checked: bool,
    pub injected: bool,
}

pub(super) fn draw(streams: Streams, configured: &[Kind], workload: &Workload, minimum: u64) -> Vec<Draw> {
    let mut rng = streams.rng(Stream::Faults, "funds");
    let mut draws = Vec::new();
    for kind in [open::START_WITH_NO_BUDGET, open::NEW_PARTY_ASKS_FOR_FUNDS, open::DELEGATING_RUN_AT_ITS_MINIMUM] {
        if !configured.contains(&kind) {
            continue;
        }
        let chosen = rng.below(2) == 0;
        let candidates: Vec<_> = workload
            .tasks
            .iter()
            .enumerate()
            .filter_map(|(index, task)| {
                if task.parent.is_some() {
                    return None;
                }
                let eligible = if kind == open::NEW_PARTY_ASKS_FOR_FUNDS {
                    task.party == Party::Member
                        && !workload
                            .tasks
                            .iter()
                            .take(index)
                            .any(|other| other.parent.is_none() && other.party == Party::Member)
                } else if kind == open::DELEGATING_RUN_AT_ITS_MINIMUM {
                    // One leaf delegate can share a near-minimum budget with its run.
                    // Larger subtrees retain the workload's drawn funding.
                    let children: Vec<_> =
                        workload.tasks.iter().enumerate().filter(|(_, child)| child.parent == Some(index)).collect();
                    children.len() == 1
                        && !workload.tasks.iter().any(|child| child.parent == Some(children[0].0))
                        && !draws
                            .iter()
                            .any(|draw: &Draw| draw.target == index && draw.kind == open::START_WITH_NO_BUDGET)
                } else {
                    true
                };
                eligible.then_some(index)
            })
            .collect();
        let target = rng.next_u64();
        let budget = 2 * minimum + rng.below(3);
        if chosen && !candidates.is_empty() {
            let target = candidates[usize::try_from(target % u64::try_from(candidates.len()).expect("bounded tasks"))
                .expect("bounded index")];
            draws.push(Draw { kind, target, budget, checked: false, injected: false });
        }
    }
    draws
}

fn funding(world: &World, funder: tasks::Funder) -> Option<&tasks::FundingRecord> {
    match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ledger(funder)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row)))) => Some(row),
        _ => None,
    }
}

impl Driver {
    pub(super) fn join(&mut self, world: &mut World) {
        if !self.joining
            || !self
                .workload
                .tasks
                .iter()
                .enumerate()
                .any(|(index, task)| task.party == Party::Owner && self.progress[index].number.is_some())
        {
            return;
        }
        let sign_in = fixtures::second_party(world, b"workload-member", requests::Role::Member);
        self.sign_ins[Party::Member.index()] = sign_in;
        self.persons[Party::Member.index()] = person(world, sign_in);
        self.joining = false;
    }

    pub(super) fn fund(&mut self, world: &mut World, target: usize, sweep: &Sweep) -> Result<(), String> {
        for draw in &mut self.funds {
            if draw.target != target || draw.checked {
                continue;
            }
            let task = &self.workload.tasks[target];
            let minimum = world.restart_config.0.core.defaults.rules.minimum_run_spend;
            let mut holding = Vec::new();
            if draw.kind == open::START_WITH_NO_BUDGET
                && self.workload.tasks.iter().enumerate().any(|(index, other)| {
                    index != target
                        && other.parent.is_none()
                        && other.party == task.party
                        && self.progress[index].number.is_none_or(|n| !super::ended(world, n))
                })
            {
                holding.push("its pool funds another request");
            }
            if draw.kind == open::NEW_PARTY_ASKS_FOR_FUNDS {
                let period = funding(world, tasks::Funder::Period { project: 1, period: 1 });
                if period.is_some_and(|period| {
                    period.numbers.budget.saturating_sub(period.numbers.spent).saturating_sub(period.numbers.reserved)
                        == 0
                }) {
                    holding.push("its period is fully allocated");
                }
            }
            if draw.kind == open::DELEGATING_RUN_AT_ITS_MINIMUM && draw.budget / 2 <= minimum {
                holding.push("half its budget is at most the minimum run spend");
            }
            let now = world.clocks().0;
            let ledger = world.fault_ledger.as_mut().expect("workload ledger");
            draw.checked = true;
            if let Some(exclusion) = sweep.leaves_out(draw.kind, &holding) {
                ledger.leave_out(exclusion.scenario, draw.kind, now);
                continue;
            }
            let entry = ledger.inject(draw.kind, now, Struck { request: Some(task.name.clone()), ..Struck::default() });
            ledger.stop(entry, now)?;
            draw.injected = true;
            if draw.kind == open::DELEGATING_RUN_AT_ITS_MINIMUM {
                self.workload.tasks[target].budget = draw.budget;
                for task in &mut self.workload.tasks {
                    if task.parent == Some(target) {
                        task.budget = if task.kind == TaskKind::Question { 0 } else { minimum + 1 };
                    }
                }
            }
        }
        Ok(())
    }
}
