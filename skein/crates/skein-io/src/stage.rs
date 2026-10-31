//! io's stages of the loop's two passes (io.md, 2; programming-model.md, 2),
//! for a service's loop to call in place of driving the four entry points by
//! hand.
//!
//! In the up pass, [`up`] takes the ready list, then the completions (only
//! once the ready list is empty, so that what the down pass made is told
//! first), then the deadlines (only once every completion is taken, so that
//! progress that came wins over a deadline that passed while the loop
//! waited). In the down pass, [`down`] takes each request while io can hold
//! a refusal. Both take an entry only while the kernel's submission queue has
//! room for the most it may submit (`MAX_OUT_*`, the kernel's edge), and
//! leave the rest where it is for the next iteration.

use skein_lib::{Env, Queue, Time};

use crate::kernel::{Complete, Submit};
use crate::{Event, Io, Limits, MAX_OUT_DOWN, MAX_OUT_FIRE, MAX_OUT_RESUME, MAX_OUT_UP, MaxOut, Request};

/// io's stage of the up pass: ready work, then completions, then deadlines, each entry taken only
/// while what it may submit fits (`MAX_OUT_*`). Each entity is resumed at most once a stage.
pub fn up(
    io: &mut Io,
    env: &Env<Limits>,
    completions: &mut Queue<Complete>,
    told: &mut Queue<Event>,
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
        crate::resume(io, env, told, submissions);
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
        crate::up(io, env, complete, told, submissions);
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
        crate::fire(io, env, told, submissions);
    }
}

/// io's stage of the down pass: each request in `asked`, in order, while io takes one
/// (`Io::takes`) and what it submits fits (`MAX_OUT_DOWN`). Requests it cannot take stay queued.
pub fn down(io: &mut Io, env: &Env<Limits>, asked: &mut Queue<Request>, submissions: &mut Queue<Submit>) {
    for _ in 0..asked.len() {
        if !io.takes() || !room(submissions, MAX_OUT_DOWN) {
            break;
        }
        let Some(request) = asked.pop() else {
            break;
        };
        crate::down(io, env, request, submissions);
    }
}

/// Whether the kernel's submission queue holds the most an entry point submits.
fn room(submissions: &Queue<Submit>, max: MaxOut) -> bool {
    submissions.room() >= max.submissions
}
