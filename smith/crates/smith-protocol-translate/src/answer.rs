//! Domain answer and refusal translation (smith/docs/design/protocol/channel.md, section 5).
use crate::{Error, encode_result};
use channel::Record;
use skein_lib::Token;
use smith_channel as channel;
use smith_charter as wire;
use smith_domain::run;

/// Translate the domain's single Start terminal into the channel's last word.
pub fn answer_record(
    now: skein_lib::Time,
    answer: run::Answer,
    read: Option<Token>,
    bodies: &channel::Limits,
    charter: &wire::v5::Limits,
) -> Result<channel::Answer, Error> {
    let (turns, spent, result) = match answer {
        run::Answer::Refused(refusal) => {
            let reason = match refusal {
                run::Refusal::Busy => channel::StartRefusal::Busy,
                run::Refusal::Invalid(invalid) => {
                    let value = channel::InvalidStartValue { value: invalid_start(invalid, bodies)? };
                    channel::StartRefusal::Invalid(value)
                }
            };
            let refused = channel::Refused { reason };
            (0, 0, channel::RunResult::Refused(refused))
        }
        run::Answer::Parked { spent, turns } => (turns, spent.units, channel::RunResult::Parked),
        run::Answer::Accepted { outcome, spent, turns } => {
            let result = encode_result(&outcome, charter)?;
            let accepted = channel::Accepted { result };
            (turns, spent.units, channel::RunResult::Accepted(accepted))
        }
        run::Answer::Failed { failure, spent, turns } => {
            let reason = failure_record(failure, now);
            let failed = channel::Failed { reason };
            (turns, spent.units, channel::RunResult::Failed(failed))
        }
    };
    let last_read = match read {
        Some(name) => Some(name.raw()),
        None => None,
    };
    let record = channel::Answer { turns, spent, last_read, result };
    match record.measure(bodies) {
        Ok(_) => Ok(record),
        Err(problem) => Err(Error::ChannelBody(problem)),
    }
}

/// Translate a refused charter into the channel refusal vocabulary.
pub fn invalid_start(invalid: run::Invalid, limits: &channel::Limits) -> Result<channel::InvalidStart, Error> {
    let value = match invalid {
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
        run::Invalid::Budget(refusal) => {
            channel::InvalidStart::Budget(channel::BudgetRefusalValue { value: budget_refusal(refusal) })
        }
        run::Invalid::Price(rate) => channel::InvalidStart::Price(channel::PriceRateValue { value: price_rate(rate) }),
        run::Invalid::Llm => channel::InvalidStart::Llm,
        run::Invalid::Effort => channel::InvalidStart::Effort,
        run::Invalid::Conversation => channel::InvalidStart::Conversation,
        run::Invalid::HostPart { bound } => channel::InvalidStart::HostPart(channel::WindowBound { bound }),
        run::Invalid::Room { bound } => channel::InvalidStart::Room(channel::WindowBound { bound }),
    };
    match value.measure(limits) {
        Ok(_) => Ok(value),
        Err(problem) => Err(Error::ChannelBody(problem)),
    }
}

fn failure_record(failure: run::RunFailure, now: skein_lib::Time) -> channel::RunFailure {
    match failure {
        run::RunFailure::Transcript(refusal) => {
            let value = match refusal {
                run::TranscriptRefusal::Version => channel::TranscriptRefusal::Version,
                run::TranscriptRefusal::Malformed => channel::TranscriptRefusal::Malformed,
                run::TranscriptRefusal::Unresolved => channel::TranscriptRefusal::Unresolved,
                run::TranscriptRefusal::TooLarge => channel::TranscriptRefusal::TooLarge,
            };
            channel::RunFailure::Transcript(channel::TranscriptRefusalValue { value })
        }
        run::RunFailure::Model(fault) => {
            let value = model_fault(fault, now);
            channel::RunFailure::Model(channel::ModelFaultValue { value })
        }
        run::RunFailure::Budget { exhausted, used, bound } => {
            let value = budget_failure(exhausted);
            channel::RunFailure::Budget(channel::BudgetFailureValue { value, used, bound })
        }
        run::RunFailure::Policy(run::Policy::Unfinished { nudges, rejected }) => {
            channel::RunFailure::Policy(channel::PolicyFailure { nudges, rejected })
        }
        run::RunFailure::Cancelled => channel::RunFailure::Cancelled,
        run::RunFailure::Stale => channel::RunFailure::Stale,
    }
}

fn model_fault(fault: run::Fault, now: skein_lib::Time) -> channel::ModelFault {
    match fault {
        run::Fault::Completion { failure, evidence, model, attempts, report } => {
            let (failure, retry_after) = completion_failure(failure);
            let evidence = match evidence {
                run::CompletionEvidence::Unsent => channel::CompletionEvidence::Unsent,
                run::CompletionEvidence::Unknown => channel::CompletionEvidence::Unknown,
                run::CompletionEvidence::Response { status } => {
                    channel::CompletionEvidence::Response(channel::ResponseEvidence { status })
                }
            };
            channel::ModelFault::Completion(channel::CompletionFault {
                failure,
                evidence,
                retry_after,
                model,
                attempts,
                code: report.code,
                request_id: report.request,
                field: report.field,
                status: report.status,
            })
        }
        run::Fault::Limited { account, reset_at } => channel::ModelFault::Limited(channel::LimitedFault {
            account,
            reset: match reset_at {
                Some(at) => Some(at.saturating_since(now)),
                None => None,
            },
        }),
        run::Fault::Refused { account, code } => channel::ModelFault::Refused(channel::RefusedFault { account, code }),
        run::Fault::Unauthorized { account } => {
            channel::ModelFault::Unauthorized(channel::UnauthorizedFault { account })
        }
        run::Fault::Program { name, exit, detail } => {
            channel::ModelFault::Program(channel::ProgramFault { name, exit, detail })
        }
        run::Fault::Untooled { name, detail } => {
            channel::ModelFault::Untooled(channel::ProgramFault { name, exit: None, detail })
        }
        run::Fault::Provider => channel::ModelFault::Provider,
        run::Fault::Declined => channel::ModelFault::Declined,
        run::Fault::Truncated => channel::ModelFault::Truncated,
        run::Fault::Malformed => channel::ModelFault::Malformed,
    }
}

fn budget_failure(exhausted: run::Exhausted) -> channel::BudgetFailure {
    match exhausted {
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
            channel::BudgetFailure::Tokens(channel::ReceivingLimitValue { value })
        }
        run::Exhausted::Overflow(overflow) => {
            let value = match overflow {
                run::Overflow::Spend => channel::BudgetOverflow::Spend,
                run::Overflow::Usage => channel::BudgetOverflow::Usage,
            };
            channel::BudgetFailure::Overflow(channel::OverflowValue { value })
        }
    }
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

fn budget_refusal(refusal: run::BudgetRefusal) -> channel::BudgetRefusal {
    match refusal {
        run::BudgetRefusal::Ceiling(ceiling) => {
            let value = match ceiling {
                run::Ceiling::Turns(value) => channel::BudgetCeiling::Turns(channel::BudgetTurns { value }),
                run::Ceiling::Spend(value) => channel::BudgetCeiling::Spend(channel::BudgetSpend { value }),
                run::Ceiling::Time(value) => channel::BudgetCeiling::Time(channel::BudgetTime { value }),
            };
            channel::BudgetRefusal::Ceiling(channel::BudgetCeilingValue { value })
        }
        run::BudgetRefusal::Empty(part) => {
            channel::BudgetRefusal::Empty(channel::BudgetPartValue { value: budget_part(part) })
        }
        run::BudgetRefusal::Reserve(part) => {
            channel::BudgetRefusal::Reserve(channel::BudgetPartValue { value: budget_part(part) })
        }
        run::BudgetRefusal::Completion { most } => {
            channel::BudgetRefusal::Completion(channel::BudgetCompletion { most })
        }
        run::BudgetRefusal::Checks { reserve, checks } => {
            channel::BudgetRefusal::Checks(channel::BudgetChecks { reserve, checks })
        }
    }
}

fn completion_limit(which: run::Limit) -> channel::CompletionLimit {
    match which {
        run::Limit::Pool { reservation } => channel::CompletionLimit::Pool(channel::PoolLimit { reservation }),
        run::Limit::Path(cap) => channel::CompletionLimit::Path(channel::PathCap { value: completion_cap(cap) }),
        run::Limit::History => channel::CompletionLimit::History,
        run::Limit::Window => channel::CompletionLimit::Window,
        run::Limit::Call => channel::CompletionLimit::Call,
        run::Limit::Message => channel::CompletionLimit::Message,
    }
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
    }
}

const fn completion_phase(phase: run::CompletionPhase) -> channel::CompletionPhase {
    match phase {
        run::CompletionPhase::Connect => channel::CompletionPhase::Connect,
        run::CompletionPhase::Handshake => channel::CompletionPhase::Handshake,
        run::CompletionPhase::Head => channel::CompletionPhase::Head,
        run::CompletionPhase::Idle => channel::CompletionPhase::Idle,
        run::CompletionPhase::Whole => channel::CompletionPhase::Whole,
    }
}

fn completion_failure(failure: run::CompletionFailure) -> (channel::CompletionFailure, Option<skein_lib::Duration>) {
    match failure {
        run::CompletionFailure::Limit { which, bound } => {
            (channel::CompletionFailure::Limit(channel::LimitFailure { which: completion_limit(which), bound }), None)
        }
        run::CompletionFailure::ApplicationLimit => (channel::CompletionFailure::ApplicationLimit, None),
        run::CompletionFailure::Protocol => (channel::CompletionFailure::Protocol, None),
        run::CompletionFailure::Cancelled => (channel::CompletionFailure::Cancelled, None),
        run::CompletionFailure::Overloaded => (channel::CompletionFailure::Overloaded, None),
        run::CompletionFailure::Unavailable => (channel::CompletionFailure::Unavailable, None),
        run::CompletionFailure::TimedOut { phase } => {
            (channel::CompletionFailure::TimedOut(channel::TimeoutFailure { phase: completion_phase(phase) }), None)
        }
        run::CompletionFailure::ContextTooLong => (channel::CompletionFailure::ContextTooLong, None),
        run::CompletionFailure::Invalid => (channel::CompletionFailure::Invalid, None),
        run::CompletionFailure::Refused => (channel::CompletionFailure::Refused, None),
        run::CompletionFailure::Unsupported => (channel::CompletionFailure::Unsupported, None),
        run::CompletionFailure::Unauthorized => (channel::CompletionFailure::Unauthorized, None),
        run::CompletionFailure::RateLimited { retry_after } => {
            (channel::CompletionFailure::RateLimited, Some(retry_after))
        }
        run::CompletionFailure::Exhausted { retry_after } => (channel::CompletionFailure::Exhausted, retry_after),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untooled_answer_names_the_program_without_an_exit() {
        let detail = b"smith: the program started without smith's tools";
        let answer = answer_record(
            skein_lib::Time::ZERO,
            run::Answer::Failed {
                failure: run::RunFailure::Model(run::Fault::Untooled {
                    name: b"claude_code".as_slice().into(),
                    detail: detail.as_slice().into(),
                }),
                spent: run::Spend::ZERO,
                turns: 0,
            },
            None,
            &channel::CEILINGS,
            &wire::CEILINGS,
        )
        .expect("answer within bounds");
        let channel::RunResult::Failed(failed) = answer.result else { panic!("failed answer") };
        let channel::RunFailure::Model(model) = failed.reason else { panic!("model fault") };
        let channel::ModelFault::Untooled(program) = model.value else { panic!("untooled fault") };
        assert_eq!(program.name.as_ref(), b"claude_code");
        assert_eq!(program.exit, None);
        assert_eq!(program.detail.as_ref(), detail);
    }
}
