//! io's stage of each pass (io.md, 2): the loop calls `up` in its up pass
//! and `down` in its down pass, with the kernel's capped queues.

use skein_lib::{Env, Queue};

use crate::kernel::{Complete, Submit};
use crate::{Event, Io, Limits, MAX_OUT_DOWN, MAX_OUT_FIRE, MAX_OUT_RESUME, MAX_OUT_UP, Request};

/// `resume` while the ready list holds an entry, then `up` for each
/// completion only once it is empty, then `fire` while a deadline is due
/// at `env.now`. Each step waits for room in `subs` for what it may
/// submit (`MAX_OUT_RESUME`, `MAX_OUT_UP`, `MAX_OUT_FIRE`): without it the
/// stage stops there, and the loop's next iteration takes it up.
pub fn up(
    io: &mut Io,
    env: &Env<Limits>,
    completions: &mut Queue<Complete>,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    let ready = io.ready_len();
    let completed = completions.len();
    let deadlines = io.deadlines_len();
    for _ in 0..ready {
        if !io.is_ready() || subs.room() < MAX_OUT_RESUME.submissions {
            break;
        }
        crate::resume(io, env, up, subs);
    }
    if io.is_ready() {
        return;
    }
    for _ in 0..completed {
        if subs.room() < MAX_OUT_UP.submissions {
            break;
        }
        let Some(complete) = completions.pop() else {
            break;
        };
        crate::up(io, env, complete, up, subs);
    }
    if !completions.is_empty() {
        return;
    }
    for _ in 0..deadlines {
        if !io.is_due(env.now) || subs.room() < MAX_OUT_FIRE.submissions {
            break;
        }
        crate::fire(io, env, up, subs);
    }
}

/// `down` for each request in order, while io takes one (`Io::takes`) and
/// `subs` has room for `MAX_OUT_DOWN`; the rest stay queued, in order.
pub fn down(io: &mut Io, env: &Env<Limits>, requests: &mut Queue<Request>, subs: &mut Queue<Submit>) {
    for _ in 0..requests.len() {
        if !io.takes() || subs.room() < MAX_OUT_DOWN.submissions {
            break;
        }
        let Some(request) = requests.pop() else {
            break;
        };
        crate::down(io, env, request, subs);
    }
}
