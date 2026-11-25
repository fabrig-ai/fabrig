//! Core rows carried by their generated family (protocol/store/records.md, 1).
//! Translation ownership: skein/docs/design/data/codec/maps.md, 4.2.
use crate::core_v6::{self as wire, Record};
use crate::generated::core as maps;
use crate::tasks_v13 as task_wire;
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core_vocabulary::{
    self as domain,
    waits::{WaitFloors, WaitKind},
};
use jig_domain_fleet as fleet;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Reader, Wall};
pub(crate) mod projections;
use super::View;

pub(super) fn limits(
    domain: &domain::Limits,
    tasks: &task_wire::Limits,
    notes: &crate::notes_v1::Limits,
) -> Option<wire::Limits> {
    let [phase, change, end, problem] = skein_codec::bounds::batch(
        task_wire::DECLARATIONS,
        tasks.values().as_slice(),
        task_wire::CEILINGS.values().as_slice(),
        ["PhaseRecord", "ChangeRecord", "EndRecord", "TaskProblem"],
    )?;
    let core = wire::Limits {
        settings_refusal_value: u32::try_from(crate::people_v17::Refusal::encoded_bound(&crate::people_v17::CEILINGS)?)
            .ok()?,
        phase_bytes_value: u32::try_from(phase.encoded).ok()?,
        change_bytes_value: u32::try_from(change.encoded).ok()?,
        end_bytes_value: u32::try_from(end.encoded).ok()?,
        projection_milestone_words: domain.tasks.message_bytes.max(domain.tasks.result_bytes),
        projection_goal_hot_pending: domain.connectors,
        projection_words_words: domain.tasks.spec_bytes,
        run_proof_accounts: tasks::MODELS,
        run_proof_credentials: domain.accounts.credentials,
        run_proof_outstanding: domain.tasks.run_inbox,
        turn_record_transcript: wire::CEILINGS.turn_record_transcript,
        wait_floors_entries: wire::CEILINGS.wait_floors_entries,
        part_record_body: wire::CEILINGS.part_record_body,

        escalation_decision_reject_reason: domain.escalation_reason_bytes.max(domain.people.words),
        settled_answer_host_body: domain.call_answer_bytes,
        settled_answer_delivery_evidence: domain.call_answer_bytes,
        settled_call_tool: domain.call_answer_bytes,
        call_part_effect_denied_findings: jig_domain_authority::stored_finding_limit(&domain.authority)?,
        call_part_tool_denied_findings: jig_domain_authority::stored_finding_limit(&domain.authority)?,
        call_part_delegated_value: domain.tasks.batch,
        call_part_delegation_denied_findings: jig_domain_authority::stored_finding_limit(&domain.authority)?,
        call_part_note_written_recorded: domain.tasks.inbox_messages,
        call_part_note_written_unknown: domain.tasks.inbox_messages,
        call_part_words_recorded_recorded: domain.tasks.inbox_messages,
        call_part_words_recorded_unknown: domain.tasks.inbox_messages,
        call_part_note_recalled_entries: domain.notes.recalled,
        task_bytes_value: u32::try_from(problem.encoded).ok()?,
        note_bytes_value: u32::try_from(crate::notes_v1::Row::encoded_bound(notes)?).ok()?,
    };
    wire::check_limits(&core).ok()?;
    Some(core)
}
pub fn encode(row: &domain::CoreRecord, limits: &View) -> Result<Box<[u8]>, wire::Problem> {
    row_to_record(row, limits)?.encode(&limits.core)
}
pub fn decode(bytes: &[u8], limits: &View) -> Result<domain::CoreRecord, wire::Problem> {
    row_to_domain(wire::Row::decode(&limits.core, &mut Reader::new(bytes))?, limits)
}
pub(crate) fn row_to_record(value: &domain::CoreRecord, limits: &View) -> Result<wire::Row, wire::Problem> {
    Ok(wire::Row { value: value_to_record(value, limits)? })
}
pub(crate) fn row_to_domain(value: wire::Row, limits: &View) -> Result<domain::CoreRecord, wire::Problem> {
    value_to_domain(value.value, limits)
}
pub(crate) fn value_to_record(value: &domain::CoreRecord, limits: &View) -> Result<wire::Value, wire::Problem> {
    Ok(match value {
        domain::CoreRecord::Call(value) => wire::Value::Call(maps::call_record_to_record(value, limits)?),
        domain::CoreRecord::Part(value) => wire::Value::Part(maps::part_record_to_record(value)),
        domain::CoreRecord::Settings(_) => return Err(wire::paths::VALUE_TAG.problem(skein_codec::Reason::Bound)),
        domain::CoreRecord::Deployment(value) => wire::Value::Deployment(maps::deployment_to_record(value)),
        domain::CoreRecord::Projection(value) => {
            wire::Value::Projection(wire::ProjectionValue { value: projection_record_to_record(value, limits)? })
        }
        domain::CoreRecord::RunProof(value) => wire::Value::RunProof(maps::run_proof_to_record(value, limits)?),
        domain::CoreRecord::Turn(value) => wire::Value::Turn(maps::turn_record_to_record(value)),
        domain::CoreRecord::Terminal(value) => wire::Value::Terminal(maps::terminal_record_to_record(value, limits)?),
        domain::CoreRecord::ProposalDecision(value) => {
            wire::Value::ProposalDecision(maps::proposal_decision_record_to_record(value))
        }
        domain::CoreRecord::EscalationDecision(value) => {
            wire::Value::EscalationDecision(maps::escalation_decision_record_to_record(value))
        }
    })
}
pub(crate) fn value_to_domain(value: wire::Value, limits: &View) -> Result<domain::CoreRecord, wire::Problem> {
    Ok(match value {
        wire::Value::Call(value) => domain::CoreRecord::Call(maps::call_record_to_domain(value, limits)?),
        wire::Value::Part(value) => domain::CoreRecord::Part(maps::part_record_to_domain(value)),
        wire::Value::Deployment(value) => domain::CoreRecord::Deployment(maps::deployment_to_domain(value)?),
        wire::Value::Projection(value) => {
            domain::CoreRecord::Projection(projection_record_to_domain(value.value, limits)?)
        }
        wire::Value::RunProof(value) => domain::CoreRecord::RunProof(maps::run_proof_to_domain(value, limits)?),
        wire::Value::Turn(value) => domain::CoreRecord::Turn(maps::turn_record_to_domain(value)),
        wire::Value::Terminal(value) => domain::CoreRecord::Terminal(maps::terminal_record_to_domain(value, limits)?),
        wire::Value::ProposalDecision(value) => {
            domain::CoreRecord::ProposalDecision(maps::proposal_decision_record_to_domain(value))
        }
        wire::Value::EscalationDecision(value) => {
            domain::CoreRecord::EscalationDecision(maps::escalation_decision_record_to_domain(value))
        }
    })
}
fn problem(reason: skein_codec::Reason) -> wire::Problem {
    wire::paths::ROW_VALUE.problem(reason)
}
fn task_problem(value: crate::tasks_v13::Problem) -> wire::Problem {
    wire::paths::TASK_BYTES_VALUE.problem(value.reason)
}
fn note_problem(value: crate::notes_v1::Problem) -> wire::Problem {
    wire::paths::NOTE_BYTES_VALUE.problem(value.reason)
}
fn people_problem(value: crate::people_v17::Problem) -> wire::Problem {
    problem(value.reason)
}
fn too_large(_: core::num::TryFromIntError) -> wire::Problem {
    problem(skein_codec::Reason::Bound)
}
pub(crate) use projections::{
    decode_projection_record as projection_record_to_domain, encode_projection_record as projection_record_to_record,
};

/// The versioned body stored in a transcript frame (protocol/store/transcripts.md, 3).
pub fn turn(value: &domain::TurnRecord, limits: &View) -> Result<Box<[u8]>, wire::Problem> {
    maps::turn_record_to_record(value).encode(&limits.core)
}
/// Decode a transcript frame's versioned core turn.
pub fn read_turn(bytes: &[u8], limits: &View) -> Result<domain::TurnRecord, wire::Problem> {
    Ok(maps::turn_record_to_domain(wire::TurnRecord::decode(&limits.core, &mut Reader::new(bytes))?))
}

#[must_use]
pub fn encode_proposer(value: &people::Proposer) -> wire::Proposer {
    maps::proposer_to_record(value)
}

#[must_use]
pub fn decode_proposer(value: wire::Proposer) -> people::Proposer {
    maps::proposer_to_domain(value)
}

#[must_use]
pub fn encode_proposal_choice(value: &people::ProposalChoice) -> wire::ProposalChoice {
    maps::proposal_choice_to_record(value)
}

#[must_use]
pub fn decode_proposal_choice(value: wire::ProposalChoice) -> people::ProposalChoice {
    maps::proposal_choice_to_domain(value)
}

#[must_use]
pub fn encode_escalation_decision(value: &people::EscalationDecision) -> wire::EscalationDecision {
    maps::escalation_decision_to_record(value)
}

#[must_use]
pub fn decode_escalation_decision(value: wire::EscalationDecision) -> people::EscalationDecision {
    maps::escalation_decision_to_domain(value)
}

#[must_use]
pub fn encode_proposal_kind(value: &tasks::ProposalKind) -> wire::ProposalKind {
    maps::proposal_kind_to_record(value)
}

#[must_use]
pub fn decode_proposal_kind(value: wire::ProposalKind) -> tasks::ProposalKind {
    maps::proposal_kind_to_domain(value)
}

#[must_use]
pub fn encode_escalation_outcome(value: &tasks::EscalationOutcome) -> wire::EscalationOutcome {
    maps::escalation_outcome_to_record(value)
}

#[must_use]
pub fn decode_escalation_outcome(value: wire::EscalationOutcome) -> tasks::EscalationOutcome {
    maps::escalation_outcome_to_domain(value)
}

#[must_use]
pub fn encode_proposal_outcome(value: &tasks::ProposalOutcome) -> wire::ProposalOutcome {
    maps::proposal_outcome_to_record(value)
}

#[must_use]
pub fn decode_proposal_outcome(value: wire::ProposalOutcome) -> tasks::ProposalOutcome {
    maps::proposal_outcome_to_domain(value)
}

#[must_use]
pub fn encode_source(value: &authority::Source) -> wire::Source {
    maps::source_to_record(value)
}

#[must_use]
pub fn decode_source(value: wire::Source) -> authority::Source {
    maps::source_to_domain(value)
}

#[must_use]
pub fn encode_lacks(value: &authority::Lacks) -> wire::Lacks {
    maps::lacks_to_record(value)
}

#[must_use]
pub fn decode_lacks(value: wire::Lacks) -> authority::Lacks {
    maps::lacks_to_domain(value)
}

#[must_use]
pub fn encode_judge(value: &authority::Judge) -> wire::Judge {
    maps::judge_to_record(value)
}

#[must_use]
pub fn decode_judge(value: wire::Judge) -> authority::Judge {
    maps::judge_to_domain(value)
}

#[must_use]
pub fn encode_answer(value: &authority::Answer) -> wire::Answer {
    maps::answer_to_record(value)
}

#[must_use]
pub fn decode_answer(value: wire::Answer) -> authority::Answer {
    maps::answer_to_domain(value)
}

#[must_use]
pub fn encode_finding(value: &authority::Finding) -> wire::Finding {
    maps::finding_to_record(value)
}

#[must_use]
pub fn decode_finding(value: wire::Finding) -> authority::Finding {
    maps::finding_to_domain(value)
}

#[must_use]
pub fn encode_host_kind(value: &fleet::HostKind) -> wire::HostKind {
    maps::host_kind_to_record(value)
}

#[must_use]
pub fn decode_host_kind(value: wire::HostKind) -> fleet::HostKind {
    maps::host_kind_to_domain(value)
}

#[must_use]
pub fn encode_outbox_outcome(value: &domain::effects::connector::OutboxOutcome) -> wire::OutboxOutcome {
    maps::outbox_outcome_to_record(value)
}

#[must_use]
pub fn decode_outbox_outcome(value: wire::OutboxOutcome) -> domain::effects::connector::OutboxOutcome {
    maps::outbox_outcome_to_domain(value)
}

#[must_use]
pub fn encode_delivery_outcome(value: &domain::DeliveryOutcome) -> wire::DeliveryOutcome {
    maps::delivery_outcome_to_record(value)
}

#[must_use]
pub fn decode_delivery_outcome(value: wire::DeliveryOutcome) -> domain::DeliveryOutcome {
    maps::delivery_outcome_to_domain(value)
}

pub(crate) fn task_bytes_to_record(value: &tasks::Problem, limits: &View) -> Result<wire::TaskBytes, wire::Problem> {
    let value = match crate::codec::tasks::carried::problem(value, &limits.tasks) {
        Ok(value) => value,
        Err(error) => return Err(task_problem(error)),
    };
    Ok(wire::TaskBytes { value })
}
pub(crate) fn wait_floors_to_record(value: &WaitFloors) -> wire::WaitFloors {
    let mut entries = List::with_capacity(value.count());
    for kind in WaitKind::ALL {
        if let Some(at) = value.floor(kind) {
            entries
                .push(wire::WaitFloor { kind: maps::wait_kind_to_record(&kind), at: at.as_nanos() })
                .expect("one entry per floor");
        }
    }
    wire::WaitFloors { entries }
}

pub(crate) fn wait_floors_to_domain(value: wire::WaitFloors) -> Result<WaitFloors, wire::Problem> {
    let mut floors = WaitFloors::EMPTY;
    for entry in value.entries.into_boxed() {
        if !floors.set_once(maps::wait_kind_to_domain(entry.kind), Wall::from_nanos(entry.at)) {
            return Err(wire::paths::WAIT_FLOORS_ENTRIES.problem(skein_codec::Reason::Malformed));
        }
    }
    Ok(floors)
}

pub(crate) fn task_bytes_to_domain(value: wire::TaskBytes, limits: &View) -> Result<tasks::Problem, wire::Problem> {
    match crate::codec::tasks::carried::read_problem(&value.value, &limits.tasks) {
        Ok(value) => Ok(value),
        Err(error) => Err(task_problem(error)),
    }
}
pub(crate) fn settings_refusal_to_record(value: &people::Refusal) -> Result<wire::SettingsRefusal, wire::Problem> {
    let value = match crate::codec::people::refusal_bytes(value) {
        Ok(value) => value,
        Err(error) => return Err(people_problem(error)),
    };
    Ok(wire::SettingsRefusal { value })
}
pub(crate) fn settings_refusal_to_domain(value: wire::SettingsRefusal) -> Result<people::Refusal, wire::Problem> {
    match crate::codec::people::read_refusal(&value.value) {
        Ok(value) => Ok(value),
        Err(error) => Err(people_problem(error)),
    }
}
pub(crate) fn note_bytes_to_record(value: &notes::Entry, limits: &View) -> Result<wire::NoteBytes, wire::Problem> {
    let value = match crate::codec::notes::encode(&notes::Record::Entry(value.clone()), &limits.notes) {
        Ok(value) => value,
        Err(error) => return Err(note_problem(error)),
    };
    Ok(wire::NoteBytes { value })
}
pub(crate) fn note_bytes_to_domain(value: wire::NoteBytes, limits: &View) -> Result<notes::Entry, wire::Problem> {
    let record = match crate::codec::notes::decode(&value.value, &limits.notes) {
        Ok(value) => value,
        Err(error) => return Err(note_problem(error)),
    };
    match record {
        notes::Record::Entry(value) => Ok(value),
        notes::Record::Line(_) => Err(problem(skein_codec::Reason::Tag)),
    }
}
pub(crate) fn phase_bytes_to_record(value: &tasks::Phase, limits: &View) -> Result<wire::PhaseBytes, wire::Problem> {
    let value = match crate::codec::tasks::carried::phase(value, &limits.tasks) {
        Ok(value) => value,
        Err(error) => return Err(task_problem(error)),
    };
    Ok(wire::PhaseBytes { value })
}
pub(crate) fn phase_bytes_to_domain(value: wire::PhaseBytes, limits: &View) -> Result<tasks::Phase, wire::Problem> {
    match crate::codec::tasks::carried::read_phase(&value.value, &limits.tasks) {
        Ok(value) => Ok(value),
        Err(error) => Err(task_problem(error)),
    }
}
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps by-hand pairs borrow the domain value")]
pub(crate) fn change_bytes_to_record(value: &tasks::Change, limits: &View) -> Result<wire::ChangeBytes, wire::Problem> {
    let value = match crate::codec::tasks::carried::change(value, &limits.tasks) {
        Ok(value) => value,
        Err(error) => return Err(task_problem(error)),
    };
    Ok(wire::ChangeBytes { value })
}
pub(crate) fn change_bytes_to_domain(value: wire::ChangeBytes, limits: &View) -> Result<tasks::Change, wire::Problem> {
    match crate::codec::tasks::carried::read_change(&value.value, &limits.tasks) {
        Ok(value) => Ok(value),
        Err(error) => Err(task_problem(error)),
    }
}
pub(crate) fn end_bytes_to_record(value: &tasks::End, limits: &View) -> Result<wire::EndBytes, wire::Problem> {
    let value = match crate::codec::tasks::carried::end(value, &limits.tasks) {
        Ok(value) => value,
        Err(error) => return Err(task_problem(error)),
    };
    Ok(wire::EndBytes { value })
}
pub(crate) fn end_bytes_to_domain(value: wire::EndBytes, limits: &View) -> Result<tasks::End, wire::Problem> {
    match crate::codec::tasks::carried::read_end(&value.value, &limits.tasks) {
        Ok(value) => Ok(value),
        Err(error) => Err(task_problem(error)),
    }
}
pub(crate) fn created_to_record(value: &(u64, u64)) -> wire::Created {
    let (first, last) = *value;
    wire::Created { first, last }
}
pub(crate) fn created_to_domain(value: wire::Created) -> (u64, u64) {
    let wire::Created { first, last } = value;
    (first, last)
}
pub(crate) fn call_part_delegated_to_record(values: &[domain::Made]) -> Result<wire::CallPartDelegated, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    };
    let mut result = List::with_capacity(count);
    for value in values {
        result.push(maps::made_to_record(value)).expect("same number of members");
    }
    Ok(wire::CallPartDelegated { value: result })
}
pub(crate) fn call_part_delegated_to_domain(value: wire::CallPartDelegated) -> Box<[domain::Made]> {
    let mut result = List::with_capacity(value.value.len());
    for value in value.value.into_boxed() {
        result.push(maps::made_to_domain(value)).expect("same number of members");
    }
    result.into_boxed()
}
