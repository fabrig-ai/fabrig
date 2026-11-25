//! Turn the wire last word into the host domain's concrete result (protocol/channel.md, sections 4 and 5;
//! domain/host.md, sections 2 and 6).
//!
//! This module keeps no state, never learns policy or credential values, and
//! enters through `decode_answer` after the channel body decoder succeeds.

use smith_channel as wire;
use smith_host_domain::channel as host;

use crate::{Error, generated::channel};

pub(crate) fn decode_answer(record: &wire::Answer, limits: &smith_charter::v5::Limits) -> Result<host::Answer, Error> {
    let result = match &record.result {
        wire::RunResult::Refused(refused) => {
            let refusal = channel::start_refusal_to_domain(refused.reason.clone());
            host::RunResult::Refused { refusal }
        }
        wire::RunResult::Accepted(accepted) => {
            let Ok(value) = smith_protocol_translate::decode_declared(accepted.result.as_ref(), limits) else {
                return Err(Error::InvalidAnswer);
            };
            host::RunResult::Accepted {
                outcome: smith_host_domain::Declared::new(value, u64::MAX).ok_or(Error::InvalidAnswer)?,
            }
        }
        wire::RunResult::Parked => host::RunResult::Parked,
        wire::RunResult::Failed(failed) => host::RunResult::Failed { failure: failure(&failed.reason)? },
    };
    Ok(host::Answer {
        read: match &record.last_read {
            Some(name) => Some(skein_lib::Token::new(*name)),
            None => None,
        },
        turns: record.turns,
        spent: record.spent,
        result,
    })
}

fn failure(source: &wire::RunFailure) -> Result<host::RunFailure, Error> {
    let result = match source {
        wire::RunFailure::Transcript(record) => {
            host::RunFailure::Transcript(channel::transcript_refusal_to_domain(record.value.clone()))
        }
        wire::RunFailure::Model(record) => host::RunFailure::Model(model_fault(record)?),
        wire::RunFailure::Budget(record) => {
            let exhausted = channel::budget_failure_to_domain(record.value.clone());
            match exhausted {
                host::Exhausted::Turns | host::Exhausted::Spend | host::Exhausted::Time => {
                    if record.used.is_none() || record.bound.is_none() {
                        return Err(Error::InvalidAnswer);
                    }
                }
                host::Exhausted::Tokens(_) | host::Exhausted::Overflow(_) => {
                    if record.used.is_some() || record.bound.is_some() {
                        return Err(Error::InvalidAnswer);
                    }
                }
            }
            host::RunFailure::Budget { exhausted, used: record.used, bound: record.bound }
        }
        wire::RunFailure::Policy(record) => {
            host::RunFailure::Policy(host::Policy::Unfinished { nudges: record.nudges, rejected: record.rejected })
        }
        wire::RunFailure::Cancelled => host::RunFailure::Cancelled,
        wire::RunFailure::Stale => host::RunFailure::Stale,
    };
    Ok(result)
}

fn model_fault(record: &wire::ModelFaultValue) -> Result<host::ModelFault, Error> {
    let result = match &record.value {
        wire::ModelFault::Completion(record) => {
            let failure = match &record.failure {
                wire::CompletionFailure::Limit(record) => host::CompletionFailure::Limit {
                    which: channel::completion_limit_to_domain(record.which.clone()),
                    bound: record.bound,
                },
                wire::CompletionFailure::ApplicationLimit => host::CompletionFailure::ApplicationLimit,
                wire::CompletionFailure::Protocol => host::CompletionFailure::Protocol,
                wire::CompletionFailure::Cancelled => host::CompletionFailure::Cancelled,
                wire::CompletionFailure::Overloaded => host::CompletionFailure::Overloaded,
                wire::CompletionFailure::Unavailable => host::CompletionFailure::Unavailable,
                wire::CompletionFailure::TimedOut(record) => host::CompletionFailure::TimedOut {
                    phase: channel::completion_phase_to_domain(record.phase.clone()),
                },
                wire::CompletionFailure::ContextTooLong => host::CompletionFailure::ContextTooLong,
                wire::CompletionFailure::Refused => host::CompletionFailure::Refused,
                wire::CompletionFailure::Unsupported => host::CompletionFailure::Unsupported,
                wire::CompletionFailure::Invalid => host::CompletionFailure::Invalid,
                wire::CompletionFailure::Unauthorized => host::CompletionFailure::Unauthorized,
                wire::CompletionFailure::RateLimited => host::CompletionFailure::RateLimited {
                    retry_after: record.retry_after.ok_or(Error::InvalidAnswer)?,
                },
                wire::CompletionFailure::Exhausted => {
                    host::CompletionFailure::Exhausted { retry_after: record.retry_after }
                }
            };
            let evidence = channel::completion_evidence_to_domain(record.evidence.clone());
            host::ModelFault::Completion {
                failure,
                evidence,
                model: record.model,
                attempts: record.attempts,
                report: host::Report {
                    status: record.status,
                    code: record.code.as_ref().into(),
                    request: record.request_id.as_ref().into(),
                    field: record.field.as_ref().into(),
                },
            }
        }
        wire::ModelFault::Limited(value) => host::ModelFault::Limited { account: value.account, reset: value.reset },
        wire::ModelFault::Refused(value) => {
            host::ModelFault::Refused { account: value.account, code: value.code.as_ref().into() }
        }
        wire::ModelFault::Unauthorized(value) => host::ModelFault::Unauthorized { account: value.account },
        wire::ModelFault::Program(value) => host::ModelFault::Program {
            name: value.name.as_ref().into(),
            exit: value.exit,
            detail: value.detail.as_ref().into(),
        },
        wire::ModelFault::Untooled(value) => {
            host::ModelFault::Untooled { name: value.name.as_ref().into(), detail: value.detail.as_ref().into() }
        }
        wire::ModelFault::Provider => host::ModelFault::Provider,
        wire::ModelFault::Declined => host::ModelFault::Declined,
        wire::ModelFault::Truncated => host::ModelFault::Truncated,
        wire::ModelFault::Malformed => host::ModelFault::Malformed,
    };
    Ok(result)
}
