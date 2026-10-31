//! Downward passes from queued domain requests to lower components.
//! Facts, messages and lower requests stay in the service; this stage never
//! chooses their domain meaning. Contract: smith/docs/design/protocol/agent.md, sections 4–6.

use super::request::domain_request;
use super::routing::component_owner;
use super::{Failure, Frame, Owner, PendingMessage, Stages, face, program};
use skein_channel::Lower as ChannelLower;
use skein_io::{self as io, file, file_layer, kernel};
use skein_lib::stream;
use smith_domain as domain;
use smith_protocol_machine as machine;

pub(super) fn domain_down(service: &mut Stages, mut frame: Option<&mut Frame>) {
    drain_facts(service, frame.as_deref_mut());
    match face::domain(frame.as_deref()) {
        Some(domain) if domain.has_facts() => return,
        Some(_) | None => {}
    }
    if let Some(run) = service.admitted {
        for _ in 0..service.pending_messages.capacity() {
            if !super::pending_message_ready(service) {
                break;
            }
            let Some(PendingMessage { finish_requested, name, label, text }) = service.pending_messages.pop() else {
                break;
            };
            service.domain_events.push(domain::Event::Message { finish_requested, run, name, label, text });
        }
    }
    for _ in 0..service.limits.queue {
        if !super::domain_request_credit(service, frame.as_deref()) {
            break;
        }
        if program::wait_for_close(service) {
            break;
        }
        let Some(request) = service.domain_requests.pop() else { break };
        domain_request(service, frame.as_deref_mut(), request);
    }
    match service.admitted {
        Some(run) if service.cancel_pending => {
            service.domain_events.push(domain::Event::Cancel { run });
            service.cancel_pending = false;
        }
        Some(_) | None => {}
    }
    drain_facts(service, frame);
}

/// A queued request waits on kernel output credit; a colocated runtime has no channel.
pub(super) fn channel_takes(frame: Option<&Frame>) -> bool {
    match face::channel(frame) {
        Some(channel) => channel.takes(),
        None => true,
    }
}

/// A single take offers each observation to the independent sinks. The trace
/// continues through answer and settlement; channel projection owns its phase.
pub(super) fn drain_facts(service: &mut Stages, mut frame: Option<&mut Frame>) {
    for _ in 0..service.limits.queue {
        if !super::sinks_take(service) {
            break;
        }
        let fact = match face::domain_mut(frame.as_deref_mut()) {
            Some(domain) => match domain.pop_fact() {
                Some(fact) => fact,
                None => break,
            },
            None => break,
        };
        let token = service.next_send();
        if let Some(channel) = face::channel_mut(frame.as_deref_mut())
            && channel
                .send_fact(&fact, service.run_started, token, &mut service.channel_events, &mut service.channel_below)
                .is_err()
        {
            service.mark_failed(Failure::Send);
        }
        if let Some(events) = &mut service.events {
            events.fact(&fact);
        }
    }
    let drained = match face::domain(frame.as_deref()) {
        Some(domain) => !domain.has_facts(),
        None => true,
    };
    if drained && let Some(events) = &mut service.events {
        for _ in 0..service.limits.domain.run.conversations {
            if !events.takes() || !events.facts_drained() {
                break;
            }
        }
    }
}

#[expect(clippy::too_many_lines, reason = "bounded lower routing stays in one down-pass stage")]
pub(super) fn component_down(service: &mut Stages) {
    for _ in 0..service.channel_below.capacity() {
        let Some(request) = service.channel_below.pop() else { break };
        match request {
            ChannelLower::Read(down) => {
                if let Some(input) = service.input {
                    service.io_requests.push(io::Request::Stream { stream: input, down });
                }
            }
            ChannelLower::Write(down) => {
                if let Some(output) = service.output {
                    service.io_requests.push(io::Request::Output { stream: output, down });
                }
            }
            ChannelLower::FinishWrite => {
                if let Some(output) = service.output {
                    service.io_requests.push(io::Request::Stream { stream: output, down: stream::Down::Finish });
                }
            }
        }
    }
    program::down(service);
    for _ in 0..service.limits.queue {
        let Some(request) = service.llm_below.pop() else { break };
        match request {
            io::Request::Connect { owner, addr } => {
                let route = service.owners.insert(Owner::Llm(owner)).expect("LLM connection route capacity");
                service.io_requests.push(io::Request::Connect { owner: component_owner(route.token()), addr });
            }
            other @ (io::Request::Listen { .. }
            | io::Request::Bind { .. }
            | io::Request::Reject { .. }
            | io::Request::Stream { .. }
            | io::Request::Output { .. }
            | io::Request::Spawn { .. }
            | io::Request::Signal { .. }
            | io::Request::Window { .. }
            | io::Request::Usage { .. }
            | io::Request::Close { .. }
            | io::Request::Abort { .. }) => service.io_requests.push(other),
        }
    }
    for _ in 0..service.limits.queue {
        let waits_for_file = match service.machine_below.iter().next() {
            Some(
                machine::Below::Remove { .. }
                | machine::Below::File { .. }
                | machine::Below::OpenRead { .. }
                | machine::Below::OpenDirectory { .. },
            ) => !service.files.takes(),
            Some(machine::Below::CancelFile { .. } | machine::Below::Spawn { .. } | machine::Below::Process(_))
            | None => false,
        };
        if waits_for_file {
            break;
        }
        let Some(request) = service.machine_below.pop() else { break };
        match request {
            machine::Below::File { request, deadline } => {
                service.files.seed_randomness(service.next_operation);
                file_layer::down_until(
                    &mut service.files,
                    deadline,
                    request,
                    &mut service.file_events,
                    &mut service.file_submissions,
                );
            }
            machine::Below::CancelFile { owner } => {
                file_layer::cancel(&mut service.files, owner, &mut service.file_submissions);
            }
            machine::Below::OpenRead { owner, root, path, deadline } => {
                let descriptor = service.files.descriptor(root).expect("machine admitted an installed root");
                file_layer::down_until(
                    &mut service.files,
                    deadline,
                    file::Request::OpenRead { owner, root: descriptor, name: path },
                    &mut service.file_events,
                    &mut service.file_submissions,
                );
            }
            machine::Below::Remove { owner, parent, name, directory, deadline } => {
                let descriptor = service.files.descriptor(parent).expect("removal owns its parent");
                file_layer::down_until(
                    &mut service.files,
                    deadline,
                    file::Request::Remove { owner, root: descriptor, name, directory },
                    &mut service.file_events,
                    &mut service.file_submissions,
                );
            }
            machine::Below::OpenDirectory { owner, root, path, no_follow, deadline } => {
                let descriptor = service.files.descriptor(root).expect("machine admitted an installed root");
                file_layer::down_until(
                    &mut service.files,
                    deadline,
                    file::Request::OpenDirectory { owner, root: descriptor, name: path, no_follow },
                    &mut service.file_events,
                    &mut service.file_submissions,
                );
            }
            machine::Below::Spawn { owner, root, spawn } => {
                let descriptor = service.files.descriptor(root).expect("machine admitted an installed root");
                let spawn = kernel::Spawn {
                    program: spawn.program,
                    args: spawn.args,
                    env: spawn.env,
                    root: descriptor,
                    dir: spawn.dir,
                    pipes: spawn.pipes,
                };
                let route = service.owners.insert(Owner::Machine(owner)).expect("process route capacity");
                service.io_requests.push(io::Request::Spawn { owner: component_owner(route.token()), spawn });
            }
            machine::Below::Process(request) => service.io_requests.push(request),
        }
    }
}
