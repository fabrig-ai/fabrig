//! Pure projection of the exact answer sent by the agent, without host or provider IO.
//! Only an accepted answer owns result content. The caller supplies its run fence,
//! wind-down and elapsed time; no observation of stdout establishes acceptance.
//! Contract: smith/docs/design/protocol/events.md, sections 3.3 and 3.10.

#![expect(clippy::manual_map, reason = "the strict subset requires explicit matches instead of closures")]

use skein_lib::{Duration, List};
use smith_domain::run;
use smith_events::{
    AnswerClass, AnswerFailure, CompletionFailure, Evidence, FailureClass, Form, RunCompleted, RunResult, Status, Usage,
};

use crate::sinks::Capture;

/// Parts of a run terminal supplied by the writer that meets its answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answered {
    pub status: Status,
    pub failure: Option<AnswerFailure>,
    pub result: Option<RunResult>,
    pub turns: u64,
    pub spent: u64,
    pub read: Option<u64>,
}

/// The run owner's final fence and measurements, supplied with its answer.
#[derive(Clone, Copy, Debug)]
pub struct AnswerMeta {
    pub now: skein_lib::Time,
    pub run: u64,
    pub wind_down: bool,
    pub read: Option<u64>,
    pub unread: u64,
    pub elapsed: Duration,
}

/// Project the exact terminal the service sends, preserving its accepted counters.
#[must_use]
pub fn answer_record(meta: AnswerMeta, answer: &run::Answer, capture: Capture) -> RunCompleted {
    let (status, failure, result, turns, spent) = match answer {
        run::Answer::Parked { spent, turns } => (Status::Parked, None, None, *turns, *spent),
        run::Answer::Refused(refusal) => (Status::Refused, Some(refusal_record(*refusal)), None, 0, run::Spend::ZERO),
        run::Answer::Accepted { outcome, spent, turns } => {
            (Status::Accepted, None, Some(accepted(outcome, capture)), *turns, *spent)
        }
        run::Answer::Failed { failure, spent, turns } => {
            (Status::Failed, Some(failure_record(failure.clone(), meta.now)), None, *turns, *spent)
        }
    };
    RunCompleted {
        run: meta.run,
        wind_down: meta.wind_down,
        status,
        failure,
        result,
        turns: u64::from(turns),
        spent: spent.units,
        usage: Usage {
            input_tokens: spent.input,
            cache_read_tokens: spent.cache_read,
            cache_write_tokens: spent.cache_write,
            output_tokens: spent.output,
            reasoning_tokens: spent.reasoning,
        },
        read: meta.read,
        unread: meta.unread,
        duration_ms: meta.elapsed.as_millis(),
    }
}

/// Project the accepted result under the writer's capture policy.
#[must_use]
pub fn accepted(outcome: &run::outcome::Declared, capture: Capture) -> RunResult {
    let (form, label, text, fields) = match outcome {
        run::outcome::Declared::Change(change) => (Form::Change, None, None, &change.fields),
        run::outcome::Declared::Verdict(verdict) => {
            (Form::Verdict, Some(verdict.name.clone()), Some(&verdict.text), &verdict.fields)
        }
        run::outcome::Declared::Report(report) => (Form::Report, None, Some(&report.text), &report.fields),
        run::outcome::Declared::Failure(failure) => (Form::Failure, None, Some(&failure.reason), &failure.fields),
    };
    let (text, fields) = match capture {
        Capture::None | Capture::Calls => (None, None),
        Capture::Everything => {
            let mut copied = List::with_capacity(u32::try_from(fields.len()).expect("admitted result field count"));
            for field in fields {
                copied
                    .push(smith_events::Field { name: field.name.clone(), text: field.value.clone() })
                    .expect("admitted result fields");
            }
            let text = match text {
                Some(text) => Some(text.clone()),
                None => None,
            };
            (text, Some(copied))
        }
    };
    RunResult { form, label, text, fields }
}

fn failure(class: AnswerClass, which: Option<&[u8]>) -> AnswerFailure {
    AnswerFailure {
        state: None,
        code: None,
        program: None,
        exit: None,
        detail: None,
        reset_ms: None,
        used: None,
        bound: None,
        dropped: None,
        ended: None,
        turn: None,
        endpoint: None,
        model: None,
        attempts: None,
        class,
        completion: None,
        account: None,
        which: match which {
            Some(which) => Some(which.into()),
            None => None,
        },
        reason: None,
    }
}

/// Project a refused entrance into the event vocabulary.
#[must_use]
pub fn refusal_record(refusal: run::Refusal) -> AnswerFailure {
    match refusal {
        run::Refusal::Busy => failure(AnswerClass::Busy, None),
        run::Refusal::Invalid(invalid) => failure(AnswerClass::Invalid, Some(invalid_name(invalid))),
    }
}

fn invalid_name(invalid: run::Invalid) -> &'static [u8] {
    match invalid {
        run::Invalid::HostPart { .. } => b"host_part",
        run::Invalid::Room { .. } => b"room",
        run::Invalid::Messages => b"messages",
        run::Invalid::CharterVersion => b"charter_version",
        run::Invalid::MalformedCharter => b"malformed_charter",
        run::Invalid::Endpoint => b"endpoint",
        run::Invalid::Activation => b"activation",
        run::Invalid::Window => b"window",
        run::Invalid::Conventions => b"conventions",
        run::Invalid::TooLarge => b"too_large",
        run::Invalid::Workspace => b"workspace",
        run::Invalid::Grants => b"grants",
        run::Invalid::Outcome => b"outcome",
        run::Invalid::Budget(why) => match why {
            run::BudgetRefusal::Reserve(part) => match part {
                run::Part::Turns => b"budget_reserve_turns",
                run::Part::Spend => b"budget_reserve_spend",
                run::Part::Time => b"budget_reserve_time",
            },
            run::BudgetRefusal::Ceiling(_)
            | run::BudgetRefusal::Empty(_)
            | run::BudgetRefusal::Checks { .. }
            | run::BudgetRefusal::Completion { .. } => b"budget",
        },
        run::Invalid::Price(_) => b"price",
        run::Invalid::Llm => b"llm",
        run::Invalid::Effort => b"effort",
        run::Invalid::Conversation => b"conversation",
    }
}

/// Project a domain failure into the event vocabulary.
#[must_use]
pub fn failure_record(why: run::Failure, now: skein_lib::Time) -> AnswerFailure {
    match why {
        run::Failure::Model(fault) => match fault {
            run::Fault::Completion { failure: why, evidence, model: _, attempts: _, report } => {
                let mut result = failure(AnswerClass::Model, None);
                let mut completed = completion(why, evidence);
                if !report.field.is_empty() {
                    completed.field = Some(report.field);
                }
                if !report.code.is_empty() {
                    completed.code = Some(report.code);
                }
                if !report.request.is_empty() {
                    completed.request_id = Some(report.request);
                }
                result.completion = Some(completed);
                result
            }
            run::Fault::Limited { account, reset_at } => {
                let mut result = failure(AnswerClass::Model, None);
                result.account = Some(account);
                result.state = Some(smith_events::AccountState::Limited);
                result.reset_ms = match reset_at {
                    Some(at) => Some(at.saturating_since(now).as_millis()),
                    None => None,
                };
                result
            }
            run::Fault::Provider => failure(AnswerClass::Model, Some(b"provider")),
            run::Fault::Refused { account, code } => {
                let mut result = failure(AnswerClass::Model, None);
                result.account = Some(account);
                result.state = Some(smith_events::AccountState::Refused);
                result.code = Some(code);
                result
            }
            run::Fault::Declined => failure(AnswerClass::Model, Some(b"refused")),
            run::Fault::Truncated => failure(AnswerClass::Model, Some(b"truncated")),
            run::Fault::Unauthorized { .. } => failure(AnswerClass::Model, Some(b"unauthorized")),
            run::Fault::Program { name, exit, detail } => {
                let mut result = failure(AnswerClass::Model, None);
                result.program = Some(name);
                result.exit = match exit {
                    Some(code) => Some(smith_events::AgentExit::Code(i64::from(code))),
                    None => None,
                };
                result.detail = Some(detail);
                result
            }
            run::Fault::Malformed => failure(AnswerClass::Model, Some(b"malformed")),
        },
        run::Failure::Budget { exhausted, used, bound } => {
            let which: &[u8] = match exhausted {
                run::Exhausted::Turns => b"turns",
                run::Exhausted::Spend => b"spend",
                run::Exhausted::Time => b"time",
                run::Exhausted::Tokens(receiving) => match receiving {
                    run::ReceivingLimit::Input => b"input_tokens",
                    run::ReceivingLimit::Output => b"output_tokens",
                    run::ReceivingLimit::CacheRead => b"cache_read_tokens",
                    run::ReceivingLimit::CacheWrite => b"cache_write_tokens",
                },
                run::Exhausted::Overflow(overflow) => match overflow {
                    run::Overflow::Spend => b"spend_overflow",
                    run::Overflow::Usage => b"usage_overflow",
                },
            };
            let mut result = failure(AnswerClass::Budget, Some(which));
            result.used = used;
            result.bound = bound;
            result
        }
        run::Failure::Policy(run::Policy::Unfinished { .. }) => failure(AnswerClass::Policy, None),
        run::Failure::Cancelled => failure(AnswerClass::Cancelled, None),
        run::Failure::Stale => failure(AnswerClass::Stale, None),
        run::Failure::Transcript(refusal) => {
            let which: &[u8] = match refusal {
                run::TranscriptRefusal::Version => b"version",
                run::TranscriptRefusal::Malformed => b"malformed",
                run::TranscriptRefusal::Unresolved => b"unresolved",
                run::TranscriptRefusal::TooLarge => b"too_large",
            };
            let mut result = failure(AnswerClass::Transcript, None);
            result.reason = Some(which.into());
            result
        }
    }
}

fn completion(why: run::CompletionFailure, evidence: run::CompletionEvidence) -> CompletionFailure {
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
            run::Phase::Connect => smith_events::FailurePhase::Connect,
            run::Phase::Handshake => smith_events::FailurePhase::Handshake,
            run::Phase::Head => smith_events::FailurePhase::Head,
            run::Phase::Idle => smith_events::FailurePhase::Idle,
            run::Phase::Whole => smith_events::FailurePhase::Whole,
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
            run::Cap::AccountId => b"account_id",
        },
    }
}

#[cfg(test)]
mod tests;
