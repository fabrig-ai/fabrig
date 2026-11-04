//! The run handle and a cancel waiting for admission share one state.
//! No handle is known until the domain names it. `cancel` accepts cancellation,
//! `admit` retains the handle, and `follow` delivers an owed cancel after requests.
//! Contract: smith/docs/design/protocol/agent.md, section 6.

use super::Stages;
use core::mem;
use skein_lib::Token;
use smith_domain as domain;

/// The domain's run handle and any cancel waiting for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Admission {
    /// The domain has not named the run.
    Waiting,
    /// A cancel came before the handle and waits for it.
    Cancelled,
    /// The handle reached a waiting cancel; this pass delivers it.
    Owed { run: Token },
    /// The domain named the run.
    Admitted { run: Token },
}

impl Admission {
    pub(super) const fn run(self) -> Option<Token> {
        match self {
            Self::Waiting | Self::Cancelled => None,
            Self::Owed { run } | Self::Admitted { run } => Some(run),
        }
    }

    pub(super) fn admit(&mut self, run: Token) {
        let state = mem::replace(self, Self::Waiting);
        *self = match state {
            Self::Waiting => Self::Admitted { run },
            Self::Cancelled => Self::Owed { run },
            Self::Owed { .. } | Self::Admitted { .. } => unreachable!("one Admitted per Start"),
        };
    }
}

/// The one cancel entrance for a host cancel or a live run's shutdown.
pub(super) fn cancel(service: &mut Stages) {
    let state = mem::replace(&mut service.admission, Admission::Waiting);
    service.admission = match state {
        Admission::Waiting | Admission::Cancelled => Admission::Cancelled,
        state @ (Admission::Owed { run } | Admission::Admitted { run }) => {
            service.domain_events.push(domain::Event::Cancel { run });
            state
        }
    };
}

/// Deliver a waiting cancel after this pass's requests, before queued messages flush.
pub(super) fn follow(service: &mut Stages) {
    let state = mem::replace(&mut service.admission, Admission::Waiting);
    service.admission = match state {
        Admission::Owed { run } => {
            service.domain_events.push(domain::Event::Cancel { run });
            Admission::Admitted { run }
        }
        state @ (Admission::Waiting | Admission::Cancelled | Admission::Admitted { .. }) => state,
    };
}
