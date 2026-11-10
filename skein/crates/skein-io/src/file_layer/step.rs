//! File steps driven by the loop's environment (skein/docs/design/runtime/io/files.md).
use super::{
    Cancelling, Complete, Env, Error, Event, FileIo, Limits, Queue, Request, Stop, Submit, Time, Work, cancelled,
    down_inner, follow_cancel, follow_work, mem, stopped, submit_cancel, up_inner,
};

pub fn down(io: &mut FileIo, env: &Env<Limits>, request: Request, events: &mut Queue<Event>, subs: &mut Queue<Submit>) {
    down_until(io, env, env.now.saturating_add(env.limits.timeout), request, events, subs);
}

pub fn down_until(
    io: &mut FileIo,
    env: &Env<Limits>,
    deadline: Time,
    request: Request,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    assert!(io.takes(), "a file request completes before the next begins");
    io.work = down_inner(io, env, request, events, subs);
    follow_work(io, Some(deadline));
}

pub fn expire(io: &mut FileIo, env: &Env<Limits>, subs: &mut Queue<Submit>) {
    if !io.is_due(env.now) {
        return;
    }
    let work = mem::replace(&mut io.work, Work::Idle);
    io.work = match work {
        Work::Active { pending, op, deadline: Some(deadline), stop: Stop::Running } if deadline <= env.now => {
            Work::Active { pending, op, deadline: None, stop: Stop::Deadline }
        }
        other @ (Work::Idle | Work::Active { .. }) => {
            io.work = other;
            return;
        }
    };
    follow_cancel(io, subs);
}

pub fn up(io: &mut FileIo, env: &Env<Limits>, complete: Complete, events: &mut Queue<Event>, subs: &mut Queue<Submit>) {
    let cancelling = mem::replace(&mut io.cancelling, Cancelling::Quiet);
    io.cancelling = match cancelling {
        Cancelling::Asked { op, target } if op == complete.op => {
            let same_target = match io.work {
                Work::Idle => false,
                Work::Active { op, .. } => op == target,
            };
            io.cancelling = if crate::layer::unsubmitted(complete.result) && same_target {
                submit_cancel(io, target, subs)
            } else {
                Cancelling::Quiet
            };
            return;
        }
        other @ (Cancelling::Quiet | Cancelling::Asked { .. }) => other,
    };
    let work = mem::replace(&mut io.work, Work::Idle);
    let (next, deadline) = match work {
        Work::Idle => unreachable!("a completion has an active file request"),
        Work::Active { pending, op, deadline, stop } => {
            assert!(op == complete.op, "completion names the outstanding file operation");
            let next = match stop {
                Stop::Deadline => stopped(io, env, pending, complete, Error::TimedOut, events, subs),
                Stop::Cancel => cancelled(io, env, pending, complete, events, subs),
                Stop::Running => up_inner(io, env, pending, complete, events, subs),
            };
            (next, deadline)
        }
    };
    io.work = next;
    follow_work(io, deadline);
}
