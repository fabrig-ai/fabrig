//! The run handle and a drain or cancel waiting for admission share one state.
//! No handle is known until the domain names it. `drain` and `cancel` retain controls,
//! `admit` retains the handle, and `follow` delivers the owed control after requests.
//! Contract: smith/docs/design/protocol/agent.md, section 6.

use super::Stages;
use skein_lib::Token;
use smith_domain as domain;

/// The domain's run handle and any drain or cancel waiting for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Admission {
    /// The domain has not named the run.
    Waiting,
    /// A cancel came before the handle and waits for it.
    Cancelled,
    /// A drain came before the handle and waits for it.
    Draining,
    /// The handle reached a waiting cancel; this pass delivers it.
    Owed { run: Token },
    /// The handle reached a waiting drain; this pass delivers it.
    DrainOwed { run: Token },
    /// The domain named the run.
    Admitted { run: Token },
}

impl Admission {
    pub(super) const fn run(self) -> Option<Token> {
        match self {
            Self::Waiting | Self::Cancelled | Self::Draining => None,
            Self::Owed { run } | Self::DrainOwed { run } | Self::Admitted { run } => Some(run),
        }
    }

    pub(super) fn admit(&mut self, run: Token) {
        *self = match *self {
            Self::Waiting => Self::Admitted { run },
            Self::Cancelled => Self::Owed { run },
            Self::Draining => Self::DrainOwed { run },
            Self::Owed { .. } | Self::DrainOwed { .. } | Self::Admitted { .. } => {
                unreachable!("one Admitted per Start")
            }
        };
    }
}

/// The one cancel entrance for a host cancel or a live run's shutdown.
pub(super) fn cancel(service: &mut Stages) {
    service.admission = match service.admission {
        Admission::Waiting | Admission::Cancelled | Admission::Draining => Admission::Cancelled,
        Admission::DrainOwed { run } => {
            service.domain_events.push(domain::Event::Cancel { run });
            Admission::Admitted { run }
        }
        state @ (Admission::Owed { run } | Admission::Admitted { run }) => {
            service.domain_events.push(domain::Event::Cancel { run });
            state
        }
    };
}

/// Hold a drain until the run is named, as for cancellation.
pub(super) fn drain(service: &mut Stages) {
    service.admission = match service.admission {
        Admission::Waiting => Admission::Draining,
        state @ (Admission::Cancelled | Admission::Draining | Admission::Owed { .. } | Admission::DrainOwed { .. }) => {
            state
        }
        state @ Admission::Admitted { run } => {
            service.domain_events.push(domain::Event::Drain { run });
            state
        }
    };
}

/// Deliver a waiting cancel after this pass's requests, before queued messages flush.
pub(super) fn follow(service: &mut Stages) {
    service.admission = match service.admission {
        Admission::Owed { run } => {
            service.domain_events.push(domain::Event::Cancel { run });
            Admission::Admitted { run }
        }
        Admission::DrainOwed { run } => {
            service.domain_events.push(domain::Event::Drain { run });
            Admission::Admitted { run }
        }
        state @ (Admission::Waiting | Admission::Cancelled | Admission::Draining | Admission::Admitted { .. }) => state,
    };
}
