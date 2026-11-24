//! Durable outcomes in logical names (`temper/docs/design/testing/workloads.md`, 7).
use std::collections::{BTreeMap, BTreeSet};

use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::Time;
use skein_world::faults::FaultLedger;
use temper_engine_domain::Record;
use temper_engine_domain_forge as forge;
use temper_faults::restarts::{ENGINE_CRASH, ENGINE_CRASH_MID_TURN};
use temper_workload::{
    draw::Workload,
    names::Names,
    outcome::{End, GoalOutcome, Outcome},
};

use crate::store::Store;

/// Read committed task rows and the forge's main tree, not the driver's intentions.
#[must_use]
pub fn read(
    store: &Store,
    main: &BTreeMap<String, String>,
    names: &Names,
    workload: &Workload,
    ledger: &FaultLedger,
) -> Outcome {
    let goals = workload
        .goals
        .iter()
        .map(|goal| {
            let number = names.task(&goal.name).expect("workload goal named");
            let members = members(store, number);
            let mut landed: Vec<_> = goal
                .changes
                .iter()
                .filter(|change| change.files.iter().all(|file| main.get(&file.path) == Some(&file.text)))
                .map(|change| change.name.clone())
                .collect();
            landed.sort();
            let mut holds: Vec<_> = store
                .rows
                .values()
                .filter_map(|record| {
                    let Record::Core(core::Record::Tasks(tasks::Stored::Hold(row))) = record else { return None };
                    members.contains(&row.task).then(|| hold(row.held.why).to_owned())
                })
                .collect();
            holds.sort();
            let tries = members
                .iter()
                .filter_map(|number| crate::acts::task(store, *number))
                .map(|row| {
                    let tries = row.tries;
                    let total: u64 = [
                        tries.transient,
                        tries.permanent,
                        tries.run,
                        tries.agent,
                        tries.lost,
                        tries.invalid,
                        tries.infrastructure,
                    ]
                    .into_iter()
                    .map(u64::from)
                    .sum();
                    let lost = store
                        .rows
                        .values()
                        .filter(|record| {
                            let Record::Core(core::Record::Tasks(tasks::Stored::Attempt(attempt))) = record else {
                                return false;
                            };
                            attempt.task == row.number
                                && matches!(attempt.end, Some(tasks::EndKind::Failed { class: tasks::Class::Lost, .. }))
                                && attempt.ended.is_some_and(|at| {
                                    ledger.entries().iter().any(|fault| {
                                        [ENGINE_CRASH, ENGINE_CRASH_MID_TURN].contains(&fault.kind)
                                            && fault.in_force(Time::from_nanos(at.as_nanos()))
                                    })
                                })
                        })
                        .count();
                    // A successful park resets the counters; older lost attempts cannot
                    // cancel failures of another class in the current activation.
                    total - u64::from(tries.lost).min(u64::try_from(lost).expect("attempt count"))
                })
                .sum();
            let end = match &crate::acts::task(store, number).expect("goal row").phase {
                tasks::Phase::Ended(tasks::Ending::Done(_)) if branch_landed(store, number) => End::Landed,
                tasks::Phase::Ended(tasks::Ending::Done(_)) => End::Done,
                tasks::Phase::Ended(tasks::Ending::Failed { .. }) => End::Failed,
                tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => End::Cancelled,
                tasks::Phase::Held { why, .. } => End::Held(hold(*why).into()),
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) => End::Waiting,
            };
            (goal.name.clone(), GoalOutcome { end, landed, holds, tries })
        })
        .collect();
    let prefixes: Vec<_> = workload.goals.iter().map(|goal| format!("{}/", goal.name)).collect();
    Outcome {
        goals,
        main: main
            .iter()
            .filter(|(path, _)| prefixes.iter().any(|prefix| path.starts_with(prefix)))
            .map(|(path, text)| (path.clone(), text.clone()))
            .collect(),
    }
}

pub(crate) fn members(store: &Store, goal: u64) -> BTreeSet<u64> {
    let mut members = BTreeSet::from([goal]);
    loop {
        let before = members.len();
        for record in store.rows.values() {
            if let Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) = record
                && let tasks::Party::Task(parent) = row.requester
                && members.contains(&parent)
            {
                members.insert(row.number);
            }
        }
        if members.len() == before {
            return members;
        }
    }
}

fn branch_landed(store: &Store, goal: u64) -> bool {
    store.rows.values().any(|record| {
        let Record::Forge { row } = record else { return false };
        let forge::Stored::Closer(closer) = row.as_ref() else { return false };
        closer.goal == goal
            && closer.closer.landing.is_some_and(|landing| {
                store.change_hot(landing).is_some_and(|row| {
                    matches!(
                        row.change.state,
                        temper_engine_domain_forge_change::State::Landed { .. }
                            | temper_engine_domain_forge_change::State::Red { .. }
                            | temper_engine_domain_forge_change::State::Cleared { .. }
                            | temper_engine_domain_forge_change::State::Uncleared { .. }
                    )
                })
            })
    })
}

/// Deadlines observed while armed, including procedure bounds from the world's adapter.
#[derive(Default)]
pub struct Deadlines(BTreeSet<u64>);

impl Deadlines {
    #[must_use]
    pub fn armed(root: &temper_engine_domain::Domain, store: &Store, now: Time) -> BTreeSet<(u64, Time)> {
        let mut armed = BTreeSet::new();
        for record in store.rows.values() {
            if let Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record
                && let Some(at) = root.tasks().deadline(row.number, skein_lib::Wall::from_nanos(now.as_nanos()))
                && at.as_nanos() >= now.as_nanos()
            {
                armed.insert((row.number, Time::from_nanos(at.as_nanos())));
            }
        }
        let mut inspection = skein_world::inspect::Inspection::new("engine", temper_engine_inspect::KINDS);
        temper_engine_inspect::inspect(root, &mut inspection);
        for wait in inspection.waits() {
            if let Some(at) = wait.bound
                && at >= now
                && let Some(task) = wait.waiter.strip_prefix("task ").and_then(|number| number.parse().ok())
            {
                armed.insert((task, at));
            }
        }
        armed
    }

    pub fn struck(&mut self, armed: &BTreeSet<(u64, Time)>, faults: &[skein_world::faults::Injected]) {
        self.0.extend(
            armed.iter().filter(|(_, at)| faults.iter().any(|fault| fault.in_force(*at))).map(|(task, _)| *task),
        );
    }

    #[must_use]
    pub fn left(&self, store: &Store, names: &Names, workload: &Workload) -> Vec<temper_workload::outcome::Left> {
        workload
            .goals
            .iter()
            .filter_map(|goal| {
                let tasks = members(store, names.task(&goal.name).expect("goal named"));
                self.0.iter().any(|task| tasks.contains(task)).then(|| temper_workload::outcome::Left {
                    goal: goal.name.clone(),
                    why: "a deadline racing a restart",
                })
            })
            .collect()
    }
}

// The kind's words, without identities or the hold's instance-specific explanation.
fn hold(why: tasks::Hold) -> &'static str {
    match why {
        tasks::Hold::Exceeded(_) => "exceeded",
        tasks::Hold::Failures(_) => "failures",
        tasks::Hold::Stalled => "stalled",
        tasks::Hold::Dependency(_) => "dependency",
        tasks::Hold::Stopped => "stopped",
        tasks::Hold::Placed { .. } => "placed",
        tasks::Hold::Drift => "drift",
        tasks::Hold::Effects => "effects",
        tasks::Hold::EffectFailed => "effect failed",
        tasks::Hold::Uncertain { .. } => "uncertain",
        tasks::Hold::Budget => "budget",
        tasks::Hold::Deadline => "deadline",
        tasks::Hold::HoldsWaited => "holds waited",
        tasks::Hold::ResourceUnavailable => "resource unavailable",
        tasks::Hold::ResourceAccess => "resource access",
        tasks::Hold::PoolLost { .. } => "pool lost",
        tasks::Hold::Procedure => "procedure",
        tasks::Hold::Circle => "circle",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_world::faults::Struck;
    use temper_workload::draw::{Goal, Party};

    #[test]
    fn only_armed_deadlines_inside_a_restart_leave_out_their_goal() {
        let workload = Workload {
            tier: false,
            goals: (1..=3)
                .map(|number| Goal {
                    name: format!("g{number}"),
                    party: Party::First,
                    after: None,
                    train: false,
                    changes: Vec::new(),
                })
                .collect(),
        };
        let mut names = Names::new();
        for number in 1..=3 {
            names.name(&format!("g{number}"), number).expect("unique name");
        }
        let mut ledger = FaultLedger::new();
        let crash = ledger.inject(ENGINE_CRASH, Time::from_nanos(10), Struck::default());
        ledger.stop(crash, Time::from_nanos(20)).expect("closed interval");
        let mut deadlines = Deadlines::default();
        // g3's old deadline is no longer armed. g2's remains outside the interval.
        deadlines.struck(&BTreeSet::from([(1, Time::from_nanos(20)), (2, Time::from_nanos(21))]), ledger.entries());
        let left = deadlines.left(&Store::new(), &names, &workload);
        assert_eq!(left, [temper_workload::outcome::Left { goal: "g1".into(), why: "a deadline racing a restart" }]);
    }
}
