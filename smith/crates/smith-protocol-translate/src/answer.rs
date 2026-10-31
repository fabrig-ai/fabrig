//! Domain answer and refusal translation (smith/docs/design/protocol/channel.md, section 5).
use crate::{Error, encode_result};
use skein_lib::Token;
use smith_channel as channel;
use smith_charter as wire;
use smith_domain::run;

/// Translate the domain's single Start terminal into the channel's last word.
#[expect(clippy::manual_map, reason = "the protocol subset keeps option projection exhaustive")]
pub fn answer_record(
    now: skein_lib::Time,
    answer: run::Answer,
    read: Option<Token>,
    bodies: &channel::Limits,
    charter: &wire::v4::Limits,
) -> Result<channel::Answer, Error> {
    let (turns, spent, result) = match answer {
        run::Answer::Refused(refusal) => {
            let reason = match refusal {
                run::Refusal::Busy => channel::StartRefusal::Busy,
                run::Refusal::Invalid(invalid) => {
                    let value = match channel::InvalidStartValue::new(
                        bodies,
                        channel::InvalidStartValueParts { value: invalid_start(invalid, bodies)? },
                    ) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::ChannelBody(problem)),
                    };
                    channel::StartRefusal::Invalid(value)
                }
            };
            let refused = match channel::Refused::new(bodies, channel::RefusedParts { reason }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            };
            (0, 0, channel::RunResult::Refused(refused))
        }
        run::Answer::Parked { spent, turns } => (turns, spent.units, channel::RunResult::Parked),
        run::Answer::Accepted { outcome, spent, turns } => {
            let result = encode_result(&outcome, charter)?;
            let accepted = match channel::Accepted::new(bodies, channel::AcceptedParts { result }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            };
            (turns, spent.units, channel::RunResult::Accepted(accepted))
        }
        run::Answer::Failed { failure, spent, turns } => {
            let reason = failure_record(failure, bodies, now)?;
            let failed = match channel::Failed::new(bodies, channel::FailedParts { reason }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            };
            (turns, spent.units, channel::RunResult::Failed(failed))
        }
    };
    let last_read = match read {
        Some(name) => Some(name.raw()),
        None => None,
    };
    Ok(match channel::Answer::new(bodies, channel::AnswerParts { turns, spent, last_read, result }) {
        Ok(value) => value,
        Err(problem) => return Err(Error::ChannelBody(problem)),
    })
}

/// Translate a refused charter into the channel refusal vocabulary.
pub fn invalid_start(invalid: run::Invalid, limits: &channel::Limits) -> Result<channel::InvalidStart, Error> {
    Ok(match invalid {
        run::Invalid::CharterVersion => channel::InvalidStart::CharterVersion,
        run::Invalid::MalformedCharter => channel::InvalidStart::MalformedCharter,
        run::Invalid::Endpoint => channel::InvalidStart::Endpoint,
        run::Invalid::Messages => channel::InvalidStart::Messages,
        run::Invalid::Activation => channel::InvalidStart::Activation,
        run::Invalid::Window => channel::InvalidStart::Window,
        run::Invalid::Conventions => channel::InvalidStart::Conventions,
        run::Invalid::TooLarge => channel::InvalidStart::TooLarge,
        run::Invalid::Workspace => channel::InvalidStart::Workspace,
        run::Invalid::Grants => channel::InvalidStart::Grants,
        run::Invalid::Outcome => channel::InvalidStart::Outcome,
        run::Invalid::Budget(refusal) => channel::InvalidStart::Budget(
            match channel::BudgetRefusalValue::new(
                limits,
                channel::BudgetRefusalValueParts { value: budget_refusal(refusal, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Invalid::Price(rate) => channel::InvalidStart::Price(
            match channel::PriceRateValue::new(limits, channel::PriceRateValueParts { value: price_rate(rate) }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Invalid::Llm => channel::InvalidStart::Llm,
        run::Invalid::Effort => channel::InvalidStart::Effort,
        run::Invalid::Conversation => channel::InvalidStart::Conversation,
        run::Invalid::HostPart { bound } => channel::InvalidStart::HostPart(
            match channel::WindowBound::new(limits, channel::WindowBoundParts { bound }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Invalid::Room { bound } => {
            channel::InvalidStart::Room(match channel::WindowBound::new(limits, channel::WindowBoundParts { bound }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            })
        }
    })
}

fn failure_record(
    failure: run::Failure,
    limits: &channel::Limits,
    now: skein_lib::Time,
) -> Result<channel::RunFailure, Error> {
    let record = match failure {
        run::Failure::Transcript(refusal) => {
            let value = match refusal {
                run::TranscriptRefusal::Version => channel::TranscriptRefusal::Version,
                run::TranscriptRefusal::Malformed => channel::TranscriptRefusal::Malformed,
                run::TranscriptRefusal::Unresolved => channel::TranscriptRefusal::Unresolved,
                run::TranscriptRefusal::TooLarge => channel::TranscriptRefusal::TooLarge,
            };
            channel::RunFailure::Transcript(
                match channel::TranscriptRefusalValue::new(limits, channel::TranscriptRefusalValueParts { value }) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            )
        }
        run::Failure::Model(fault) => {
            let value = model_fault(fault, now, limits)?;
            channel::RunFailure::Model(
                match channel::ModelFaultValue::new(limits, channel::ModelFaultValueParts { value }) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            )
        }
        run::Failure::Budget { exhausted, used, bound } => {
            let value = budget_failure(exhausted, limits)?;
            channel::RunFailure::Budget(
                match channel::BudgetFailureValue::new(limits, channel::BudgetFailureValueParts { value, used, bound })
                {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            )
        }
        run::Failure::Policy(run::Policy::Unfinished { nudges, rejected }) => channel::RunFailure::Policy(
            match channel::PolicyFailure::new(limits, channel::PolicyFailureParts { nudges, rejected }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Failure::Cancelled => channel::RunFailure::Cancelled,
        run::Failure::Stale => channel::RunFailure::Stale,
    };
    Ok(record)
}

#[expect(clippy::manual_map, reason = "the protocol subset projects the optional cooldown without a closure")]
fn model_fault(
    fault: run::Fault,
    now: skein_lib::Time,
    limits: &channel::Limits,
) -> Result<channel::ModelFault, Error> {
    let value = match fault {
        run::Fault::Completion { failure, evidence, model, attempts, report } => {
            let (failure, retry_after) = completion_failure(failure, limits)?;
            let evidence = match evidence {
                run::CompletionEvidence::Unsent => channel::CompletionEvidence::Unsent,
                run::CompletionEvidence::Unknown => channel::CompletionEvidence::Unknown,
                run::CompletionEvidence::Response { status } => channel::CompletionEvidence::Response(
                    match channel::ResponseEvidence::new(limits, channel::ResponseEvidenceParts { status }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::ChannelBody(problem)),
                    },
                ),
            };
            channel::ModelFault::Completion(
                match channel::CompletionFault::new(
                    limits,
                    channel::CompletionFaultParts {
                        failure,
                        evidence,
                        retry_after,
                        model,
                        attempts,
                        code: report.code,
                        request_id: report.request,
                        field: report.field,
                    },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            )
        }
        run::Fault::Limited { account, reset_at } => channel::ModelFault::Limited(
            match channel::LimitedFault::new(
                limits,
                channel::LimitedFaultParts {
                    account,
                    reset: match reset_at {
                        Some(at) => Some(at.saturating_since(now)),
                        None => None,
                    },
                },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Fault::Refused { account, code } => channel::ModelFault::Refused(
            match channel::RefusedFault::new(limits, channel::RefusedFaultParts { account, code }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Fault::Unauthorized { account } => channel::ModelFault::Unauthorized(
            match channel::UnauthorizedFault::new(limits, channel::UnauthorizedFaultParts { account }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Fault::Program { name, exit, detail } => channel::ModelFault::Program(
            match channel::ProgramFault::new(limits, channel::ProgramFaultParts { name, exit, detail }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Fault::Provider => channel::ModelFault::Provider,
        run::Fault::Declined => channel::ModelFault::Declined,
        run::Fault::Truncated => channel::ModelFault::Truncated,
        run::Fault::Malformed => channel::ModelFault::Malformed,
    };
    Ok(value)
}

fn budget_failure(exhausted: run::Exhausted, limits: &channel::Limits) -> Result<channel::BudgetFailure, Error> {
    let value = match exhausted {
        run::Exhausted::Turns => channel::BudgetFailure::Turns,
        run::Exhausted::Spend => channel::BudgetFailure::Spend,
        run::Exhausted::Time => channel::BudgetFailure::Time,
        run::Exhausted::Tokens(receiving) => {
            let value = match receiving {
                run::ReceivingLimit::Input => channel::ReceivingLimit::Input,
                run::ReceivingLimit::Output => channel::ReceivingLimit::Output,
                run::ReceivingLimit::CacheRead => channel::ReceivingLimit::CacheRead,
                run::ReceivingLimit::CacheWrite => channel::ReceivingLimit::CacheWrite,
            };
            channel::BudgetFailure::Tokens(
                match channel::ReceivingLimitValue::new(limits, channel::ReceivingLimitValueParts { value }) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            )
        }
        run::Exhausted::Overflow(overflow) => {
            let value = match overflow {
                run::Overflow::Spend => channel::BudgetOverflow::Spend,
                run::Overflow::Usage => channel::BudgetOverflow::Usage,
            };
            channel::BudgetFailure::Overflow(
                match channel::OverflowValue::new(limits, channel::OverflowValueParts { value }) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            )
        }
    };
    Ok(value)
}

fn price_rate(rate: run::Rate) -> channel::PriceRate {
    match rate {
        run::Rate::Input => channel::PriceRate::Input,
        run::Rate::Cached => channel::PriceRate::Cached,
        run::Rate::CacheWrite => channel::PriceRate::CacheWrite,
        run::Rate::Output => channel::PriceRate::Output,
    }
}

fn budget_part(part: run::Part) -> channel::BudgetPart {
    match part {
        run::Part::Turns => channel::BudgetPart::Turns,
        run::Part::Spend => channel::BudgetPart::Spend,
        run::Part::Time => channel::BudgetPart::Time,
    }
}

fn budget_refusal(refusal: run::BudgetRefusal, limits: &channel::Limits) -> Result<channel::BudgetRefusal, Error> {
    Ok(match refusal {
        run::BudgetRefusal::Ceiling(ceiling) => {
            let value = match ceiling {
                run::Ceiling::Turns(value) => channel::BudgetCeiling::Turns(
                    match channel::BudgetTurns::new(limits, channel::BudgetTurnsParts { value }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::ChannelBody(problem)),
                    },
                ),
                run::Ceiling::Spend(value) => channel::BudgetCeiling::Spend(
                    match channel::BudgetSpend::new(limits, channel::BudgetSpendParts { value }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::ChannelBody(problem)),
                    },
                ),
                run::Ceiling::Time(value) => channel::BudgetCeiling::Time(
                    match channel::BudgetTime::new(limits, channel::BudgetTimeParts { value }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::ChannelBody(problem)),
                    },
                ),
            };
            channel::BudgetRefusal::Ceiling(
                match channel::BudgetCeilingValue::new(limits, channel::BudgetCeilingValueParts { value }) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            )
        }
        run::BudgetRefusal::Empty(part) => channel::BudgetRefusal::Empty(
            match channel::BudgetPartValue::new(limits, channel::BudgetPartValueParts { value: budget_part(part) }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::BudgetRefusal::Reserve(part) => channel::BudgetRefusal::Reserve(
            match channel::BudgetPartValue::new(limits, channel::BudgetPartValueParts { value: budget_part(part) }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::BudgetRefusal::Completion { most } => channel::BudgetRefusal::Completion(
            match channel::BudgetCompletion::new(limits, channel::BudgetCompletionParts { most }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::BudgetRefusal::Checks { reserve, checks } => channel::BudgetRefusal::Checks(
            match channel::BudgetChecks::new(limits, channel::BudgetChecksParts { reserve, checks }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
    })
}

fn completion_limit(which: run::Limit, limits: &channel::Limits) -> Result<channel::CompletionLimit, Error> {
    let which = match which {
        run::Limit::Pool { reservation } => channel::CompletionLimit::Pool(
            match channel::PoolLimit::new(limits, channel::PoolLimitParts { reservation }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Limit::Path(cap) => channel::CompletionLimit::Path(
            match channel::PathCap::new(limits, channel::PathCapParts { value: completion_cap(cap) }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::ChannelBody(problem)),
            },
        ),
        run::Limit::History => channel::CompletionLimit::History,
        run::Limit::Window => channel::CompletionLimit::Window,
        run::Limit::Call => channel::CompletionLimit::Call,
        run::Limit::Message => channel::CompletionLimit::Message,
    };
    Ok(which)
}

const fn completion_cap(cap: run::Cap) -> channel::Cap {
    match cap {
        run::Cap::Request => channel::Cap::Request,
        run::Cap::RequestHead => channel::Cap::RequestHead,
        run::Cap::ResponseHead => channel::Cap::ResponseHead,
        run::Cap::ResponseFields => channel::Cap::ResponseFields,
        run::Cap::Skip => channel::Cap::Skip,
        run::Cap::Field => channel::Cap::Field,
        run::Cap::Retained => channel::Cap::Retained,
        run::Cap::Tokens => channel::Cap::Tokens,
        run::Cap::Depth => channel::Cap::Depth,
        run::Cap::Strings => channel::Cap::Strings,
        run::Cap::Number => channel::Cap::Number,
        run::Cap::Reasoning => channel::Cap::Reasoning,
        run::Cap::OutputItems => channel::Cap::OutputItems,
        run::Cap::Metadata => channel::Cap::Metadata,
        run::Cap::Tools => channel::Cap::Tools,
        run::Cap::HistoryItems => channel::Cap::HistoryItems,
        run::Cap::Output => channel::Cap::Output,
        run::Cap::Answer => channel::Cap::Answer,
        run::Cap::ErrorBody => channel::Cap::ErrorBody,
        run::Cap::AccessToken => channel::Cap::AccessToken,
        run::Cap::AccountId => channel::Cap::AccountId,
    }
}

const fn completion_phase(phase: run::Phase) -> channel::CompletionPhase {
    match phase {
        run::Phase::Connect => channel::CompletionPhase::Connect,
        run::Phase::Handshake => channel::CompletionPhase::Handshake,
        run::Phase::Head => channel::CompletionPhase::Head,
        run::Phase::Idle => channel::CompletionPhase::Idle,
        run::Phase::Whole => channel::CompletionPhase::Whole,
    }
}

fn completion_failure(
    failure: run::CompletionFailure,
    limits: &channel::Limits,
) -> Result<(channel::CompletionFailure, Option<skein_lib::Duration>), Error> {
    Ok(match failure {
        run::CompletionFailure::Limit { which, bound } => (
            channel::CompletionFailure::Limit(
                match channel::LimitFailure::new(
                    limits,
                    channel::LimitFailureParts { which: completion_limit(which, limits)?, bound },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            ),
            None,
        ),
        run::CompletionFailure::ApplicationLimit => (channel::CompletionFailure::ApplicationLimit, None),
        run::CompletionFailure::Protocol => (channel::CompletionFailure::Protocol, None),
        run::CompletionFailure::Cancelled => (channel::CompletionFailure::Cancelled, None),
        run::CompletionFailure::Overloaded => (channel::CompletionFailure::Overloaded, None),
        run::CompletionFailure::Unavailable => (channel::CompletionFailure::Unavailable, None),
        run::CompletionFailure::TimedOut { phase } => (
            channel::CompletionFailure::TimedOut(
                match channel::TimeoutFailure::new(
                    limits,
                    channel::TimeoutFailureParts { phase: completion_phase(phase) },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::ChannelBody(problem)),
                },
            ),
            None,
        ),
        run::CompletionFailure::ContextTooLong => (channel::CompletionFailure::ContextTooLong, None),
        run::CompletionFailure::Invalid => (channel::CompletionFailure::Invalid, None),
        run::CompletionFailure::Refused => (channel::CompletionFailure::Refused, None),
        run::CompletionFailure::Unsupported => (channel::CompletionFailure::Unsupported, None),
        run::CompletionFailure::Unauthorized => (channel::CompletionFailure::Unauthorized, None),
        run::CompletionFailure::RateLimited { retry_after } => {
            (channel::CompletionFailure::RateLimited, Some(retry_after))
        }
        run::CompletionFailure::Exhausted { retry_after } => (channel::CompletionFailure::Exhausted, retry_after),
    })
}
