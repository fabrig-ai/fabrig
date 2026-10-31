//! Upward passes from IO completions through components to the domain.
//! Queued terminals and observations stay in the service; these passes never
//! choose a domain outcome. Contract: smith/docs/design/protocol/agent.md, sections 4–6.

use super::down::drain_facts;
use super::routing::route_io_event;
use super::teardown::Teardown;
use super::{COMPONENT_OWNER_BIT, Failure, Operation, Owner, PendingMessage, Service, credential, program, teardown};
use skein_io::{self as io, file_layer, kernel};
use skein_lib::{Id, Time, Token};
use smith_domain::{self as domain, run};
use smith_protocol_channel as channel;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

pub(super) fn io_up(service: &mut Service) {
    if let Some(io) = &mut service.io {
        for _ in 0..service.limits.io.sockets {
            if !io.is_ready() {
                break;
            }
            io::resume(io, &service.io_env, &mut service.io_events, &mut service.io_submissions);
        }
    }
    for _ in 0..service.completions.capacity() {
        let Some(complete) = service.completions.pop() else { break };
        let route = service.operations.remove(&complete.op).expect("one route per submitted operation");
        match route {
            Operation::Io(original) | Operation::Signals(original) => {
                let complete = kernel::Complete { op: original, kind: complete.kind, result: complete.result };
                io::up(
                    service.io.as_mut().expect("standalone IO completion"),
                    &service.io_env,
                    complete,
                    &mut service.io_events,
                    &mut service.io_submissions,
                );
            }
            Operation::File(original) => {
                let complete = kernel::Complete { op: original, kind: complete.kind, result: complete.result };
                file_layer::up(&mut service.files, complete, &mut service.file_events, &mut service.file_submissions);
            }
        }
    }
    if let Some(io) = &mut service.io {
        for _ in 0..service.limits.io.sockets {
            if !io.is_due(service.io_env.now) {
                break;
            }
            io::fire(io, &service.io_env, &mut service.io_events, &mut service.io_submissions);
        }
    }
    if service.files.is_due(service.io_env.now) {
        file_layer::expire(&mut service.files, service.io_env.now, &mut service.file_submissions);
    }
}

pub(super) fn lower_events(service: &mut Service) {
    for _ in 0..service.limits.queue {
        if service.channel.is_some()
            && (service.channel_events.room() < channel::edge_room(&service.limits.channel).to_domain
                || service.channel_below.room() < channel::edge_room(&service.limits.channel).below)
        {
            break;
        }
        let Some(event) = service.io_events.pop() else { break };
        let signal_ready = match &event {
            io::Event::Shutdown { .. } => service.llm.measures_drained(),
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Failed { .. }
            | io::Event::Closed { .. } => true,
        };
        if !signal_ready {
            service.io_events.push(event);
            continue;
        }
        if !service.llm.measures_drained() && llm_io(service, &event) {
            service.io_events.push(event);
            continue;
        }
        route_io_event(service, event);
    }
    for _ in 0..service.limits.queue {
        let Some(event) = service.file_events.pop() else { break };
        if event.owner() == Token::new(u64::MAX) && service.root_close_pending {
            service.root_close_pending = false;
            continue;
        }
        let Some(event) = program::file_event(service, event) else { continue };
        service.machine.from_below(
            &service.machine_env,
            machine::BelowEvent::File(event),
            &mut service.machine_events,
            &mut service.machine_below,
        );
    }
}

fn machine_up(service: &mut Service) {
    for _ in 0..service.limits.queue {
        let Some(event) = service.machine_events.pop() else { break };
        let event = match event {
            machine::ToDomain::Done { owner, done } => domain::Event::Done { owner, done },
            machine::ToDomain::Read { owner, read } => domain::Event::Read { owner, read },
            machine::ToDomain::Probed { owner, executable } => domain::Event::Probed { owner, executable },
            machine::ToDomain::Checked { owner, ran } => domain::Event::Checked { owner, ran },
            machine::ToDomain::Aborted { owner } => domain::Event::Aborted { owner },
        };
        service.domain_events.push(event);
    }
}

fn llm_io(service: &Service, event: &io::Event) -> bool {
    let Some(owner) = event.owner() else { return false };
    if service.pipe_routes.contains_key(&owner) || owner.raw() & COMPONENT_OWNER_BIT == 0 {
        return false;
    }
    let id = Id::<Owner>::from_token(Token::new(owner.raw() & !COMPONENT_OWNER_BIT));
    match service.owners.get(id) {
        Some(Owner::Llm(_)) => true,
        Some(Owner::Machine(_) | Owner::Program(_)) | None => false,
    }
}

pub(super) fn live_step(service: &mut Service, action: crate::live::Action<'_>) {
    let Some(channel) = &mut service.channel else {
        return;
    };
    let ending = match &action {
        crate::live::Action::End { .. } => true,
        crate::live::Action::Call { .. }
        | crate::live::Action::Arguments { .. }
        | crate::live::Action::Open { .. }
        | crate::live::Action::Delta { .. }
        | crate::live::Action::Pump => false,
    };
    let mut room = crate::live::Connection {
        channel,
        above: &mut service.channel_events,
        below: &mut service.channel_below,
        next: &mut service.next_send,
        failed: false,
        room: channel::edge_room(&service.limits.channel),
        ending,
    };
    service.live.act(action, service.io_env.now, &mut room);
    if room.failed {
        service.mark_failed(Failure::Send);
    }
}

fn llm_measures(service: &mut Service) {
    let pending = program::pending_measures(service);
    if !llm_measure_batch(service, pending, false) {
        return;
    }
    // Recording the one prepared prompt can append one finite child batch.
    // The second snapshot drains it; further work needs another gated entrance.
    for _ in 0_u32..2 {
        let pending = service.llm.pending_measures();
        if !llm_measure_batch(service, pending, true) {
            return;
        }
    }
}

fn llm_measure_batch(service: &mut Service, pending: u32, direct: bool) -> bool {
    for _ in 0..pending {
        match &service.events {
            Some(events) if !events.takes() => return false,
            Some(_) | None => {}
        }
        let measure = if direct { service.llm.pop_measure() } else { program::pop_measure(service) }
            .expect("the batch retains its counted observations");
        let prompt_owner = match &measure {
            llm::Measure::Prompt { owner, .. } => Some(*owner),
            llm::Measure::Completion { .. }
            | llm::Measure::Call { .. }
            | llm::Measure::Arguments { .. }
            | llm::Measure::Delta { .. }
            | llm::Measure::Terminal { .. }
            | llm::Measure::Retried { .. } => None,
        };
        match &measure {
            llm::Measure::Delta { owner, text, .. } => {
                live_step(service, crate::live::Action::Delta { owner: *owner, bytes: text });
            }
            llm::Measure::Call { owner, index, id, name, .. } => {
                live_step(service, crate::live::Action::Call { owner: *owner, index: *index, id, tool: name });
            }
            llm::Measure::Arguments { owner, index, bytes, .. } => {
                live_step(service, crate::live::Action::Arguments { owner: *owner, index: *index, bytes });
            }
            llm::Measure::Prompt { .. }
            | llm::Measure::Completion { .. }
            | llm::Measure::Terminal { .. }
            | llm::Measure::Retried { .. } => {}
        }
        match &mut service.events {
            Some(events) => events.measure(measure),
            None => match measure {
                llm::Measure::Terminal { .. }
                | llm::Measure::Call { .. }
                | llm::Measure::Arguments { .. }
                | llm::Measure::Delta { .. }
                | llm::Measure::Retried { .. } => {}
                llm::Measure::Prompt { .. } | llm::Measure::Completion { .. } => {
                    unreachable!("only an event owner selects component content")
                }
            },
        }
        if direct && let Some(owner) = prompt_owner {
            service.llm.prompt_recorded(&service.llm_env, owner, &mut service.llm_events, &mut service.llm_below);
        }
    }
    true
}

pub(super) fn component_up(service: &mut Service) {
    program::fire(service);
    llm_measures(service);
    let channel_room = channel::edge_room(&service.limits.channel);
    if service.channel_events.room() >= channel_room.to_domain
        && service.channel_below.room() >= channel_room.below
        && let Some(channel) = &mut service.channel
    {
        channel.fire(&mut service.channel_events, &mut service.channel_below);
    }
    live_step(service, crate::live::Action::Pump);
    let llm_due = match service.llm.next_deadline() {
        Some(at) => at <= service.llm_env.now,
        None => false,
    };
    if (service.llm.has_work() || llm_due) && service.llm.measures_drained() {
        service.llm.fire(&service.llm_env, &mut service.llm_events, &mut service.llm_below);
    }
    llm_measures(service);
    let machine_due = match service.machine.next_deadline() {
        Some(at) => at <= service.machine_env.now,
        None => false,
    };
    if machine_due {
        service.machine.fire(&service.machine_env, &mut service.machine_events, &mut service.machine_below);
    }
    for _ in 0..service.channel_events.capacity() {
        if !super::channel_event_ready(service) {
            break;
        }
        let Some(event) = service.channel_events.pop() else { break };
        channel_event(service, event);
    }
    llm_events_up(service);
    machine_up(service);
}

fn llm_events_up(service: &mut Service) {
    for _ in 0..service.limits.queue {
        match &service.events {
            Some(events) if !events.takes() => break,
            Some(_) | None => {}
        }
        if !service.llm.measures_drained()
            || (service.channel.is_some()
                && (service.channel_events.room() < channel::edge_room(&service.limits.channel).to_domain
                    || service.channel_below.room() < channel::edge_room(&service.limits.channel).below))
        {
            break;
        }
        let Some(event) = service.llm_events.pop() else { break };
        match event {
            llm::ToDomain::ReadVersion { owner, place, version } => {
                service.domain_events.push(domain::Event::ReadVersion { owner, place, version });
            }
            llm::ToDomain::BuiltinResult { owner, id, text, error } => {
                service.domain_events.push(domain::Event::BuiltinResult { owner, id, text, error });
            }
            llm::ToDomain::Completed { owner, completion } => {
                if let Some(events) = &mut service.events {
                    events.sinks().attempt_ready(owner);
                    let conversation = events.sinks().received(owner, &completion);
                    events.received(conversation, &completion);
                }
                service.domain_events.push(domain::Event::Completed { owner, completion });
            }
            llm::ToDomain::Failed { owner, failure, evidence, detail, report } => {
                if let Some(events) = &mut service.events {
                    events.sinks().attempt_ready(owner);
                    events.sinks().failed(owner, failure, evidence, &detail, &report);
                }
                service.domain_events.push(domain::Event::Failed { owner, failure, evidence, detail, report });
            }
            llm::ToDomain::Cancelled { owner } => {
                if let Some(events) = &mut service.events {
                    events.sinks().attempt_ready(owner);
                }
                service.domain_events.push(domain::Event::Cancelled { owner });
            }
            llm::ToDomain::Text { owner: _, bytes } => {
                if service.channel.is_none() {
                    continue;
                }
                let token = service.next_send();
                if service
                    .channel
                    .as_mut()
                    .expect("framed agent owns its channel")
                    .send_text_arrived(
                        service.domain_env.now.saturating_since(Time::ZERO),
                        u64::from(bytes),
                        token,
                        &mut service.channel_events,
                        &mut service.channel_below,
                    )
                    .is_err()
                {
                    service.mark_failed(Failure::Send);
                }
            }
        }
    }
}

pub(super) fn channel_event(service: &mut Service, event: channel::OpenEvent) {
    match event {
        channel::OpenEvent::Opened { version: _ } => {}
        channel::OpenEvent::Start { start } => {
            service.live.ask(start.live.clone());
            let count = match &start.mounts {
                Some(mounts) => u32::try_from(mounts.len()).unwrap_or(u32::MAX),
                None => 0,
            };
            if count > service.limits.machine.roots {
                service.refuse_start(run::Invalid::Workspace);
                return;
            }
            let empty = match &start.mounts {
                Some(mounts) => mounts.is_empty(),
                None => true,
            };
            if let Some(events) = &mut service.events {
                events.starting(
                    start.activation,
                    u64::try_from(start.messages.len()).expect("bounded messages"),
                    &start.charter,
                );
            }
            service.pending_start = Some(start);
            if empty {
                service.start_ready();
            }
        }
        channel::OpenEvent::Message { finish_requested, name, label, text } => {
            if let Some(run) = service.admitted {
                service.domain_events.push(domain::Event::Message { finish_requested, run, name, label, text });
            } else {
                service.pending_messages.push(PendingMessage { finish_requested, name, label, text });
            }
        }
        channel::OpenEvent::HostReturned { relay, reply } => {
            service.domain_events.push(domain::Event::HostReturned { relay, reply });
        }
        channel::OpenEvent::Delivered { owner, delivery } => {
            service.domain_events.push(domain::Event::Delivered { owner, delivery: *delivery });
        }
        channel::OpenEvent::DeliveryUnsent { owner } => {
            service.domain_events.push(domain::Event::Delivered { owner, delivery: run::Delivery::Stale });
        }
        channel::OpenEvent::Acknowledged { turn } => {
            if let Some(run) = service.admitted {
                service.domain_events.push(domain::Event::Acknowledge { run, turn });
            }
        }
        channel::OpenEvent::Grant { grant } => {
            let value =
                service.channel.as_ref().expect("framed agent owns its channel").grant_value(grant.name).cloned();
            if let Some(value) = &value
                && program::renew(service, grant, value)
            {
                return;
            }
            match service.channel.as_ref().expect("framed agent owns its channel").grant_value(grant.name) {
                Some(value) => match credential(value) {
                    Some(value) => {
                        let lapses = service.domain_env.now.saturating_add(grant.valid);
                        if service.llm.grant(grant.name, value, lapses).is_ok() {
                            service.domain_events.push(domain::Event::Grant { grant });
                        } else {
                            service.mark_failed(Failure::Credential);
                        }
                    }
                    None => service.mark_failed(Failure::Credential),
                },
                None => service.mark_failed(Failure::Credential),
            }
        }
        channel::OpenEvent::Cancel => {
            if let Some(run) = service.admitted {
                service.domain_events.push(domain::Event::Cancel { run });
            } else {
                service.cancel_pending = true;
            }
        }
        channel::OpenEvent::WriteFailed => service.mark_failed(Failure::WriteFailed),
        channel::OpenEvent::Ended { why: _ } => match service.teardown {
            Teardown::Live => {
                service.mark_failed(Failure::ChannelEnded);
                if let Some(run) = service.admitted {
                    service.domain_events.push(domain::Event::Cancel { run });
                } else {
                    teardown::answered(service);
                }
            }
            Teardown::Answered { .. } | Teardown::Closing { .. } | Teardown::Closed { .. } => {}
        },
    }
}

pub(super) fn domain_up(service: &mut Service) {
    drain_facts(service);
    // An entrance's facts drain to the sinks before the next entrance.
    for _ in 0..service.limits.domain.run.conversations {
        if !super::domain_requests_dispatched(service)
            || !service.domain.as_ref().expect("framed agent owns its domain").is_ready()
            || service.domain.as_ref().expect("framed agent owns its domain").has_facts()
        {
            break;
        }
        domain::resume(
            service.domain.as_mut().expect("framed agent owns its domain"),
            &service.domain_env,
            &mut service.domain_requests,
        );
        drain_content(service);
        drain_facts(service);
    }
    for _ in 0..service.limits.queue {
        if !super::domain_requests_dispatched(service)
            || service.domain.as_ref().expect("framed agent owns its domain").has_facts()
        {
            break;
        }
        let Some(event) = service.domain_events.pop() else { break };
        observe_messages(service, &event);
        domain::step(
            service.domain.as_mut().expect("framed agent owns its domain"),
            &service.domain_env,
            event,
            &mut service.domain_requests,
        );
        drain_content(service);
        drain_facts(service);
    }
    if super::domain_requests_dispatched(service)
        && service.domain.as_ref().expect("framed agent owns its domain").is_due(service.domain_env.now)
        && !service.domain.as_ref().expect("framed agent owns its domain").has_facts()
    {
        domain::fire(
            service.domain.as_mut().expect("framed agent owns its domain"),
            &service.domain_env,
            &mut service.domain_requests,
        );
        drain_content(service);
        drain_facts(service);
    }
}

// A capture copy waits only on this entrance's facts, never in the input
// queue. The next entrance is held until every prior fact reaches its sinks.
fn observe_messages(service: &mut Service, event: &domain::Event) {
    let Some(events) = &mut service.events else { return };
    match event {
        domain::Event::Start { messages, .. } => {
            for message in messages {
                events.message(message.name, &message.label, &message.text);
            }
        }
        domain::Event::Message { name, label, text, .. } => events.message(*name, label, text),
        domain::Event::HostReturned { .. }
        | domain::Event::Acknowledge { .. }
        | domain::Event::Grant { .. }
        | domain::Event::Cancel { .. }
        | domain::Event::Delivered { .. }
        | domain::Event::Completed { .. }
        | domain::Event::Failed { .. }
        | domain::Event::Cancelled { .. }
        | domain::Event::Done { .. }
        | domain::Event::Read { .. }
        | domain::Event::Probed { .. }
        | domain::Event::Checked { .. }
        | domain::Event::BuiltinResult { .. }
        | domain::Event::ReadVersion { .. }
        | domain::Event::Aborted { .. } => {}
    }
}

// The root's optional content observer is not the typed event stream. Its
// standalone API remains available to domain hosts; this composed service
// captures provider content before decoding and drains each root entrance.
fn drain_content(service: &mut Service) {
    for _ in 0..service.domain.as_ref().expect("framed agent owns its domain").pending_content() {
        match service.domain.as_mut().expect("framed agent owns its domain").pop_content() {
            Some(_) => {}
            None => break,
        }
    }
}

#[cfg(test)]
mod tests;
