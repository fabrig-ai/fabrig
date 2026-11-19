//! Named calls and settled answers (protocol/store/records.md, 1).
use super::{
    Box, List, View, Wall, authority, decode_answer, decode_delivery_outcome, decode_escalation_outcome,
    decode_finding, decode_outbox_outcome, decode_proposal_outcome, domain, encode_answer, encode_delivery_outcome,
    encode_escalation_outcome, encode_finding, encode_outbox_outcome, encode_proposal_outcome, note_problem, notes,
    problem, task_problem, tasks, too_large, wire,
};
mod moved;
pub(super) fn encode_settled_answer(value: &domain::SettledAnswer, _limits: &View) -> wire::SettledAnswer {
    match value {
        domain::SettledAnswer::Host { error, body } => {
            wire::SettledAnswer::Host(wire::SettledAnswerHost { error: *error, body: body.clone() })
        }
        domain::SettledAnswer::Delivery { outcome, evidence } => {
            wire::SettledAnswer::Delivery(wire::SettledAnswerDelivery {
                outcome: encode_delivery_outcome(outcome),
                evidence: evidence.clone(),
            })
        }
    }
}
pub(super) fn decode_settled_answer(value: wire::SettledAnswer) -> domain::SettledAnswer {
    match value {
        wire::SettledAnswer::Host(value) => {
            let wire::SettledAnswerHost { error, body } = value;
            domain::SettledAnswer::Host { error, body }
        }
        wire::SettledAnswer::Delivery(value) => {
            let wire::SettledAnswerDelivery { outcome, evidence } = value;
            domain::SettledAnswer::Delivery { outcome: decode_delivery_outcome(outcome), evidence }
        }
    }
}

pub(super) fn encode_settled_call(value: &domain::SettledCall, limits: &View) -> wire::SettledCall {
    let domain::SettledCall { serial, tool, answer } = value;
    wire::SettledCall { serial: *serial, tool: tool.clone(), answer: encode_settled_answer(answer, limits) }
}
pub(super) fn decode_settled_call(value: wire::SettledCall) -> domain::SettledCall {
    let wire::SettledCall { serial, tool, answer } = value;
    domain::SettledCall { serial, tool, answer: decode_settled_answer(answer) }
}

pub(super) fn encode_call_key(value: &domain::CallKey, _limits: &View) -> wire::CallKey {
    let domain::CallKey { task, attempt, completion, position } = value;
    wire::CallKey { task: *task, attempt: *attempt, completion: *completion, position: *position }
}
pub(super) fn decode_call_key(value: wire::CallKey) -> domain::CallKey {
    let wire::CallKey { task, attempt, completion, position } = value;
    domain::CallKey { task, attempt, completion, position }
}

pub(super) fn encode_call_part(value: &domain::CallPart, limits: &View) -> Result<wire::CallPart, wire::Problem> {
    Ok(match value {
        domain::CallPart::SettingsRefused(value) => encode_settings_refusal(value, limits)?,
        domain::CallPart::Effect { connector, entry, deadline, outcome } => {
            encode_part_effect(*connector, *entry, *deadline, *outcome, limits)?
        }
        domain::CallPart::Connector { connector } => {
            wire::CallPart::Connector(wire::CallPartConnector { connector: *connector })
        }
        domain::CallPart::EffectDenied { answer, findings } => encode_part_effect_denied(*answer, findings, limits)?,
        domain::CallPart::EffectRefused(value) => {
            wire::CallPart::EffectRefused(wire::CallPartEffectRefused { value: encode_task_problem(value, limits)? })
        }
        domain::CallPart::ToolDenied { answer, findings } => encode_part_tool_denied(*answer, findings, limits)?,
        domain::CallPart::EscalationDecided { task, revision, outcome } => {
            encode_part_escalation_decided(*task, *revision, *outcome, limits)?
        }
        domain::CallPart::EscalationRefused(value) => {
            wire::CallPart::EscalationRefused(wire::CallPartEscalationRefused {
                value: encode_task_problem(value, limits)?,
            })
        }
        domain::CallPart::Proposed { proposal, holder } => wire::CallPart::Proposed(wire::CallPartProposed {
            proposal: *proposal,
            holder: encode_holder(holder, limits),
        }),
        domain::CallPart::ProposalDecided { proposal, outcome } => {
            encode_part_proposal_decided(*proposal, *outcome, limits)
        }
        domain::CallPart::ProposalRefused(value) => wire::CallPart::ProposalRefused(wire::CallPartProposalRefused {
            value: encode_task_problem(value, limits)?,
        }),
        domain::CallPart::HoldOwner { task, hold, revision, owner } => {
            encode_hold_owner(*task, *hold, *revision, owner, limits)
        }
        domain::CallPart::Again { task, required } => encode_again(*task, *required, limits),
        domain::CallPart::PassedUp { task } => encode_passed_up(*task, limits),
        domain::CallPart::Controlled => wire::CallPart::Controlled,
        domain::CallPart::Revised { task, revision } => {
            wire::CallPart::Revised(wire::CallPartRevised { task: *task, revision: *revision })
        }
        domain::CallPart::Moved { task, now } => encode_moved_part(*task, now, limits),
        domain::CallPart::ControlRefused(value) => {
            wire::CallPart::ControlRefused(wire::CallPartControlRefused { value: encode_task_problem(value, limits)? })
        }
        domain::CallPart::ControlDenied { answer } => {
            wire::CallPart::ControlDenied(wire::CallPartControlDenied { answer: encode_answer(answer) })
        }
        domain::CallPart::Sent { message } => wire::CallPart::Sent(wire::CallPartSent { message: *message }),
        domain::CallPart::Introduced => wire::CallPart::Introduced,
        domain::CallPart::MessageRefused(value) => {
            wire::CallPart::MessageRefused(wire::CallPartMessageRefused { value: encode_task_problem(value, limits)? })
        }
        domain::CallPart::Subscribed { subscription } => {
            wire::CallPart::Subscribed(wire::CallPartSubscribed { subscription: *subscription })
        }
        domain::CallPart::Unsubscribed => wire::CallPart::Unsubscribed,
        domain::CallPart::SubscriptionRefused(value) => {
            wire::CallPart::SubscriptionRefused(wire::CallPartSubscriptionRefused {
                value: encode_task_problem(value, limits)?,
            })
        }
        domain::CallPart::Delegated(value) => wire::CallPart::Delegated(wire::CallPartDelegated {
            value: encode_call_part_delegated_value(value, limits)?,
        }),
        domain::CallPart::DelegationDenied { answer, findings } => {
            encode_part_delegation_denied(*answer, findings, limits)?
        }
        domain::CallPart::DelegationRefused(value) => {
            wire::CallPart::DelegationRefused(wire::CallPartDelegationRefused {
                value: encode_task_problem(value, limits)?,
            })
        }
        domain::CallPart::NoteDeleted { name } => {
            wire::CallPart::NoteDeleted(wire::CallPartNoteDeleted { name: *name })
        }
        domain::CallPart::NoteWritten { name, revision, lines, added, removed, first } => {
            wire::CallPart::NoteWritten(wire::CallPartNoteWritten {
                name: *name,
                revision: *revision,
                lines: *lines,
                added: *added,
                removed: *removed,
                first: *first,
            })
        }
        domain::CallPart::NoteRecalled { entries, more } => encode_part_note_recalled(entries, *more, limits)?,
        domain::CallPart::NoteRefused(value) => encode_part_note_refused(*value, limits),
        domain::CallPart::Unavailable => wire::CallPart::Unavailable,
    })
}
#[expect(clippy::too_many_lines, reason = "one exhaustive translation preserves the stored call vocabulary")]
pub(super) fn decode_call_part(value: wire::CallPart, limits: &View) -> Result<domain::CallPart, wire::Problem> {
    Ok(match value {
        wire::CallPart::SettingsRefused(value) => decode_settings_refusal(value)?,
        wire::CallPart::Effect(value) => decode_part_effect(value),
        wire::CallPart::Connector(value) => {
            let wire::CallPartConnector { connector } = value;
            domain::CallPart::Connector { connector }
        }
        wire::CallPart::EffectDenied(value) => {
            let wire::CallPartEffectDenied { answer, findings } = value;
            domain::CallPart::EffectDenied {
                answer: decode_answer(answer),
                findings: decode_call_part_effect_denied_findings(findings),
            }
        }
        wire::CallPart::EffectRefused(value) => {
            let wire::CallPartEffectRefused { value } = value;
            domain::CallPart::EffectRefused(decode_task_problem(value, limits)?)
        }
        wire::CallPart::ToolDenied(value) => {
            let wire::CallPartToolDenied { answer, findings } = value;
            domain::CallPart::ToolDenied {
                answer: decode_answer(answer),
                findings: decode_call_part_tool_denied_findings(findings),
            }
        }
        wire::CallPart::EscalationDecided(value) => {
            let wire::CallPartEscalationDecided { task, revision, outcome } = value;
            domain::CallPart::EscalationDecided { task, revision, outcome: decode_escalation_outcome(outcome) }
        }
        wire::CallPart::EscalationRefused(value) => {
            let wire::CallPartEscalationRefused { value } = value;
            domain::CallPart::EscalationRefused(decode_task_problem(value, limits)?)
        }
        wire::CallPart::Proposed(value) => {
            let wire::CallPartProposed { proposal, holder } = value;
            domain::CallPart::Proposed { proposal, holder: decode_holder(holder) }
        }
        wire::CallPart::ProposalDecided(value) => {
            let wire::CallPartProposalDecided { proposal, outcome } = value;
            domain::CallPart::ProposalDecided { proposal, outcome: decode_proposal_outcome(outcome) }
        }
        wire::CallPart::ProposalRefused(value) => {
            let wire::CallPartProposalRefused { value } = value;
            domain::CallPart::ProposalRefused(decode_task_problem(value, limits)?)
        }
        wire::CallPart::HoldOwner(value) => {
            let wire::CallPartHoldOwner { task, hold, revision, owner } = value;
            domain::CallPart::HoldOwner { task, hold, revision, owner: moved::decode_hold_owner(owner) }
        }
        wire::CallPart::Again(value) => decode_again(value),
        wire::CallPart::PassedUp(value) => decode_passed_up(value),
        wire::CallPart::Controlled => domain::CallPart::Controlled,
        wire::CallPart::Revised(value) => decode_revised(value),
        wire::CallPart::Moved(value) => decode_moved(value),
        wire::CallPart::ControlRefused(value) => {
            let wire::CallPartControlRefused { value } = value;
            domain::CallPart::ControlRefused(decode_task_problem(value, limits)?)
        }
        wire::CallPart::ControlDenied(value) => {
            let wire::CallPartControlDenied { answer } = value;
            domain::CallPart::ControlDenied { answer: decode_answer(answer) }
        }
        wire::CallPart::Sent(value) => {
            let wire::CallPartSent { message } = value;
            domain::CallPart::Sent { message }
        }
        wire::CallPart::Introduced => domain::CallPart::Introduced,
        wire::CallPart::MessageRefused(value) => {
            let wire::CallPartMessageRefused { value } = value;
            domain::CallPart::MessageRefused(decode_task_problem(value, limits)?)
        }
        wire::CallPart::Subscribed(value) => {
            let wire::CallPartSubscribed { subscription } = value;
            domain::CallPart::Subscribed { subscription }
        }
        wire::CallPart::Unsubscribed => domain::CallPart::Unsubscribed,
        wire::CallPart::SubscriptionRefused(value) => {
            let wire::CallPartSubscriptionRefused { value } = value;
            domain::CallPart::SubscriptionRefused(decode_task_problem(value, limits)?)
        }
        wire::CallPart::Delegated(value) => {
            let wire::CallPartDelegated { value } = value;
            domain::CallPart::Delegated(decode_call_part_delegated_value(value))
        }
        wire::CallPart::DelegationDenied(value) => {
            let wire::CallPartDelegationDenied { answer, findings } = value;
            domain::CallPart::DelegationDenied {
                answer: decode_answer(answer),
                findings: decode_call_part_delegation_denied_findings(findings),
            }
        }
        wire::CallPart::DelegationRefused(value) => {
            let wire::CallPartDelegationRefused { value } = value;
            domain::CallPart::DelegationRefused(decode_task_problem(value, limits)?)
        }
        wire::CallPart::NoteDeleted(value) => domain::CallPart::NoteDeleted { name: value.name },
        wire::CallPart::NoteWritten(value) => {
            let wire::CallPartNoteWritten { name, revision, lines, added, removed, first } = value;
            domain::CallPart::NoteWritten { name, revision, lines, added, removed, first }
        }
        wire::CallPart::NoteRecalled(value) => {
            let wire::CallPartNoteRecalled { entries, more } = value;
            domain::CallPart::NoteRecalled { entries: decode_call_part_note_recalled_entries(entries, limits)?, more }
        }
        wire::CallPart::NoteRefused(value) => {
            let wire::CallPartNoteRefused { value } = value;
            domain::CallPart::NoteRefused(decode_note_refusal(value))
        }
        wire::CallPart::Unavailable => domain::CallPart::Unavailable,
    })
}

pub(super) fn encode_call_record(value: &domain::CallRecord, limits: &View) -> Result<wire::CallRecord, wire::Problem> {
    let domain::CallRecord { key, part, settled } = value;
    Ok(wire::CallRecord {
        key: encode_call_key(key, limits),
        part: encode_call_part(part, limits)?,
        settled: match settled {
            Some(value) => Some(encode_settled_call(value, limits)),
            None => None,
        },
    })
}
pub(super) fn decode_call_record(value: wire::CallRecord, limits: &View) -> Result<domain::CallRecord, wire::Problem> {
    let wire::CallRecord { key, part, settled } = value;
    Ok(domain::CallRecord {
        key: decode_call_key(key),
        part: decode_call_part(part, limits)?,
        settled: match settled {
            Some(value) => Some(decode_settled_call(value)),
            None => None,
        },
    })
}

fn encode_call_part_effect_denied_findings(
    values: &[authority::Finding],
    limits: &View,
) -> Result<List<wire::Finding>, wire::Problem> {
    let mut result = List::with_capacity(match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    });
    for value in values {
        result.push(encode_finding(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_call_part_effect_denied_findings(values: List<wire::Finding>) -> Box<[authority::Finding]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_finding(value)).expect("same number of members");
    }
    result.into_boxed()
}

fn encode_call_part_tool_denied_findings(
    values: &[authority::Finding],
    limits: &View,
) -> Result<List<wire::Finding>, wire::Problem> {
    let mut result = List::with_capacity(match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    });
    for value in values {
        result.push(encode_finding(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_call_part_tool_denied_findings(values: List<wire::Finding>) -> Box<[authority::Finding]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_finding(value)).expect("same number of members");
    }
    result.into_boxed()
}

fn encode_call_part_delegated_value(
    values: &[domain::Made],
    _limits: &View,
) -> Result<List<wire::Made>, wire::Problem> {
    let mut result = List::with_capacity(match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    });
    for value in values {
        result
            .push(wire::Made { task: value.task, revision: value.revision, given: value.given })
            .expect("same number of members");
    }
    Ok(result)
}
fn decode_call_part_delegated_value(values: List<wire::Made>) -> Box<[domain::Made]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let wire::Made { task, revision, given } = value;
        result.push(domain::Made { task, revision, given }).expect("same number of members");
    }
    result.into_boxed()
}

fn encode_call_part_delegation_denied_findings(
    values: &[authority::Finding],
    limits: &View,
) -> Result<List<wire::Finding>, wire::Problem> {
    let mut result = List::with_capacity(match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    });
    for value in values {
        result.push(encode_finding(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_call_part_delegation_denied_findings(values: List<wire::Finding>) -> Box<[authority::Finding]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_finding(value)).expect("same number of members");
    }
    result.into_boxed()
}

fn encode_call_part_note_recalled_entries(
    values: &[notes::Entry],
    limits: &View,
) -> Result<List<wire::NoteBytes>, wire::Problem> {
    let mut result = List::with_capacity(match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    });
    for value in values {
        result
            .push(wire::NoteBytes {
                value: match crate::codec::notes::encode(&notes::Record::Entry(value.clone()), &limits.notes) {
                    Ok(value) => value,
                    Err(error) => return Err(note_problem(error)),
                },
            })
            .expect("same number of members");
    }
    Ok(result)
}
fn decode_call_part_note_recalled_entries(
    values: List<wire::NoteBytes>,
    limits: &View,
) -> Result<Box<[notes::Entry]>, wire::Problem> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result
            .push(
                match match crate::codec::notes::decode(&value.value, &limits.notes) {
                    Ok(value) => value,
                    Err(error) => return Err(note_problem(error)),
                } {
                    notes::Record::Entry(entry) => entry,
                    notes::Record::Line(_) => return Err(problem(skein_codec::Reason::Tag)),
                },
            )
            .expect("same number of members");
    }
    Ok(result.into_boxed())
}
pub(super) fn encode_note_refusal(value: notes::Refusal, _limits: &View) -> wire::NoteRefusal {
    match value {
        notes::Refusal::Busy => wire::NoteRefusal::Busy,
        notes::Refusal::Oversized => wire::NoteRefusal::Oversized,
        notes::Refusal::Full => wire::NoteRefusal::Full,
        notes::Refusal::Exists => wire::NoteRefusal::Exists,
        notes::Refusal::Missing => wire::NoteRefusal::Missing,
        notes::Refusal::Moved { revision } => wire::NoteRefusal::Moved(wire::NoteRefusalMoved { revision }),
        notes::Refusal::PastEnd { lines } => wire::NoteRefusal::PastEnd(wire::NoteRefusalPastEnd { lines }),
        notes::Refusal::PastBound { bytes, bound } => {
            wire::NoteRefusal::PastBound(wire::NoteRefusalPastBound { bytes, bound })
        }
        notes::Refusal::RevisionExhausted => wire::NoteRefusal::RevisionExhausted,
    }
}
pub(super) fn decode_note_refusal(value: wire::NoteRefusal) -> notes::Refusal {
    match value {
        wire::NoteRefusal::Busy => notes::Refusal::Busy,
        wire::NoteRefusal::Oversized => notes::Refusal::Oversized,
        wire::NoteRefusal::Full => notes::Refusal::Full,
        wire::NoteRefusal::Exists => notes::Refusal::Exists,
        wire::NoteRefusal::Missing => notes::Refusal::Missing,
        wire::NoteRefusal::Moved(value) => {
            let wire::NoteRefusalMoved { revision } = value;
            notes::Refusal::Moved { revision }
        }
        wire::NoteRefusal::PastEnd(value) => {
            let wire::NoteRefusalPastEnd { lines } = value;
            notes::Refusal::PastEnd { lines }
        }
        wire::NoteRefusal::PastBound(value) => {
            let wire::NoteRefusalPastBound { bytes, bound } = value;
            notes::Refusal::PastBound { bytes, bound }
        }
        wire::NoteRefusal::RevisionExhausted => notes::Refusal::RevisionExhausted,
    }
}

fn encode_task_problem(value: &jig_domain_tasks::Problem, limits: &View) -> Result<wire::TaskBytes, wire::Problem> {
    let value = match crate::codec::tasks::carried::problem(value, &limits.tasks) {
        Ok(value) => value,
        Err(error) => return Err(task_problem(error)),
    };
    Ok(wire::TaskBytes { value })
}
fn decode_task_problem(value: wire::TaskBytes, limits: &View) -> Result<jig_domain_tasks::Problem, wire::Problem> {
    match crate::codec::tasks::carried::read_problem(&value.value, &limits.tasks) {
        Ok(value) => Ok(value),
        Err(error) => Err(task_problem(error)),
    }
}

fn encode_part_effect(
    connector: u16,
    entry: u64,
    deadline: Wall,
    outcome: Option<domain::effects::connector::OutboxOutcome>,
    limits: &View,
) -> Result<wire::CallPart, wire::Problem> {
    Ok(wire::CallPart::Effect(wire::CallPartEffect {
        connector,
        entry,
        deadline: deadline.as_nanos(),
        outcome: match outcome {
            Some(value) => Some(encode_outbox_outcome(&value, limits)?),
            None => None,
        },
    }))
}

fn encode_part_effect_denied(
    answer: authority::Answer,
    findings: &[authority::Finding],
    limits: &View,
) -> Result<wire::CallPart, wire::Problem> {
    Ok(wire::CallPart::EffectDenied(wire::CallPartEffectDenied {
        answer: encode_answer(&answer),
        findings: encode_call_part_effect_denied_findings(findings, limits)?,
    }))
}

fn encode_part_tool_denied(
    answer: authority::Answer,
    findings: &[authority::Finding],
    limits: &View,
) -> Result<wire::CallPart, wire::Problem> {
    Ok(wire::CallPart::ToolDenied(wire::CallPartToolDenied {
        answer: encode_answer(&answer),
        findings: encode_call_part_tool_denied_findings(findings, limits)?,
    }))
}

fn encode_part_delegation_denied(
    answer: authority::Answer,
    findings: &[authority::Finding],
    limits: &View,
) -> Result<wire::CallPart, wire::Problem> {
    Ok(wire::CallPart::DelegationDenied(wire::CallPartDelegationDenied {
        answer: encode_answer(&answer),
        findings: encode_call_part_delegation_denied_findings(findings, limits)?,
    }))
}

fn encode_part_note_recalled(
    entries: &[notes::Entry],
    more: bool,
    limits: &View,
) -> Result<wire::CallPart, wire::Problem> {
    Ok(wire::CallPart::NoteRecalled(wire::CallPartNoteRecalled {
        entries: encode_call_part_note_recalled_entries(entries, limits)?,
        more,
    }))
}

fn encode_part_escalation_decided(
    task: u64,
    revision: u64,
    outcome: tasks::EscalationOutcome,
    limits: &View,
) -> Result<wire::CallPart, wire::Problem> {
    Ok(wire::CallPart::EscalationDecided(wire::CallPartEscalationDecided {
        task,
        revision,
        outcome: encode_escalation_outcome(&outcome, limits)?,
    }))
}

fn encode_part_proposal_decided(proposal: u64, outcome: tasks::ProposalOutcome, _limits: &View) -> wire::CallPart {
    wire::CallPart::ProposalDecided(wire::CallPartProposalDecided {
        proposal,
        outcome: encode_proposal_outcome(&outcome),
    })
}

fn decode_part_effect(value: wire::CallPartEffect) -> domain::CallPart {
    let wire::CallPartEffect { connector, entry, deadline, outcome } = value;
    domain::CallPart::Effect {
        connector,
        entry,
        deadline: Wall::from_nanos(deadline),
        outcome: match outcome {
            Some(value) => Some(decode_outbox_outcome(value)),
            None => None,
        },
    }
}

fn decode_revised(value: wire::CallPartRevised) -> domain::CallPart {
    let wire::CallPartRevised { task, revision } = value;
    domain::CallPart::Revised { task, revision }
}
fn decode_moved(value: wire::CallPartMoved) -> domain::CallPart {
    let wire::CallPartMoved { task, now } = value;
    domain::CallPart::Moved { task, now: moved::decode_moved(now) }
}

fn encode_again(task: u64, required: bool, _limits: &View) -> wire::CallPart {
    wire::CallPart::Again(wire::CallPartAgain { task, required })
}
fn encode_passed_up(task: u64, _limits: &View) -> wire::CallPart {
    wire::CallPart::PassedUp(wire::CallPartPassedUp { task })
}
fn decode_again(value: wire::CallPartAgain) -> domain::CallPart {
    let wire::CallPartAgain { task, required } = value;
    domain::CallPart::Again { task, required }
}
fn decode_passed_up(value: wire::CallPartPassedUp) -> domain::CallPart {
    let wire::CallPartPassedUp { task } = value;
    domain::CallPart::PassedUp { task }
}

fn encode_holder(value: &tasks::ProposalHolder, _limits: &View) -> wire::ProposalHolder {
    match value {
        tasks::ProposalHolder::Task(value) => wire::ProposalHolder::Task(wire::Number { value: *value }),
        tasks::ProposalHolder::Person(value) => wire::ProposalHolder::Person(wire::Number { value: *value }),
        tasks::ProposalHolder::Policy { project, kind } => wire::ProposalHolder::Policy(wire::PolicyHolder {
            project: *project,
            kind: super::encode_proposal_kind(kind),
        }),
    }
}
fn decode_holder(value: wire::ProposalHolder) -> tasks::ProposalHolder {
    match value {
        wire::ProposalHolder::Task(value) => tasks::ProposalHolder::Task(value.value),
        wire::ProposalHolder::Person(value) => tasks::ProposalHolder::Person(value.value),
        wire::ProposalHolder::Policy(value) => {
            let wire::PolicyHolder { project, kind } = value;
            tasks::ProposalHolder::Policy { project, kind: super::decode_proposal_kind(kind) }
        }
    }
}

fn people_problem(value: crate::people_v17::Problem) -> wire::Problem {
    problem(value.reason)
}

fn encode_settings_refusal(
    value: &jig_domain_people::Refusal,
    _limits: &View,
) -> Result<wire::CallPart, wire::Problem> {
    let value = match crate::codec::people::refusal_bytes(value) {
        Ok(value) => value,
        Err(problem) => return Err(people_problem(problem)),
    };
    Ok(wire::CallPart::SettingsRefused(wire::SettingsRefusal { value }))
}
fn decode_settings_refusal(value: wire::SettingsRefusal) -> Result<domain::CallPart, wire::Problem> {
    match crate::codec::people::read_refusal(&value.value) {
        Ok(value) => Ok(domain::CallPart::SettingsRefused(value)),
        Err(problem) => Err(people_problem(problem)),
    }
}

fn encode_part_note_refused(value: notes::Refusal, limits: &View) -> wire::CallPart {
    wire::CallPart::NoteRefused(wire::CallPartNoteRefused { value: encode_note_refusal(value, limits) })
}

fn encode_hold_owner(
    task: u64,
    hold: u64,
    revision: u64,
    owner: &jig_domain_people::HoldOwner,
    limits: &View,
) -> wire::CallPart {
    wire::CallPart::HoldOwner(wire::CallPartHoldOwner {
        task,
        hold,
        revision,
        owner: moved::encode_hold_owner(owner, &limits.core),
    })
}

fn encode_moved_part(task: u64, now: &jig_domain_people::Moved, limits: &View) -> wire::CallPart {
    wire::CallPart::Moved(wire::CallPartMoved { task, now: moved::encode_moved(now, &limits.core) })
}
