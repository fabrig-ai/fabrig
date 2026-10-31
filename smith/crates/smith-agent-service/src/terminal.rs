//! Retains the answer sent by the framed service for its final operator line.
//! No event destination decides whether that answer crossed the channel.
//! Contract: smith/docs/design/protocol/agent.md, section 6.

use alloc::boxed::Box;
use smith_domain::run;
use smith_events::AnswerFailure;

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
    let failure = match service.final_failure.clone() {
        Some(why) => smith_protocol_events::failure_record(why, service.final_answer_at),
        None => match service.final_refusal {
            Some(why) => smith_protocol_events::refusal_record(why),
            None => return Some(Last::Quiet),
        },
    };
    Some(Last::Failed { failure: Box::new(failure), detail: service.final_detail.clone() })
}

pub(crate) fn remember(service: &mut crate::Service, answer: &run::Answer, detail: Box<[u8]>, now: skein_lib::Time) {
    service.final_detail = detail;
    service.final_answer_at = now;
    match answer {
        run::Answer::Failed { failure, .. } => service.final_failure = Some(failure.clone()),
        run::Answer::Refused(refusal) => service.final_refusal = Some(*refusal),
        run::Answer::Accepted { .. } | run::Answer::Parked { .. } => {}
    }
}
