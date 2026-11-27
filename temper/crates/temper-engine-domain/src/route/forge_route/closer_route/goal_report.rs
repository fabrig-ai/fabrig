//! The closer's goal-scope report read (temper/docs/design/domain/forge/branches.md, 2).
use crate::boundary::{Read, Work};
use crate::{CoreRange, Decision, Delivery, Domain, Key, Limits, Range, Record, Request, loads};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_notes as notes;
use skein_lib::{Duration, Env, Id, List, Queue, Token, Wall};
use temper_engine_domain_forge_closer as c;

/// An observation can serve the containment and CI reads of one offered step.
/// A dependency admitted during a read invalidates it even if that work ends
/// before the read answers. Dependencies are retained in admission order.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct At {
    pub(super) step: u64,
    pub(super) dependencies: usize,
}

#[derive(Hash, Debug)]
pub(crate) struct Observation {
    at: At,
    delay: Option<Duration>,
    state: Observed,
}

#[derive(Hash, Debug)]
enum Observed {
    Reading(Token),
    Read(Option<Box<[u8]>>),
    Failed,
    Waiting(Wall),
}

#[derive(Hash, Debug)]
pub(crate) struct Flight {
    task: u64,
    scope: notes::Scope,
    phase: Phase,
    newest: Option<Newest>,
}

#[derive(Hash, Debug)]
struct Newest {
    changed: Wall,
    name: u64,
    left: Option<Box<[u8]>>,
}

#[derive(Hash, Debug)]
enum Phase {
    Index { after: Option<u64> },
    Entries { names: List<(u64, u32)>, at: u32, after: Option<u64> },
}

pub(super) fn observed(domain: &mut Domain, env: &Env<Limits>, task: u64, at: At) -> Option<c::ReportRead> {
    let Some(observation) = domain.closer_reports.get_mut(&task) else { return Some(c::ReportRead::Unread) };
    match &observation.state {
        Observed::Reading(_) => None,
        Observed::Read(left) if observation.at == at => Some(c::ReportRead::Read(left.clone())),
        Observed::Read(_) => Some(c::ReportRead::Unread),
        Observed::Failed => {
            let client = &env.limits.forge.client;
            let delay = match observation.delay {
                Some(previous) => Duration::from_nanos(previous.as_nanos().saturating_mul(2)),
                None => client.backoff,
            };
            let delay = delay.min(client.poll_max);
            let until = Wall::from_nanos(env.wall.as_nanos().saturating_add(delay.as_nanos()));
            observation.delay = Some(delay);
            observation.state = Observed::Waiting(until);
            domain.forge.wait_change(&env.with(env.limits.forge), task, Some(until));
            Some(c::ReportRead::Failed)
        }
        Observed::Waiting(until) => {
            // An unrelated wake does not shorten a failed read's wait.
            if env.wall < *until { Some(c::ReportRead::Failed) } else { Some(c::ReportRead::Unread) }
        }
    }
}

pub(super) fn start(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    task: u64,
    scope: notes::Scope,
    at: At,
) {
    let range = index_range(&scope, None);
    let flight = Flight { task, scope, phase: Phase::Index { after: None }, newest: None };
    let delay = match domain.closer_reports.remove(&task) {
        Some(previous) => previous.delay,
        None => None,
    };
    let state = match domain.archive_reads.insert(Some(Read::CloserReport(Box::new(flight)))) {
        Ok(id) => {
            crate::domain::emit(decision, &env.limits, Delivery::Load { waiter: id.token(), range, after: None });
            Observed::Reading(id.token())
        }
        Err(_read) => {
            domain.work.push(Work::CloserReportReady { task });
            Observed::Failed
        }
    };
    domain.closer_reports.insert(task, Observation { at, delay, state }).expect("one observation per live closer");
}

pub(super) fn submitted(domain: &mut Domain, task: u64) {
    if let Some(observation) = domain.closer_reports.get(&task)
        && let Observed::Waiting(_) = observation.state
    {
        return;
    }
    forget(domain, task);
}

pub(crate) fn forget(domain: &mut Domain, task: u64) {
    if let Some(observation) = domain.closer_reports.remove(&task)
        && let Observed::Reading(waiter) = observation.state
    {
        crate::archive::retire_read(domain, waiter);
    }
}

pub(crate) fn failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::CloserReport(flight)) = crate::archive::take_read(domain, waiter) else { return };
    finish(domain, waiter, flight.task, Err(()));
}

pub(crate) fn loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
    out: &mut Queue<Request>,
) {
    if cut.is_some() {
        failed(domain, waiter);
        return;
    }
    let Some(Read::CloserReport(mut flight)) = crate::archive::take_read(domain, waiter) else { return };
    let result = match &flight.phase {
        Phase::Index { after } => {
            let after = *after;
            index(&mut flight, rows, next, after, &domain.limits.notes)
        }
        Phase::Entries { .. } => entry(&mut flight, rows, next, &domain.limits.notes),
    };
    match result {
        Ok(Some(range)) => {
            *domain.archive_reads.get_mut(Id::from_token(waiter)).expect("report read slot") =
                Some(Read::CloserReport(flight));
            crate::restart::request_load(domain, waiter, range, None, out);
        }
        Ok(None) => {
            let left = match flight.newest {
                Some(newest) => newest.left,
                None => None,
            };
            finish(domain, waiter, flight.task, Ok(left));
        }
        Err(()) => finish(domain, waiter, flight.task, Err(())),
    }
}

fn finish(domain: &mut Domain, waiter: Token, task: u64, answer: Result<Option<Box<[u8]>>, ()>) {
    domain.archive_reads.retire(Id::from_token(waiter));
    let Some(observation) = domain.closer_reports.get_mut(&task) else { return };
    let Observed::Reading(owner) = observation.state else { return };
    if owner != waiter {
        return;
    }
    observation.state = match answer {
        Ok(line) => {
            observation.delay = None;
            Observed::Read(line)
        }
        Err(()) => Observed::Failed,
    };
    domain.work.push(Work::CloserReportReady { task });
}

fn index_range(scope: &notes::Scope, after: Option<u64>) -> Range {
    Range::Core(CoreRange::ApplicationNotes(notes::Range::Lines { scope: scope.clone(), after }))
}

fn index(
    flight: &mut Flight,
    rows: Box<[Record]>,
    next: Option<Key>,
    after: Option<u64>,
    limits: &notes::Limits,
) -> Result<Option<Range>, ()> {
    let mut last = after;
    let mut names = List::with_capacity(u32::try_from(rows.len()).expect("bounded note page"));
    for row in rows {
        let Record::Core(core::Record::Notes(notes::Record::Line(line))) = row else { return Err(()) };
        if line.scope != flight.scope || !notes::valid_line(&line, limits) || Some(line.name) <= last {
            return Err(());
        }
        last = Some(line.name);
        if line.description.starts_with(c::REPORT_OPENING) {
            names.push((line.name, line.revision)).expect("no more matches than lines");
        }
    }
    let after = if next.is_some() {
        if last == after {
            return Err(());
        }
        last
    } else {
        None
    };
    flight.phase = Phase::Entries { names, at: 0, after };
    Ok(next_range(flight))
}

fn entry(
    flight: &mut Flight,
    rows: Box<[Record]>,
    next: Option<Key>,
    limits: &notes::Limits,
) -> Result<Option<Range>, ()> {
    let Phase::Entries { names, at, .. } = &mut flight.phase else { unreachable!("entry load has a name") };
    let &(name, revision) = names.get(*at).expect("requested candidate");
    if next.is_some() || rows.len() != 1 {
        return Err(());
    }
    let Some(Record::Core(core::Record::Notes(notes::Record::Entry(entry)))) = rows.into_iter().next() else {
        return Err(());
    };
    if entry.name != name
        || entry.scope != flight.scope
        || entry.revision != revision
        || !entry.description.starts_with(c::REPORT_OPENING)
        || !notes::valid_entry(&entry, limits)
    {
        return Err(());
    }
    let newer = match &flight.newest {
        Some(previous) => (entry.changed, name) > (previous.changed, previous.name),
        None => true,
    };
    if newer {
        let left = match c::goal_left(&entry.body) {
            Some(line) => Some(skein_lib::bytes::copy_of(line)),
            None => None,
        };
        flight.newest = Some(Newest { changed: entry.changed, name, left });
    }
    *at = at.checked_add(1).expect("bounded candidate index");
    Ok(next_range(flight))
}

fn next_range(flight: &mut Flight) -> Option<Range> {
    let Phase::Entries { names, at, after } = &flight.phase else { unreachable!("candidate page selected") };
    if let Some(&(name, _revision)) = names.get(*at) {
        return Some(Range::Core(CoreRange::ApplicationNotes(notes::Range::Entry { name })));
    }
    let after = (*after)?;
    flight.phase = Phase::Index { after: Some(after) };
    Some(index_range(&flight.scope, Some(after)))
}
