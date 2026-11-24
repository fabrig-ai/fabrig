//! The run handle and a drain or cancel waiting for admission share one state.
//! No handle is known until the domain names it. `drain` and `cancel` retain controls,
//! `admit` retains the handle, and `follow` delivers the owed control after requests.
//! Contract: smith/docs/design/protocol/agent.md, section 6.

use super::Stages;
use skein_lib::{Duration, Token};
use smith_domain as domain;

/// The domain's run handle and any drain or cancel waiting for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Admission {
    /// The domain has not named the run.
    Waiting,
    /// A cancel came before the handle and waits for it.
    Cancelled,
    /// A drain came before the handle and waits for it.
    Draining { grace: Option<Duration> },
    /// The handle reached a waiting cancel; this pass delivers it.
    Owed { run: Token },
    /// The handle reached a waiting drain; this pass delivers it.
    DrainOwed { run: Token, grace: Option<Duration> },
    /// The domain named the run.
    Admitted { run: Token },
}

impl Admission {
    pub(super) const fn run(self) -> Option<Token> {
        match self {
            Self::Waiting | Self::Cancelled | Self::Draining { .. } => None,
            Self::Owed { run } | Self::DrainOwed { run, .. } | Self::Admitted { run } => Some(run),
        }
    }

    pub(super) fn admit(&mut self, run: Token) {
        *self = match *self {
            Self::Waiting => Self::Admitted { run },
            Self::Cancelled => Self::Owed { run },
            Self::Draining { grace } => Self::DrainOwed { run, grace },
            Self::Owed { .. } | Self::DrainOwed { .. } | Self::Admitted { .. } => {
                unreachable!("one Admitted per Start")
            }
        };
    }
}

/// The one cancel entrance for a host cancel or a live run's shutdown.
pub(super) fn cancel(service: &mut Stages) {
    service.host_word_closed = true;
    service.pending_signal = None;
    service.admission = match service.admission {
        Admission::Waiting | Admission::Cancelled | Admission::Draining { .. } => Admission::Cancelled,
        Admission::DrainOwed { run, .. } => {
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
pub(super) fn drain(service: &mut Stages, grace: Option<Duration>) {
    service.admission = match service.admission {
        Admission::Waiting => Admission::Draining { grace },
        state @ (Admission::Cancelled
        | Admission::Draining { .. }
        | Admission::Owed { .. }
        | Admission::DrainOwed { .. }) => state,
        state @ Admission::Admitted { run } => {
            service.domain_events.push(domain::Event::Drain { run, grace });
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
        Admission::DrainOwed { run, grace } => {
            service.domain_events.push(domain::Event::Drain { run, grace });
            Admission::Admitted { run }
        }
        state @ (Admission::Waiting
        | Admission::Cancelled
        | Admission::Draining { .. }
        | Admission::Admitted { .. }) => state,
    };
}

/// A signal belongs to the host while its word remains open.
pub(super) fn signalled(service: &mut Stages, signal: skein_io::kernel::ServiceSignal, from: Option<Token>) {
    match service.teardown {
        super::Teardown::Live { .. } => {}
        super::Teardown::Answered { .. } | super::Teardown::Closing { .. } | super::Teardown::Closed { .. } => {
            super::teardown::shutdown(service);
            return;
        }
    }
    if service.host_word_closed {
        cancel(service);
        return;
    }
    if service.signalled {
        return;
    }
    let signal = match signal {
        skein_io::kernel::ServiceSignal::Interrupt => smith_channel::Signal::Interrupt,
        skein_io::kernel::ServiceSignal::Terminate => smith_channel::Signal::Terminate,
        skein_io::kernel::ServiceSignal::Hangup => {
            super::teardown::shutdown(service);
            return;
        }
    };
    let from = match from {
        Some(sender) => u32::try_from(sender.raw()).ok(),
        None => None,
    };
    service.signalled = true;
    service.pending_signal = Some(smith_channel::Signalled { signal, from });
}

/// Send the retained notice only after admission, before another run record.
pub(super) fn tell_signal(service: &mut Stages, frame: &mut super::Frame) -> bool {
    if service.pending_signal.is_none() || service.admission.run().is_none() {
        return true;
    }
    if !frame.channel.takes() || !super::channel_room(service, Some(frame)) {
        return false;
    }
    let record = service.pending_signal.take().expect("one retained signal");
    let token = service.next_send();
    if frame.channel.send_signalled(record, token, &mut service.channel_events, &mut service.channel_below).is_err() {
        service.mark_failed(super::Failure::WriteFailed);
        cancel(service);
    }
    true
}
