//! A run's whole hand-over and messages (jig/docs/design/domain/hosts/charters.md and jig/docs/design/domain/hosts/README.md, 7).
use crate::Section;
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{List, Token};

/// A root's own part of a run's hand-over (domain/hosts/charters.md).
#[derive(Debug)]
pub struct RootParts<'a> {
    pub tools: Box<[crate::Tool]>,
    pub endpoints: &'a [crate::EndpointName],
    pub deployment: &'a [u8; 16],
    pub limits: &'a smith_charter::v4::Limits,
}

/// Assemble the hub's hand-over, refusing unsupported sections or exceeded bounds.
#[must_use]
pub fn hosted(assignment: &core::Assignment, parts: RootParts<'_>) -> Option<host::Assignment> {
    let mut sections = List::with_capacity(u32::try_from(assignment.brief.len()).ok()?);
    for placed in &assignment.brief {
        let section = match placed {
            brief::GatherPlaced::Core { kind, text } => Section {
                title: if *kind == brief::Core::Now { now_title(text) } else { title(*kind).into() },
                text: text.clone(),
            },
            brief::GatherPlaced::CoreMissing { kind, .. } => {
                Section { title: title(*kind).into(), text: b"[missing]".as_slice().into() }
            }
            brief::GatherPlaced::Missing { .. } => {
                Section { title: b"Context".as_slice().into(), text: b"[missing]".as_slice().into() }
            }
            brief::GatherPlaced::Connector { .. } => return None,
        };
        sections.push(section).ok()?;
    }
    let resumes = !assignment.transcript.is_empty();
    let value = crate::assignment((*assignment.run).clone(), sections.into_boxed(), parts.tools, resumes)?;
    let encoded = crate::encode(crate::charter(value)?, parts.endpoints, parts.limits)?;
    let mut answered = List::with_capacity(u32::try_from(assignment.answered.len()).ok()?);
    for core::Settled { key, call } in &assignment.answered {
        let answer = match &call.answer {
            core::SettledAnswer::Host { error, body } => {
                host::SettledAnswer::Host { error: *error, body: body.clone() }
            }
            core::SettledAnswer::Delivery { outcome, evidence } => {
                host::SettledAnswer::Delivery { outcome: delivery(*outcome), evidence: evidence.clone() }
            }
        };
        let name = crate::call_name(&crate::CallKey {
            activation: key.attempt,
            completion: key.completion,
            position: key.position,
        });
        answered.push(host::AnsweredCall { name, tool: call.tool.clone(), answer }).ok()?;
    }
    let mut messages = List::with_capacity(u32::try_from(assignment.messages.len()).ok()?);
    for word in &assignment.messages {
        messages.push(start_message(word)).ok()?;
    }
    let live = assignment.live;
    Some(host::Assignment {
        messages: messages.into_boxed(),
        assignment: host::RunAssignment {
            identity: crate::run_identity(parts.deployment, assignment.task, assignment.conversation),
            run: Token::new(assignment.task),
            attempt: Token::new(assignment.attempt),
            workspace: None,
            save: false,
            live: host::LiveAsk { text: live.text, reasoning: live.reasoning, input: live.input },
            charter: encoded,
            grants: assignment.grants.clone(),
        },
        turns: assignment.transcript.clone(),
        answered: answered.into_boxed(),
    })
}

/// A task's message, as the host carries it (domain/hosts/README.md, 7).
#[must_use]
pub fn message(run: Token, attempt: Token, word: &tasks::Word) -> host::LinkDown {
    let host::StartMessage { name, sender, finish_requested, words } = start_message(word);
    host::LinkDown::Message { run, attempt, name, sender, finish_requested, words }
}
fn start_message(word: &tasks::Word) -> host::StartMessage {
    host::StartMessage {
        name: Token::new(word.number),
        sender: word.from.sender(),
        finish_requested: word.finish_requested(),
        words: jig_protocol_tools::words(word),
    }
}
/// Title a Now with its rendered note identity, revision and write time
/// (`jig/docs/design/domain/tasks/standing.md`, section 6).
#[must_use]
pub fn now_title(text: &[u8]) -> Box<[u8]> {
    let end = skein_lib::bytes::find(text, b"\n").unwrap_or(text.len());
    let heading = text.get(..end).expect("first line within text");
    skein_lib::bytes::concat(&[b"Now: ", heading])
}

const fn title(kind: brief::Core) -> &'static [u8] {
    match kind {
        brief::Core::Task => b"Task",
        brief::Core::Now => b"Now",
        brief::Core::Lineage => b"Requesters",
        brief::Core::Results => b"Results",
        brief::Core::Plan => b"Plan",
        brief::Core::Attempts => b"Attempts",
        brief::Core::Calls => b"Calls",
        brief::Core::Waiting => b"Waiting",
        brief::Core::NotesIndex => b"Notes",
        brief::Core::Preferences => b"Preferences",
        brief::Core::TranscriptTail => b"Conversation",
    }
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
