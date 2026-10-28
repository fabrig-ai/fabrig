//! Hand-outs waiting for the protocol layer (jig's `domain/hosts.md`, 6.8):
//! each of smith's payloads goes out to be encoded, and what it was for waits
//! here, in order, until it comes back, once.

use jig_domain_host as host;
use skein_lib::Token;
use smith_host_domain as agent;

/// What a payload handed out is for.
#[derive(Debug)]
pub(crate) enum Pending {
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
            Pending::Turn { owner } | Pending::Call { owner, .. } | Pending::Result { owner, .. } => *owner,
        }
    }
}

/// How the run finished, kept typed beside its encoded result.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Finish {
    Ended,
    Parked,
    Failed(host::RunFailure),
    /// smith refused the start before any run began.
    Refused(host::StartCause),
}

impl Finish {
    pub(crate) const fn of(result: &agent::RunResult) -> Finish {
        match result {
            agent::RunResult::Accepted { .. } => Finish::Ended,
            agent::RunResult::Parked => Finish::Parked,
            agent::RunResult::Failed { failure } => Finish::Failed(crate::translate::run_failure(failure)),
            agent::RunResult::Refused { refusal } => Finish::Refused(match refusal {
                agent::Refusal::Invalid(why) => crate::translate::start_cause(*why),
                agent::Refusal::Busy => host::StartCause::Other,
            }),
        }
    }
}
