//! Assemble outputs in the decision that owns them. A copying application
//! replaces its tools and connector results; charter conversion stays shared
//! (domain/root.md, 7; domain/hosts.md, 8).
use crate::domain::Work;
use crate::{Domain, Limits};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_ops_domain_connectors::Connector;
use jig_protocol_charter as charter;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(crate) fn brief(order: Box<[brief::GatherPlaced]>) -> Box<[charter::Section]> {
    let mut sections = List::with_capacity(u32::try_from(order.len()).expect("bounded sections"));
    for placed in order {
        let section = match placed {
            brief::GatherPlaced::Core { kind, text } => charter::Section { title: Box::from(title(kind)), text },
            brief::GatherPlaced::CoreMissing { kind, .. } => {
                charter::Section { title: Box::from(title(kind)), text: Box::from(&b"[missing]"[..]) }
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
#[expect(clippy::too_many_arguments, reason = "assemble the owners parts without a shared sibling type")]
pub(crate) fn assignment(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    conversation: u64,
    live: core::LiveAsk,
    run: core::RunCharter,
    sections: Box<[charter::Section]>,
    transcript: Box<[Box<[u8]>]>,
    messages: Box<[tasks::Word]>,
    grants: Box<[accounts::Grant]>,
) -> Option<host::Assignment> {
    let value = charter::charter(charter::assignment(run, sections, domain.tools.clone(), !transcript.is_empty())?)?;
    let encoded = charter::encode(value, &domain.charter_endpoints, &env.limits.charter)?;
    let mut answered = List::with_capacity(env.limits.core.call_records);
    for core::Settled { key, call: value } in domain.core.settled_calls(task, attempt) {
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

pub(crate) fn ending(value: host::Ending) -> tasks::End {
    match value {
        host::Ending::Refused(host::Refusal::Busy) => tasks::End::Refused,
        host::Ending::Refused(host::Refusal::Invalid(_)) => tasks::End::Failed(tasks::Class::Invalid),
        host::Ending::Parked { .. } => tasks::End::Parked,
        host::Ending::Failed { failure, .. } => match failure {
            host::Failure::Run(host::RunFailure::Exceeded(value)) => tasks::End::Exceeded(tasks::Exceeded {
                bound: match value.bound {
                    host::Bound::Reasoning => tasks::Bound::Reasoning,
                    host::Bound::Metadata => tasks::Bound::Metadata,
                    host::Bound::Items => tasks::Bound::Items,
                    host::Bound::Answer => tasks::Bound::Answer,
                    host::Bound::Tools => tasks::Bound::Tools,
                    host::Bound::HistoryItems => tasks::Bound::HistoryItems,
                    host::Bound::History => tasks::Bound::History,
                    host::Bound::Window => tasks::Bound::Window,
                    host::Bound::Output => tasks::Bound::Output,
                },
                value: value.value,
            }),
            host::Failure::Unprepared(host::Preparation::Transient) => tasks::End::Failed(tasks::Class::Transient),
            host::Failure::Unprepared(host::Preparation::Permanent { .. }) => {
                tasks::End::Failed(tasks::Class::Permanent)
            }
            host::Failure::Run(
                host::RunFailure::Model
                | host::RunFailure::Budget
                | host::RunFailure::Policy
                | host::RunFailure::Cancelled
                | host::RunFailure::Stale
                | host::RunFailure::Limited { .. }
                | host::RunFailure::Refused { .. },
            ) => tasks::End::Failed(tasks::Class::Run),
            host::Failure::Agent(_) | host::Failure::Cancelled(_) => tasks::End::Failed(tasks::Class::Agent),
        },
        host::Ending::Ended { .. } => unreachable!("ended results leave for protocol reading"),
    }
}

pub(crate) fn answer_payload(domain: &mut Domain, run: Token, attempt: Token, payload: Token, work: &mut Queue<Work>) {
    let ending = match domain.payloads.get(&payload).expect("fenced answer payload") {
        crate::boundary::Payload::Answer { task, attempt: retained, ending, .. } => {
            assert!(*task == run.raw() && *retained == attempt.raw(), "fenced answer identity");
            ending.clone()
        }
        crate::boundary::Payload::Turn { .. }
        | crate::boundary::Payload::Word(_)
        | crate::boundary::Payload::Settled(_) => unreachable!("answer family"),
    };
    match ending {
        host::Ending::Ended { outcome, .. } => {
            crate::domain::now(domain, crate::Output::ReadResult { payload, outcome });
        }
        other @ (host::Ending::Refused(_) | host::Ending::Parked { .. } | host::Ending::Failed { .. }) => {
            result_read(domain, payload, self::ending(other), work);
        }
    }
}

pub(crate) fn result_read(domain: &Domain, payload: Token, end: tasks::End, work: &mut Queue<Work>) {
    let Some(crate::boundary::Payload::Answer { task, attempt, spent, read, refused, .. }) =
        domain.payloads.get(&payload)
    else {
        return;
    };
    work.push(Work::Core(core::Event::AnswerPayload {
        run: Token::new(*task),
        attempt: Token::new(*attempt),
        payload,
        cumulative: *spent,
        end,
        saved: None,
        invalid_saved: false,
        read: *read,
        refused: refusal_names(refused),
    }));
}

/// Take the owner's answer once, preserving the core's held or now mark.
pub(crate) fn relayed(
    domain: &mut Domain,
    run: Token,
    attempt: Token,
    call: Box<[u8]>,
    answer: Token,
) -> Option<crate::Released> {
    let value = match domain.payloads.remove(&answer).expect("settled call token") {
        crate::boundary::Payload::Settled(value) => value,
        crate::boundary::Payload::Turn { .. }
        | crate::boundary::Payload::Answer { .. }
        | crate::boundary::Payload::Word(_) => unreachable!("settled call family"),
    };
    let binding = domain.host_calls.remove(&call)?;
    assert!(binding.run == run && binding.attempt == attempt, "hub relay fence");
    Some(crate::Released::Host(host::Event::Down(host::LinkDown::Relayed {
        run,
        attempt,
        call,
        answer: host_answer(value.answer),
    })))
}

pub(crate) fn read_answer(
    domain: &mut Domain,
    call: crate::domain::ReadCall,
    error: bool,
    body: Box<[u8]>,
    work: &mut Queue<Work>,
) {
    let answer = domain.payload(crate::boundary::Payload::Settled(core::SettledCall {
        serial: 0,
        tool: call.tool,
        answer: core::SettledAnswer::Host { error, body },
    }));
    work.push(Work::Core(core::Event::Fleet(jig_domain_fleet::Event::Relayed { to: ReplyTo::new(call.to), answer })));
}

pub(crate) use crate::message_refusals::{message_read, refusal_names, refusals};

pub(crate) fn connector_read(
    domain: &mut Domain,
    to: ReplyTo,
    task: u64,
    tool: Box<[u8]>,
    read: jig_ops_domain_observability::Read,
    work: &mut Queue<Work>,
) {
    let binding = crate::domain::ReadCall { to: to.into_token(), tool };
    let number = jig_ops_domain_connectors::number(Connector::Observability);
    let description = crate::translate::read_description(number, &read);
    let mut findings = Queue::uncapped();
    if domain.core.connector_read_admit(task, description, &mut findings) != jig_domain_authority::Answer::Allow
        || domain.read_calls.len() == domain.read_calls.capacity()
    {
        read_answer(domain, binding, true, Box::from(&b"refused"[..]), work);
    } else {
        let token = domain.token();
        domain.read_calls.insert(token, binding).expect("admitted read continuation");
        work.push(Work::Observability(jig_ops_domain_observability::Event::Read { token, read }));
    }
}

pub(crate) fn fleet_grants(grants: &[host::Grant]) -> Box<[fleet::Grant]> {
    Box::from(grants)
}
