//! Provider failure classification and retry timing. Time and randomness are
//! inputs; the session machine follows the returned state.
//! Contract: smith/docs/design/domain/session.md, section 4.

use alloc::boxed::Box;

use super::{
    Conversation, End, Env, Failure, Id, Limits, OUT_OF_TIME, Queue, Request, Rng, Session, State, ask, compaction,
    finish,
};
use crate::llm::{Cap, CompletionPhase, Evidence, Limit};
use skein_lib::{Duration, Time};

/// One policy class for every provider failure.
#[derive(Clone, Copy)]
enum Class {
    Gone,
    Context,
    Transient { floor: Duration },
    TimeUp,
    Lasting,
}

fn class(failure: Failure) -> Class {
    match failure {
        Failure::SessionGone => Class::Gone,
        Failure::ContextTooLong | Failure::Limit { which: Limit::Path(Cap::Request), .. } => Class::Context,
        Failure::TimedOut { phase } => match phase {
            CompletionPhase::Whole => Class::TimeUp,
            CompletionPhase::Connect | CompletionPhase::Handshake | CompletionPhase::Head | CompletionPhase::Idle => {
                Class::Transient { floor: Duration::ZERO }
            }
        },
        Failure::Overloaded | Failure::Unavailable | Failure::Unauthorized => {
            Class::Transient { floor: Duration::ZERO }
        }
        Failure::RateLimited { retry_after } => Class::Transient { floor: retry_after },
        Failure::Limit { .. }
        | Failure::ApplicationLimit
        | Failure::Protocol
        | Failure::Cancelled
        | Failure::Refused
        | Failure::SignedOut
        | Failure::Program { .. }
        | Failure::Unsupported
        | Failure::Invalid
        | Failure::Exhausted { .. } => Class::Lasting,
    }
}

/// Completing, failed: recover its window, wait for a retry, or end once.
#[expect(
    clippy::too_many_arguments,
    reason = "the failed completion carries evidence and its issued credential retry policy"
)]
pub(super) fn failed(
    attempt: u32,
    failure: Failure,
    evidence: Evidence,
    retry_ok: bool,
    conversation: &mut Conversation,
    id: Id<Session>,
    rng: &mut Rng,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let class = class(failure);
    match class {
        Class::Gone => {
            return match compaction::session_gone(conversation) {
                Some(end) => finish(end),
                None => ask(conversation, id, 0, env, out),
            };
        }
        Class::Context => {
            return match compaction::start(
                conversation,
                &env.limits,
                evidence,
                attempt.checked_add(1).expect("a completion has a one-based attempt"),
            ) {
                Some(end) => finish(end),
                None => ask(conversation, id, 0, env, out),
            };
        }
        Class::Transient { .. } | Class::TimeUp | Class::Lasting => {}
    }
    let response = match evidence {
        Evidence::Response { .. } => true,
        Evidence::Unsent | Evidence::Unknown => false,
    };
    let decision = if failure == Failure::Unauthorized && (conversation.account.is_none() || (!retry_ok && response)) {
        Retry::Fail
    } else {
        decision(class, attempt, env.now, conversation.expires, &env.limits, rng)
    };
    match decision {
        Retry::After(delay) => {
            State::Backoff { attempt: attempt.saturating_add(1), until: env.now.saturating_add(delay) }
        }
        Retry::Fail => finish(End::Failed {
            failure,
            evidence,
            detail: Box::default(),
            attempts: attempt.checked_add(1).expect("an admitted completion leaves room for its first attempt"),
        }),
        Retry::TimeUp => finish(OUT_OF_TIME),
    }
}

enum Retry {
    After(Duration),
    Fail,
    TimeUp,
}

fn decision(class: Class, attempt: u32, now: Time, expiry: Time, limits: &Limits, rng: &mut Rng) -> Retry {
    if now >= expiry {
        return Retry::TimeUp;
    }
    match class {
        Class::Transient { floor } if attempt < limits.retries => Retry::After(backoff(floor, attempt, limits, rng)),
        Class::Transient { .. } | Class::Lasting => Retry::Fail,
        Class::TimeUp => Retry::TimeUp,
        Class::Gone | Class::Context => unreachable!("window recovery precedes retry policy"),
    }
}

fn backoff(floor: Duration, attempt: u32, limits: &Limits, rng: &mut Rng) -> Duration {
    // Exponential and capped, with equal jitter: half fixed, half random.
    let factor = 1_u64.checked_shl(attempt).unwrap_or(u64::MAX);
    let ceiling = limits.backoff_base.saturating_mul(factor).min(limits.backoff_max);
    let half = ceiling.as_nanos() / 2;
    let jittered = Duration::from_nanos(half.saturating_add(rng.below(half.saturating_add(1))));
    jittered.max(floor)
}
