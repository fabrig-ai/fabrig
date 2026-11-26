//! Ordering and readback of converted entries (temper/docs/design/domain/forge/writes.md, 3–4 and 7).
use super::{
    Check, Checked, Commit, Domain, Env, Found, Limits, Name, Operation, Queue, Request, Sender, Then, WriteState,
};
use super::{entry, forget, schedule};
use crate::writes::{Actor, Seen};
use skein_forge::api::{Answer, Error};
use temper_engine_domain_forge_change::Status;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum WriterRead {
    Observed,
    Answered,
    Lost,
}

/// The old waiting head was already ours. Order it before judging even a
/// read unrelated to that head. A lost writer owes a fresh branch read, not
/// a verdict on the first cached head a caller happened to bring.
pub(in crate::domain::writes) fn owed(
    d: &mut Domain,
    name: &Name,
    observed: Option<Commit>,
    actor: Actor,
    out: &mut Queue<Request>,
) -> bool {
    let Some(row) = d.writes.get(name) else { return false };
    for written in &row.entries {
        if written.state.unsettled()
            && let Operation::Compare { after } = written.operation
        {
            let order = written.order;
            return compare(d, name, order, after, out);
        }
    }
    let mut waiting = false;
    for written in &row.entries {
        if !written.state.unsettled() {
            continue;
        }
        if let Operation::Writer { .. } = written.operation {
            if actor == Actor::Other
                && let Some(observed) = observed
            {
                // A compatibility proof in flight for this observation cannot
                // explain a tip already attributed to someone else.
                d.write_reads
                    .checked
                    .insert((name.clone(), written.order), Checked { head: row.head, observed })
                    .expect("one check per write");
            }
            waiting |= waiting_for_writer(d, name, written.order)
                || (written.state == WriteState::Uncertain && !crate::domain::fresh_reads_in(d));
        }
    }
    waiting
}

fn waiting_for_writer(d: &Domain, name: &Name, order: u64) -> bool {
    let key = (name.clone(), order);
    if let Some(job) = d.write_reads.pending.get(&key)
        && terminal(job.check)
    {
        return true;
    }
    if let Some(flight) = &d.write_reads.flight {
        return flight.live && flight.key == key && terminal(flight.check);
    }
    false
}

fn terminal(check: Check) -> bool {
    match check {
        Check::Branch { .. }
        | Check::Contains { then: Then::Writer { mode: WriterRead::Answered | WriterRead::Lost, .. }, .. } => true,
        Check::Contains { .. } | Check::Pull | Check::Order { .. } | Check::PreviousCompare { .. } => false,
    }
}

/// True if the candidate needs a comparison between distinct known commits.
pub(super) fn compare(d: &mut Domain, name: &Name, order: u64, waiting: Commit, out: &mut Queue<Request>) -> bool {
    if let Some(head) = super::super::known_head(d, name)
        && head != waiting
    {
        schedule(d, name, order, Check::PreviousCompare { head, waiting });
        return true;
    }
    ordered(d, name, order, waiting, waiting, out);
    false
}

pub(super) fn compared(head: Commit, waiting: Commit, result: Result<Answer, Error>) -> Found {
    let older = match crate::domain::ancestry_status(result) {
        Status::Passed => true,
        Status::Failed => false,
        Status::Unknown | Status::Pending => return Found::Retry,
    };
    Found::Compared { head, waiting, older }
}

pub(super) fn ordered(
    d: &mut Domain,
    name: &Name,
    order: u64,
    newer: Commit,
    superseded: Commit,
    out: &mut Queue<Request>,
) {
    let row = d.writes.get_mut(name).expect("converted record");
    for written in &mut row.entries {
        if written.order == order && written.state.unsettled() {
            // Once ordered, retain the superseded head, not an unordered
            // candidate that would need another comparison at the next start.
            written.operation = Operation::Compare { after: superseded };
            written.state = WriteState::Confirmed;
            let previous = row.head;
            row.head = Some(newer);
            super::save_changed(row, previous, &mut d.write_heads, out);
            forget(d, name, order);
            return;
        }
    }
}

/// The converted head must be contained, and the read must also move
/// forward from the head recorded since conversion, if that head differs.
pub(super) fn writer_proof(
    before: Option<Commit>,
    head: Option<Commit>,
    tip: Commit,
    mode: WriterRead,
) -> Option<Check> {
    let mut contains = head;
    let mut next = None;
    if let Some(before) = before
        && before != tip
    {
        contains = Some(before);
        if head != Some(before) && head != Some(tip) {
            next = head;
        }
    }
    Some(Check::Contains { tip, contains: contains?, then: Then::Writer { head, next, mode } })
}

pub(super) fn writer_proved(head: Option<Commit>, next: Option<Commit>, tip: Commit, mode: WriterRead) -> Found {
    if let Some(contains) = next {
        Found::Check(Check::Contains { tip, contains, then: Then::Writer { head, next: None, mode } })
    } else {
        Found::WriterProgress { head, tip, mode }
    }
}

pub(super) fn writer_read(d: &Domain, name: &Name, before: Option<Commit>, tip: Option<Commit>, lost: bool) -> Found {
    let head = super::super::known_head(d, name);
    if let Some(tip) = tip
        && Some(tip) != head
        && !other_actor(d, name, tip)
    {
        let mode = if lost { WriterRead::Lost } else { WriterRead::Answered };
        return match writer_proof(before, head, tip, mode) {
            Some(check) => Found::Check(check),
            None => Found::WriterProgress { head, tip, mode },
        };
    }
    if lost { Found::NoProof { observed: None } } else { Found::Refuse }
}

pub(in crate::domain::writes) fn first_head(
    d: &mut Domain,
    name: &Name,
    order: u64,
    tip: Commit,
    out: &mut Queue<Request>,
) -> Option<Seen> {
    let Operation::Writer { before } = entry(d, name, order)?.operation else { return None };
    if other_actor(d, name, tip) {
        return Some(super::super::judge::drift(d.writes.get(name)?));
    }
    if before.is_none() && super::super::known_head(d, name).is_none() && !super::checked(d, name, order, None, tip) {
        progress(d, name, order, None, tip, WriterRead::Observed, out);
        return Some(Seen::Explained { order });
    }
    None
}

pub(super) fn progress(
    d: &mut Domain,
    name: &Name,
    order: u64,
    head: Option<Commit>,
    tip: Commit,
    mode: WriterRead,
    out: &mut Queue<Request>,
) {
    if super::super::known_head(d, name) != head {
        match mode {
            WriterRead::Observed => {}
            WriterRead::Answered => schedule(d, name, order, Check::Branch { lost: false }),
            WriterRead::Lost => schedule(d, name, order, Check::Branch { lost: true }),
        }
        return;
    }
    if other_actor(d, name, tip) || super::checked(d, name, order, head, tip) {
        if mode == WriterRead::Answered {
            super::settle(d, name, order, WriteState::Refused, None, None, out);
        }
        return;
    }
    let row = d.writes.get_mut(name).expect("converted record");
    for written in &mut row.entries {
        if written.order == order && written.state.unsettled() {
            // A live design writer can deliver again. Only its end or a
            // lost writer's proved move settles the compatibility entry.
            if written.state == WriteState::Uncertain || mode != WriterRead::Observed {
                written.state = WriteState::Confirmed;
            }
            row.head = Some(tip);
            super::advance(row);
            super::save_changed(row, head, &mut d.write_heads, out);
            forget(d, name, order);
            return;
        }
    }
}

fn other_actor(d: &Domain, name: &Name, tip: Commit) -> bool {
    if let Some(hold) = d.holds.get(name)
        && let Some(drift) = &hold.drift
        && let crate::DriftChange::Branch { observed, .. } = drift.change
    {
        return observed == Some(tip);
    }
    false
}

/// The old build had already ended this claim. Its first successful fresh
/// branch read transfers responsibility for any late push to the entry alone.
pub(super) fn end_rereading(
    d: &mut Domain,
    env: &Env<Limits>,
    name: &Name,
    sender: Sender,
    check: Check,
    found: Found,
    out: &mut Queue<Request>,
) {
    if !terminal(check) || !read_in(found) {
        return;
    }
    let Sender::Attempt { task, attempt } = sender else { return };
    let Some(hold) = d.holds.get_mut(name) else { return };
    if !hold.design_writers.contains(&crate::DesignWriter { task, attempt, rereading: true }) {
        return;
    }
    let mut retained = skein_lib::List::with_capacity(env.limits.holds);
    for writer in &hold.design_writers {
        if *writer != (crate::DesignWriter { task, attempt, rereading: true }) {
            retained.push(*writer).expect("removing from a restored claim list");
        }
    }
    hold.design_writers = retained.into_boxed();
    crate::domain::save_hold(out, hold.clone());
    crate::domain::lost_release(d, env, name, task, out);
}

fn read_in(found: Found) -> bool {
    match found {
        Found::WriterProgress { .. } | Found::Refuse | Found::NoProof { .. } => true,
        Found::Check(_) | Found::Retry | Found::Confirm { .. } | Found::Ordered { .. } | Found::Compared { .. } => {
            false
        }
    }
}
