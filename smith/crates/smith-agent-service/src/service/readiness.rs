//! Admission and downstream credit shared by the passes and shell readiness.
//! The predicates inspect lower queues and optional framed consumers without
//! advancing them. Contract: skein/docs/foundation/programming-model.md, section 7.2.

use super::stages::backlog;
use super::{Frame, Stages, face};
use smith_protocol_channel as channel;

// Dispatch an entrance's effects before advancing the domain again. Its facts
// must not overtake the preceding turn and channel notifications.
pub(super) fn domain_requests_dispatched(service: &Stages) -> bool {
    service.domain_requests.is_empty()
}

fn admits(service: &Stages) -> bool {
    backlog(service) < u64::from(service.limits.queue)
}

// The drain and shell readiness agree on both admission and the retained
// pending-message edge credit before offering the head channel event.
pub(super) fn channel_event_ready(service: &Stages) -> bool {
    if service.admitted.is_none() && service.pending_messages.room() == 0 {
        return false;
    }
    match service.channel_events.iter().next() {
        Some(channel::OpenEvent::Start { .. } | channel::OpenEvent::Message { .. }) => admits(service),
        Some(
            channel::OpenEvent::Opened { .. }
            | channel::OpenEvent::HostReturned { .. }
            | channel::OpenEvent::Delivered { .. }
            | channel::OpenEvent::DeliveryUnsent { .. }
            | channel::OpenEvent::Acknowledged { .. }
            | channel::OpenEvent::Grant { .. }
            | channel::OpenEvent::Cancel
            | channel::OpenEvent::WriteFailed
            | channel::OpenEvent::Ended { .. },
        ) => true,
        None => false,
    }
}

pub(super) fn sinks_take(service: &Stages) -> bool {
    match &service.events {
        Some(events) => events.takes(),
        None => true,
    }
}

pub(super) fn channel_room(service: &Stages, frame: Option<&Frame>) -> bool {
    if frame.is_none() {
        return true;
    }
    let room = channel::edge_room(&service.limits.channel);
    service.channel_events.room() >= room.to_domain && service.channel_below.room() >= room.below
}

pub(super) fn pending_message_ready(service: &Stages) -> bool {
    service.admitted.is_some() && admits(service)
}

pub(super) fn domain_entrance_ready(service: &Stages, frame: Option<&Frame>) -> bool {
    domain_requests_dispatched(service)
        && match face::domain(frame) {
            Some(domain) => !domain.has_facts(),
            None => true,
        }
}

pub(super) fn llm_events_ready(service: &Stages, frame: Option<&Frame>) -> bool {
    sinks_take(service) && service.llm.measures_drained() && channel_room(service, frame)
}

pub(super) fn domain_request_credit(service: &Stages, frame: Option<&Frame>) -> bool {
    llm_events_ready(service, frame) && super::down::channel_takes(frame)
}

pub(super) fn submissions_ready(service: &Stages) -> bool {
    service.submissions.room() > 0 && service.operations.len() < service.operations.capacity()
}
