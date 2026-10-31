//! Translation of one typed domain request to its component or host channel.
//! The service retains delivery and observation state; translation never chooses
//! a domain result. Contract: smith/docs/design/protocol/agent.md, sections 4–6.

use super::up::live_step;
use super::{Frame, Stages, link, program};
use smith_domain as domain;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

#[expect(clippy::too_many_lines, reason = "one exhaustive root-domain request translation")]
pub(super) fn domain_request(service: &mut Stages, mut frame: Option<&mut Frame>, request: domain::Request) {
    match request {
        domain::Request::Complete {
            owner,
            name,
            grant,
            prompt,
            deadlines,
            max_completion_bytes,
            max_completion_blocks,
            max_failure_bytes,
            decoded_call_bytes,
        } => {
            let observation = match &mut service.events {
                Some(events) => events.request(service.domain_env.now, owner, &prompt),
                None => None,
            };
            let wanted = service.live.wants(&prompt);
            if wanted {
                live_step(service, frame.as_deref_mut(), crate::live::Action::Open { owner, retry: name.retry });
            }
            let deltas = service.live.deltas(&prompt, observation.is_some());
            let outcome = service.outcome.clone().expect("an admitted Start retains its result contract");
            let request = llm::FromDomain::Complete {
                owner,
                grant,
                prompt,
                deadlines,
                bounds: llm::Receiving {
                    max_completion_bytes,
                    max_completion_blocks,
                    max_failure_bytes,
                    decoded_call_bytes,
                },
                outcome,
                deliver: service.delivery.clone(),
                observation,
                deltas,
            };
            program::complete(service, request);
        }
        domain::Request::Yield { owner } => {
            program::complete(service, llm::FromDomain::Yield { owner });
        }
        domain::Request::Cancel { owner } => {
            program::complete(service, llm::FromDomain::Cancel { owner });
        }
        domain::Request::Io { owner, op, deadline } => {
            service.machine.from_domain(
                &service.machine_env,
                machine::FromDomain::Op { owner, op, deadline },
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
        domain::Request::Job { owner, act } => {
            service.machine.from_domain(
                &service.machine_env,
                machine::FromDomain::Job { owner, act },
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
        domain::Request::CancelIo { owner } => {
            service.machine.from_domain(
                &service.machine_env,
                machine::FromDomain::Cancel { owner },
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
        domain::Request::Read { owner, at, max, deadline } => {
            service.machine.from_domain(
                &service.machine_env,
                machine::FromDomain::Read { owner, at, max, deadline },
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
        domain::Request::Probe { owner, at, deadline } => {
            service.machine.from_domain(
                &service.machine_env,
                machine::FromDomain::Probe { owner, at, deadline },
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
        domain::Request::Check { owner, program, deadline, tail } => {
            service.machine.from_domain(
                &service.machine_env,
                machine::FromDomain::Check { owner, program, deadline, tail },
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
        domain::Request::Abort { owner } => {
            service.machine.from_domain(
                &service.machine_env,
                machine::FromDomain::Abort { owner },
                &mut service.machine_events,
                &mut service.machine_below,
            );
        }
        request @ (domain::Request::Waiting { .. }
        | domain::Request::Turn { .. }
        | domain::Request::HostCall { .. }
        | domain::Request::WithdrawHost { .. }
        | domain::Request::MessageRefused { .. }
        | domain::Request::Admitted { .. }
        | domain::Request::Answer { .. }
        | domain::Request::Long { .. }
        | domain::Request::LongDone { .. }
        | domain::Request::Deliver { .. }
        | domain::Request::Unauthorized { .. }
        | domain::Request::Limited { .. }
        | domain::Request::Refused { .. }) => {
            let Some(frame) = frame else { unreachable!("Effects admits only lower requests") };
            link::request(service, frame, request);
        }
    }
}
