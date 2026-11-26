//! The framed agent's lost host and the run's one cancellation grace.
//! The first loss withdraws the channel and cancels the run; no peer can
//! extend the grace. Contract: smith/docs/design/domain/host.md, section 4.

use super::{Failure, Frame, Stages, Teardown, admission, teardown};
use alloc::boxed::Box;
use skein_lib::Time;
use smith_domain as domain;
use smith_protocol_channel::Heard;

#[derive(Clone, Copy, Debug)]
pub(super) enum Host {
    Open,
    Lost { drop_at: Time },
    Dropped,
}

impl Host {
    pub(super) const fn gone(self) -> bool {
        match self {
            Self::Open => false,
            Self::Lost { .. } | Self::Dropped => true,
        }
    }

    pub(super) const fn dropped(self) -> bool {
        match self {
            Self::Open | Self::Lost { .. } => false,
            Self::Dropped => true,
        }
    }
}

/// The host is gone: start cancel, grace, drop (domain/host.md, section 4).
/// A later loss keeps the first failure.
pub(super) fn host_lost(service: &mut Stages, failure: Failure) {
    if service.host.gone() {
        return;
    }
    let live = match service.teardown {
        Teardown::Live { .. } => true,
        Teardown::Answered { .. } | Teardown::Closing { .. }
            if failure == Failure::WriteFailed || failure == Failure::HostSilent =>
        {
            false
        }
        Teardown::Answered { .. } | Teardown::Closing { .. } | Teardown::Closed { .. } => return,
    };
    service.mark_failed(failure);
    service.host = Host::Lost { drop_at: service.io_env.now.saturating_add(service.limits.link.cancel_grace()) };
    service.host_word_closed = true;
    service.pending_signal = None;
    service.pending_messages.clear();
    service.channel_below.clear();
    super::routing::abort_channel(service);
    if !live {
        return;
    }
    match service.admission.run() {
        Some(run) => disconnect(service, run),
        None => match &service.opening {
            super::opening::Opening::Started { .. } => admission::cancel(service),
            super::opening::Opening::Idle | super::opening::Opening::Attaching { .. } => {
                service.opening.refuse();
                teardown::answered(service);
            }
        },
    }
}

// A loss already read from IO does not wait for an earlier blocked host message.
pub(super) fn detect_blocked_loss(service: &mut Stages) {
    for event in &service.channel_events {
        let failure = match event {
            Heard::WriteFailed => Failure::WriteFailed,
            Heard::Ended { .. } => Failure::ChannelEnded,
            Heard::Opened { .. }
            | Heard::Start { .. }
            | Heard::Message { .. }
            | Heard::Event { .. }
            | Heard::DeliveryUnsent { .. }
            | Heard::Acknowledged { .. }
            | Heard::Grant { .. }
            | Heard::Cancel
            | Heard::Drain { .. }
            | Heard::PartKept { .. } => continue,
        };
        host_lost(service, failure);
        return;
    }
}

pub(super) fn disconnect(service: &mut Stages, run: skein_lib::Token) {
    let pending = service.domain_events.len();
    service.domain_events.push(domain::Event::Disconnected { run });
    for _ in 0..pending {
        let earlier = service.domain_events.pop().expect("counted entrances");
        service.domain_events.push(earlier);
    }
}

pub(super) fn abandon(service: &mut Stages, frame: &mut Frame) {
    if service.host.gone() && frame.channel.abandon(&mut service.channel_events, &mut service.channel_below) {
        frame.channel.lose_live(service.live.abandon());
        service.channel_below.clear();
    }
}

/// Drain the channel's single terminals for calls which already crossed it.
pub(super) fn channel_terminal(service: &mut Stages, event: Heard) {
    if service.host.dropped() {
        return;
    }
    match event {
        Heard::Event { event } => service.domain_events.push(*event),
        Heard::DeliveryUnsent { owner } => lost_delivery(service, owner),
        Heard::Message { .. }
        | Heard::Opened { .. }
        | Heard::Start { .. }
        | Heard::Acknowledged { .. }
        | Heard::Grant { .. }
        | Heard::Cancel
        | Heard::Drain { .. }
        | Heard::PartKept { .. }
        | Heard::WriteFailed
        | Heard::Ended { .. } => {}
    }
}

fn lost_delivery(service: &mut Stages, owner: skein_lib::Token) {
    service.domain_events.push(domain::Event::Delivered { owner, delivery: super::run::Delivery::Stale });
}

/// Settle requests still queued at the lost edge without sending them.
pub(super) fn host_request(service: &mut Stages, request: domain::Request) {
    use domain::Request;
    match request {
        Request::Admitted { run, .. } => service.admission.admit(run),
        Request::HostCall { relay, .. } => service.domain_events.push(domain::Event::HostReturned {
            relay,
            reply: super::run::HostReply::Unanswered(super::run::Unanswered::Lost),
        }),
        Request::Save { owner, .. } | Request::Deliver { owner, .. } => lost_delivery(service, owner),
        Request::Answer { to, answer, read, detail } => {
            assert!(to.into_token() == super::START_REPLY, "one Start reply right per process");
            crate::terminal::remember(service, &answer, detail, service.domain_env.now);
            if let Some(events) = &mut service.events {
                events.answer(service.domain_env.now, service.run_started, &answer, read);
            }
            teardown::answered(service);
        }
        Request::Turn { turn, .. } => super::program::settled(service, &turn),
        Request::Lower(_) => unreachable!("lower requests are dispatched before the framed face"),
        Request::Waiting { .. }
        | Request::Part { .. }
        | Request::WithdrawHost { .. }
        | Request::MessageRefused { .. }
        | Request::Long { .. }
        | Request::LongDone { .. }
        | Request::Unauthorized { .. }
        | Request::Usage { .. }
        | Request::Limited { .. }
        | Request::Refused { .. } => {}
    }
}

/// Return the terminal of work which the run requested but the service did not start.
pub(super) fn refuse_start(service: &mut Stages, request: &domain::Request) -> bool {
    use domain::{Below, Lower, Request};
    let event = match request {
        Request::Lower(Lower::Complete { owner, .. }) => Below::Cancelled { owner: *owner },
        Request::Lower(Lower::Io { owner, .. }) => Below::Done { owner: *owner, done: domain::tools::Done::Cancelled },
        Request::Lower(Lower::Read { owner, .. }) => Below::Read { owner: *owner, read: super::run::Read::Failed },
        Request::Lower(Lower::Probe { owner, .. }) => Below::Probed { owner: *owner, executable: false },
        Request::Lower(Lower::Check { owner, .. }) => Below::Checked {
            owner: *owner,
            ran: super::run::Ran {
                exit: super::run::Exit::Unstarted,
                said_not_judged: false,
                output: Box::default(),
                cut: 0,
            },
        },
        Request::Lower(
            Lower::Cancel { .. }
            | Lower::Yield { .. }
            | Lower::Close { .. }
            | Lower::Job { .. }
            | Lower::CancelIo { .. }
            | Lower::Abort { .. }
            | Lower::Classify { .. },
        )
        | Request::Admitted { .. }
        | Request::Waiting { .. }
        | Request::Part { .. }
        | Request::Turn { .. }
        | Request::HostCall { .. }
        | Request::WithdrawHost { .. }
        | Request::MessageRefused { .. }
        | Request::Answer { .. }
        | Request::Long { .. }
        | Request::LongDone { .. }
        | Request::Save { .. }
        | Request::Deliver { .. }
        | Request::Unauthorized { .. }
        | Request::Usage { .. }
        | Request::Limited { .. }
        | Request::Refused { .. } => return false,
    };
    service.domain_events.push(domain::Event::Below(event));
    true
}

pub(super) const fn deadline(service: &Stages) -> Option<Time> {
    match service.teardown {
        Teardown::Live { .. } => match service.host {
            Host::Open | Host::Dropped => None,
            Host::Lost { drop_at } => Some(drop_at),
        },
        Teardown::Answered { .. } | Teardown::Closing { .. } | Teardown::Closed { .. } => None,
    }
}

pub(super) fn expire(service: &mut Stages) {
    match deadline(service) {
        Some(at) if service.io_env.now >= at => {
            service.host = Host::Dropped;
            service.domain_events.clear();
            service.domain_requests.clear();
            teardown::dropped(service);
            super::program::close(service);
            service.machine.abort(&mut service.machine_below);
        }
        Some(_) | None => {}
    }
}
