//! Assemble outputs in the decision that owns them. A copying application
//! replaces its tools and connector results; charter conversion stays shared
//! (domain/root.md, 7; domain/hosts.md, 8).
use crate::domain::Work;
use crate::{Domain, Limits};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_ops_domain_connectors::Connector;
use jig_protocol_charter as charter;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(crate) fn brief(order: &[brief::GatherPlaced]) -> Box<[charter::Section]> {
    let mut sections = List::with_capacity(u32::try_from(order.len()).expect("bounded sections"));
    for placed in order {
        let section = match placed {
            brief::GatherPlaced::Core { kind, text } => {
                charter::Section { title: Box::from(title(*kind)), text: text.clone() }
            }
            brief::GatherPlaced::CoreMissing { kind, .. } => {
                charter::Section { title: Box::from(title(*kind)), text: Box::from(&b"[missing]"[..]) }
            }
            brief::GatherPlaced::Missing { .. } => {
                charter::Section { title: Box::from(&b"Context"[..]), text: Box::from(&b"[missing]"[..]) }
            }
            brief::GatherPlaced::Connector { .. } => unreachable!("ops offers no connector brief sections"),
        };
        sections.push(section).expect("one section per placement");
    }
    sections.into_boxed()
}
const fn title(kind: brief::Core) -> &'static [u8] {
    match kind {
        brief::Core::Task => b"Task",
        brief::Core::Lineage => b"Requesters",
        brief::Core::Results => b"Results",
        brief::Core::Plan => b"Plan",
        brief::Core::Attempts => b"Attempts",
        brief::Core::Calls => b"Calls",
        brief::Core::Waiting => b"Waiting",
        brief::Core::NotesIndex => b"Notes",
        brief::Core::TranscriptTail => b"Conversation",
    }
}
pub(crate) fn assignment(
    domain: &Domain,
    env: &Env<Limits>,
    assignment: &jig_domain_hub_link::Assignment,
) -> Option<host::Assignment> {
    let task = assignment.task;
    let attempt = assignment.attempt;
    let conversation = assignment.conversation;
    let live = assignment.live;
    let sections = brief(&assignment.brief);
    let run = (*assignment.run).clone();
    let transcript = assignment.transcript.clone();
    let messages = assignment.messages.clone();
    let grants = assignment.grants.clone();
    let value = charter::charter(charter::assignment(run, sections, domain.tools.clone(), !transcript.is_empty())?)?;
    let encoded = charter::encode(value, &domain.charter_endpoints, &env.limits.charter)?;
    let mut answered = List::with_capacity(env.limits.core.call_records);
    for core::Settled { key, call: value } in assignment.answered.clone() {
        let answer = match value.answer {
            core::SettledAnswer::Host { error, body } => host::SettledAnswer::Host { error, body },
            core::SettledAnswer::Delivery { outcome, evidence } => {
                host::SettledAnswer::Delivery { outcome: delivery(outcome), evidence }
            }
        };
        let name = charter::call_name(&charter::CallKey {
            activation: key.attempt,
            completion: key.completion,
            position: key.position,
        });
        answered.push(host::AnsweredCall { name, tool: value.tool, answer }).ok()?;
    }
    let mut start_messages = List::with_capacity(env.limits.core.tasks.run_inbox);
    for word in messages {
        start_messages
            .push(host::StartMessage {
                name: Token::new(word.number),
                sender: charter::sender(word.from),
                finish_requested: word.finish_requested(),
                words: jig_protocol_tools::words(&word),
            })
            .ok()?;
    }
    Some(host::Assignment {
        messages: start_messages.into_boxed(),
        assignment: host::RunAssignment {
            identity: charter::run_identity(&domain.deployment, task, conversation),
            run: Token::new(task),
            attempt: Token::new(attempt),
            workspace: None,
            save: false,
            live: host::LiveAsk { text: live.text, reasoning: live.reasoning, input: live.input },
            charter: encoded,
            grants,
        },
        turns: transcript,
        answered: answered.into_boxed(),
    })
}
const fn delivery(value: core::DeliveryOutcome) -> host::DeliveryOutcome {
    match value {
        core::DeliveryOutcome::Delivered => host::DeliveryOutcome::Delivered,
        core::DeliveryOutcome::Nothing => host::DeliveryOutcome::Nothing,
        core::DeliveryOutcome::Stale => host::DeliveryOutcome::Stale,
        core::DeliveryOutcome::Refused | core::DeliveryOutcome::Invalid => host::DeliveryOutcome::Refused,
        core::DeliveryOutcome::Failed => host::DeliveryOutcome::Failed,
    }
}
pub(crate) fn host_answer(value: core::SettledAnswer) -> host::Relayed {
    match value {
        core::SettledAnswer::Host { error, body } => host::Relayed::Answer { error, text: body },
        core::SettledAnswer::Delivery { .. } => unreachable!("engine charter has no delivery tool"),
    }
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
    to: ReplyTo,
    task: u64,
    tool: Box<[u8]>,
    read: jig_ops_domain_observability::Read,
    work: &mut Queue<Work>,
) {
    let binding = jig_domain_hub_link::ReadCall { to: to.into_token(), tool };
    let number = jig_ops_domain_connectors::number(Connector::Observability);
    let description = crate::translate::read_description(number, &read);
    let mut findings = Queue::uncapped();
    if domain.core.connector_read_admit(task, description, &mut findings) == jig_domain_authority::Answer::Allow {
        let token = domain.token();
        domain.hub_link.keep_read_call(token, binding);
        work.push(Work::Observability(jig_ops_domain_observability::Event::Read { token, read }));
    } else {
        read_answer(domain, binding, true, Box::from(&b"refused"[..]), work);
    }
}
