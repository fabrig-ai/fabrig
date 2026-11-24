//! Taking the outcomes of a task's waits (jig/docs/design/domain/engine/contracts.md, 2).

mod words;

use super::{activity, own};
use crate::{AtBound, Bound, Core, Figures, Limits, Request, Wait, WaitKind, WaitObject};
use alloc::boxed::Box;
use jig_domain_people::EntryKind;
use jig_domain_tasks::{self as tasks, Active, Hold, HoldSource, Phase, TaskRecord};
use skein_lib::{Deadlines, Duration, Env, Map, Queue, Set, Time, Wall};

#[derive(Hash, Debug)]
pub(crate) struct Alarms {
    timers: Deadlines<u64>,
    armed: Map<u64, Armed>,
    dirty: Set<u64>,
    changing_runs: Set<u64>,
    run_waits_changed: bool,
    parking: Map<u64, Park>,
}

#[derive(Hash, Debug)]
struct Armed {
    wall: Wall,
    due: Time,
}

#[derive(Hash, Debug)]
struct Park {
    attempt: u64,
    why: Hold,
    source: Box<HoldSource>,
}

impl Alarms {
    pub(crate) fn new(tasks: u32) -> Self {
        Self {
            timers: Deadlines::with_capacity(tasks),
            armed: Map::with_capacity(tasks),
            dirty: Set::with_capacity(tasks),
            changing_runs: Set::with_capacity(tasks),
            run_waits_changed: false,
            parking: Map::with_capacity(tasks),
        }
    }

    pub(crate) fn next(&self) -> Option<Time> {
        self.timers.next()
    }

    pub(crate) fn at(&self, task: u64) -> Option<Time> {
        Some(self.armed.get(&task)?.due)
    }

    pub(crate) fn changed(&mut self, task: u64) {
        let _new = self.dirty.insert(task).expect("one bound update per live task");
    }

    pub(crate) fn changed_run_waits(&mut self) {
        self.run_waits_changed = true;
    }

    pub(crate) fn forget(&mut self, task: u64) {
        self.cancel(task);
        self.dirty.remove(&task);
        self.changing_runs.remove(&task);
        drop(self.parking.remove(&task));
    }

    /// A valid cancellation terminal keeps its admitted expense and saved
    /// resources, but does not spend a try or close the activation's session.
    pub(crate) fn park_terminal(&self, event: &mut tasks::Event) -> Option<u64> {
        let tasks::Event::Activation { task, attempt, end, cause, .. } = event else { return None };
        let park = self.parking.get(task)?;
        if *attempt != park.attempt {
            return None;
        }
        match end {
            tasks::End::Parked
            | tasks::End::Failed(tasks::Class::Agent | tasks::Class::Lost)
            | tasks::End::Refused
            | tasks::End::Recheck => {}
            tasks::End::Finished { .. }
            | tasks::End::FinishedWithFollowups { .. }
            | tasks::End::Exceeded(_)
            | tasks::End::Failed(_)
            | tasks::End::FailedWithReason { .. }
            | tasks::End::RefusedWithReason { .. }
            | tasks::End::Suspended(_) => return None,
        }
        let cumulative = match *cause {
            tasks::Cause::Priced { cumulative } => Some(cumulative),
            tasks::Cause::Unpriced => None,
            tasks::Cause::Parked { cumulative } => cumulative,
        };
        *cause = tasks::Cause::Parked { cumulative };
        Some(*task)
    }

    pub(crate) fn parked(&mut self, tasks: &tasks::Domain, task: u64) -> Option<tasks::Event> {
        let Park { why, source, .. } = self.parking.remove(&task)?;
        // A different hold or an ending that arrived during cancellation wins.
        if let Phase::Active(_) = tasks.task(task)?.phase {
            Some(tasks::Event::HoldFrom { task, why, source })
        } else {
            None
        }
    }

    fn cancel(&mut self, task: u64) {
        self.timers.cancel(task);
        let _previous = self.armed.remove(&task);
    }

    fn arm(&mut self, task: u64, at: Option<Wall>, env: &Env<Limits>) {
        let Some(at) = at else { return self.cancel(task) };
        if let Some(armed) = self.armed.get(&task)
            && armed.wall == at
        {
            return;
        }
        let due = env.now.saturating_add(Duration::from_nanos(at.as_nanos().saturating_sub(env.wall.as_nanos())));
        let _old = self.armed.insert(task, Armed { wall: at, due }).expect("one wall bound per live task");
        self.timers.arm(task, due).expect("one bound alarm per live task");
    }
}

pub(crate) fn restore(core: &mut Core, env: &Env<Limits>) {
    for task in core.tasks.view_tasks() {
        core.wait_alarms.changed(task.number);
    }
    refresh(core, env);
}

/// Recompute after the decision's sibling routes have updated their facts.
pub(crate) fn refresh(core: &mut Core, env: &Env<Limits>) {
    if !core.start_opened() {
        return;
    }
    if core::mem::replace(&mut core.wait_alarms.run_waits_changed, false) {
        for task in &core.wait_alarms.changing_runs {
            let _new = core.wait_alarms.dirty.insert(*task).expect("one bound update per live task");
        }
    }
    for _ in 0..core.wait_alarms.dirty.len() {
        let task = *core.wait_alarms.dirty.first().expect("one update per dirty task");
        core.wait_alarms.dirty.remove(&task);
        let Some(row) = core.tasks.task(task) else {
            core.wait_alarms.forget(task);
            continue;
        };
        if let Some(park) = core.wait_alarms.parking.get(&task)
            && tasks::run_attempt(&row.phase) != Some(park.attempt)
        {
            drop(core.wait_alarms.parking.remove(&task));
        }
        // These waits can move without a task save: a placement can lose its
        // observation, or a host, slot, account or credential can change.
        let observed = activity(core, row);
        let changing = match row.phase {
            Phase::Active(Active::Due) => match row.executor {
                tasks::Executor::Agent { .. } => true,
                tasks::Executor::Job { .. } | tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => false,
            },
            Phase::Active(Active::Claimed { .. } | Active::Running { .. }) => observed.1.is_some(),
            Phase::Active(Active::Idle | Active::Preparing | Active::BackingOff { .. })
            | Phase::Waiting
            | Phase::Held { .. }
            | Phase::Closing(_)
            | Phase::Ended(_) => false,
        };
        if changing {
            let _new = core.wait_alarms.changing_runs.insert(task).expect("one run wait per live task");
        } else {
            core.wait_alarms.changing_runs.remove(&task);
        }
        let at = match agenda(core, row, observed) {
            Some(agenda) => Some(agenda.first.at),
            None => None,
        };
        core.wait_alarms.arm(task, at, env);
    }
}

pub(crate) fn fire(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if !core.start_opened() {
        return out;
    }
    let Some(task) = core.wait_alarms.timers.expire(env.now) else { return out };
    let _previous = core.wait_alarms.armed.remove(&task);
    let Some(row) = core.tasks.task(task) else { return out };
    let Some(agenda) = agenda(core, row, activity(core, row)) else { return out };
    if agenda.first.at > env.wall {
        // A correction of the wall, or a wait which no longer has this bound.
        core.wait_alarms.arm(task, Some(agenda.first.at), env);
        return out;
    }
    let why = agenda.first.why;
    let source = Box::new(HoldSource::Wait { words: words::source(core.wait_limits.waits, &agenda) });
    if let Some(attempt) = agenda.attempt {
        assert!(core.wait_alarms.parking.insert(task, Park { attempt, why, source }).is_ok(), "one park per task");
        out.push(Request::Held(Box::new(crate::Held::StopRun { task, attempt })));
    } else {
        crate::routing::children::append_tasks(core, env, tasks::Event::HoldFrom { task, why, source }, &mut out);
    }
    out
}

#[derive(Clone, Copy)]
struct Outcome {
    at: Wall,
    why: Hold,
}

struct Agenda {
    first: Outcome,
    waits: own::Records,
    deadline: Option<Wall>,
    attempt: Option<u64>,
}

fn agenda(core: &Core, row: &TaskRecord, observed: (bool, Option<Wall>)) -> Option<Agenda> {
    if let Phase::Held { .. } | Phase::Closing(_) | Phase::Ended(_) = row.phase {
        return None;
    }
    if observed.0 || core.wait_alarms.parking.contains_key(&row.number) {
        return None;
    }
    let deadline = tasks::deadline(row, core.run_wall);
    if deadline.is_none()
        && let Phase::Waiting | Phase::Active(Active::Preparing) = row.phase
    {
        return None;
    }
    // An open suspension advances the cap as time passes. Its own window
    // rechecks placement; leaving suspension fixes the deadline again.
    let (deadline, moving) = match deadline {
        Some(at) if row.suspension.is_some() && at > core.run_wall => (None, Some(at)),
        Some(_) | None => (deadline, None),
    };
    let waits = core.own_waits(row, observed);
    let mut first = None;
    for wait in waits.values() {
        if let Some(next) = dispatch(wait, core.wait_limits.waits, moving) {
            consider(&mut first, next);
        }
        // The task's deadline applies even if this wait has no clock of its
        // own, or another owner takes the outcome of that clock.
        if let Some(at) = deadline {
            consider(&mut first, Outcome { at: at.max(wait.since), why: Hold::Deadline });
        }
    }
    // The actions coalesce into one hold bearing the words of all the waits.
    Some(Agenda { first: first?, waits, deadline, attempt: tasks::run_attempt(&row.phase) })
}

fn consider(first: &mut Option<Outcome>, next: Outcome) {
    if let Some(prior) = *first
        && (prior.at < next.at || prior.at == next.at && (prior.why == Hold::Deadline || next.why != Hold::Deadline))
    {
        return;
    }
    *first = Some(next);
}

/// What the alarm takes at a record's bound. Leave the question arms for
/// pass-up; resource queues, stalls and procedures' clocks have other owners.
fn dispatch(wait: &Wait, figures: Figures, moving: Option<Wall>) -> Option<Outcome> {
    let at = match (wait.kind, &wait.object, wait.bound) {
        (WaitKind::News | WaitKind::Host | WaitKind::Account, _, Bound::At { at, then: AtBound::Hold }) => at,
        (WaitKind::Question, _, _)
        | (WaitKind::Party, WaitObject::Entry { kind: EntryKind::Question { .. }, .. }, _) => {
            return None;
        }
        (
            WaitKind::Dependency
            | WaitKind::Delegate
            | WaitKind::News
            | WaitKind::Host
            | WaitKind::Account
            | WaitKind::Resource
            | WaitKind::Party
            | WaitKind::Step
            | WaitKind::Room
            | WaitKind::Turn
            | WaitKind::SignOff,
            _,
            _,
        ) => return None,
    };
    let at = if let Some(cap) = moving
        && at == cap.max(wait.since)
    {
        let (figure, from) = clock(wait, figures).expect("figure wait has a clock");
        own::figure_at(from, figure)
    } else {
        at
    };
    Some(Outcome { at, why: Hold::Waited })
}

fn clock(wait: &Wait, figures: Figures) -> Option<(Duration, Wall)> {
    match &wait.object {
        WaitObject::News { .. } => Some((figures.news, wait.since)),
        WaitObject::Run(run) => {
            let (_, figure, reset) = own::run_clock(figures, *run);
            Some((figure, wait.since.max(reset.unwrap_or(wait.since))))
        }
        WaitObject::Entry { .. }
        | WaitObject::Question { .. }
        | WaitObject::Task(_)
        | WaitObject::Resource { .. }
        | WaitObject::Procedure { .. } => None,
    }
}
