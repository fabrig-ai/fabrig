//! Mixed finite faults over the composed draw (temper/docs/design/testing/workloads.md, 8.2).
use super::World;
use skein_lib::Time;
use skein_world::{
    faults::{Counter, FaultLedger, Injection, Kind, Struck},
    inspect::{Inspection, Release},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use temper_engine_domain_world::acts::Watch;
use temper_faults::{bounds, forge as forge_faults, parks, people, restarts, stale};
use temper_worker_inspect::faults::{DISK_FULL, FETCH_CUT};
use temper_workload::parties::{Act, Action};

mod actions;
mod blame;
mod branch_holds;
mod cells;
mod draw;
pub use draw::FaultDraw;
mod forge;
mod runs;
#[cfg(test)]
mod tests;
mod worker;

/// The composed mixed sweep's kinds at the gate.
// The workload's repairs carry gate evidence, not a PullFiles read: the
// bounds family's next reachable kind is the failed job's bounded log.
// Its dependent goal starts after the predecessor landed, so it has no
// early checkout with a squashed parent: move the base during landing.
// COMPOSED has too few concurrent writes to fill the stock outbox.
// Its read-admission table is bounded to two in this faulted world.
// The worker's fake checkout supplies a byte capacity for DISK_FULL.
pub const GATE_KINDS: &[Kind] = &[
    forge_faults::TIMED_OUT,
    forge_faults::HEAD_BEHIND,
    forge_faults::JOB_LOG,
    forge_faults::CLOSED_BY_ANOTHER,
    temper_blame::LOST,
    stale::MOVED_BASE,
    skein_llm_kit::faults::RATE_WINDOW,
    restarts::ENGINE_CRASH_MID_TURN,
    DISK_FULL,
    people::HOLDS_A_MEMBER,
    people::RELEASES_A_MEMBER,
    parks::DELIVERED_THEN_PARKED,
    bounds::FORGE_READ_ADMISSIONS,
    cells::JOB_RUNNING,
    cells::CELL,
];

/// Additional kinds configured in a campaign or replay, in family order.
pub const CAMPAIGN_KINDS: &[Kind] = &[
    // busy
    forge_faults::UNAVAILABLE,
    forge_faults::LIMITED,
    // lagging
    forge_faults::BASE_BEHIND,
    forge_faults::MERGE_LATE,
    // bounds
    // refusing
    forge_faults::CLOSE_REFUSED,
    forge_faults::MERGED_BY_ANOTHER,
    // ci
    temper_blame::WRONG_TREE,
    temper_blame::SET_UP,
    temper_blame::SILENT,
    temper_blame::MISSING_REPORT,
    // stale
    stale::REOPENED,
    // provider
    skein_llm_kit::faults::SPENT_CREDITS,
    skein_llm_kit::faults::EXPIRED_SIGN_IN,
    // worker
    FETCH_CUT,
    // people and parties
    // runs that park, resume and move
    // the engine's own bounds
    bounds::BRANCH_HOLDS,
    bounds::LANDING_ROWS,
    // restarts
    restarts::ENGINE_RESTART_MID_TURN,
];

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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
    unavailable_until: Option<Time>,
    limited_until: Option<Time>,
    lag_until: Option<Time>,
    base_behind_configured: bool,
    base_lag: Option<forge::BaseLag>,
    merge_lag: Option<forge::MergeLag>,
    log_restore: Option<(u64, u32, Box<[u8]>, Injection)>,
    closed: Option<(u64, Injection, Time)>,
    close_calls: VecDeque<forge::CloseCall>,
    close_refusal: Option<forge::CloseRefusal>,
    close_refusal_exclusion: Option<&'static str>,
    other_merge_exclusion: Option<&'static str>,
    close_permission_restored: Option<u64>,
    restored: Option<u64>,
    stale_release: Option<u64>,
    silent_workflow: Option<Box<[u8]>>,
    silent_run: Option<(u64, Injection)>,
    silent_timeout: Option<u64>,
    settling: Option<String>,
    opening: Option<Injection>,
    rejoining: Option<usize>,
    provider: Option<(u32, Time)>,
    account_strikes: Vec<runs::AccountStrike>,
    runs: smith_faults::Runs,
    observed_runs: BTreeMap<(u64, u64), runs::ObservedRun>,
    provider_failures: Vec<((u64, u64), Injection)>,
    reactions: Vec<temper_blame::Observed>,
    retries: BTreeSet<(u64, u64, u32)>,
    repairs: BTreeSet<u64>,
    holds: BTreeSet<String>,
    judged_red_trees: BTreeSet<u64>,
    missing_report: Option<blame::MissingReport>,
    bounds: Vec<Kind>,
    branch_holds: Option<BTreeMap<u64, branch_holds::Load>>,
    actions: Vec<Action>,
    release_configured: bool,
    release_from: Option<(u64, u64)>,
    pub(super) released_member: Option<(u64, Time)>,
    watch: Watch,
    disk: worker::Disk,
    fetch_cut: Option<worker::FetchCut>,
    cell: Option<cells::Driver>,
    park: Option<(String, u64, u64)>,
    parked: bool,
    read_latency: skein_lib::Duration,
    read_replies: Vec<(Time, crate::composed::engine::Event)>,
}

impl Faults {
    pub(super) fn new(seed: u64, draw: FaultDraw) -> Self {
        let FaultDraw {
            plans,
            cell,
            actions,
            ineligible,
            bounds,
            disk,
            read_latency,
            release_configured,
            base_behind_configured,
            ..
        } = draw;
        let missing_report = plans.iter().any(|plan| plan.kind == temper_blame::MISSING_REPORT);
        let mut ledger = FaultLedger::new();
        for kind in ineligible {
            ledger.ineligible(kind);
        }
        let branch_holds = bounds.contains(&bounds::BRANCH_HOLDS).then(BTreeMap::new);
        Self {
            merge_lag: plans.iter().any(|plan| plan.kind == forge_faults::MERGE_LATE).then(forge::MergeLag::default),
            bounds,
            branch_holds,
            release_configured,
            watch: Watch::new(actions.iter().filter(|action| action.act == Act::HoldMember).cloned().collect()),
            release_from: None,
            released_member: None,
            actions,
            disk,
            fetch_cut: None,
            cell: cell.map(cells::Driver::new),
            park: None,
            parked: false,
            read_latency,
            read_replies: Vec::new(),
            seed,
            plans,
            steps: BTreeMap::new(),
            ledger,
            last: Time::ZERO,
            unavailable_until: None,
            limited_until: None,
            lag_until: None,
            base_behind_configured,
            base_lag: None,
            log_restore: None,
            closed: None,
            close_calls: VecDeque::new(),
            close_refusal: None,
            close_refusal_exclusion: None,
            other_merge_exclusion: None,
            close_permission_restored: None,
            restored: None,
            stale_release: None,
            silent_workflow: None,
            silent_run: None,
            silent_timeout: None,
            settling: None,
            opening: None,
            rejoining: None,
            provider: None,
            account_strikes: Vec::new(),
            runs: smith_faults::Runs::new(),
            observed_runs: BTreeMap::new(),
            provider_failures: Vec::new(),
            reactions: Vec::new(),
            retries: BTreeSet::new(),
            repairs: BTreeSet::new(),
            holds: BTreeSet::new(),
            judged_red_trees: BTreeSet::new(),
            missing_report: missing_report.then_some(blame::MissingReport::Waiting),
        }
    }

    pub(super) fn needs_small_steps(&self) -> bool {
        self.provider.is_some() || self.settling.is_some() || self.opening.is_some() || self.rejoining.is_some()
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

    /// Keep the shared exclusion; the close injector reads its condition when it would strike.
    pub fn exclude_workload_close(&mut self, sweep: &skein_world::sweep::Sweep) {
        self.workload_faults.as_mut().expect("faulted workload").close_refusal_exclusion = sweep
            .leaves_out(forge_faults::CLOSE_REFUSED, temper_engine_forge_world::open::CLOSE_REFUSED_WHEN)
            .map(|exclusion| exclusion.scenario);
    }

    /// Keep the shared exclusion for outside merges before a change's record.
    pub fn exclude_workload_other_merge(&mut self, sweep: &skein_world::sweep::Sweep) {
        let exclusion = temper_engine_forge_world::open::MERGED_BY_ANOTHER_UNRECORDED;
        self.workload_faults.as_mut().expect("faulted workload").other_merge_exclusion =
            sweep.leaves_out(exclusion.fault, exclusion.when).map(|exclusion| exclusion.scenario);
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
                && f.fetch_cut.is_none()
                && (f.park.is_none() || f.parked)
                && f.closed.is_none()
                && f.close_calls.is_empty()
                && f.close_refusal.is_none()
                && f.provider.is_none()
                && f.account_strikes.is_empty()
                && f.settling.is_none()
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
            if let Some(cell) = &mut f.cell {
                cell.retire(&mut f.ledger);
            }
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
        let Some(f) = &self.workload_faults else { return };
        if f.bounds.is_empty() {
            return;
        }
        let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
        temper_engine_inspect::inspect(&self.root, &mut inspection);
        let landing_rows = bounds::LANDING_ROWS;
        let settled = f.bounds.contains(&landing_rows)
            && self.workload_goals_ended()
            && inspection.tables().iter().any(|table| {
                table.name == landing_rows.name && table.entries.iter().all(|entry| entry.release == Release::Kept)
            });
        let f = self.workload_faults.as_mut().expect("faulted workload");
        f.ledger.saturation(&f.bounds, &[&inspection], self.env.now).expect("quiet-point bounds");
        if settled {
            // Terminal rows keep their places until another admission. Retire
            // only this observation once those places can all be reclaimed.
            f.ledger.saturation(&[landing_rows], &[], self.env.now).expect("landing rows settled at a quiet point");
            f.bounds.retain(|&kind| kind != landing_rows);
        }
        if let Some(at) = f.ledger.entries().iter().filter_map(|entry| entry.until).max() {
            f.stopped(at);
        }
    }

    /// Judge runs and gates once, after the ordinary workload's checks.
    pub fn judge_workload_faults(&mut self) -> Result<(), String> {
        self.judge_workload_park()?;
        self.judge_workload_cell()?;
        self.count_provider_tries();
        let f = self.workload_faults.as_mut().expect("faulted workload");
        super::super::accounts::judge_account_runs(&mut f.ledger, &mut f.runs)?;
        let findings = temper_blame::judge(&mut f.ledger, &f.reactions, f.seed);
        if !findings.is_empty() {
            return Err(findings.join("\n"));
        }
        let mut setup_faults = f.ledger.entries().iter().filter(|entry| entry.kind == temper_blame::SET_UP);
        if let Some(entry) = setup_faults.next()
            && (setup_faults.next().is_some()
                || entry.count(Counter::GateRetries) != 1
                || entry.count(Counter::Repairs) != 0)
        {
            return Err(format!("seed {}: ci/set_up must fall once and be retried without repair", f.seed));
        }
        if let Some(entry) = f.ledger.entries().iter().find(|entry| entry.kind == temper_blame::SILENT) {
            let held_run = f.silent_timeout.map(|run| run.to_string());
            if held_run.is_none() || held_run != entry.struck.gate_run || entry.count(Counter::Repairs) != 0 {
                return Err(format!(
                    "seed {}: ci/silent on run {:?} must explain a runner timeout hold without repair; saw hold on {:?}, {} repairs",
                    f.seed,
                    entry.struck.gate_run,
                    held_run,
                    entry.count(Counter::Repairs)
                ));
            }
        }
        if let Some(entry) = f.ledger.entries().iter().find(|entry| entry.until.is_none()) {
            return Err(format!("{:?} has no last moment", entry.kind));
        }
        eprintln!("seed {}: faults stopped by {:?}", f.seed, f.last);
        for entry in f.ledger.entries() {
            if entry.kind == temper_blame::MISSING_REPORT && entry.count(skein_world::faults::Counter::GateRetries) != 1
            {
                return Err(format!("the missing report on run {:?} was not retried once", entry.struck.gate_run));
            }
            eprintln!(
                "{:?}: {:?}..{:?}; {:?}; gate retries {}, repairs {}, task tries {}",
                entry.kind,
                entry.from,
                entry.until,
                entry.struck,
                entry.count(Counter::GateRetries),
                entry.count(Counter::Repairs),
                entry.count(Counter::TaskTries)
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
        let acts = self
            .workload_cell_tick(&mut f)
            .and_then(|()| self.workload_worker_tick(&mut f))
            .and_then(|()| self.workload_actions_tick(&mut f));
        if let Err(finding) = acts {
            self.workload_faults = Some(f);
            return Err(finding);
        }
        self.workload_restart_tick(&mut f);
        self.ends.faults_stop(f.last);
        self.workload_faults = Some(f);
        Ok(())
    }

    // Arm the first outstanding point for this goal; after rejoining, the
    // same activation may continue to a later point.
    fn workload_restart_query(&self, f: &Faults, task: u64, past: usize) -> Option<usize> {
        f.plans
            .iter()
            .filter(|p| {
                restarts::MID_TURN.contains(&p.kind)
                    && !p.fired
                    && p.query > past
                    && self.workload_names().task(&p.goal) == Some(task)
            })
            .map(|p| p.query)
            .min()
    }

    fn workload_restart_tick(&mut self, f: &mut Faults) {
        if f.settling.is_none() && f.opening.is_none() && f.rejoining.is_none() {
            self.workload_restart_at_hold(f);
        }
        if self.absence.drained
            && let Some(goal) = f.settling.take()
        {
            self.reopen_after_workload_stop(f, restarts::ENGINE_RESTART_MID_TURN, goal);
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
            self.lanes[0].process.held_query = self.lanes[0].process.held_query.and_then(|query| {
                let task = self.lanes[0].assignment.as_ref()?.task;
                self.workload_restart_query(f, task, query)
            });
            f.rejoining = None;
        }
    }

    fn workload_restart_at_hold(&mut self, f: &mut Faults) {
        let lane = &self.lanes[0];
        let (Some(assignment), Some(query)) = (&lane.assignment, lane.process.held_query) else { return };
        if !lane.process.model_held()
            || lane.worker.retained_turns() != 0
            || !self.store.idle()
            || !self.root_events.is_empty()
            || !f.close_calls.is_empty()
            || !lane.worker_events.is_empty()
        {
            return;
        }
        let Some(kind) = f.plans.iter().find_map(|p| {
            (restarts::MID_TURN.contains(&p.kind)
                && !p.fired
                && p.query == query
                && self.workload_names().task(&p.goal) == Some(assignment.task)
                && f.due(p.kind))
            .then_some(p.kind)
        }) else {
            return;
        };
        let goal = f.fire(kind);
        // Once one restart strikes this held call, a second fault cannot
        // use the same call, even if a new activation repeats the script.
        for plan in &mut f.plans {
            if !plan.fired && restarts::MID_TURN.contains(&plan.kind) && plan.goal == goal && plan.query == query {
                plan.fired = true;
                f.ledger.ineligible(plan.kind);
            }
        }
        if kind == restarts::ENGINE_RESTART_MID_TURN {
            f.settling = Some(goal);
            self.begin_engine_settle();
        } else {
            eprintln!(
                "crash at {:?}, iteration {}, assignment {:?}, queries {}",
                self.env.now,
                self.iteration,
                self.lanes[0].assignment.as_ref().map(|a| (a.task, a.attempt)),
                query
            );
            self.reopen_after_workload_stop(f, kind, goal);
        }
    }

    fn reopen_after_workload_stop(&mut self, f: &mut Faults, kind: Kind, goal: String) {
        self.cut_link();
        // These replies address the old forge client's tokens, not the
        // reopened client's independently numbered calls.
        f.read_replies.clear();
        f.close_calls.clear();
        self.stop_engine();
        let injection = f.ledger.inject(kind, self.env.now, Struck { request: Some(goal), ..Struck::default() });
        // The composed settings and all durable rows survive the root, not its queues.
        let config = temper_engine_domain_world::oracle::config::copy(&self.restart_config);
        self.reopen_with_config(config);
        f.opening = Some(injection);
    }
}
