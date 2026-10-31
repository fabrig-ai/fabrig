//! io's stage in each pass of the loop (io.md, 2.1): its entry points called
//! in their order, each only while the kernel's submission queue has room for
//! the most that entry point submits (its `MAX_OUT`, programming-model.md,
//! section 2). What does not fit waits in its queue for the next iteration.

use skein_lib::{Env, Queue};

use crate::kernel::{Complete, Submit};
use crate::layer::{self, Io};
use crate::limits::{self, Limits, MAX_OUT_DOWN, MAX_OUT_FIRE, MAX_OUT_RESUME, MAX_OUT_UP, MaxOut};
use crate::records::{Event, Request};

/// io's stage of the up pass: the ready list, then the kernel's completions,
/// then the deadlines due at `env.now`.
///
/// - **The ready list first:** `resume` while io [`is_ready`](Io::is_ready).
///   An entity readied while the stage runs waits for the next one, so each
///   is resumed at most once a stage, and each refusal is told once.
/// - **Then the completions,** only once the ready list is empty, so that
///   what the down pass made is told first: `up` for each completion the
///   queue held when the stage began.
/// - **Then the deadlines,** only once every completion is taken, so that
///   progress that came wins over a deadline that passed while the loop
///   waited: `fire` while one [`is_due`](Io::is_due).
///
/// Each call waits for room for its entry point's `MAX_OUT` in `subs`; what
/// waits stays where it is (on the ready list, in `completions`, in io's
/// deadline table) for the next iteration's stage.
pub fn up(
    io: &mut Io,
    env: &Env<Limits>,
    completions: &mut Queue<Complete>,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    // Each entity once, and each refusal held.
    let entries = env.limits.sockets.saturating_add(env.limits.refusals);
    for _ in 0..entries {
        if !io.is_ready() || !room(subs, MAX_OUT_RESUME) {
            break;
        }
        layer::resume(io, env, events, subs);
    }
    if io.is_ready() {
        return;
    }
    for _ in 0..completions.len() {
        if !room(subs, MAX_OUT_UP) {
            break;
        }
        let Some(complete) = completions.pop() else {
            break;
        };
        layer::up(io, env, complete, events, subs);
    }
    if !completions.is_empty() {
        return;
    }
    // A close's deadline and a retry's, per entity.
    let timers = limits::timers(&env.limits).unwrap_or(u32::MAX);
    for _ in 0..timers {
        if !io.is_due(env.now) || !room(subs, MAX_OUT_FIRE) {
            break;
        }
        layer::fire(io, env, events, subs);
    }
}

/// io's stage of the down pass: `down` for each request `requests` held when
/// the stage began, oldest first, while io [`takes`](Io::takes) one and
/// `subs` has room for `MAX_OUT_DOWN`. The rest wait in `requests`, in
/// order, for the next iteration.
pub fn down(io: &mut Io, env: &Env<Limits>, requests: &mut Queue<Request>, subs: &mut Queue<Submit>) {
    for _ in 0..requests.len() {
        if !io.takes() || !room(subs, MAX_OUT_DOWN) {
            break;
        }
        let Some(request) = requests.pop() else {
            break;
        };
        layer::down(io, env, request, subs);
    }
}

/// Whether `subs` holds what an entry point bounded by `max` may submit.
fn room(subs: &Queue<Submit>, max: MaxOut) -> bool {
    subs.room() >= max.submissions
}
