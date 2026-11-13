//! Landing proof observations, written with their durable transitions
//! (temper/docs/design/domain/landing-history.md, 2.2 and 3.2–3.4).
use crate::{Domain, LandingRow, LandingRun, Limits, Operation, Request, history, runs};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Env, List, Queue, Wall};
use temper_engine_domain_forge_landing as policy;

fn step(row: &LandingRow, what: history::What, head: Option<api::Commit>) -> history::Step {
    history::Step {
        what,
        landing: None,
        head,
        base: row.landing.members.first().and_then(member_base),
        operation: history::Operation::Of(row.landing.task),
        attempt: u32::from(row.landing.preparation),
        source_at: None,
    }
}
fn member_base(member: &policy::Member) -> Option<api::Commit> { member.base }

fn members(d: &mut Domain, wall: Wall, row: &LandingRow, through: Option<u16>, step: history::Step, out: &mut Queue<Request>) {
    for (index, member) in row.landing.members.iter().enumerate() {
        if member.candidate.is_none() || through.is_some_and(|top| index > usize::from(top)) {
            continue;
        }
        let mut step = step.clone();
        step.landing = Some(history::InLanding { task: row.landing.task, round: row.landing.preparation, unit: member.unit });
        d.write_event(wall, member.change, step, out);
    }
}

/// The entry's observation mark, not the in-memory follower, decides whether
/// either interval end is new. A completed run first seen has no invented start.
pub(crate) fn run(
    d: &mut Domain, env: &Env<Limits>, row: &mut LandingRow, flight: &runs::Follow,
    verdict: Option<history::GateVerdict>, out: &mut Queue<Request>,
) {
    let Some(run) = &flight.run else { return };
    let Some(index) = row.entries.iter().position(|entry| entry.entry == flight.entry) else { return };
    let entry = *row.entries.get(index).expect("flight entry");
    if entry.preparation != Some(row.landing.preparation) { return; }
    let mut attempt = 1;
    for job in &run.jobs { attempt = attempt.max(job.attempt); }
    // A provider may omit jobs on a listing; the committed rerun names its attempt.
    if entry.operation == Operation::Rerun
        && let Some(kept) = d.entries.get(&entry.entry)
        && let api::Write::Rerun { attempt: job } = &kept.entry.effect.write
    { attempt = attempt.max(job.attempt.saturating_add(1)); }
    let same = entry.run.is_some_and(|mark| mark.id == run.id && mark.attempt == attempt);
    if same && entry.run.is_some_and(|mark| mark.ended) { return; }
    let key = crate::landing::input_commit(&flight.inputs, b"temper_key").expect("gate key");
    let mut event = step(row, history::What::Gate { gate: entry.gate, key }, Some(flight.commit));
    event.base = None;
    event.operation = history::Operation::Of(run.id);
    event.attempt = attempt;
    let mut changed = false;
    if !same && verdict.is_none() && run.state != api::RunState::Completed {
        event.source_at = Some(Wall::from_nanos(
            env.wall.as_nanos().saturating_sub(env.now.as_nanos().saturating_sub(run.created.as_nanos()))
                .saturating_add(run.created.as_nanos().saturating_sub(env.now.as_nanos())),
        ));
        members(d, env.wall, row, None, event.clone(), out);
        changed = true;
    }
    if let Some(verdict) = verdict {
        event.what = history::What::GateEnded { gate: entry.gate, key, verdict };
        members(d, env.wall, row, None, event, out);
        changed = true;
    }
    if changed {
        row.entries.get_mut(index).expect("flight entry").run = Some(LandingRun { id: run.id, attempt, ended: verdict.is_some() });
    }
}

/// A saved search-bound transition also starts the next probe, if any.
pub(crate) fn search(d: &mut Domain, wall: Wall, old: &LandingRow, row: &LandingRow, out: &mut Queue<Request>) {
    if old.landing.preparation != row.landing.preparation || old.landing.round.search == row.landing.round.search { return; }
    let Some(search) = row.landing.round.search else { return };
    if let Some(previous) = old.landing.round.search {
        if let Some(probe) = policy::probe(&old.landing) {
            let prefix = old.landing.members.get(usize::from(probe)).expect("probe").candidate;
            let event = step(old, history::What::ProbeEnded { green: search.green != previous.green }, prefix);
            let event = history::Step { base: None, operation: history::Operation::Of(u64::from(old.landing.preparation)), attempt: 0, ..event };
            members(d, wall, old, None, event, out);
        }
    } else {
        let head = row.landing.members.get(usize::from(search.red.checked_sub(1).expect("red"))).expect("red").candidate;
        members(d, wall, row, None, step(row, history::What::Search, head), out);
    }
    if let Some(probe) = policy::probe(&row.landing) {
        let head = row.landing.members.get(usize::from(probe)).expect("probe").candidate;
        let event = history::Step { base: None, operation: history::Operation::Of(u64::from(row.landing.preparation)), attempt: 0, ..step(row, history::What::Probe, head) };
        members(d, wall, row, None, event, out);
    } else {
        let red = row.landing.members.get(usize::from(search.red.checked_sub(1).expect("red"))).expect("red");
        let green = search.green.checked_sub(1).and_then(|index| row.landing.members.get(usize::from(index))).and_then(member_candidate);
        let what = history::What::Searched { green, red: red.candidate.expect("red prefix"), blamed: red.unit };
        members(d, wall, row, None, step(row, what, None), out);
    }
}
fn member_candidate(member: &policy::Member) -> Option<api::Commit> { member.candidate }

pub(crate) fn proved(d: &mut Domain, wall: Wall, old: &LandingRow, row: &LandingRow, out: &mut Queue<Request>) {
    if !matches_proving(&old.landing.phase) { return; }
    let Some(top) = crate::landing::round_top(row) else { return };
    let member = row.landing.members.get(usize::from(top)).expect("proved top");
    let mut proofs = List::with_capacity(u32::try_from(member.gates.len()).expect("gates"));
    for (number, gate) in member.gates.iter().enumerate() {
        if let policy::Proof::Passed { .. } = gate.proof {
            let number = u16::try_from(number).expect("gate");
            let mut run = None;
            for entry in &row.entries {
                if entry.preparation == Some(row.landing.preparation) && entry.member == top && entry.gate == number {
                    if let Some(mark) = entry.run && mark.ended { run = Some(mark.id); }
                }
            }
            proofs.push(history::Proof { gate: number, run }).expect("gates");
        }
    }
    let event = step(row, history::What::Proved { proofs: proofs.into_boxed() }, member.candidate);
    members(d, wall, row, Some(top), event, out);
}
fn matches_proving(phase: &policy::Phase) -> bool { if let policy::Phase::Proving = phase { true } else { false } }

/// Successful outbox settlement removes the merge/record entry in this decision.
pub(crate) fn made(d: &mut Domain, wall: Wall, row: &LandingRow, entry: crate::LandingEntry, out: &mut Queue<Request>) {
    let member = row.landing.members.get(usize::from(entry.member)).expect("merge member");
    if entry.operation == Operation::Merge {
        let count = row.landing.members.iter().take(usize::from(entry.member).saturating_add(1)).filter(|m| m.left.is_none() && m.candidate.is_some()).count();
        let event = step(row, history::What::Turn { members: u16::try_from(count).expect("members") }, member.candidate);
        members(d, wall, row, Some(entry.member), event, out);
    } else if entry.operation == Operation::Record {
        let mut event = step(row, history::What::Recorded { state: history::RecordedState::Merged }, member.landed);
        event.base = None;
        event.operation = history::Operation::Of(member.pull);
        event.attempt = 0;
        event.landing = Some(history::InLanding { task: row.landing.task, round: row.landing.preparation, unit: member.unit });
        d.write_event(wall, member.change, event, out);
    }
}
