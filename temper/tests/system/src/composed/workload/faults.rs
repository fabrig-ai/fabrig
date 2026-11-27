//! Mixed finite faults over the composed draw (temper/docs/design/testing/workloads.md, 8.2).
use super::World;
use skein_lib::Time;
use skein_world::faults::{FaultLedger, Injection, Kind, Struck};
use std::collections::{BTreeMap, BTreeSet};
use temper_engine_domain_world::acts::Watch;
use temper_faults::{bounds, forge as forge_faults, parks, restarts, stale};
use temper_worker_inspect::faults::DISK_FULL;
use temper_workload::parties::{Act, Action};

mod actions;
mod blame;
mod draw;
pub use draw::FaultDraw;
mod forge;
mod runs;
#[cfg(test)]
mod tests;
mod worker;

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
    provider_failures: Vec<((u64, u64), Injection)>,
    reactions: Vec<temper_blame::Observed>,
    retries: BTreeSet<(u64, u64, u32)>,
    repairs: BTreeSet<u64>,
    holds: BTreeSet<String>,
    wrong: BTreeSet<u64>,
    bounds: Vec<Kind>,
    actions: Vec<Action>,
    release_configured: bool,
    release_from: Option<(u64, u64)>,
    pub(super) released_member: Option<(u64, Time)>,
    watch: Watch,
    disk: worker::Disk,
    park: Option<(String, u64, u64)>,
    parked: bool,
    read_latency: skein_lib::Duration,
    read_replies: Vec<(Time, crate::composed::engine::Event)>,
}

impl Faults {
    pub(super) fn new(seed: u64, draw: FaultDraw) -> Self {
        let FaultDraw { plans, actions, ineligible, bounds, disk, read_latency, release_configured, .. } = draw;
        let mut ledger = FaultLedger::new();
        for kind in ineligible {
            ledger.ineligible(kind);
        }
        Self {
            bounds,
            release_configured,
            watch: Watch::new(actions.iter().filter(|action| action.act == Act::HoldMember).cloned().collect()),
            release_from: None,
            released_member: None,
            actions,
            disk,
            park: None,
            parked: false,
            read_latency,
            read_replies: Vec::new(),
            seed,
            plans,
            steps: BTreeMap::new(),
            ledger,
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
            provider_failures: Vec::new(),
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
                && f.read_replies.is_empty()
                && f.actions.is_empty()
                && f.disk.stopped()
                && (f.park.is_none() || f.parked)
                && f.closed.is_none()
                && f.provider.is_none()
                && f.opening.is_none()
                && f.rejoining.is_none()
                && f.lag_until.is_none()
                && f.log_restore.is_none()
                && f.ledger.entries().iter().all(|entry| entry.until.is_some())
                && self.env.now >= f.last
        })
    }

    pub(super) fn retire_workload_plans(&mut self) {
        if let Some(f) = &mut self.workload_faults {
            for plan in &mut f.plans {
                // With all goals ended, no new target can reach a planned moment.
                if !plan.fired {
                    plan.fired = true;
                    f.ledger.ineligible(plan.kind);
                }
            }
        }
    }

    pub(in crate::composed) fn workload_forge_reply(
        &mut self,
        op: &skein_forge::api::Op,
        reply: crate::composed::engine::Event,
    ) {
        if let Some(f) = &mut self.workload_faults
            && f.bounds.contains(&bounds::FORGE_READ_ADMISSIONS)
            && !f.ledger.entries().iter().any(|entry| entry.kind == bounds::FORGE_READ_ADMISSIONS)
            && matches!(op, skein_forge::api::Op::Read(_))
        {
            f.read_replies.push((self.env.now.saturating_add(f.read_latency), reply));
        } else {
            self.root_events.push_back(reply);
        }
    }

    pub(in crate::composed) fn workload_bounds(&mut self) {
        let Some(f) = &mut self.workload_faults else { return };
        if f.bounds.is_empty() {
            return;
        }
        let mut inspection = skein_world::inspect::Inspection::new("engine", temper_engine_inspect::KINDS);
        temper_engine_inspect::inspect(&self.root, &mut inspection);
        f.ledger.saturation(&f.bounds, &[&inspection], self.env.now).expect("quiet-point bounds");
        if let Some(at) = f.ledger.entries().iter().filter_map(|entry| entry.until).max() {
            f.stopped(at);
        }
    }

    /// Judge runs and gates once, after the ordinary workload's checks.
    pub fn judge_workload_faults(&mut self) -> Result<(), String> {
        self.judge_workload_park()?;
        self.count_provider_tries();
        let f = self.workload_faults.as_mut().expect("faulted workload");
        super::super::accounts::judge_account_runs(&mut f.ledger, &mut f.runs)?;
        let findings = temper_blame::judge(&mut f.ledger, &f.reactions, f.seed);
        if !findings.is_empty() {
            return Err(findings.join("\n"));
        }
        if let Some(entry) = f.ledger.entries().iter().find(|entry| entry.until.is_none()) {
            return Err(format!("{:?} has no last moment", entry.kind));
        }
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
        Ok(())
    }

    pub(in crate::composed) fn workload_fault_tick(&mut self) -> Result<(), String> {
        let Some(mut f) = self.workload_faults.take() else { return Ok(()) };
        let mut waiting = Vec::new();
        for (at, reply) in f.read_replies.drain(..) {
            if at <= self.env.now {
                self.root_events.push_back(reply);
            } else {
                waiting.push((at, reply));
            }
        }
        f.read_replies = waiting;
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
        let acts = self.workload_worker_tick(&mut f).and_then(|()| self.workload_actions_tick(&mut f));
        if let Err(finding) = acts {
            self.workload_faults = Some(f);
            return Err(finding);
        }
        let restart_task = f
            .plans
            .iter()
            .find(|p| p.kind == restarts::ENGINE_CRASH_MID_TURN)
            .and_then(|p| self.workload_names().task(&p.goal));
        if f.due(restarts::ENGINE_CRASH_MID_TURN)
            && self.lanes[0].assignment.as_ref().is_some_and(|a| Some(a.task) == restart_task)
            && !self.lanes[0].process.prepared()
            && self.lanes[0].process.held_query.is_some_and(|query| {
                self.lanes[0].process.agent().prompts().len() == query
                    && self.lanes[0].process.agent().turns().len() + 1 == query
            })
            && self.lanes[0].worker.retained_turns() == 0
            && self.store.idle()
            && self.root_events.is_empty()
            && self.lanes[0].worker_events.is_empty()
        {
            let goal = f.fire(restarts::ENGINE_CRASH_MID_TURN);
            eprintln!(
                "crash at {:?}, iteration {}, assignment {:?}, queries {}",
                self.env.now,
                self.iteration,
                self.lanes[0].assignment.as_ref().map(|a| (a.task, a.attempt)),
                self.lanes[0].process.agent().prompts().len()
            );
            self.cut_link();
            // These replies address the old forge client's tokens, not the
            // reopened client's independently numbered calls.
            f.read_replies.clear();
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
            self.lanes[0].worker_events.push_back(super::super::worker::Event::Ready);
        }
        if f.rejoining.is_some_and(|before| self.absence_admissions() > before) {
            self.lanes[0].process.held_query = None;
            f.rejoining = None;
        }
        self.ends.faults_stop(f.last);
        self.workload_faults = Some(f);
        Ok(())
    }
}
