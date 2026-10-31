//! io's stage of the loop (io.md, 2): the up pass takes the ready list, then the
//! completions, then the due deadlines; the down pass takes the requests. Each
//! step runs only while the submissions have room for its `MAX_OUT_*` (the
//! kernel's edge, testing-strategy.md, 6), so a full submission queue leaves
//! work waiting, never lost.

use crate::{
    Event, Io, Limits, MAX_OUT_DOWN, MAX_OUT_FIRE, MAX_OUT_RESUME, MAX_OUT_UP, MaxOut, Request,
    kernel::{Complete, Submit},
};
use skein_lib::{Env, Queue, Time};

/// io's stage of the up pass: each ready entity and refusal at most once, then,
/// only once nothing is ready, each completion, then, only once no completion
/// waits, each due deadline. Stops early, leaving the rest queued, when the
/// submissions cannot hold the next step's `MAX_OUT_*`.
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

/// io's stage of the down pass: each request in order, while io takes one
/// (`Io::takes`) and the submissions can hold `MAX_OUT_DOWN`; the rest stay queued.
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

fn room(submissions: &Queue<Submit>, max: MaxOut) -> bool {
    submissions.room() >= max.submissions
}
