//! Total parent/root translations (domain/host.md, section 9.4).
//! Translations move values into the root; its admission checks decide their
//! semantics. It knows no codec, policy interpretation or credential values.

use skein_lib::{List, ReplyTo, Token};
use smith_domain::{self as smith, run};
use smith_host_domain as host;

use crate::Limits;

pub(crate) fn start(client: Token, source: host::Start, limits: &Limits) -> Result<smith::Event, host::Invalid> {
    let count = checked_count(source.messages.len())?;
    if count > limits.smith.run.messages {
        return Err(host::Invalid::Messages);
    }
    let count = checked_count(source.grants.len())?;
    if count > limits.smith.accounts {
        return Err(host::Invalid::Grants);
    }
    let mut grants = List::with_capacity(count);
    for item in source.grants {
        match item.form {
            host::Form::Program(_) => return Err(host::Invalid::Grants),
            host::Form::Credential(_) | host::Form::Withdrawn => {}
        }
        grants.push(grant(item)).expect("checked count");
    }
    let count = checked_count(source.answered.len())?;
    if count > limits.smith.run.answered_calls {
        return Err(host::Invalid::Answered);
    }
    let mut answered = List::with_capacity(count);
    for call in source.answered {
        let answer = match call.reply {
            host::SavedReply::Host { error, body } => {
                smith::Answered::Host(run::HostAnswer::new(body, error).ok_or(host::Invalid::Answered)?)
            }
            host::SavedReply::Delivery(value) => smith::Answered::Delivery(value),
            host::SavedReply::TooLarge => smith::Answered::TooLarge,
        };
        answered.push(smith::AnsweredCall { name: call.name, tool: call.tool, answer }).expect("checked count");
    }
    let transcript = match source.transcript {
        Some(value) => Some(value.into_value()),
        None => None,
    };
    Ok(smith::Event::Start {
        reply_to: ReplyTo::new(client),
        host_run: source.logical_run,
        activation: source.activation,
        identity: source.identity,
        window: limits.window,
        messages: source.messages,
        charter: source.charter.into_value(),
        workspace: crate::workspace::start(source.workspace, source.directories, limits)?,
        transcript,
        answered: answered.into_boxed(),
        grants: grants.into_boxed(),
    })
}

fn checked_count(length: usize) -> Result<u32, host::Invalid> {
    match u32::try_from(length) {
        Ok(count) => Ok(count),
        Err(_) => Err(host::Invalid::Limits),
    }
}

pub(crate) const fn grant(value: host::Grant) -> smith::Grant {
    let renewable = match value.form {
        host::Form::Credential(scheme) => match scheme {
            host::Scheme::Key => false,
            host::Scheme::Bearer => true,
        },
        host::Form::Withdrawn | host::Form::Program(_) => true,
    };
    smith::Grant {
        name: smith::GrantName { account: value.account, generation: value.generation },
        valid: value.valid,
        renewable,
    }
}

pub(crate) fn host_reply(value: host::Reply) -> Option<run::HostReply> {
    match value {
        host::Reply::Host { error, body } => match run::HostAnswer::new(body, error) {
            Some(answer) => Some(run::HostReply::Answered(answer)),
            None => Some(run::HostReply::TooLarge),
        },
        host::Reply::Busy => Some(run::HostReply::Busy),
        host::Reply::Unavailable => Some(run::HostReply::Unanswered(run::Unanswered::Lost)),
        host::Reply::Withdrawn => Some(run::HostReply::Withdrawn),
        host::Reply::TooLarge => Some(run::HostReply::TooLarge),
        host::Reply::Delivery(_) => None,
    }
}

pub(crate) fn answer(value: run::Answer, read: Option<Token>, now: skein_lib::Time) -> host::Answer {
    match value {
        run::Answer::Parked { spent, turns } => {
            host::Answer { read, turns, spent: spent.units, result: host::RunResult::Parked }
        }
        run::Answer::Refused(value) => {
            host::Answer { read, turns: 0, spent: 0, result: host::RunResult::Refused { refusal: value } }
        }
        run::Answer::Accepted { outcome, spent, turns } => host::Answer {
            read,
            turns,
            spent: spent.units,
            result: host::RunResult::Accepted {
                outcome: host::Declared::new(outcome, u64::MAX).expect("checked root outcome"),
            },
        },
        run::Answer::Failed { failure: value, spent, turns } => host::Answer {
            read,
            turns,
            spent: spent.units,
            result: host::RunResult::Failed { failure: failure(value, now) },
        },
    }
}

fn failure(value: run::Failure, now: skein_lib::Time) -> host::RunFailure {
    match value {
        run::Failure::Transcript(value) => host::RunFailure::Transcript(value),
        run::Failure::Model(value) => host::RunFailure::Model(model_fault(value, now)),
        run::Failure::Budget { exhausted: value, used, bound } => {
            host::RunFailure::Budget { exhausted: value, used, bound }
        }
        run::Failure::Policy(policy) => host::RunFailure::Policy(policy),
        run::Failure::Cancelled => host::RunFailure::Cancelled,
        run::Failure::Stale => host::RunFailure::Stale,
    }
}

fn model_fault(value: run::Fault, now: skein_lib::Time) -> host::ModelFault {
    match value {
        run::Fault::Completion { failure, evidence, model, attempts, report } => {
            host::ModelFault::Completion { failure, evidence, model, attempts, report }
        }
        run::Fault::Limited { account, reset_at } => host::ModelFault::Limited {
            account,
            reset: match reset_at {
                Some(at) => Some(at.saturating_since(now)),
                None => None,
            },
        },
        run::Fault::Refused { account, code } => host::ModelFault::Refused { account, code },
        run::Fault::Declined => host::ModelFault::Declined,
        run::Fault::Unauthorized { account } => host::ModelFault::Unauthorized { account },
        run::Fault::Program { name, exit, detail } => host::ModelFault::Program { name, exit, detail },
        run::Fault::Provider => host::ModelFault::Provider,
        run::Fault::Truncated => host::ModelFault::Truncated,
        run::Fault::Malformed => host::ModelFault::Malformed,
    }
}
