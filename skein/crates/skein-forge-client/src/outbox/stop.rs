//! The final client phase: finish sent entries, without starting another write (forge-client.md, 7.8).
use super::{Domain, Env, Limits, Op, Outbox, Queue, Request, Time};
use crate::calls::Owner;
use skein_lib::List;

#[derive(Debug)]
pub(crate) enum State {
    Running,
    Settling(Time),
    Settled,
}
pub(crate) fn active(outbox: &Outbox) -> bool {
    match outbox.stop {
        State::Running => false,
        State::Settling(_) | State::Settled => true,
    }
}
pub(crate) fn deadline(outbox: &Outbox) -> Option<Time> {
    match outbox.stop {
        State::Settling(at) => Some(at),
        State::Running | State::Settled => None,
    }
}
pub(crate) fn start(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if active(&d.outbox) {
        return;
    }
    let span = env.limits.lifetime.saturating_add(env.limits.clock_margin).saturating_mul(2);
    d.outbox.stop = State::Settling(env.now.saturating_add(span));
    advance(d, env, out);
}
pub(crate) fn advance(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(until) = deadline(&d.outbox) else { return };
    let mut left = List::with_capacity(env.limits.entries);
    for (number, writing) in &d.outbox.entries {
        if writing.awaiting_outcome {
            left.push(*number).expect("one unfinished result per entry");
        }
    }
    if left.is_empty() || env.now >= until {
        d.outbox.stop = State::Settled;
        out.push(Request::Settled { left: left.into_boxed() });
    }
}
/// Only entries sent by this client and still awaiting an outcome may continue.
pub(crate) fn entry(outbox: &Outbox, number: u64) -> bool {
    if !active(outbox) {
        return true;
    }
    if deadline(outbox).is_none() {
        return false;
    }
    match outbox.entries.get(&number) {
        Some(writing) => writing.awaiting_outcome,
        None => false,
    }
}
/// A queued write is held even if its entry previously sent another attempt.
pub(crate) fn call(outbox: &Outbox, owner: &Owner, op: &Op) -> bool {
    if !active(outbox) {
        return true;
    }
    match op {
        Op::Write(_) => false,
        Op::Read(_) => match owner {
            Owner::Entry(number) => entry(outbox, *number),
            Owner::Read(_) | Owner::Resource(_) | Owner::Repository(_) => false,
        },
    }
}
