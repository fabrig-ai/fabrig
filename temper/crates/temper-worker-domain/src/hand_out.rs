//! Hand-outs waiting for the protocol layer (jig's `domain/hosts/hub.md`, 8):
//! each of smith's payloads goes out to be encoded, and what it was for waits
//! here, in order, until it comes back, once.

use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{List, Token};
use smith_host_domain as agent;

/// An encoded start refusal waiting for process cleanup and its diagnostic.
#[derive(Debug)]
pub(crate) struct StartRefusal {
    /// Whether smith lacked room or found the start invalid.
    pub(crate) class: StartClass,
    /// Encoded refusal words, outside the model's transcript.
    pub(crate) why: Box<[u8]>,
}

/// A start's refusal policy (`temper/docs/design/domain/worker/hosted-runs.md`, 1.3).
#[derive(Clone, Copy, Debug)]
pub(crate) enum StartClass {
    Busy,
    Invalid(host::StartCause),
}

impl StartClass {
    pub(crate) fn event(self, owner: Token, why: Box<[u8]>) -> host::FromAgent {
        match self {
            StartClass::Busy => host::FromAgent::StartBusy { owner },
            StartClass::Invalid(cause) => host::FromAgent::StartRefused { owner, cause, why },
        }
    }
}

/// What a payload handed out is for.
#[derive(Debug)]
pub(crate) enum Pending {
    /// A part of the run `owner`, for the hub.
    Part { owner: Token },
    /// A turn of the run `owner`, for the hub.
    Turn { owner: Token },
    /// The name of the call `call` of the run `owner`.
    Call { owner: Token, call: Token },
    /// The run's last word, with what comes beside its result.
    Result { owner: Token, turns: u32, spent: u64, read: Option<Token>, finish: Finish },
}

impl Pending {
    /// The hosted run the hand-out is for.
    pub(crate) const fn owner(&self) -> Token {
        match self {
            Pending::Part { owner }
            | Pending::Turn { owner }
            | Pending::Call { owner, .. }
            | Pending::Result { owner, .. } => *owner,
        }
    }
}

/// A charter LLM's bounded names, retained until its run ends (domain/worker/hosted-runs.md, 1.3).
#[derive(Debug)]
pub(crate) struct ModelName {
    endpoint: Box<[u8]>,
    model: Box<[u8]>,
}

pub(crate) fn model_names(start: &agent::Start, endpoints: Box<[Box<[u8]>]>) -> Box<[ModelName]> {
    let mut names = List::with_capacity(u32::try_from(endpoints.len()).expect("decoded charter model count"));
    for (index, endpoint) in endpoints.into_iter().enumerate() {
        let index = u32::try_from(index).expect("decoded charter model index");
        let llm = start.charter.value().model(index).expect("one endpoint name per charter LLM");
        names
            .push(ModelName { endpoint: short_name(&endpoint), model: short_name(&llm.model) })
            .expect("one name per charter LLM");
    }
    names.into_boxed()
}

fn short_name(bytes: &[u8]) -> Box<[u8]> {
    let bound = skein_lib::usize_of(host::ModelFailure::NAME_BYTES);
    for end in (0..=bytes.len().min(bound)).rev() {
        // A UTF-8 continuation byte cannot begin the omitted suffix.
        if let Some(byte) = bytes.get(end)
            && byte & 0xc0 == 0x80
        {
            continue;
        }
        return bytes.get(..end).expect("bounded name prefix").into();
    }
    Box::new([])
}

fn model_failure(failure: &agent::RunFailure, models: &[ModelName]) -> Option<host::ModelFailure> {
    let agent::RunFailure::Model(agent::ModelFault::Completion { failure, evidence, model, report, .. }) = failure
    else {
        return None;
    };
    let name = models.get(skein_lib::usize_of(*model))?;
    let (evidence, status) = match evidence {
        agent::CompletionEvidence::Unsent => (host::ModelEvidence::Unsent, report.status),
        agent::CompletionEvidence::Unknown => (host::ModelEvidence::Unknown, report.status),
        agent::CompletionEvidence::Response { status } => (host::ModelEvidence::Response, status.or(report.status)),
    };
    let kind = crate::translate::model_failure_kind(failure);
    let unsent = match evidence {
        host::ModelEvidence::Unsent => true,
        host::ModelEvidence::Unknown | host::ModelEvidence::Response => false,
    };
    let refused = match kind {
        host::ModelFailureKind::Invalid | host::ModelFailureKind::Unsupported => true,
        host::ModelFailureKind::Unavailable
        | host::ModelFailureKind::Overloaded
        | host::ModelFailureKind::RateLimited
        | host::ModelFailureKind::Exhausted
        | host::ModelFailureKind::TimedOut
        | host::ModelFailureKind::ContextTooLong
        | host::ModelFailureKind::Refused
        | host::ModelFailureKind::Unauthorized
        | host::ModelFailureKind::Protocol
        | host::ModelFailureKind::Cancelled
        | host::ModelFailureKind::Limit => false,
    };
    let refusal = if unsent && status.is_none() && refused { refusal_code(&report.code) } else { None };
    Some(host::ModelFailure {
        endpoint: name.endpoint.clone(),
        model: name.model.clone(),
        kind,
        evidence,
        status,
        refusal,
    })
}

fn refusal_code(code: &[u8]) -> Option<Box<[u8]>> {
    if code.is_empty() || code.len() > skein_lib::usize_of(host::ModelFailure::REFUSAL_BYTES) {
        return None;
    }
    for byte in code {
        if !byte.is_ascii_lowercase() && *byte != b'-' {
            return None;
        }
    }
    Some(code.into())
}

/// How the run finished, kept typed beside its encoded result.
#[derive(Debug)]
pub(crate) enum Finish {
    Ended,
    Parked,
    Failed {
        failure: host::RunFailure,
        model: Option<host::ModelFailure>,
    },
    /// smith refused the start before any run began.
    Refused(StartClass),
}

impl Finish {
    pub(crate) fn of(result: &agent::RunResult, models: &[ModelName]) -> Finish {
        match result {
            agent::RunResult::Accepted { .. } => Finish::Ended,
            agent::RunResult::Parked => Finish::Parked,
            agent::RunResult::Failed { failure } => {
                let translated = crate::translate::run_failure(failure);
                let model = match translated {
                    host::RunFailure::Model => model_failure(failure, models),
                    host::RunFailure::Exceeded(_)
                    | host::RunFailure::Untooled
                    | host::RunFailure::Transcript
                    | host::RunFailure::Budget
                    | host::RunFailure::Policy
                    | host::RunFailure::Cancelled
                    | host::RunFailure::Stale
                    | host::RunFailure::Limited { .. }
                    | host::RunFailure::Refused { .. } => None,
                };
                Finish::Failed { failure: translated, model }
            }
            agent::RunResult::Refused { refusal } => Finish::Refused(match refusal {
                agent::Refusal::Invalid(why) => StartClass::Invalid(crate::translate::start_cause(*why)),
                agent::Refusal::Busy => StartClass::Busy,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_refusals_keep_only_unsent_application_codes() {
        use agent::CompletionEvidence::{Response, Unknown, Unsent};
        use agent::CompletionFailure as C;
        let names =
            [ModelName { endpoint: b"anthropic-api".as_slice().into(), model: b"claude-opus-5-5".as_slice().into() }];
        for (failure, eligible) in [
            (C::Invalid, true),
            (C::Unsupported, true),
            (C::Unavailable, false),
            (C::Overloaded, false),
            (C::RateLimited { retry_after: skein_lib::Duration::from_secs(1) }, false),
            (C::Exhausted { retry_after: None }, false),
            (C::TimedOut { phase: smith_domain_run::CompletionPhase::Head }, false),
            (C::ContextTooLong, false),
            (C::Refused, false),
            (C::Unauthorized, false),
            (C::Protocol, false),
            (C::Cancelled, false),
            (C::ApplicationLimit, false),
            (C::Limit { which: agent::Limit::Path(agent::Cap::Request), bound: 2048 }, false),
        ] {
            for (evidence, status, code, accepted) in [
                (Unsent, None, b"empty-message".as_slice(), true),
                (Unsent, None, b"a", true),
                (Unsent, None, b"abcdefghijklmnopqrstuvwx", true),
                (Unsent, None, b"", false),
                (Unsent, None, b"abcdefghijklmnopqrstuvwxy", false),
                (Unsent, None, b"Empty_Message", false),
                (Unsent, None, b"empty1", false),
                (Unsent, None, b"empty message", false),
                (Unsent, None, b"empty\x00message", false),
                (Unsent, None, b"empty\xffmessage", false),
                (Unknown, None, b"empty-message", false),
                (Response { status: None }, None, b"provider-code", false),
                (Response { status: Some(400) }, None, b"provider-code", false),
                (Response { status: None }, Some(400), b"provider-code", false),
                (Unsent, Some(400), b"provider-code", false),
            ] {
                let result = agent::RunResult::Failed {
                    failure: agent::RunFailure::Model(agent::ModelFault::Completion {
                        failure,
                        evidence,
                        model: 0,
                        attempts: 1,
                        report: agent::Report { status, code: code.into(), request: Box::new([]), field: Box::new([]) },
                    }),
                };
                match Finish::of(&result, &names) {
                    Finish::Failed { model, .. } => {
                        let model = model.expect("a completion names its call");
                        let expected = if eligible && accepted { Some(code) } else { None };
                        assert_eq!(model.refusal.as_deref(), expected, "{failure:?} {evidence:?} {status:?} {code:?}");
                    }
                    Finish::Ended | Finish::Parked | Finish::Refused(_) => unreachable!("a failed model call"),
                }
            }
        }
    }
}
