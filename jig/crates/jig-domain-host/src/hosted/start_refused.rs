//! A refused start frees its slot without keeping an unstarted agent (domain/hosts/hub.md, 2 and 3).
use super::{Hosted, State, conclude, tail};
use crate::{Ending, Hub, Invalid, Limits, LinkUp, Refusal, Request, RunAnswer};
use alloc::boxed::Box;
use core::mem;
use skein_lib::{Env, Id, Queue, Token};

pub(crate) fn start_refused(
    domain: &mut Hub,
    env: &Env<Limits>,
    owner: Token,
    why: Box<[u8]>,
    cause: crate::StartCause,
    out: &mut Queue<Request>,
) {
    refused(domain, env, owner, why, Reason::Invalid(cause), out);
}

pub(crate) fn start_busy(domain: &mut Hub, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    refused(domain, env, owner, Box::new([]), Reason::Busy, out);
}

pub(crate) fn start_version(
    domain: &mut Hub,
    env: &Env<Limits>,
    owner: Token,
    mismatch: crate::Mismatch,
    out: &mut Queue<Request>,
) {
    refused(domain, env, owner, Box::new([]), Reason::Version(mismatch), out);
}

enum Reason {
    Invalid(crate::StartCause),
    Busy,
    Version(crate::Mismatch),
}

fn refused(
    domain: &mut Hub,
    env: &Env<Limits>,
    owner: Token,
    why: Box<[u8]>,
    reason: Reason,
    out: &mut Queue<Request>,
) {
    let id = Id::<Hosted>::from_token(owner);
    let entry = domain.hosted.get(id).expect("a start lives until its terminal");
    match &entry.state {
        // A cancellation already won: keep its established terminal path.
        State::Unwanted { .. } => {
            super::gone(domain, env, owner, why, out);
            return;
        }
        State::Starting { .. } => {}
        State::Preparing { .. }
        | State::Cancelling { .. }
        | State::Active { .. }
        | State::Waiting { .. }
        | State::Stopping { .. }
        | State::Saving { .. }
        | State::Closed => unreachable!("only a pending start can be refused"),
    }
    let entry = domain.hosted.get_mut(id).expect("pending start");
    let State::Starting { workspace, held, fault: _ } = mem::replace(&mut entry.state, State::Closed) else {
        unreachable!("checked the pending start above");
    };
    crate::refusals::ending(held, &mut entry.refusals);
    if let Some(workspace) = workspace {
        out.push(Request::Release { workspace });
    }
    let refusal = match reason {
        Reason::Invalid(cause) => Refusal::Invalid(Invalid::Start { cause, why: tail(why, &env.limits) }),
        Reason::Busy => Refusal::Busy,
        Reason::Version(mismatch) => Refusal::Version(mismatch),
    };
    let answer =
        RunAnswer { turns: 0, spent: 0, read: None, refused: entry.refusals.take(), ending: Ending::Refused(refusal) };
    out.push(Request::Up(LinkUp::Answer { run: entry.run, attempt: entry.attempt, answer }));
    conclude(domain, id, out);
}
