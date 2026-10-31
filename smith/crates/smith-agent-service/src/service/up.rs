//! Upward passes from IO completions through components to the domain.
//! Queued terminals and observations stay in the service; these passes never
//! choose a domain outcome. Contract: smith/docs/design/protocol/agent.md, sections 4–6.

use super::down::drain_facts;
use super::routing::route_io_event;
use super::teardown::Teardown;
use super::{
    COMPONENT_OWNER_BIT, Failure, Frame, Operation, Owner, PendingMessage, Stages, credential, face, program, teardown,
};
use skein_io::{self as io, file_layer, kernel};
use skein_lib::{Id, Time, Token};
use smith_domain::{self as domain, run};
use smith_protocol_channel as channel;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

pub(super) fn io_up(service: &mut Stages, mut frame: Option<&mut Frame>) {
    if let Some(io) = face::io_mut(frame.as_deref_mut()) {
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
                let Some(frame) = frame.as_deref_mut() else {
                    unreachable!("only framed IO issues native completions")
                };
                io::up(&mut frame.io, &service.io_env, complete, &mut service.io_events, &mut service.io_submissions);
            }
            Operation::File(original) => {
                let complete = kernel::Complete { op: original, kind: complete.kind, result: complete.result };
                file_layer::up(&mut service.files, complete, &mut service.file_events, &mut service.file_submissions);
            }
        }
    }
    if let Some(io) = face::io_mut(frame) {
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

pub(super) fn lower_events(service: &mut Stages, mut frame: Option<&mut Frame>) {
    for _ in 0..service.limits.queue {
        if !super::channel_room(service, frame.as_deref()) {
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
        route_io_event(service, frame.as_deref_mut(), event);
    }
    for _ in 0..service.limits.queue {
        let Some(event) = service.file_events.pop() else { break };
        if event.owner() == super::ROOT_CLOSE_OWNER && service.root_close_pending {
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

fn lower_event(service: &mut Stages, framed: bool, event: domain::Below) {
    if framed {
        service.domain_events.push(domain::Event::Below(event));
    } else {
        service.lower_events.push(event);
    }
}

fn machine_up(service: &mut Stages, framed: bool) {
    for _ in 0..service.limits.queue {
        let Some(event) = service.machine_events.pop() else { break };
        let event = match event {
            machine::ToDomain::Done { owner, done } => domain::Below::Done { owner, done },
            machine::ToDomain::Read { owner, read } => domain::Below::Read { owner, read },
            machine::ToDomain::Probed { owner, executable } => domain::Below::Probed { owner, executable },
            machine::ToDomain::Checked { owner, ran } => domain::Below::Checked { owner, ran },
            machine::ToDomain::Aborted { owner } => domain::Below::Aborted { owner },
        };
        lower_event(service, framed, event);
    }
}

fn llm_io(service: &Stages, event: &io::Event) -> bool {
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

pub(super) fn live_step(service: &mut Stages, frame: Option<&mut Frame>, action: crate::live::Action<'_>) {
    let Some(channel) = face::channel_mut(frame) else {
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
    let mut room = super::link::HostLink {
        channel,
        above: &mut service.channel_events,
        below: &mut service.channel_below,
        next: &mut service.next_send,
        failed: false,
        room: channel::edge_room(),
        ending,
    };
    service.live.act(action, service.io_env.now, &mut room);
    let succeeded = !room.failed;
    super::link::sent(service, succeeded);
}

fn llm_measures(service: &mut Stages, mut frame: Option<&mut Frame>) {
    let pending = program::pending_measures(service);
    if !llm_measure_batch(service, frame.as_deref_mut(), pending, false) {
        return;
    }
    // Recording the one prepared prompt can append one finite child batch.
    // The second snapshot drains it; further work needs another gated entrance.
    for _ in 0_u32..2 {
        let pending = service.llm.pending_measures();
        if !llm_measure_batch(service, frame.as_deref_mut(), pending, true) {
            return;
        }
    }
}

fn llm_measure_batch(service: &mut Stages, mut frame: Option<&mut Frame>, pending: u32, direct: bool) -> bool {
    for _ in 0..pending {
        if !super::sinks_take(service) {
            return false;
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
                live_step(service, frame.as_deref_mut(), crate::live::Action::Delta { owner: *owner, bytes: text });
            }
            llm::Measure::Call { owner, index, id, name, .. } => {
                live_step(
                    service,
                    frame.as_deref_mut(),
                    crate::live::Action::Call { owner: *owner, index: *index, id, tool: name },
                );
            }
            llm::Measure::Arguments { owner, index, bytes, .. } => {
                live_step(
                    service,
                    frame.as_deref_mut(),
                    crate::live::Action::Arguments { owner: *owner, index: *index, bytes },
                );
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

pub(super) fn component_up(service: &mut Stages, mut frame: Option<&mut Frame>) {
    program::fire(service);
    llm_measures(service, frame.as_deref_mut());
    if super::channel_room(service, frame.as_deref())
        && let Some(channel) = face::channel_mut(frame.as_deref_mut())
    {
        channel.fire(&mut service.channel_events, &mut service.channel_below);
    }
    live_step(service, frame.as_deref_mut(), crate::live::Action::Pump);
    let llm_due = match service.llm.next_deadline() {
        Some(at) => at <= service.llm_env.now,
        None => false,
    };
    if (service.llm.has_work() || llm_due) && service.llm.measures_drained() {
        service.llm.fire(&service.llm_env, &mut service.llm_events, &mut service.llm_below);
    }
    llm_measures(service, frame.as_deref_mut());
    let machine_due = match service.machine.next_deadline() {
        Some(at) => at <= service.machine_env.now,
        None => false,
    };
    if machine_due {
        service.machine.fire(&service.machine_env, &mut service.machine_events, &mut service.machine_below);
    }
    if let Some(frame) = frame.as_deref_mut() {
        for _ in 0..service.channel_events.capacity() {
            if !super::channel_event_ready(service) {
                break;
            }
            let Some(event) = service.channel_events.pop() else { break };
            channel_event(service, frame, event);
        }
    }
    let framed = frame.is_some();
    llm_events_up(service, frame);
    machine_up(service, framed);
}

fn llm_events_up(service: &mut Stages, mut frame: Option<&mut Frame>) {
    for _ in 0..service.limits.queue {
        if !super::llm_events_ready(service, frame.as_deref()) {
            break;
        }
        let Some(event) = service.llm_events.pop() else { break };
        match event {
            llm::ToDomain::ReadVersion { owner, place, version } => {
                lower_event(service, frame.is_some(), domain::Below::ReadVersion { owner, place, version });
            }
            llm::ToDomain::BuiltinResult { owner, id, text, error } => {
                lower_event(service, frame.is_some(), domain::Below::BuiltinResult { owner, id, text, error });
            }
            llm::ToDomain::Completed { owner, completion } => {
                if let Some(events) = &mut service.events {
                    events.sinks().attempt_ready(owner);
                    let conversation = events.sinks().received(owner, &completion);
                    events.received(conversation, &completion);
                }
                lower_event(service, frame.is_some(), domain::Below::Completed { owner, completion });
            }
            llm::ToDomain::Failed { owner, failure, evidence, detail, report } => {
                if let Some(events) = &mut service.events {
                    events.sinks().attempt_ready(owner);
                    events.sinks().failed(owner, failure, evidence, &detail, &report);
                }
                lower_event(
                    service,
                    frame.is_some(),
                    domain::Below::Failed { owner, failure, evidence, detail, report },
                );
            }
            llm::ToDomain::Cancelled { owner } => {
                if let Some(events) = &mut service.events {
                    events.sinks().attempt_ready(owner);
                }
                lower_event(service, frame.is_some(), domain::Below::Cancelled { owner });
            }
            llm::ToDomain::Text { owner: _, bytes } => {
                let Some(frame) = frame.as_deref_mut() else { continue };
                let token = service.next_send();
                let result = frame.channel.send_text_arrived(
                    service.domain_env.now.saturating_since(Time::ZERO),
                    u64::from(bytes),
                    token,
                    &mut service.channel_events,
                    &mut service.channel_below,
                );
                super::link::sent(service, result.is_ok());
            }
        }
    }
}

pub(super) fn channel_event(service: &mut Stages, frame: &mut Frame, event: channel::Heard) {
    match event {
        channel::Heard::Opened { version: _ } => {}
        channel::Heard::Start { start } => {
            service.live.ask(start.live.clone());
            let count = match &start.mounts {
                Some(mounts) => u32::try_from(mounts.len()).unwrap_or(u32::MAX),
                None => 0,
            };
            if count > service.limits.machine.roots {
                service.refuse_start(frame, run::Invalid::Workspace);
                return;
            }
            let empty = match &start.mounts {
                Some(mounts) => mounts.is_empty(),
                None => true,
            };
            if let Some(events) = &mut service.events {
                events.starting(start.activation, skein_lib::len_u64(start.messages.len()), &start.charter);
            }
            service.pending_start = Some(start);
            if empty {
                service.start_ready(frame);
            }
        }
        channel::Heard::Message { finish_requested, name, label, text } => {
            if let Some(run) = service.admitted {
                service.domain_events.push(domain::Event::Message { finish_requested, run, name, label, text });
            } else {
                service.pending_messages.push(PendingMessage { finish_requested, name, label, text });
            }
        }
        channel::Heard::Event { event } => service.domain_events.push(*event),
        channel::Heard::DeliveryUnsent { owner } => {
            service.domain_events.push(domain::Event::Delivered { owner, delivery: run::Delivery::Stale });
        }
        channel::Heard::Acknowledged { turn } => {
            if let Some(run) = service.admitted {
                service.domain_events.push(domain::Event::Acknowledge { run, turn });
            }
        }
        channel::Heard::Grant { grant } => {
            let value = frame.channel.grant_value(grant.name).cloned();
            if let Some(value) = &value
                && program::renew(service, grant, value)
            {
                return;
            }
            match frame.channel.grant_value(grant.name) {
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
        channel::Heard::Cancel => {
            if let Some(run) = service.admitted {
                service.domain_events.push(domain::Event::Cancel { run });
            } else {
                service.cancel_pending = true;
            }
        }
        channel::Heard::WriteFailed => service.mark_failed(Failure::WriteFailed),
        channel::Heard::Ended { why: _ } => match service.teardown {
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

pub(super) fn domain_up(service: &mut Stages, frame: &mut Frame) {
    drain_facts(service, Some(&mut *frame));
    // An entrance's facts drain to the sinks before the next entrance.
    for _ in 0..service.limits.domain.run.conversations {
        if !super::domain_entrance_ready(service, Some(&*frame)) || !frame.domain.is_ready() {
            break;
        }
        domain::resume(&mut frame.domain, &service.domain_env, &mut service.domain_requests);
        drain_content(frame);
        drain_facts(service, Some(&mut *frame));
    }
    for _ in 0..service.limits.queue {
        if !super::domain_entrance_ready(service, Some(&*frame)) {
            break;
        }
        let Some(event) = service.domain_events.pop() else { break };
        observe_messages(service, &event);
        domain::step(&mut frame.domain, &service.domain_env, event, &mut service.domain_requests);
        drain_content(frame);
        drain_facts(service, Some(&mut *frame));
    }
    if super::domain_entrance_ready(service, Some(&*frame)) && frame.domain.is_due(service.domain_env.now) {
        domain::fire(&mut frame.domain, &service.domain_env, &mut service.domain_requests);
        drain_content(frame);
        drain_facts(service, Some(&mut *frame));
    }
}

// A capture copy waits only on this entrance's facts, never in the input
// queue. The next entrance is held until every prior fact reaches its sinks.
fn observe_messages(service: &mut Stages, event: &domain::Event) {
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
        | domain::Event::Below(
            smith_domain::Below::Completed { .. }
            | smith_domain::Below::Failed { .. }
            | smith_domain::Below::Cancelled { .. }
            | smith_domain::Below::Done { .. }
            | smith_domain::Below::Read { .. }
            | smith_domain::Below::Probed { .. }
            | smith_domain::Below::Checked { .. }
            | smith_domain::Below::BuiltinResult { .. }
            | smith_domain::Below::ReadVersion { .. }
            | smith_domain::Below::Aborted { .. },
        ) => {}
    }
}

// The root's optional content observer is not the typed event stream. Its
// standalone API remains available to domain hosts; this composed service
// captures provider content before decoding and drains each root entrance.
fn drain_content(frame: &mut Frame) {
    for _ in 0..frame.domain.pending_content() {
        match frame.domain.pop_content() {
            Some(_) => {}
            None => break,
        }
    }
}

#[cfg(test)]
mod tests;
