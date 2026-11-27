//! Domain answer and refusal translation (smith/docs/design/protocol/channel.md, section 5).
use crate::{Error, encode_result, generated::channel as mapped};
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
            let reason = mapped::start_refusal_to_record(&refusal);
            match &reason {
                channel::StartRefusal::Busy => (),
                channel::StartRefusal::Invalid(invalid) => check_invalid_start(&invalid.value, bodies)?,
            }
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
    let value = crate::channel_hand::invalid_start_to_record(&invalid);
    check_invalid_start(&value, limits)?;
    Ok(value)
}

fn check_invalid_start(value: &channel::InvalidStart, limits: &channel::Limits) -> Result<(), Error> {
    match value.measure(limits) {
        Ok(_) => Ok(()),
        Err(problem) => Err(Error::ChannelBody(problem)),
    }
}

fn failure_record(failure: run::RunFailure, now: skein_lib::Time) -> channel::RunFailure {
    match failure {
        run::RunFailure::Transcript(refusal) => {
            let value = mapped::transcript_refusal_to_record(&refusal);
            channel::RunFailure::Transcript(channel::TranscriptRefusalValue { value })
        }
        run::RunFailure::Model(fault) => {
            let value = model_fault(fault, now);
            channel::RunFailure::Model(channel::ModelFaultValue { value })
        }
        run::RunFailure::Budget { exhausted, used, bound } => {
            let value = mapped::budget_failure_to_record(&exhausted);
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
            let evidence = mapped::completion_evidence_to_record(&evidence);
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

fn completion_failure(failure: run::CompletionFailure) -> (channel::CompletionFailure, Option<skein_lib::Duration>) {
    match failure {
        run::CompletionFailure::Limit { which, bound } => (
            channel::CompletionFailure::Limit(channel::LimitFailure {
                which: mapped::completion_limit_to_record(&which),
                bound,
            }),
            None,
        ),
        run::CompletionFailure::ApplicationLimit => (channel::CompletionFailure::ApplicationLimit, None),
        run::CompletionFailure::Protocol => (channel::CompletionFailure::Protocol, None),
        run::CompletionFailure::Cancelled => (channel::CompletionFailure::Cancelled, None),
        run::CompletionFailure::Overloaded => (channel::CompletionFailure::Overloaded, None),
        run::CompletionFailure::Unavailable => (channel::CompletionFailure::Unavailable, None),
        run::CompletionFailure::TimedOut { phase } => (
            channel::CompletionFailure::TimedOut(channel::TimeoutFailure {
                phase: mapped::completion_phase_to_record(&phase),
            }),
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
