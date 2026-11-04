//! Retains the answer sent by the framed service for its final operator line.
//! No event destination decides whether that answer crossed the channel.
//! Contract: smith/docs/design/protocol/agent.md, section 6.

use alloc::boxed::Box;
use skein_lib::Time;
use smith_domain::run;
use smith_events::AnswerFailure;

/// The settled agent's last operator line (smith/docs/design/protocol/agent.md, section 6).
#[derive(Clone, Debug)]
pub enum Last {
    /// No answer yet, or an accepted or parked answer that needs no failure line.
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
    Some(service.stages.line.clone())
}

pub(crate) fn remember(service: &mut crate::service::Stages, answer: &run::Answer, detail: Box<[u8]>, now: Time) {
    service.line = match answer {
        run::Answer::Failed { failure, .. } => {
            Last::Failed { failure: Box::new(smith_protocol_events::failure_record(failure.clone(), now)), detail }
        }
        run::Answer::Refused(refusal) => {
            Last::Failed { failure: Box::new(smith_protocol_events::refusal_record(*refusal)), detail }
        }
        run::Answer::Accepted { .. } | run::Answer::Parked { .. } => Last::Quiet,
    };
}
