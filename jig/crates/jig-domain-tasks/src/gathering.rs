//! A gathering's concluding word and delegate gate (jig/docs/design/domain/tasks/gatherings.md, 2–4).
use crate::domain::{Domain, publish, record, refused, task_mut};
use crate::{Closing, Contract, Ending, Executor, Limits, New, Party, Phase, Refusal, Request, Stage, TaskResult};
use skein_lib::{Decimal, Env, Queue, ReplyTo, Wall, Writer};

/// The first party to conclude a gathering and the decision's wall time (gatherings.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Concluded {
    pub by: u64,
    pub at: Wall,
}

/// Whether this executor is a gathering (domain/tasks/gatherings.md, 2).
#[must_use]
pub fn executor(executor: Executor) -> bool {
    match executor {
        Executor::Procedure { connector, code } => connector == u16::MAX && code == 2,
        Executor::Agent { .. } | Executor::Person(_) | Executor::Job { .. } => false,
    }
}

pub(crate) fn shape(new: &New) -> bool {
    !new.spec.asks
        && new.spec.parameters.is_empty()
        && new.dependencies.is_empty()
        && match new.contract {
            Contract::Report { .. } => true,
            Contract::Verdict { .. } | Contract::Change { .. } | Contract::Approval { .. } | Contract::Job { .. } => {
                false
            }
        }
}

pub(crate) fn conclude(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    by: u64,
    number: u64,
    out: &mut Queue<Request>,
) {
    if !domain.ready() {
        return refused(to, Some(number), Refusal::NotReady, out);
    }
    let Some(row) = record(domain, number) else { return refused(to, Some(number), Refusal::Unknown, out) };
    if !executor(row.executor) || by == 0 {
        return refused(to, Some(number), Refusal::Contract, out);
    }
    if row.concluded.is_none() {
        if row.revision == u64::MAX {
            return refused(to, Some(number), Refusal::State, out);
        }
        task_mut(domain, number).expect("live gathering").record.concluded = Some(Concluded { by, at: env.wall });
        crate::control::history(domain, number, env.wall, Party::Person(by), crate::Change::Concluded, &[], out);
        publish(domain, env, number, out);
    }
    settle(domain, env, number, out);
    out.push(Request::Done { reply_to: to });
}

pub(crate) fn settle(domain: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    let Some(row) = record(domain, number) else { return };
    if !executor(row.executor) || row.concluded.is_none() || !row.delegates.is_empty() {
        return;
    }
    match row.phase {
        Phase::Active(_) => {}
        Phase::Waiting | Phase::Held { .. } | Phase::Closing(_) | Phase::Ended(_) => return,
    }
    let done = Decimal::of(u64::from(row.ended_delegates.done));
    let failed = Decimal::of(u64::from(row.ended_delegates.failed));
    let cancelled = Decimal::of(u64::from(row.ended_delegates.cancelled));
    let parts = [
        b"Concluded: ".as_slice(),
        done.as_bytes(),
        b" done, ",
        failed.as_bytes(),
        b" failed, ",
        cancelled.as_bytes(),
        b" cancelled.",
    ];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("three bounded decimal counts");
    }
    let mut words = Writer::new(length);
    for part in parts {
        words.put(part).expect("measured report words");
    }
    let result = TaskResult::Report { words: words.finish() };
    task_mut(domain, number).expect("live gathering").record.phase =
        Phase::Closing(Closing { stage: Stage::Delegates, ending: Ending::Done(result) });
    publish(domain, env, number, out);
}
