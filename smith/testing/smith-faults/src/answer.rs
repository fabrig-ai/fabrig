//! Classifies the answer the host reads, without inferring a cause from prose
//! (`smith/docs/design/domain/run.md`, 9; `smith/docs/design/domain/session.md`, 4).

use smith_domain_run::{Answer, CompletionFailure, CompletionPhase, Fault, RunFailure};

/// The end a world's host observed for a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    /// The run returned an accepted result or parked.
    Completed,
    /// The account spent its allowance.
    Limited,
    /// The provider refused the account.
    Refused,
    /// The program source lost its login.
    Unauthorized,
    /// The host's program lacked smith's tools.
    Untooled,
    /// A completion exhausted its transient retries.
    Outage,
    /// Another failure or an entrance refusal ended the run.
    Failed,
}

/// The end a run's answer names, as its host reads it.
#[must_use]
pub fn ended(answer: &Answer) -> Ended {
    match answer {
        Answer::Accepted { .. } | Answer::Parked { .. } => Ended::Completed,
        Answer::Refused(_) => Ended::Failed,
        Answer::Failed { failure, .. } => match failure {
            RunFailure::Model(fault) => model_end(fault),
            RunFailure::Transcript(_)
            | RunFailure::Budget { .. }
            | RunFailure::Policy(_)
            | RunFailure::Cancelled
            | RunFailure::Stale => Ended::Failed,
        },
    }
}

fn model_end(fault: &Fault) -> Ended {
    match fault {
        Fault::Limited { .. } => Ended::Limited,
        Fault::Refused { .. } => Ended::Refused,
        Fault::Unauthorized { .. } => Ended::Unauthorized,
        Fault::Untooled { .. } => Ended::Untooled,
        Fault::Completion { failure, .. } => completion_end(*failure),
        Fault::Program { .. } | Fault::Provider | Fault::Declined | Fault::Truncated | Fault::Malformed => {
            Ended::Failed
        }
    }
}

fn completion_end(failure: CompletionFailure) -> Ended {
    match failure {
        CompletionFailure::Overloaded | CompletionFailure::Unavailable | CompletionFailure::RateLimited { .. } => {
            Ended::Outage
        }
        CompletionFailure::TimedOut { phase } => match phase {
            CompletionPhase::Connect | CompletionPhase::Handshake | CompletionPhase::Head | CompletionPhase::Idle => {
                Ended::Outage
            }
            CompletionPhase::Whole => Ended::Failed,
        },
        CompletionFailure::Limit { .. }
        | CompletionFailure::ApplicationLimit
        | CompletionFailure::Protocol
        | CompletionFailure::Cancelled
        | CompletionFailure::ContextTooLong
        | CompletionFailure::Invalid
        | CompletionFailure::Refused
        | CompletionFailure::Unsupported
        | CompletionFailure::Unauthorized
        | CompletionFailure::Exhausted { .. } => Ended::Failed,
    }
}
