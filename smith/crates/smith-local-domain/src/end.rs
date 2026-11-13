//! The first cause of a run's unanswering end (protocol/events.md, section 3.10;
//! shell/commands.md, section 3). Local policy keeps the cause through settlement;
//! process exit evidence is not known until the host kit supplies it.

use crate::{ExitStatus, StoreFailure};

/// The last turn told in this activation, independently of its durable save.
#[derive(Debug)]
pub(crate) struct Told {
    pub(crate) turns: u32,
    pub(crate) spent: u64,
    pub(crate) read: Option<skein_lib::Token>,
}

impl Told {
    pub(crate) const fn new() -> Self {
        Self { turns: 0, spent: 0, read: None }
    }
}

/// The first cause recorded by local policy; settlement determines when the host exits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndCause {
    /// The person asked to drop or the inline cancellation grace expired.
    Cancelled,
    /// The store failed while the run went on; its turn is absent for a non-turn write.
    Host { turn: Option<u32>, reason: StoreFailure },
    /// The child ended without answering; Gone supplies any process exit the kit observed.
    Agent { ended: Option<smith_host_domain::Exit> },
}

pub(crate) const fn exit_status(cause: Option<EndCause>) -> ExitStatus {
    match cause {
        Some(EndCause::Cancelled) => ExitStatus::Interrupted,
        Some(EndCause::Host { .. } | EndCause::Agent { .. }) | None => ExitStatus::RunFailed,
    }
}

/// Classify the answer before rendering moves its concrete result.
pub(crate) fn answer_status(answer: &smith_host_domain::Answer, account: Option<u32>) -> ExitStatus {
    use smith_host_domain::{Refusal, RunFailure, RunResult};
    match &answer.result {
        RunResult::Accepted { .. } => ExitStatus::Accepted,
        RunResult::Parked => ExitStatus::NeedsInput,
        RunResult::Refused { refusal } => match refusal {
            Refusal::Busy => ExitStatus::RunFailed,
            Refusal::Invalid(_) => ExitStatus::Refused,
        },
        RunResult::Failed { failure } => match failure {
            RunFailure::Budget { .. } => ExitStatus::Budget,
            RunFailure::Cancelled => ExitStatus::Interrupted,
            RunFailure::Model(fault) => match fault {
                smith_host_domain::ModelFault::Completion {
                    failure: smith_host_domain::CompletionFailure::Unauthorized,
                    ..
                } => match account {
                    Some(account) => ExitStatus::Credentials { account },
                    None => ExitStatus::RunFailed,
                },
                smith_host_domain::ModelFault::Refused { account, .. }
                | smith_host_domain::ModelFault::Unauthorized { account } => {
                    ExitStatus::Credentials { account: *account }
                }
                smith_host_domain::ModelFault::Completion { .. }
                | smith_host_domain::ModelFault::Limited { .. }
                | smith_host_domain::ModelFault::Provider
                | smith_host_domain::ModelFault::Declined
                | smith_host_domain::ModelFault::Program { .. }
                | smith_host_domain::ModelFault::Truncated
                | smith_host_domain::ModelFault::Malformed => ExitStatus::RunFailed,
            },
            RunFailure::Transcript(_) | RunFailure::Policy(_) | RunFailure::Stale => ExitStatus::RunFailed,
        },
    }
}

/// Cancellation and store failure retain the first cause until settlement.
#[derive(Debug)]
pub(crate) enum End {
    Open { cause: Option<EndCause> },
    Cancelling { cause: Option<EndCause>, requested: skein_lib::Time },
    StoreFailed { cause: Option<EndCause>, requested: Option<skein_lib::Time> },
}

impl End {
    pub(crate) const fn new() -> Self {
        Self::Open { cause: None }
    }

    pub(crate) const fn cause(&self) -> Option<EndCause> {
        match self {
            Self::Open { cause } | Self::Cancelling { cause, .. } | Self::StoreFailed { cause, .. } => *cause,
        }
    }

    pub(crate) fn requested(&self) -> Option<skein_lib::Time> {
        match self {
            Self::Open { .. } => None,
            Self::Cancelling { requested, .. } => Some(*requested),
            Self::StoreFailed { requested, .. } => *requested,
        }
    }

    pub(crate) fn store_failed(&self) -> bool {
        match self {
            Self::StoreFailed { .. } => true,
            Self::Open { .. } | Self::Cancelling { .. } => false,
        }
    }

    pub(crate) fn record(&mut self, incoming: EndCause) {
        let state = core::mem::replace(self, Self::new());
        *self = match state {
            Self::Open { cause } => Self::Open { cause: cause.or(Some(incoming)) },
            Self::Cancelling { cause, requested } => Self::Cancelling { cause: cause.or(Some(incoming)), requested },
            Self::StoreFailed { cause, requested } => Self::StoreFailed { cause, requested },
        };
    }

    pub(crate) fn cancel(&mut self, requested: skein_lib::Time) {
        let state = core::mem::replace(self, Self::new());
        *self = match state {
            Self::Open { cause } | Self::Cancelling { cause, .. } => Self::Cancelling { cause, requested },
            Self::StoreFailed { cause, .. } => Self::StoreFailed { cause, requested: Some(requested) },
        };
    }

    pub(crate) fn fail_store(&mut self) {
        let state = core::mem::replace(self, Self::new());
        *self = match state {
            Self::Open { cause } => Self::StoreFailed { cause, requested: None },
            Self::Cancelling { cause, requested } => Self::StoreFailed { cause, requested: Some(requested) },
            Self::StoreFailed { cause, requested } => Self::StoreFailed { cause, requested },
        };
    }

    pub(crate) fn gone(&mut self, ended: Option<smith_host_domain::Exit>) {
        let state = core::mem::replace(self, Self::new());
        *self = match state {
            Self::Open { cause } => Self::Open { cause: Some(gone_cause(cause, ended)) },
            Self::Cancelling { cause, requested } => {
                Self::Cancelling { cause: Some(gone_cause(cause, ended)), requested }
            }
            Self::StoreFailed { cause, requested } => {
                Self::StoreFailed { cause: Some(gone_cause(cause, ended)), requested }
            }
        };
    }
}

fn gone_cause(cause: Option<EndCause>, ended: Option<smith_host_domain::Exit>) -> EndCause {
    match cause {
        Some(EndCause::Agent { .. }) | None => EndCause::Agent { ended },
        Some(cause @ (EndCause::Cancelled | EndCause::Host { .. })) => cause,
    }
}

#[cfg(test)]
mod tests {
    use super::{End, EndCause};

    #[test]
    fn an_agent_cause_gains_exit_evidence_after_an_intervening_store_failure() {
        let mut end = End::new();
        end.record(EndCause::Agent { ended: None });
        end.fail_store();
        end.gone(Some(smith_host_domain::Exit::Signal(9)));
        assert_eq!(end.cause(), Some(EndCause::Agent { ended: Some(smith_host_domain::Exit::Signal(9)) }));
        assert!(end.store_failed());
    }
}
