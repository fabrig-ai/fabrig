//! Bounded io stages: ready results, completions, then deadlines.
//! See `skein/docs/design/io.md, 2` and
//! `skein/docs/foundation/testing-strategy.md, 6`.

use crate::{
    Event, Io, Limits, MAX_OUT_DOWN, MAX_OUT_FIRE, MAX_OUT_RESUME, MAX_OUT_UP, MaxOut, Request,
    kernel::{Complete, Submit},
};
use skein_lib::{Env, Queue, Time};

/// Drain bounded ready work, then completions, then due timers; stop before
/// taking work whose maximum submissions do not fit the kernel queue.
pub fn up(
    io: &mut Io,
    env: &Env<Limits>,
    completions: &mut Queue<Complete>,
    events: &mut Queue<Event>,
    submissions: &mut Queue<Submit>,
    now: Time,
) {
    let limits = env.limits;
    // Each entity is resumed at most once a stage, and each refusal told once.
    let entries = limits.sockets.checked_add(limits.refusals).expect("io's limits fit a u32");
    for _ in 0..entries {
        if !io.is_ready() || !room(submissions, MAX_OUT_RESUME) {
            break;
        }
        crate::resume(io, env, events, submissions);
    }
    if io.is_ready() {
        return;
    }
    for _ in 0..completions.len() {
        if !room(submissions, MAX_OUT_UP) {
            break;
        }
        let Some(complete) = completions.pop() else {
            break;
        };
        crate::up(io, env, complete, events, submissions);
    }
    if !completions.is_empty() {
        return;
    }
    // A close's deadline and a retry's, per socket.
    let timers = limits.sockets.checked_mul(2).expect("io's limits fit a u32");
    for _ in 0..timers {
        if !io.is_due(now) || !room(submissions, MAX_OUT_FIRE) {
            break;
        }
        crate::fire(io, env, events, submissions);
    }
}

fn room(submissions: &Queue<Submit>, max: MaxOut) -> bool {
    submissions.room() >= max.submissions
}

/// Take the queued requests while io can hold a refusal and each request's
/// maximum submissions fit; leave the remaining requests queued.
pub fn down(io: &mut Io, env: &Env<Limits>, requests: &mut Queue<Request>, submissions: &mut Queue<Submit>) {
    for _ in 0..requests.len() {
        if !io.takes() || !room(submissions, MAX_OUT_DOWN) {
            break;
        }
        let Some(request) = requests.pop() else {
            break;
        };
        crate::down(io, env, request, submissions);
    }
}
