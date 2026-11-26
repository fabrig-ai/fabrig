//! The waits of a change (temper/docs/design/domain/forge/waits.md, 4).
use crate::domain::{first_in_queue, read_session};
use crate::{ChangeRow, Domain, Limits, room};
use jig_domain_core_vocabulary::waits::{AtBound, Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject};
use skein_lib::{Duration, Env, List, Wall};
use temper_client_protocol_wire::waits as code;
use temper_engine_domain_forge_change::{self as change, State};

pub(super) const THINGS: u32 = 4;

#[derive(Debug, Hash)]
pub(super) struct Observed {
    pub(super) dependencies: List<Thing>,
    pub(super) agent_live: bool,
    pub(super) policy: Option<PolicyWait>,
}

impl Observed {
    pub(super) fn empty() -> Self {
        Self { dependencies: List::with_capacity(0), agent_live: false, policy: None }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.dependencies.is_empty() && !self.agent_live && self.policy.is_none()
    }
}

/// Where a completed step's wait differs from the bound normally inferred from its state.
#[derive(Clone, Copy, Debug, Hash)]
pub(super) enum PolicyWait {
    Quiet { phase: u32, since: Wall },
    PlainChecking { since: Wall },
}

pub(super) fn observe_policy(row: &ChangeRow, decision: change::Decision, execution: bool) -> Option<PolicyWait> {
    if let change::Decision::Wait { until } = decision {
        let since = row.change.since;
        if until.is_none() {
            return Some(PolicyWait::Quiet { phase: step(&row.change.state)?, since });
        }
        if !execution && let State::Checking { .. } = row.change.state {
            return Some(PolicyWait::PlainChecking { since });
        }
    }
    None
}

pub(super) fn waits(d: &Domain, env: &Env<Limits>, task: u64) -> List<Reported> {
    let mut waits = List::with_capacity(1);
    if let Some(row) = d.changes.get(&task)
        && let Some(wait) = waited(d, env, row)
    {
        waits.push(wait).expect("one change wait");
    }
    waits
}

fn waited(d: &Domain, env: &Env<Limits>, row: &ChangeRow) -> Option<Reported> {
    let phase = step(&row.change.state)?;
    let observed = d.wait_reports.changes.get(&row.task);
    if delegate_live(row, observed) || d.change_access_lost(row) {
        return None;
    }
    // Release cleanup keeps its bound in the top; it is not a change wait.
    if row.ending.is_some() || d.releases.contains_key(&row.task) {
        return None;
    }
    if let Some(wait) = room::waited(d, env, row.task, phase) {
        return Some(wait);
    }
    if let State::Landed { since, .. } = row.change.state {
        return verification_wait(d, row, phase, since);
    }
    let since = row.change.since;
    if let State::Producing { .. } = row.change.state
        && let Some(facts) = observed
        && let Some(dependency) = facts.dependencies.as_slice().first()
    {
        return Some(record(
            phase,
            facts.dependencies.clone(),
            since,
            ReportedBound::None(Unbounded::Behind(dependency.number)),
        ));
    }
    if let Some(wait) = sign_off(d, row, phase) {
        return Some(wait);
    }
    if d.change_landing_withheld(row) {
        return Some(unbounded(phase, since, None, Unbounded::OperatorHold));
    }
    Some(step_wait(d, env, row, phase, observed))
}

fn delegate_live(row: &ChangeRow, observed: Option<&Observed>) -> bool {
    if let Some(facts) = observed
        && facts.agent_live
    {
        return true;
    }
    let Some(_) = row.delegate else { return false };
    match row.delegate_status {
        change::Status::Unknown | change::Status::Pending => true,
        change::Status::Passed | change::Status::Failed => false,
    }
}

fn sign_off(d: &Domain, row: &ChangeRow, step: u32) -> Option<Reported> {
    let since = row.change.ready_since?;
    row.pull?;
    // An approval only blocks when the change is waiting to land, not while
    // it is still producing, checking or requalifying at a new head.
    if !crate::domain::in_queue(&row.change.state) && step != code::CHANGE_READY && step != code::CHANGE_LANDING {
        return None;
    }
    let short = d.short_approvals(row);
    let first = short.first()?;
    let mut things = List::with_capacity(THINGS);
    things.push(thing(code::THING_RULE, u64::from(first.rule))).expect("one rule");
    for approval in &short {
        if approval.scope == first.scope && approval.rule == first.rule {
            let role = thing(code::THING_ROLE, u64::from(approval.role));
            if !things.as_slice().contains(&role) {
                things.push(role).expect("one rule and at most three roles");
                if things.room() == 0 {
                    break;
                }
            }
        }
    }
    Some(Reported {
        kind: WaitKind::SignOff,
        object: WaitObject::Procedure { step, things },
        since,
        bound: ReportedBound::Figure(AtBound::Overdue),
    })
}

fn verification_wait(d: &Domain, row: &ChangeRow, step: u32, since: Wall) -> Option<Reported> {
    let (bound, wait) = d.change_verification_settings(row);
    if wait == change::Wait::Landed {
        return None;
    }
    match d.change_verification(row.task) {
        change::Verification::Verifying => {
            Some(at(step, step_things(row, None), since, after(since, bound), AtBound::GiveUp))
        }
        change::Verification::Red => Some(behind(step, since, None)),
        change::Verification::Unfollowed | change::Verification::Cleared => None,
    }
}

fn step_wait(d: &Domain, env: &Env<Limits>, row: &ChangeRow, phase: u32, observed: Option<&Observed>) -> Reported {
    let since = row.change.since;
    let clock = execution_clock(row);
    if let Some((start, end)) = read_session::counting(d, row.task) {
        return at(phase, step_things(row, clock), start, end, AtBound::Hold);
    }
    let policy = match observed {
        Some(facts) => facts.policy,
        None => None,
    };
    let (quiet, plain) = match policy {
        Some(PolicyWait::Quiet { phase: at, since: start }) => (at == phase && start == since, false),
        Some(PolicyWait::PlainChecking { since: start }) => (false, start == since),
        None => (false, false),
    };
    if let State::Queued { .. } = row.change.state {
        let first = first_in_queue(d, row, env.wall, &env.limits);
        return behind(phase, since, if first == Some(row.task) { None } else { first });
    }
    if crate::landing::waiting(d, row) {
        if let Some(landing) = d.train_holder(row.repository, &row.base) {
            return behind(phase, since, Some(landing));
        }
        let cohort = d.settling_changes(row.repository, &row.base, THINGS);
        if let Some(first) = cohort.first() {
            let mut things = List::with_capacity(THINGS);
            for task in &cohort {
                things.push(thing(code::THING_TASK, *task)).expect("bounded cohort report");
            }
            return record(phase, things, since, ReportedBound::None(Unbounded::Behind(*first)));
        }
        return behind(phase, since, None);
    }
    if let State::Red { since, .. } = row.change.state {
        return behind(phase, since, None);
    }
    if quiet {
        let blocker = if let Some(task) = row.queue_repair
            && let Some(repair) = d.changes.get(&task)
            && !repair.frozen()
        {
            Some(task)
        } else {
            None
        };
        return behind(phase, since, blocker);
    }
    let limits = &env.limits.change_policy;
    let end = if let Some(clock) = clock
        && !plain
    {
        after(clock.at, clock.remaining(clock.at, limits))
    } else {
        after(since, limits.stall)
    };
    at(phase, step_things(row, clock), since, end, AtBound::Hold)
}

fn execution_clock(row: &ChangeRow) -> Option<change::Clock> {
    if let State::Checking { head } = row.change.state
        && let Some(clock) = row.change.clock
        && clock.head == head
        && clock.since == row.change.since
    {
        return Some(clock);
    }
    None
}

fn step_things(row: &ChangeRow, clock: Option<change::Clock>) -> List<Thing> {
    let mut things = List::with_capacity(3);
    if let Some(number) = row.pending {
        things.push(Thing { what: code::THING_ENTRY, number, owner: Some(row.task) }).expect("one pending entry");
    }
    if let Some(number) = row.pull {
        things.push(thing(code::THING_PULL, number)).expect("one pull request");
    }
    if let Some(clock) = clock
        && clock.run != 0
    {
        things.push(thing(code::THING_RUN, clock.run)).expect("one CI run");
    }
    things
}

pub(super) const fn thing(what: u32, number: u64) -> Thing {
    Thing { what, number, owner: None }
}

fn behind(step: u32, since: Wall, task: Option<u64>) -> Reported {
    let reason = match task {
        Some(task) => Unbounded::Behind(task),
        None => Unbounded::NoDeadline,
    };
    unbounded(step, since, task, reason)
}

fn unbounded(step: u32, since: Wall, task: Option<u64>, reason: Unbounded) -> Reported {
    let mut things = List::with_capacity(1);
    if let Some(task) = task {
        things.push(thing(code::THING_TASK, task)).expect("one blocker");
    }
    record(step, things, since, ReportedBound::None(reason))
}

fn after(since: Wall, duration: Duration) -> Wall {
    Wall::from_nanos(since.as_nanos().saturating_add(duration.as_nanos()))
}

fn at(step: u32, things: List<Thing>, since: Wall, at: Wall, then: AtBound) -> Reported {
    record(step, things, since, ReportedBound::At { at, then })
}

fn record(step: u32, things: List<Thing>, since: Wall, bound: ReportedBound) -> Reported {
    Reported { kind: WaitKind::Step, object: WaitObject::Procedure { step, things }, since, bound }
}

fn step(state: &State) -> Option<u32> {
    let code = match state {
        State::Taking { .. } => code::CHANGE_TAKING,
        State::Producing { .. } => code::CHANGE_PRODUCING,
        State::Opening { .. } => code::CHANGE_OPENING,
        State::Recreating { .. } => code::CHANGE_RECREATING,
        State::Reopening { .. } => code::CHANGE_REOPENING,
        State::Checking { .. } => code::CHANGE_CHECKING,
        State::Gating { .. } => code::CHANGE_GATING,
        State::Ready { .. } => code::CHANGE_READY,
        State::Preparing { .. } => code::CHANGE_PREPARING,
        State::CancellingGates { .. } => code::CHANGE_CANCELLING_GATES,
        State::Queued { .. } => code::CHANGE_QUEUED,
        State::First { .. } => code::CHANGE_FIRST,
        State::Updating { .. } => code::CHANGE_UPDATING,
        State::WaitingProducer { .. } => code::CHANGE_WAITING_PRODUCER,
        State::Resolving { .. } => code::CHANGE_RESOLVING,
        State::Repairing { .. } => code::CHANGE_REPAIRING,
        State::Landing { .. } => code::CHANGE_LANDING,
        State::Landed { .. } => code::CHANGE_LANDED,
        State::Red { .. } => code::CHANGE_RED,
        State::Cleared { .. } => code::CHANGE_CLEARED,
        State::Uncleared { .. } => code::CHANGE_UNCLEARED,
        State::Judging { .. } => code::CHANGE_JUDGING,
        State::Held { .. } | State::NoChange { .. } | State::Ended => return None,
    };
    Some(code)
}
