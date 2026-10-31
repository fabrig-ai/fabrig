//! io's stages check the kernel's room before taking work
//! (skein/docs/design/io.md, 2;
//! skein/docs/foundation/testing-strategy.md, 6).

use skein_lib::{Env, Queue, Time};

use crate::kernel::{Complete, Submit};
use crate::{self as io, Event, Io, Limits, MaxOut, Request};

/// io's stage of the up pass: the ready list, then, once nothing is
/// ready, `completions`, then, once none is left, the timers due at
/// `now`. Each step is taken only while `submissions` has room for its
/// `MAX_OUT_*`; a step that would not fit waits for the next pass.
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
        if !io.is_ready() || !room(submissions, io::MAX_OUT_RESUME) {
            break;
        }
        io::resume(io, env, events, submissions);
    }
    if io.is_ready() {
        return;
    }
    for _ in 0..completions.len() {
        if !room(submissions, io::MAX_OUT_UP) {
            break;
        }
        let Some(complete) = completions.pop() else {
            break;
        };
        io::up(io, env, complete, events, submissions);
    }
    if !completions.is_empty() {
        return;
    }
    // A close's deadline and a retry's, per socket.
    let timers = limits.sockets.checked_mul(2).expect("io's limits fit a u32");
    for _ in 0..timers {
        if !io.is_due(now) || !room(submissions, io::MAX_OUT_FIRE) {
            break;
        }
        io::fire(io, env, events, submissions);
    }
}

/// io's stage of the down pass: each of `requests`, oldest first,
/// while io can hold a refusal (`Io::takes`) and `submissions` has room
/// for `MAX_OUT_DOWN`. What is not taken stays queued.
pub fn down(io: &mut Io, env: &Env<Limits>, requests: &mut Queue<Request>, submissions: &mut Queue<Submit>) {
    for _ in 0..requests.len() {
        if !io.takes() || !room(submissions, io::MAX_OUT_DOWN) {
            break;
        }
        let Some(request) = requests.pop() else {
            break;
        };
        io::down(io, env, request, submissions);
    }
}

fn room(submissions: &Queue<Submit>, max: MaxOut) -> bool {
    submissions.room() >= max.submissions
}
