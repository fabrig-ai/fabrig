//! Provider failure classification and retry timing. Time and randomness are
//! inputs; the session machine follows the returned state.
//! Contract: smith/docs/design/domain/session.md, section 4.

use alloc::boxed::Box;

use super::{
    Conversation, End, Env, Failure, Id, Limits, OUT_OF_TIME, Queue, Request, Rng, Session, State, ask, compaction,
    finish,
};
use crate::llm::{Cap, CompletionPhase, Evidence, Limit};
use skein_lib::Duration;

/// One policy class for every provider failure.
#[derive(Clone, Copy)]
enum Class {
    Gone,
    Context,
    Transient,
    RateLimited { delay: Duration },
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
                Class::Transient
            }
        },
        Failure::Overloaded | Failure::Unavailable | Failure::Unauthorized => Class::Transient,
        Failure::RateLimited { retry_after } => Class::RateLimited { delay: retry_after },
        Failure::Limit { .. }
        | Failure::ApplicationLimit
        | Failure::Protocol
        | Failure::Cancelled
        | Failure::Refused
        | Failure::SignedOut
        | Failure::Program { .. }
        | Failure::Untooled
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
    let attempts = conversation.completion_attempts;
    match class {
        Class::Gone => {
            return match compaction::session_gone(conversation, &env.limits) {
                Some(end) => finish(end),
                None => ask(conversation, id, 0, env, out),
            };
        }
        Class::Context => {
            return match compaction::start(conversation, &env.limits, evidence, attempts) {
                Some(end) => finish(end),
                None => ask(conversation, id, 0, env, out),
            };
        }
        Class::Transient | Class::RateLimited { .. } | Class::TimeUp | Class::Lasting => {}
    }
    let response = match evidence {
        Evidence::Response { .. } => true,
        Evidence::Unsent | Evidence::Unknown => false,
    };
    let decision = if failure == Failure::Unauthorized && (conversation.account.is_none() || (!retry_ok && response)) {
        Retry::Fail { reset: None }
    } else {
        decision(class, attempt, conversation, env, rng)
    };
    match decision {
        Retry::After { delay, rate_limited } => {
            // The retry would need a new identity and a representable one-based count.
            if attempts == u32::MAX {
                return finish(End::Failed {
                    failure: Failure::ApplicationLimit,
                    evidence: Evidence::Unsent,
                    attempts,
                    detail: Box::default(),
                });
            }
            let until = env.now.saturating_add(delay);
            if rate_limited {
                out.push(Request::RateLimitWait { opener: conversation.opener, until });
            }
            State::Backoff { attempt: attempts, until, rate_limited }
        }
        Retry::Fail { reset } => {
            if let Some(retry_after) = reset {
                out.push(Request::RateLimitOutlastsSession { opener: conversation.opener, retry_after });
            }
            finish(End::Failed { failure, evidence, detail: Box::default(), attempts })
        }
        Retry::TimeUp => finish(OUT_OF_TIME),
    }
}

enum Retry {
    After { delay: Duration, rate_limited: bool },
    Fail { reset: Option<Duration> },
    TimeUp,
}

fn decision(class: Class, attempt: u32, conversation: &mut Conversation, env: &Env<Limits>, rng: &mut Rng) -> Retry {
    if env.now >= conversation.expires {
        return Retry::TimeUp;
    }
    match class {
        Class::RateLimited { delay } => {
            if delay != Duration::ZERO
                && conversation.account.is_some()
                && env.now.saturating_add(delay) >= conversation.expires
            {
                return Retry::Fail { reset: Some(delay) };
            }
            Retry::After { delay: backoff(delay, attempt, &env.limits, rng), rate_limited: true }
        }
        Class::Transient if conversation.retries_spent < env.limits.retries => {
            conversation.retries_spent = conversation.retries_spent.checked_add(1).expect("retry budget checked");
            Retry::After { delay: backoff(Duration::ZERO, attempt, &env.limits, rng), rate_limited: false }
        }
        Class::Transient | Class::Lasting => Retry::Fail { reset: None },
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
