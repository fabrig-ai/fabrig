//! Durable job attempts and their read-back (domain/landing-history.md, 3.2–3.3).
use crate::{Domain, LandingRow, Request, history as h};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Wall};
use temper_engine_domain_forge_landing as policy;

fn event(
    job: &h::JobRecord,
    landing: Option<h::InLanding>,
    head: Option<skein_forge::api::Commit>,
    base: Option<skein_forge::api::Commit>,
) -> h::Step {
    let what = match job.step.outcome {
        Some(outcome) => h::What::JobEnded { job: job.step.job, outcome },
        None => h::What::Job { job: job.step.job, task: job.step.task },
    };
    h::Step {
        what,
        landing,
        head,
        base,
        operation: h::Operation::Of(job.step.task),
        attempt: job.step.attempt,
        source_at: None,
    }
}
fn same(job: &h::JobRecord, step: h::JobStep) -> bool {
    job.step.task == step.task && job.step.attempt == step.attempt
}
fn end_record(old: &mut h::JobRecord, step: h::JobStep) -> bool {
    if !same(old, step) || old.written || old.step.outcome.is_some() {
        return false;
    }
    old.step.outcome = step.outcome;
    true
}

pub(crate) fn heard(d: &mut Domain, wall: Wall, task: u64, step: h::JobStep, out: &mut Queue<Request>) {
    if let Some(mut row) = d.landing.rows.get(&task).cloned() {
        if step.outcome.is_some() {
            let Some(job) = row.job.as_mut() else { return };
            if !end_record(job, step) {
                return;
            }
        } else {
            if let Some(old) = &row.job
                && same(old, step)
            {
                return;
            }
            if let Some(mut old) = row.job.take()
                && !old.written
            {
                old.step.outcome = Some(old.step.outcome.unwrap_or(h::JobOutcome::Lost));
                write_prepare(d, wall, &row, &old, &[], out);
            }
            let mut members = List::uncapped();
            for (index, member) in row.landing.members.iter().enumerate() {
                if member.left.is_none() && member.landed.is_none() && member.asked == Some(row.landing.preparation) {
                    members.push(u16::try_from(index).expect("bounded members")).expect("admitted job members");
                }
            }
            let job = h::JobRecord {
                step,
                round: row.landing.preparation,
                members: members.into_boxed(),
                head: None,
                written: false,
            };
            write_prepare(d, wall, &row, &job, &[], out);
            row.job = Some(job);
        }
        crate::landing::save(d, row, out);
        out.push(Request::WakeProcedure { task });
    } else if let Some(mut row) = d.changes.get(&task).cloned() {
        if step.outcome.is_some() {
            let Some(job) = row.job.as_mut() else { return };
            if !end_record(job, step) {
                return;
            }
        } else {
            if let Some(old) = &row.job
                && same(old, step)
            {
                return;
            }
            if let Some(mut old) = row.job.take()
                && !old.written
            {
                old.step.outcome = Some(old.step.outcome.unwrap_or(h::JobOutcome::Lost));
                d.write_event(wall, task, event(&old, None, None, None), out);
                row.history = d.change(task).expect("job owner").history;
            }
            let job =
                h::JobRecord { step, round: 0, members: Box::new([]), head: row.change.last_head, written: false };
            let base = match row.delegate {
                Some((_, temper_engine_domain_forge_change::Delegate::Merge { base, .. })) => Some(base),
                Some(_) | None => None,
            };
            d.write_event(wall, task, event(&job, None, job.head, base), out);
            row.history = d.change(task).expect("job owner").history;
            row.job = Some(job);
        }
        d.changes.insert(task, row.clone()).expect("job owner admitted");
        crate::domain::save_change(d, out, row);
        out.push(Request::WakeProcedure { task });
    }
}
fn write_prepare(
    d: &mut Domain,
    wall: Wall,
    row: &LandingRow,
    job: &h::JobRecord,
    facts: &[policy::MemberFacts],
    out: &mut Queue<Request>,
) {
    for index in &job.members {
        let member = row.landing.members.get(usize::from(*index)).expect("job member");
        let (head, base) = if job.step.outcome.is_some() {
            let head = match facts.get(usize::from(*index)) {
                Some(fact) => match fact.candidate {
                    Some((head, _)) => Some(head),
                    None => None,
                },
                None => member.candidate,
            };
            (head, None)
        } else {
            (member.head, member.base)
        };
        d.write_event(
            wall,
            member.change,
            event(job, Some(h::InLanding { task: row.landing.task, round: job.round, unit: member.unit }), head, base),
            out,
        );
    }
}
pub(crate) fn prepared(
    d: &mut Domain,
    wall: Wall,
    row: &mut LandingRow,
    facts: &[policy::MemberFacts],
    out: &mut Queue<Request>,
) {
    let Some(mut job) = row.job.take() else { return };
    if job.step.outcome.is_some() && !job.written {
        write_prepare(d, wall, row, &job, facts, out);
        job.written = true;
    }
    row.job = Some(job);
}
/// A merge job's head comes from the same coherent branch read as its change.
pub(crate) fn merged(
    d: &mut Domain,
    wall: Wall,
    row: &mut crate::ChangeRow,
    head: Option<skein_forge::api::Commit>,
    out: &mut Queue<Request>,
) {
    let Some(mut job) = row.job.take() else { return };
    if job.step.outcome.is_some() && !job.written {
        let pushed = if head == job.head { None } else { head };
        d.write_event(wall, row.task, event(&job, None, pushed, None), out);
        row.history = d.change(row.task).expect("job owner").history;
        job.written = true;
    }
    row.job = Some(job);
}
