//! Projects completion failures once for run answers and provider observations.
//! Sibling vocabulary is translated without changing retry, status or evidence.
//! No projection selects a domain outcome.
//! Contract: smith/docs/design/protocol/events.md, 3.3 and 3.10.

use smith_domain::{llm, run};
use smith_events::{CompletionFailure, Evidence, FailureClass};

pub(crate) fn from_llm(why: llm::Failure, evidence: llm::Evidence) -> CompletionFailure {
    from_run(failure(why), completion_evidence(evidence))
}

pub(crate) fn from_run(why: run::CompletionFailure, evidence: run::CompletionEvidence) -> CompletionFailure {
    let (class, retry_after_ms) = match why {
        run::CompletionFailure::Limit { .. } | run::CompletionFailure::ApplicationLimit => (FailureClass::Limit, None),
        run::CompletionFailure::Protocol => (FailureClass::Protocol, None),
        run::CompletionFailure::Cancelled => (FailureClass::Cancelled, None),
        run::CompletionFailure::Overloaded => (FailureClass::Overloaded, None),
        run::CompletionFailure::Unavailable => (FailureClass::Unavailable, None),
        run::CompletionFailure::TimedOut { .. } => (FailureClass::TimedOut, None),
        run::CompletionFailure::ContextTooLong => (FailureClass::ContextTooLong, None),
        run::CompletionFailure::Unsupported => (FailureClass::Unsupported, None),
        run::CompletionFailure::Invalid => (FailureClass::Invalid, None),
        run::CompletionFailure::Refused => (FailureClass::Refused, None),
        run::CompletionFailure::Unauthorized => (FailureClass::Unauthorized, None),
        run::CompletionFailure::RateLimited { retry_after } => {
            (FailureClass::RateLimited, Some(retry_after.as_millis()))
        }
        run::CompletionFailure::Exhausted { retry_after } => (
            FailureClass::Exhausted,
            match retry_after {
                Some(reset) => Some(reset.as_millis()),
                None => None,
            },
        ),
    };
    let status = match evidence {
        run::CompletionEvidence::Unsent | run::CompletionEvidence::Unknown => None,
        run::CompletionEvidence::Response { status } => match status {
            Some(status) => Some(u64::from(status)),
            None => None,
        },
    };
    let phase = match why {
        run::CompletionFailure::TimedOut { phase } => Some(match phase {
            run::CompletionPhase::Connect => smith_events::FailurePhase::Connect,
            run::CompletionPhase::Handshake => smith_events::FailurePhase::Handshake,
            run::CompletionPhase::Head => smith_events::FailurePhase::Head,
            run::CompletionPhase::Idle => smith_events::FailurePhase::Idle,
            run::CompletionPhase::Whole => smith_events::FailurePhase::Whole,
        }),
        run::CompletionFailure::Limit { .. }
        | run::CompletionFailure::ApplicationLimit
        | run::CompletionFailure::Protocol
        | run::CompletionFailure::Cancelled
        | run::CompletionFailure::Overloaded
        | run::CompletionFailure::Unavailable
        | run::CompletionFailure::ContextTooLong
        | run::CompletionFailure::Refused
        | run::CompletionFailure::Unsupported
        | run::CompletionFailure::Invalid
        | run::CompletionFailure::Unauthorized
        | run::CompletionFailure::RateLimited { .. }
        | run::CompletionFailure::Exhausted { .. } => None,
    };
    let limit = match why {
        run::CompletionFailure::Limit { which, bound } => {
            Some(smith_events::FailureLimit { which: limit_name(which).into(), bound })
        }
        run::CompletionFailure::ApplicationLimit
        | run::CompletionFailure::Protocol
        | run::CompletionFailure::Cancelled
        | run::CompletionFailure::Overloaded
        | run::CompletionFailure::Unavailable
        | run::CompletionFailure::TimedOut { .. }
        | run::CompletionFailure::ContextTooLong
        | run::CompletionFailure::Refused
        | run::CompletionFailure::Unsupported
        | run::CompletionFailure::Invalid
        | run::CompletionFailure::Unauthorized
        | run::CompletionFailure::RateLimited { .. }
        | run::CompletionFailure::Exhausted { .. } => None,
    };
    let evidence = match evidence {
        run::CompletionEvidence::Unsent => Evidence::Unsent,
        run::CompletionEvidence::Unknown => Evidence::MaybeSent,
        run::CompletionEvidence::Response { .. } => Evidence::Response,
    };
    CompletionFailure {
        field: None,
        code: None,
        request_id: None,
        class,
        evidence,
        retry_after_ms,
        phase,
        limit,
        status,
        detail: None,
    }
}

fn limit_name(which: run::Limit) -> &'static [u8] {
    match which {
        run::Limit::Pool { .. } => b"pool",
        run::Limit::History => b"history",
        run::Limit::Window => b"window",
        run::Limit::Call => b"call",
        run::Limit::Message => b"message",
        run::Limit::Path(which) => match which {
            run::Cap::Request => b"request",
            run::Cap::RequestHead => b"request_head",
            run::Cap::ResponseHead => b"response_head",
            run::Cap::ResponseFields => b"response_fields",
            run::Cap::Skip => b"skip",
            run::Cap::Field => b"field",
            run::Cap::Retained => b"retained",
            run::Cap::Tokens => b"tokens",
            run::Cap::Depth => b"depth",
            run::Cap::Strings => b"strings",
            run::Cap::Number => b"number",
            run::Cap::Reasoning => b"reasoning",
            run::Cap::OutputItems => b"output_items",
            run::Cap::Metadata => b"metadata",
            run::Cap::Tools => b"tools",
            run::Cap::HistoryItems => b"history_items",
            run::Cap::Output => b"output",
            run::Cap::Answer => b"answer",
            run::Cap::ErrorBody => b"error_body",
            run::Cap::AccessToken => b"access_token",
        },
    }
}

fn failure(failure: llm::Failure) -> run::CompletionFailure {
    match failure {
        llm::Failure::Limit { which, bound } => run::CompletionFailure::Limit { which: completion_limit(which), bound },
        llm::Failure::ApplicationLimit => run::CompletionFailure::ApplicationLimit,
        llm::Failure::Protocol => run::CompletionFailure::Protocol,
        llm::Failure::Cancelled => run::CompletionFailure::Cancelled,
        llm::Failure::Overloaded => run::CompletionFailure::Overloaded,
        llm::Failure::Unavailable => run::CompletionFailure::Unavailable,
        llm::Failure::TimedOut { phase } => run::CompletionFailure::TimedOut { phase: completion_phase(phase) },
        llm::Failure::ContextTooLong => run::CompletionFailure::ContextTooLong,
        llm::Failure::Invalid | llm::Failure::Program { .. } | llm::Failure::SessionGone => {
            run::CompletionFailure::Invalid
        }
        llm::Failure::Refused => run::CompletionFailure::Refused,
        llm::Failure::Unsupported => run::CompletionFailure::Unsupported,
        llm::Failure::SignedOut | llm::Failure::Unauthorized => run::CompletionFailure::Unauthorized,
        llm::Failure::RateLimited { retry_after } => run::CompletionFailure::RateLimited { retry_after },
        llm::Failure::Exhausted { retry_after } => run::CompletionFailure::Exhausted { retry_after },
    }
}

/// Transport evidence is copied exactly, without inferring absence of effects.
fn completion_evidence(evidence: llm::Evidence) -> run::CompletionEvidence {
    match evidence {
        llm::Evidence::Unsent => run::CompletionEvidence::Unsent,
        llm::Evidence::Unknown => run::CompletionEvidence::Unknown,
        llm::Evidence::Response { status } => run::CompletionEvidence::Response { status },
    }
}

const fn completion_limit(which: llm::Limit) -> run::Limit {
    match which {
        llm::Limit::Pool { reservation } => run::Limit::Pool { reservation },
        llm::Limit::Path(cap) => run::Limit::Path(completion_cap(cap)),
        llm::Limit::History => run::Limit::History,
        llm::Limit::Window => run::Limit::Window,
        llm::Limit::Call => run::Limit::Call,
        llm::Limit::Message => run::Limit::Message,
    }
}

const fn completion_cap(cap: llm::Cap) -> run::Cap {
    match cap {
        llm::Cap::Request => run::Cap::Request,
        llm::Cap::RequestHead => run::Cap::RequestHead,
        llm::Cap::ResponseHead => run::Cap::ResponseHead,
        llm::Cap::ResponseFields => run::Cap::ResponseFields,
        llm::Cap::Skip => run::Cap::Skip,
        llm::Cap::Field => run::Cap::Field,
        llm::Cap::Retained => run::Cap::Retained,
        llm::Cap::Tokens => run::Cap::Tokens,
        llm::Cap::Depth => run::Cap::Depth,
        llm::Cap::Strings => run::Cap::Strings,
        llm::Cap::Number => run::Cap::Number,
        llm::Cap::Reasoning => run::Cap::Reasoning,
        llm::Cap::OutputItems => run::Cap::OutputItems,
        llm::Cap::Metadata => run::Cap::Metadata,
        llm::Cap::Tools => run::Cap::Tools,
        llm::Cap::HistoryItems => run::Cap::HistoryItems,
        llm::Cap::Output => run::Cap::Output,
        llm::Cap::Answer => run::Cap::Answer,
        llm::Cap::ErrorBody => run::Cap::ErrorBody,
        llm::Cap::AccessToken => run::Cap::AccessToken,
    }
}

const fn completion_phase(phase: llm::CompletionPhase) -> run::CompletionPhase {
    match phase {
        llm::CompletionPhase::Connect => run::CompletionPhase::Connect,
        llm::CompletionPhase::Handshake => run::CompletionPhase::Handshake,
        llm::CompletionPhase::Head => run::CompletionPhase::Head,
        llm::CompletionPhase::Idle => run::CompletionPhase::Idle,
        llm::CompletionPhase::Whole => run::CompletionPhase::Whole,
    }
}
