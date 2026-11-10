//! Assemble outputs in the decision that owns them. A copying application
//! replaces its tools and connector results; charter conversion stays shared
//! (domain/root.md, 7; domain/hosts/charters.md).
use crate::domain::Work;
use crate::{Domain, Limits};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_ops_domain_connectors::Connector;
use jig_protocol_charter as charter;
use skein_lib::{Env, Queue, ReplyTo};

pub(crate) fn assignment(
    domain: &Domain,
    env: &Env<Limits>,
    assignment: &jig_domain_hub_link::Assignment,
) -> Option<host::Assignment> {
    charter::hosted(
        assignment,
        charter::RootParts {
            tools: domain.tools.clone(),
            endpoints: &domain.charter_endpoints,
            deployment: &domain.deployment,
            limits: &env.limits.charter,
        },
    )
}
pub(crate) fn read_answer(
    domain: &mut Domain,
    call: jig_domain_hub_link::ReadCall,
    error: bool,
    body: Box<[u8]>,
    work: &mut Queue<Work>,
) {
    let answer = domain.hub_link.retain(jig_domain_hub_link::Payload::SettledCall(core::SettledCall {
        serial: 0,
        tool: call.tool,
        answer: core::SettledAnswer::Host { error, body },
    }));
    work.push(Work::Core(core::Event::Fleet(jig_domain_fleet::Event::Relayed { to: ReplyTo::new(call.to), answer })));
}

pub(crate) fn connector_read(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    task: u64,
    tool: Box<[u8]>,
    read: jig_ops_domain_observability::Read,
    work: &mut Queue<Work>,
) {
    let binding = jig_domain_hub_link::ReadCall { to: to.into_token(), tool };
    let number = jig_ops_domain_connectors::number(Connector::Observability);
    let description = crate::translate::read_description(number, &read, &env.limits.observability.vocabulary);
    let mut findings = Queue::uncapped();
    if domain.core.connector_read_admit(task, description, &mut findings) == jig_domain_authority::Answer::Allow {
        let token = domain.token();
        domain.hub_link.keep_read_call(token, binding);
        work.push(Work::Observability(jig_ops_domain_observability::Event::Read { token, read }));
    } else {
        read_answer(domain, binding, true, Box::from(&b"refused"[..]), work);
    }
}
