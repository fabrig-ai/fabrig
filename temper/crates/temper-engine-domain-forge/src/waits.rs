//! The forge's replacement wait reports (temper/docs/design/domain/forge/waits.md, 2).
use crate::{Domain, Limits, Request, room};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::waits::{ReportRefusal, Reported};
use skein_lib::{Env, List, Map, Queue, Set, Wall};

mod change;
mod closer;
pub(crate) mod landing;

#[derive(Debug, Hash)]
pub(crate) struct State {
    // An absent report is equivalent to an empty report, except at catch-up.
    last: Map<u64, List<Reported>>,
    dirty: Set<u64>,
    changes: Map<u64, change::Observed>,
    holders: Option<room::Holders>,
    catch_up_tasks: Option<List<u64>>,
    catch_up_at: Option<Wall>,
    refusal: Option<(u64, ReportRefusal)>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            last: Map::uncapped(),
            dirty: Set::with_capacity(u32::MAX),
            changes: Map::uncapped(),
            holders: None,
            catch_up_tasks: None,
            catch_up_at: None,
            refusal: None,
        }
    }

    pub(crate) fn changed(&mut self, task: u64) {
        self.dirty.insert(task).expect("one pending report per task");
    }
}

impl Domain {
    /// Supply the live procedures for catch-up, including those that have no connector row yet.
    pub fn prepare_wait_catch_up(&mut self, tasks: List<u64>) {
        self.wait_reports.catch_up_tasks = Some(tasks);
    }

    /// The first refusal is a fault of this connector, not a procedure event.
    #[must_use]
    pub const fn inspection_wait_refusal(&self) -> Option<(u64, ReportRefusal)> {
        self.wait_reports.refusal
    }
}

fn procedures(d: &Domain) -> Set<u64> {
    let mut tasks =
        Set::with_capacity(d.changes.len().saturating_add(d.landing.rows.len()).saturating_add(d.closing.rows.len()));
    for (task, _) in &d.changes {
        tasks.insert(*task).expect("change task");
    }
    for (task, _) in &d.landing.rows {
        tasks.insert(*task).expect("landing task");
    }
    for (task, _) in &d.closing.rows {
        tasks.insert(*task).expect("closer task");
    }
    tasks
}

pub(crate) fn refused(d: &mut Domain, task: u64, why: ReportRefusal) {
    if d.wait_reports.refusal.is_none() {
        d.wait_reports.refusal = Some((task, why));
    }
}

/// Save a changed row first; report from its committed state at the end of the step.
pub(crate) fn changed(d: &mut Domain, task: u64) {
    d.wait_reports.changed(task);
}

/// The root's pending dependencies and live pushing child may outlive the gather.
pub(crate) fn change_standing(
    d: &mut Domain,
    task: u64,
    dependencies: &[temper_engine_domain_forge_change::Pending],
    agent_live: bool,
) {
    if !d.changes.contains_key(&task) {
        return;
    }
    let mut things = List::with_capacity(if dependencies.is_empty() { 0 } else { change::THINGS });
    for dependency in dependencies {
        things
            .push(change::thing(temper_client_protocol_wire::waits::THING_TASK, dependency.task))
            .expect("at most four dependencies named");
        if things.room() == 0 {
            break;
        }
    }
    let mut seen = match d.wait_reports.changes.remove(&task) {
        Some(seen) => seen,
        None => change::Observed::empty(),
    };
    seen.dependencies = things;
    seen.agent_live = agent_live;
    keep_change(d, task, seen);
}

pub(crate) fn change_policy(
    d: &mut Domain,
    row: &crate::ChangeRow,
    decision: temper_engine_domain_forge_change::Decision,
    execution: bool,
) {
    let mut seen = match d.wait_reports.changes.remove(&row.task) {
        Some(seen) => seen,
        None => change::Observed::empty(),
    };
    seen.policy = change::observe_policy(row, decision, execution);
    keep_change(d, row.task, seen);
}

fn keep_change(d: &mut Domain, task: u64, seen: change::Observed) {
    if !seen.is_empty() {
        d.wait_reports.changes.insert(task, seen).expect("observations of bounded changes");
    }
}

pub(crate) fn change_dependencies_met(d: &mut Domain, task: u64) {
    if let Some(mut seen) = d.wait_reports.changes.remove(&task) {
        seen.dependencies = List::with_capacity(0);
        seen.policy = None;
        keep_change(d, task, seen);
    }
    changed(d, task);
}

pub(crate) fn change_agent_ended(d: &mut Domain, task: u64) {
    if let Some(mut seen) = d.wait_reports.changes.remove(&task) {
        seen.agent_live = false;
        seen.policy = None;
        keep_change(d, task, seen);
    }
    changed(d, task);
}

pub(crate) fn change_forge(d: &mut Domain, forge: u16) {
    for (task, row) in &d.changes {
        if row.repository.forge == forge {
            d.wait_reports.changed(*task);
        }
    }
}

pub(crate) fn forget(d: &mut Domain, task: u64) {
    d.wait_reports.last.remove(&task);
    d.wait_reports.dirty.remove(&task);
    d.wait_reports.changes.remove(&task);
    for waiter in [room::Waiter::Change(task), room::Waiter::Landing(task), room::Waiter::Release(task)] {
        d.room.places.remove(&waiter);
    }
}

pub(crate) fn begin_catch_up(d: &mut Domain, started: Wall) {
    d.wait_reports.catch_up_at = Some(started);
}

fn catch_up(d: &mut Domain, env: &Env<Limits>, started: Wall, out: &mut Queue<Request>) {
    if let Some(tasks) = d.wait_reports.catch_up_tasks.take() {
        for task in &tasks {
            room::observe(d, *task, started);
            publish(d, env, *task, true, out);
        }
    } else {
        for task in &procedures(d) {
            if standalone_live(d, *task) {
                room::observe(d, *task, started);
                publish(d, env, *task, true, out);
            }
        }
    }
}

fn standalone_live(d: &Domain, task: u64) -> bool {
    if d.releases.contains_key(&task) {
        return true;
    }
    if let Some(row) = d.changes.get(&task) {
        return row.ending.is_none();
    }
    if let Some(row) = d.landing.rows.get(&task) {
        return match row.landing.phase {
            temper_engine_domain_forge_landing::Phase::Landed
            | temper_engine_domain_forge_landing::Phase::Cancelled => false,
            temper_engine_domain_forge_landing::Phase::Waiting
            | temper_engine_domain_forge_landing::Phase::Preparing
            | temper_engine_domain_forge_landing::Phase::Proving
            | temper_engine_domain_forge_landing::Phase::Proved { .. }
            | temper_engine_domain_forge_landing::Phase::Turn { .. }
            | temper_engine_domain_forge_landing::Phase::Held { .. } => true,
        };
    }
    if let Some(row) = d.closing.rows.get(&task) {
        return !row.done;
    }
    false
}

/// Publish changed room records when a holder settles or its owner ends, without stepping its waiters.
pub(crate) fn flush(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if let Some(started) = d.wait_reports.catch_up_at.take() {
        catch_up(d, env, started, out);
    }
    if !d.room.places.is_empty() {
        let holders = room::holders(d);
        if d.wait_reports.holders.as_ref() != Some(&holders) {
            d.wait_reports.holders = Some(holders);
            for (waiter, _) in &d.room.places {
                let task = match waiter {
                    room::Waiter::Change(task) | room::Waiter::Landing(task) | room::Waiter::Release(task) => *task,
                    room::Waiter::Making(_) => continue,
                };
                d.wait_reports.dirty.insert(task).expect("one pending report per task");
            }
        }
    }
    for _ in 0..d.wait_reports.dirty.len() {
        let task = d.wait_reports.dirty.pop_first().expect("counted pending reports");
        report(d, env, task, out);
    }
    room::refresh_deadline(d, &env.limits);
}

/// Compute the task's replacement report, suppressing an unchanged list.
pub(crate) fn report(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    room::observe(d, task, env.wall);
    publish(d, env, task, false, out);
}

fn publish(d: &mut Domain, env: &Env<Limits>, task: u64, force: bool, out: &mut Queue<Request>) {
    let waits = if d.changes.contains_key(&task) {
        change::waits(d, env, task)
    } else if d.landing.rows.contains_key(&task) {
        landing::waits(d, &env.limits, task)
    } else if d.closing.rows.contains_key(&task) {
        closer::waits(d, env, task)
    } else {
        List::with_capacity(0)
    };
    let same = match d.wait_reports.last.get(&task) {
        Some(last) => last == &waits,
        None => waits.is_empty(),
    };
    if force || !same {
        if waits.is_empty() {
            d.wait_reports.last.remove(&task);
        } else {
            d.wait_reports.last.insert(task, waits.clone()).expect("one report per task");
        }
        out.push(Request::Waits { task, waits });
    }
}

/// The bound's figure, not the amount the clock happened to overshoot it.
pub(crate) fn stalled_words(step: &[u8], figure: skein_lib::Duration, what: &[u8], last: Option<&[u8]>) -> Box<[u8]> {
    let minutes =
        skein_lib::Decimal::of(figure.as_nanos().checked_div(60_000_000_000).expect("nanoseconds per minute"));
    let (separator, answer) = match last {
        Some(answer) => (b"; the last answer: ".as_slice(), answer),
        None => (b"".as_slice(), b"".as_slice()),
    };
    skein_lib::bytes::concat(&[
        b"no progress at ",
        step,
        b" for ",
        minutes.as_bytes(),
        b" minutes: waiting for ",
        what,
        separator,
        answer,
    ])
}

pub(crate) fn step_bound(since: Wall, figure: skein_lib::Duration) -> Wall {
    Wall::from_nanos(since.as_nanos().saturating_add(figure.as_nanos()))
}

pub(crate) fn read_answer(last: Option<skein_forge::api::Error>) -> Box<[u8]> {
    match last {
        Some(error) => crate::effect_words::error_words(error),
        None => Box::from(b"no answer".as_slice()),
    }
}
