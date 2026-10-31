//! io's stage of a service's loop (skein/docs/design/io.md, 2): what every
//! host's loop does with io in each pass, the kernel's submission room
//! checked before each entry point (testing-strategy.md, 6).

use skein_lib::{Env, Queue};

use crate::kernel::{Complete, Submit};
use crate::{Event, Io, Limits, MaxOut, Request, limits};

/// The up pass's stage: `resume` while io is ready, then, once its ready
/// list is empty, `up` for each completion, then, once every completion
/// is taken, `fire` while a deadline is due at `env.now`; each only
/// while `submissions` has room for its `MAX_OUT`. What is not taken
/// stays for the next iteration.
pub fn up(
    io: &mut Io,
    env: &Env<Limits>,
    completions: &mut Queue<Complete>,
    events: &mut Queue<Event>,
    submissions: &mut Queue<Submit>,
) {
    // Every entity kind shares the slab counted by sockets(), including
    // pipes, children, adopted appends and signals. Each is resumed at most
    // once a stage; each held refusal is told once. Their u32 counts sum in u64.
    let entries =
        u64::from(io.sockets()).checked_add(u64::from(io.tables.refused.len())).expect("two u32 counts fit a u64");
    for _ in 0..entries {
        if !io.is_ready() || !room(submissions, crate::MAX_OUT_RESUME) {
            break;
        }
        crate::resume(io, env, events, submissions);
    }
    if io.is_ready() {
        return;
    }
    for _ in 0..completions.len() {
        if !room(submissions, crate::MAX_OUT_UP) {
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
    // Every entity has at most a close/write deadline and a retry. Firing
    // removes one; retries are submitted, not rearmed at the current time.
    let timers = limits::timers(&env.limits).expect("io's timer capacity fits a u32");
    for _ in 0..timers {
        if !io.is_due(env.now) || !room(submissions, crate::MAX_OUT_FIRE) {
            break;
        }
        crate::fire(io, env, events, submissions);
    }
}

/// The down pass's stage: `down` for each request in order, while io
/// takes one (`Io::takes`) and `submissions` has room for
/// `MAX_OUT_DOWN`. What is not taken stays queued.
pub fn down(io: &mut Io, env: &Env<Limits>, requests: &mut Queue<Request>, submissions: &mut Queue<Submit>) {
    for _ in 0..requests.len() {
        if !io.takes() || !room(submissions, crate::MAX_OUT_DOWN) {
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
