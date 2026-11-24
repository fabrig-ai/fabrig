//! Mixed finite faults over the composed draw (temper/docs/design/testing/workloads.md, 8.2).
use super::World;
use skein_lib::Time;
use skein_world::{
    faults::{FaultLedger, Injection, Kind, Struck},
    streams::{Stream, Streams},
};
use std::collections::{BTreeMap, BTreeSet};
use temper_faults::{forge as forge_faults, restarts, stale};
use temper_workload::draw::Workload;

mod blame;
mod forge;
mod runs;

#[derive(Debug)]
struct Plan {
    kind: Kind,
    goal: String,
    after: u64,
    fired: bool,
    query: usize,
}

pub(in crate::composed) struct Faults {
    seed: u64,
    plans: Vec<Plan>,
    steps: BTreeMap<String, u64>,
    ledger: FaultLedger,
    last: Time,
    lag_until: Option<Time>,
    log_restore: Option<(u64, u32, Box<[u8]>, Injection)>,
    closed: Option<(u64, Injection, Time)>,
    restored: Option<u64>,
    stale_release: Option<u64>,
    opening: Option<Injection>,
    rejoining: Option<usize>,
    provider: Option<(u32, Time)>,
    runs: smith_faults::Runs,
    observed_runs: BTreeMap<(u64, u64), runs::ObservedRun>,
    failed_attempts: BTreeSet<(u64, u64)>,
    reactions: Vec<temper_blame::Observed>,
    retries: BTreeSet<(u64, u64, u32)>,
    repairs: BTreeSet<u64>,
    holds: BTreeSet<String>,
    wrong: BTreeSet<u64>,
}

impl Faults {
    pub(super) fn new(seed: u64, workload: &Workload, kinds: &[Kind]) -> Self {
        let streams = Streams::new(seed);
        let mut forge_rng = streams.rng(Stream::Faults, "forge");
        let mut llm_rng = streams.rng(Stream::Faults, "llm");
        let mut restart_rng = streams.rng(Stream::Faults, restarts::PART);
        // Draw the fixed positions even for unconfigured kinds. Removing a
        // family cannot shift another family's choice within the forge part.
        let supported = [
            forge_faults::TIMED_OUT,
            forge_faults::HEAD_BEHIND,
            forge_faults::JOB_LOG,
            forge_faults::CLOSED_BY_ANOTHER,
            temper_blame::LOST,
            stale::MOVED_BASE,
            skein_llm_kit::faults::RATE_WINDOW,
            restarts::ENGINE_CRASH_MID_TURN,
        ];
        assert!(kinds.iter().all(|kind| supported.contains(kind)), "supported composed faults");
        let plans = supported
            .iter()
            .map(|kind| {
                let rng = match kind.family {
                    "provider" => &mut llm_rng,
                    "restarts" => &mut restart_rng,
                    _ => &mut forge_rng,
                };
                let eligible: Vec<_> = workload
                    .goals
                    .iter()
                    .filter(|goal| {
                        if *kind == stale::MOVED_BASE {
                            goal.train
                        } else if *kind == forge_faults::HEAD_BEHIND || *kind == forge_faults::JOB_LOG {
                            goal.changes.iter().any(|change| change.red_once)
                        } else {
                            true
                        }
                    })
                    .collect();
                assert!(!eligible.is_empty(), "a workload goal reaches {kind:?}");
                let goal = eligible[usize::try_from(rng.below(u64::try_from(eligible.len()).expect("bounded goals")))
                    .expect("goal index")];
                let after = rng.below(16);
                let query = if *kind == restarts::ENGINE_CRASH_MID_TURN {
                    1 + usize::try_from(rng.below(2)).expect("two query points")
                } else {
                    1
                };
                Plan { kind: *kind, goal: goal.name.clone(), after, fired: false, query }
            })
            .filter(|plan| kinds.contains(&plan.kind))
            .collect();
        Self {
            seed,
            plans,
            steps: BTreeMap::new(),
            ledger: FaultLedger::new(),
            last: Time::ZERO,
            lag_until: None,
            log_restore: None,
            closed: None,
            restored: None,
            stale_release: None,
            opening: None,
            rejoining: None,
            provider: None,
            runs: smith_faults::Runs::new(),
            observed_runs: BTreeMap::new(),
            failed_attempts: BTreeSet::new(),
            reactions: Vec::new(),
            retries: BTreeSet::new(),
            repairs: BTreeSet::new(),
            holds: BTreeSet::new(),
            wrong: BTreeSet::new(),
        }
    }

    pub(super) fn needs_small_steps(&self) -> bool {
        self.provider.is_some() || self.opening.is_some() || self.rejoining.is_some()
    }

    fn due(&self, kind: Kind) -> bool {
        self.plans.iter().any(|p| p.kind == kind && !p.fired && self.steps.get(&p.goal).is_some_and(|s| *s >= p.after))
    }

    fn target(&self, kind: Kind) -> Option<&str> {
        self.plans.iter().find(|plan| plan.kind == kind).map(|plan| plan.goal.as_str())
    }

    fn fire(&mut self, kind: Kind) -> String {
        let plan = self.plans.iter_mut().find(|p| p.kind == kind && !p.fired).expect("configured fault");
        plan.fired = true;
        plan.goal.clone()
    }

    fn stopped(&mut self, at: Time) {
        self.last = self.last.max(at);
    }
}

impl World {
    /// Keep the configured kinds as input so later families extend the same sweep.
    #[must_use]
    pub fn faulted_workload(seed: u64, kinds: &[Kind]) -> Self {
        Self::workload_with_faults(seed, kinds)
    }

    #[must_use]
    pub fn workload_ledger(&self) -> &FaultLedger {
        &self.workload_faults.as_ref().expect("faulted workload").ledger
    }

    pub(in crate::composed) fn workload_faults_stopped(&self) -> bool {
        self.workload_faults.as_ref().is_none_or(|f| {
            f.plans.iter().all(|p| p.fired)
                && f.closed.is_none()
                && f.provider.is_none()
                && f.opening.is_none()
                && f.rejoining.is_none()
                && f.lag_until.is_none()
                && f.log_restore.is_none()
                && self.env.now >= f.last
        })
    }

    /// Judge runs and gates once, after the ordinary workload's checks.
    pub fn judge_workload_faults(&mut self) {
        let f = self.workload_faults.as_mut().expect("faulted workload");
        super::super::accounts::judge_account_runs(&mut f.ledger, &mut f.runs).expect("account run blame");
        let findings = temper_blame::judge(&mut f.ledger, &f.reactions, f.seed);
        assert!(findings.is_empty(), "{}", findings.join("\n"));
        assert!(f.ledger.entries().iter().all(|entry| entry.until.is_some()), "all faults have a last moment");
        eprintln!("seed {}: faults stopped by {:?}", f.seed, f.last);
        for entry in f.ledger.entries() {
            eprintln!(
                "{:?}: {:?}..{:?}; {:?}; gate retries {}, repairs {}, task tries {}",
                entry.kind,
                entry.from,
                entry.until,
                entry.struck,
                entry.count(skein_world::faults::Counter::GateRetries),
                entry.count(skein_world::faults::Counter::Repairs),
                entry.count(skein_world::faults::Counter::TaskTries)
            );
        }
    }

    pub(in crate::composed) fn workload_fault_tick(&mut self) {
        let Some(mut f) = self.workload_faults.take() else { return };
        for goal in &self.workload().goals {
            if self.workload_names().task(&goal.name).is_some_and(|task| {
                self.root
                    .tasks()
                    .task(task)
                    .is_some_and(|row| !matches!(row.phase, crate::composed::tasks::Phase::Waiting))
            }) {
                *f.steps.entry(goal.name.clone()).or_default() += 1;
            }
        }
        self.copy_workload_forge_faults(&mut f);
        self.observe_workload_runs(&mut f);
        self.workload_forge_tick(&mut f);
        self.workload_provider_tick(&mut f);
        let restart_task = f
            .plans
            .iter()
            .find(|p| p.kind == restarts::ENGINE_CRASH_MID_TURN)
            .and_then(|p| self.workload_names().task(&p.goal));
        if f.due(restarts::ENGINE_CRASH_MID_TURN)
            && self.assignment.as_ref().is_some_and(|a| Some(a.task) == restart_task)
            && !self.process.prepared()
            && self.process.held_query.is_some_and(|query| {
                self.process.agent().prompts().len() == query && self.process.agent().turns().len() + 1 == query
            })
            && self.worker.retained_turns() == 0
            && self.store.idle()
            && self.root_events.is_empty()
            && self.worker_events.is_empty()
        {
            let goal = f.fire(restarts::ENGINE_CRASH_MID_TURN);
            eprintln!(
                "crash at {:?}, iteration {}, assignment {:?}, queries {}",
                self.env.now,
                self.iteration,
                self.assignment.as_ref().map(|a| (a.task, a.attempt)),
                self.process.agent().prompts().len()
            );
            self.cut_link();
            self.stop_engine();
            let injection = f.ledger.inject(
                restarts::ENGINE_CRASH_MID_TURN,
                self.env.now,
                Struck { request: Some(goal), ..Struck::default() },
            );
            // The composed settings and all durable rows survive the root, not its queues.
            let config = temper_engine_domain_world::oracle::config::copy(&self.restart_config);
            self.reopen_with_config(config);
            f.opening = Some(injection);
        }
        if self.opened
            && let Some(injection) = f.opening.take()
        {
            eprintln!("opened at {:?}, iteration {}", self.env.now, self.iteration);
            f.ledger.stop(injection, self.env.now).expect("restart opening");
            f.stopped(self.env.now);
            f.rejoining = Some(self.absence_admissions());
            self.absence.link_joined = true;
            self.worker_events.push_back(super::super::worker::Event::Ready);
        }
        if f.rejoining.is_some_and(|before| self.absence_admissions() > before) {
            self.process.held_query = None;
            f.rejoining = None;
        }
        self.ends.faults_stop(f.last);
        self.workload_faults = Some(f);
    }
}
