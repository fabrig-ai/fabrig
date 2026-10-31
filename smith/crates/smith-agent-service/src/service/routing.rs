//! IO terminals and kernel submissions routed to their original owners.
//! The service retains owner and operation routes; this stage never interprets
//! domain outcomes. Contract: smith/docs/design/protocol/agent.md, sections 4–6.

use super::teardown::{Step, Teardown};
use super::{COMPONENT_OWNER_BIT, Operation, Owner, Service, USAGE_OWNER, program, teardown};
use skein_channel::LowerEvent as ChannelLowerEvent;
use skein_io::{self as io, kernel};
use skein_lib::{Id, Token, stream};
use smith_protocol_machine as machine;

pub(super) fn record_usage(service: &mut Service, usage: io::Measured) {
    let (at, answer, close) = match service.teardown {
        Teardown::Closing { at, answer, close, step: Step::Usage } => (at, answer, close),
        Teardown::Live | Teardown::Answered { .. } | Teardown::Closing { .. } | Teardown::Closed { .. } => {
            unreachable!("one requested usage terminal")
        }
    };
    service.usage_unread = match usage {
        io::Measured::Read(_) => false,
        io::Measured::Unread => true,
    };
    #[expect(clippy::manual_map, reason = "strict production entries avoid closures")]
    let channel = match &service.channel {
        Some(channel) => Some(channel.lost_facts()),
        None => None,
    };
    let events = service.events.as_mut().expect("only the event producer requests usage");
    let teardown = match events.answer_at() {
        Some(answer) => Some(service.io_env.now.saturating_since(answer)),
        None => None,
    };
    let origin = events.origin();
    let answered = match service.product_exit {
        Some(_) => events.answer_at().is_some(),
        None => answer.sent(),
    };
    events.sinks().ended(
        service.io_env.now,
        origin,
        &smith_protocol_events::sinks::Ended {
            exit: service.product_exit.unwrap_or(u8::from(!answer.sent())),
            answered,
            aborted: match close {
                teardown::Close::Graceful => false,
                teardown::Close::Aborted => true,
            },
            teardown,
            usage,
            channel,
        },
    );
    service.teardown = Teardown::Closing { at, answer, close, step: Step::Stream };
}

pub(super) fn route_io_event(service: &mut Service, event: io::Event) {
    match &event {
        io::Event::Usage { owner, usage } if *owner == USAGE_OWNER => {
            record_usage(service, *usage);
            return;
        }
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
        | io::Event::Shutdown { .. }
        | io::Event::Failed { .. }
        | io::Event::Closed { .. } => {}
    }
    if let Some(events) = &mut service.events {
        match event.owner() {
            Some(owner) if events.owns(owner) => {
                events.sinks().up(service.io_env.now, event);
                return;
            }
            Some(_) | None => {}
        }
    }
    match event {
        io::Event::Resized | io::Event::Window { .. } => {}
        io::Event::Shutdown { .. } => teardown::shutdown(service),
        io::Event::Stream { owner, up } if Some(owner) == service.input => {
            service.channel.as_mut().expect("framed agent owns its channel").from_below(
                ChannelLowerEvent::Read(up),
                &mut service.channel_events,
                &mut service.channel_below,
            );
        }
        io::Event::Output { owner, up } if Some(owner) == service.output => {
            service.channel.as_mut().expect("framed agent owns its channel").from_below(
                ChannelLowerEvent::Write(up),
                &mut service.channel_events,
                &mut service.channel_below,
            );
        }
        io::Event::Stream { owner, up } if Some(owner) == service.output => match up {
            stream::Up::Failed(fault) => service.channel.as_mut().expect("framed agent owns its channel").from_below(
                ChannelLowerEvent::WriteFailed(fault),
                &mut service.channel_events,
                &mut service.channel_below,
            ),
            stream::Up::Bytes(_) | stream::Up::Room | stream::Up::End => {}
        },
        io::Event::Closed { owner } if Some(owner) == service.input => {
            service.input = None;
            service.channel.as_mut().expect("framed agent owns its channel").from_below(
                ChannelLowerEvent::Read(stream::Up::End),
                &mut service.channel_events,
                &mut service.channel_below,
            );
        }
        io::Event::Closed { owner } if Some(owner) == service.output => service.output = None,
        io::Event::Closed { owner } if Some(owner) == service.signals => service.signals = None,
        event @ (io::Event::Listening { .. }
        | io::Event::Accepted { .. }
        | io::Event::Connecting { .. }
        | io::Event::Connected { .. }
        | io::Event::Stream { .. }
        | io::Event::Output { .. }
        | io::Event::Spawned { .. }
        | io::Event::Exited { .. }
        | io::Event::Usage { .. }
        | io::Event::Failed { .. }
        | io::Event::Closed { .. }) => route_owned_io(service, event),
    }
}

pub(super) fn route_owned_io(service: &mut Service, event: io::Event) {
    let Some(event) = program::pipe_event(service, event) else { return };
    let owner = event.owner().expect("only signal events lack owner");
    if service.pipe_routes.contains_key(&owner) {
        let closed = event.is_closed();
        service.machine.from_below(
            &service.machine_env,
            machine::BelowEvent::Pipe(event),
            &mut service.machine_events,
            &mut service.machine_below,
        );
        if closed {
            service.pipe_routes.remove(&owner);
        }
        return;
    }
    let id = Id::<Owner>::from_token(Token::new(owner.raw() & !COMPONENT_OWNER_BIT));
    let route = *service.owners.get(id).expect("IO event has a live component route");
    let closed = event.is_closed();
    match route {
        Owner::Llm(original) => {
            let event = event.with_owner(original);
            service.llm.from_below(&service.llm_env, event, &mut service.llm_events, &mut service.llm_below);
        }
        Owner::Program(original) => {
            let event = event.with_owner(original);
            program::io_event(service, event);
        }
        Owner::Machine(original) => {
            if let io::Event::Spawned { pipes, .. } = &event {
                for pipe in pipes {
                    service.pipe_routes.insert(*pipe, original).expect("bounded process pipes");
                }
            }
            let event = event.with_owner(original);
            service.machine.from_below(
                &service.machine_env,
                machine::BelowEvent::Process(event),
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
    }
    if closed {
        service.owners.retire(id);
    }
}

pub(super) fn component_owner(route: Token) -> Token {
    assert!(route.raw() & COMPONENT_OWNER_BIT == 0, "one process cannot exhaust the component owner namespace");
    Token::new(route.raw() | COMPONENT_OWNER_BIT)
}

pub(super) fn io_down(service: &mut Service) {
    if let Some(io) = &mut service.io {
        for _ in 0..service.limits.queue {
            if !io.takes() {
                break;
            }
            let Some(request) = service.io_requests.pop() else { break };
            io::down(io, &service.io_env, request, &mut service.io_submissions);
        }
    }
    for _ in 0..service.limits.queue {
        if !super::submissions_ready(service) {
            break;
        }
        let Some(submit) = service.io_submissions.pop() else { break };
        route_submission(service, submit, false);
    }
    for _ in 0..service.limits.queue {
        if !super::submissions_ready(service) {
            break;
        }
        let Some(submit) = service.file_submissions.pop() else { break };
        route_submission(service, submit, true);
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the kernel operation is an external vocabulary and only Cancel names another operation"
)]
pub(super) fn route_submission(service: &mut Service, submit: kernel::Submit, file: bool) {
    let global = Token::new(service.next_operation);
    service.next_operation = service.next_operation.checked_add(1).expect("one run does not exhaust operation names");
    let kind = match submit.kind {
        kernel::Op::Cancel { target } => {
            let mut found = None;
            for (global, route) in &service.operations {
                match route {
                    Operation::File(original) if file && *original == target => found = Some(*global),
                    Operation::Io(original) | Operation::Signals(original) if !file && *original == target => {
                        found = Some(*global);
                    }
                    Operation::File(_) | Operation::Io(_) | Operation::Signals(_) => {}
                }
            }
            kernel::Op::Cancel { target: found.unwrap_or(Token::new(u64::MAX)) }
        }
        other => other,
    };
    let route = if file {
        Operation::File(submit.op)
    } else {
        match kind {
            kernel::Op::ReadSignal { .. } => Operation::Signals(submit.op),
            _ => Operation::Io(submit.op),
        }
    };
    service.operations.insert(global, route).expect("reserved operation route");
    service.submissions.push(kernel::Submit { op: global, kind });
}
