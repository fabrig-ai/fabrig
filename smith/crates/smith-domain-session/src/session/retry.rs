//! Provider failure classification and retry timing. The clock and random source are inputs; no attempt is scheduled outside the session machine.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use super::{Box, Conversation, Duration, End, Env, Failure, Limits, OUT_OF_TIME, Rng, State, Time, finish};

pub(super) fn retry_decision(
    failure: Failure,
    attempt: u32,
    now: Time,
    expiry: Time,
    limits: &Limits,
    rng: &mut Rng,
) -> Retry {
    if now >= expiry {
        return Retry::TimeUp;
    }
    match failure {
        Failure::TimedOut { phase } => match phase {
            crate::llm::Phase::Whole => return Retry::TimeUp,
            crate::llm::Phase::Connect
            | crate::llm::Phase::Handshake
            | crate::llm::Phase::Head
            | crate::llm::Phase::Idle => {}
        },
        Failure::Overloaded
        | Failure::Unavailable
        | Failure::Unauthorized
        | Failure::RateLimited { .. }
        | Failure::ContextTooLong
        | Failure::Refused
        | Failure::Program { .. }
        | Failure::SessionGone
        | Failure::SignedOut
        | Failure::Unsupported
        | Failure::Invalid
        | Failure::Exhausted { .. }
        | Failure::Limit { .. }
        | Failure::ApplicationLimit
        | Failure::Protocol
        | Failure::Cancelled => {}
    }
    match backoff(failure, attempt, limits, rng) {
        Some(delay) => Retry::After(delay),
        None => Retry::Fail,
    }
}

/// How long to wait before retrying a call that failed with `failure` after
/// `attempt` retries, or `None` if it is not to be retried.
pub(super) fn backoff(failure: Failure, attempt: u32, limits: &Limits, rng: &mut Rng) -> Option<Duration> {
    let floor = match failure {
        Failure::Overloaded | Failure::Unavailable | Failure::TimedOut { .. } | Failure::Unauthorized => Duration::ZERO,
        Failure::RateLimited { retry_after } => retry_after,
        Failure::ContextTooLong
        | Failure::Refused
        | Failure::Program { .. }
        | Failure::SessionGone
        | Failure::SignedOut
        | Failure::Unsupported
        | Failure::Invalid
        | Failure::Exhausted { .. }
        | Failure::Limit { .. }
        | Failure::ApplicationLimit
        | Failure::Protocol
        | Failure::Cancelled => return None,
    };
    if attempt >= limits.retries {
        return None;
    }
    // Exponential and capped, with equal jitter: half fixed, half random.
    let factor = 1_u64.checked_shl(attempt).unwrap_or(u64::MAX);
    let ceiling = limits.backoff_base.saturating_mul(factor).min(limits.backoff_max);
    let half = ceiling.as_nanos() / 2;
    let jittered = Duration::from_nanos(half.saturating_add(rng.below(half.saturating_add(1))));
    Some(jittered.max(floor))
}

/// Completing, failed: wait and call again if the failure is transient and
/// retries remain; end otherwise.
pub(super) fn completion_failed(
    attempt: u32,
    failure: Failure,
    evidence: crate::llm::Evidence,
    retry_ok: bool,
    conversation: &Conversation,
    rng: &mut Rng,
    env: &Env<Limits>,
) -> State {
    let response = match evidence {
        crate::llm::Evidence::Response { .. } => true,
        crate::llm::Evidence::Unsent | crate::llm::Evidence::Unknown => false,
    };
    let decision = match failure {
        Failure::Unauthorized if conversation.account.is_none() || (!retry_ok && response) => Retry::Fail,
        Failure::Unauthorized
        | Failure::Overloaded
        | Failure::Unavailable
        | Failure::TimedOut { .. }
        | Failure::RateLimited { .. }
        | Failure::ContextTooLong
        | Failure::Refused
        | Failure::Program { .. }
        | Failure::SessionGone
        | Failure::SignedOut
        | Failure::Unsupported
        | Failure::Invalid
        | Failure::Exhausted { .. }
        | Failure::Limit { .. }
        | Failure::ApplicationLimit
        | Failure::Protocol
        | Failure::Cancelled => retry_decision(failure, attempt, env.now, conversation.expires, &env.limits, rng),
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

/// The session's policy decision after a failed attempt (smith/docs/design/domain/session.md, section 4).
pub(super) enum Retry {
    /// A transient failure waits for this backoff before another attempt.
    After(Duration),
    /// A lasting failure or exhausted retry allowance ends the session.
    Fail,
    /// The whole call or the session time ended; no retry is armed.
    TimeUp,
}
