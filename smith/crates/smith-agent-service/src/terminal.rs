//! Retains the answer sent by the framed service for its final operator line.
//! No event destination decides whether that answer crossed the channel.
//! Contract: smith/docs/design/protocol/agent.md, section 6.

use alloc::boxed::Box;
use smith_domain::run;
use skein_lib::Time;
use smith_events::AnswerFailure;

/// The answer the settled agent's last line reports (smith/docs/design/protocol/agent.md, section 6).
#[derive(Debug)]
pub(crate) enum Line {
    /// No answer yet, or one that needs no line.
    Quiet,
    /// A failed answer, when it was told, with its operator detail.
    Failed { failure: run::Failure, at: Time, detail: Box<[u8]> },
    /// A refused answer, with its operator detail.
    Refused { refusal: run::Refusal, detail: Box<[u8]> },
}

/// The settled agent's last operator line (smith/docs/design/protocol/agent.md, section 6).
#[derive(Debug)]
pub enum Last {
    /// An accepted or parked answer needs no failure line.
    Quiet,
    /// A failed or refused answer, with its operator detail.
    Failed { failure: Box<AnswerFailure>, detail: Box<[u8]> },
    /// No answer crossed the channel, for this concrete cause.
    Unanswered(crate::Failure),
}

/// Project the settled terminal independently of event capture or destinations.
#[must_use]
pub fn last(service: &crate::Service) -> Option<Last> {
    let answered = crate::done(service)?;
    if !answered {
        return Some(Last::Unanswered(crate::failure(service).expect("an unanswered end keeps its first cause")));
    }
    match &service.stages.line {
        Line::Quiet => Some(Last::Quiet),
        Line::Failed { failure, at, detail } => Some(Last::Failed {
            failure: Box::new(smith_protocol_events::failure_record(failure.clone(), *at)),
            detail: detail.clone(),
        }),
        Line::Refused { refusal, detail } => Some(Last::Failed {
            failure: Box::new(smith_protocol_events::refusal_record(*refusal)),
            detail: detail.clone(),
        }),
    }
}

pub(crate) fn remember(
    service: &mut crate::service::Stages,
    answer: &run::Answer,
    detail: Box<[u8]>,
    now: Time,
) {
    service.line = match answer {
        run::Answer::Failed { failure, .. } => Line::Failed { failure: failure.clone(), at: now, detail },
        run::Answer::Refused(refusal) => Line::Refused { refusal: *refusal, detail },
        run::Answer::Accepted { .. } | run::Answer::Parked { .. } => Line::Quiet,
    };
}
