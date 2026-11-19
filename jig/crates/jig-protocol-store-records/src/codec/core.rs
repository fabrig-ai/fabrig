//! Core rows carried by their generated family (protocol/store/records.md, 1).
use crate::core_v6::{self as wire, Record};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core_vocabulary as domain;
use jig_domain_fleet as fleet;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Reader, Wall};
mod calls;
pub(crate) mod projections;
mod runs;
use super::View;

pub(super) fn limits(
    domain: &domain::Limits,
    tasks: &crate::tasks_v13::Limits,
    notes: &crate::notes_v1::Limits,
) -> Option<wire::Limits> {
    let core = wire::Limits {
        settings_refusal_value: u32::try_from(crate::people_v17::Refusal::encoded_bound(&crate::people_v17::CEILINGS)?)
            .ok()?,
        phase_bytes_value: u32::try_from(crate::tasks_v13::PhaseRecord::encoded_bound(tasks)?).ok()?,
        change_bytes_value: u32::try_from(crate::tasks_v13::ChangeRecord::encoded_bound(tasks)?).ok()?,
        end_bytes_value: u32::try_from(crate::tasks_v13::EndRecord::encoded_bound(tasks)?).ok()?,
        projection_milestone_words: domain.tasks.message_bytes.max(domain.tasks.result_bytes),
        projection_goal_hot_pending: domain.connectors,
        projection_words_words: domain.tasks.spec_bytes,
        run_proof_accounts: tasks::MODELS,
        run_proof_credentials: domain.accounts.credentials,
        run_proof_outstanding: domain.tasks.run_inbox,
        turn_record_transcript: wire::CEILINGS.turn_record_transcript,

        escalation_decision_reject_reason: domain.escalation_reason_bytes.max(domain.people.words),
        settled_answer_host_body: domain.call_answer_bytes,
        settled_answer_delivery_evidence: domain.call_answer_bytes,
        settled_call_tool: domain.call_answer_bytes,
        call_part_effect_denied_findings: jig_domain_authority::stored_finding_limit(&domain.authority)?,
        call_part_tool_denied_findings: jig_domain_authority::stored_finding_limit(&domain.authority)?,
        call_part_delegated_value: domain.tasks.batch,
        call_part_delegation_denied_findings: jig_domain_authority::stored_finding_limit(&domain.authority)?,
        call_part_note_recalled_entries: domain.notes.recalled,
        task_bytes_value: u32::try_from(crate::tasks_v13::TaskProblem::encoded_bound(tasks)?).ok()?,
        note_bytes_value: u32::try_from(crate::notes_v1::Row::encoded_bound(notes)?).ok()?,
    };
    wire::check_limits(&core).ok()?;
    Some(core)
}
pub fn encode(row: &domain::CoreRecord, limits: &View) -> Result<Box<[u8]>, wire::Problem> {
    let value = match row {
        domain::CoreRecord::Call(call) => wire::Value::Call(calls::encode_call_record(call, limits)?),
        domain::CoreRecord::Settings(_) => {
            return Err(wire::Problem { path: wire::paths::VALUE_TAG, reason: skein_codec::Reason::Bound });
        }
        domain::CoreRecord::Deployment(value) => wire::Value::Deployment(runs::encode_deployment(value, limits)),
        domain::CoreRecord::Projection(value) => wire::Value::Projection(wire::ProjectionValue {
            value: projections::encode_projection_record(value, limits)?,
        }),
        domain::CoreRecord::RunProof(value) => wire::Value::RunProof(runs::encode_run_proof(value, limits)?),
        domain::CoreRecord::Turn(value) => wire::Value::Turn(runs::encode_turn_record(value, limits)),
        domain::CoreRecord::Terminal(value) => wire::Value::Terminal(runs::encode_terminal_record(value, limits)?),
        domain::CoreRecord::ProposalDecision(value) => {
            wire::Value::ProposalDecision(runs::encode_proposal_decision_record(value, limits)?)
        }
        domain::CoreRecord::EscalationDecision(value) => {
            wire::Value::EscalationDecision(runs::encode_escalation_decision_record(value, limits)?)
        }
    };
    let row = wire::Row { value };
    row.encode(&limits.core)
}
pub fn decode(bytes: &[u8], limits: &View) -> Result<domain::CoreRecord, wire::Problem> {
    let wire::Row { value } = wire::Row::decode(&limits.core, &mut Reader::new(bytes))?;
    Ok(match value {
        wire::Value::Call(value) => domain::CoreRecord::Call(calls::decode_call_record(value, limits)?),
        wire::Value::Deployment(value) => domain::CoreRecord::Deployment(runs::decode_deployment(value)),
        wire::Value::Projection(value) => {
            domain::CoreRecord::Projection(projections::decode_projection_record(value.value, limits)?)
        }
        wire::Value::RunProof(value) => domain::CoreRecord::RunProof(runs::decode_run_proof(value, limits)?),
        wire::Value::Turn(value) => domain::CoreRecord::Turn(runs::decode_turn_record(value)),
        wire::Value::Terminal(value) => domain::CoreRecord::Terminal(runs::decode_terminal_record(value, limits)?),
        wire::Value::ProposalDecision(value) => {
            domain::CoreRecord::ProposalDecision(runs::decode_proposal_decision_record(value))
        }
        wire::Value::EscalationDecision(value) => {
            domain::CoreRecord::EscalationDecision(runs::decode_escalation_decision_record(value))
        }
    })
}
fn problem(reason: skein_codec::Reason) -> wire::Problem {
    wire::Problem { path: wire::paths::ROW_VALUE, reason }
}
fn task_problem(value: crate::tasks_v13::Problem) -> wire::Problem {
    wire::Problem { path: wire::paths::TASK_BYTES_VALUE, reason: value.reason }
}
fn note_problem(value: crate::notes_v1::Problem) -> wire::Problem {
    wire::Problem { path: wire::paths::NOTE_BYTES_VALUE, reason: value.reason }
}
fn too_large(_: core::num::TryFromIntError) -> wire::Problem {
    problem(skein_codec::Reason::Bound)
}
pub fn encode_proposer(value: &people::Proposer, _limits: &View) -> Result<wire::Proposer, wire::Problem> {
    Ok(match value {
        people::Proposer::Task(value) => wire::Proposer::Task(wire::ProposerTask { value: *value }),
        people::Proposer::Party(value) => wire::Proposer::Party(wire::ProposerParty { value: *value }),
    })
}
#[must_use]
pub fn decode_proposer(value: wire::Proposer) -> people::Proposer {
    match value {
        wire::Proposer::Task(value) => {
            let wire::ProposerTask { value } = value;
            people::Proposer::Task(value)
        }
        wire::Proposer::Party(value) => {
            let wire::ProposerParty { value } = value;
            people::Proposer::Party(value)
        }
    }
}

#[must_use]
pub fn encode_proposal_choice(value: &people::ProposalChoice) -> wire::ProposalChoice {
    match value {
        people::ProposalChoice::Accepted => wire::ProposalChoice::Accepted,
        people::ProposalChoice::Rejected => wire::ProposalChoice::Rejected,
        people::ProposalChoice::Passed => wire::ProposalChoice::Passed,
        people::ProposalChoice::Withdrawn => wire::ProposalChoice::Withdrawn,
        people::ProposalChoice::Stale => wire::ProposalChoice::Stale,
    }
}
#[must_use]
pub fn decode_proposal_choice(value: wire::ProposalChoice) -> people::ProposalChoice {
    match value {
        wire::ProposalChoice::Accepted => people::ProposalChoice::Accepted,
        wire::ProposalChoice::Rejected => people::ProposalChoice::Rejected,
        wire::ProposalChoice::Passed => people::ProposalChoice::Passed,
        wire::ProposalChoice::Withdrawn => people::ProposalChoice::Withdrawn,
        wire::ProposalChoice::Stale => people::ProposalChoice::Stale,
    }
}

pub fn encode_escalation_decision(
    value: &people::EscalationDecision,
    _limits: &View,
) -> Result<wire::EscalationDecision, wire::Problem> {
    Ok(match value {
        people::EscalationDecision::Release => wire::EscalationDecision::Release,
        people::EscalationDecision::Reject { reason } => {
            wire::EscalationDecision::Reject(wire::EscalationDecisionReject { reason: reason.clone() })
        }
        people::EscalationDecision::Pass => wire::EscalationDecision::Pass,
    })
}
#[must_use]
pub fn decode_escalation_decision(value: wire::EscalationDecision) -> people::EscalationDecision {
    match value {
        wire::EscalationDecision::Release => people::EscalationDecision::Release,
        wire::EscalationDecision::Reject(value) => {
            let wire::EscalationDecisionReject { reason } = value;
            people::EscalationDecision::Reject { reason }
        }
        wire::EscalationDecision::Pass => people::EscalationDecision::Pass,
    }
}

#[must_use]
pub fn encode_proposal_kind(value: &tasks::ProposalKind) -> wire::ProposalKind {
    match value {
        tasks::ProposalKind::Settings => wire::ProposalKind::Settings,
        tasks::ProposalKind::Batch => wire::ProposalKind::Batch,
        tasks::ProposalKind::Effect => wire::ProposalKind::Effect,
        tasks::ProposalKind::Amend => wire::ProposalKind::Amend,
        tasks::ProposalKind::Widen => wire::ProposalKind::Widen,
        tasks::ProposalKind::Release => wire::ProposalKind::Release,
    }
}
#[must_use]
pub fn decode_proposal_kind(value: wire::ProposalKind) -> tasks::ProposalKind {
    match value {
        wire::ProposalKind::Settings => tasks::ProposalKind::Settings,
        wire::ProposalKind::Batch => tasks::ProposalKind::Batch,
        wire::ProposalKind::Effect => tasks::ProposalKind::Effect,
        wire::ProposalKind::Amend => tasks::ProposalKind::Amend,
        wire::ProposalKind::Widen => tasks::ProposalKind::Widen,
        wire::ProposalKind::Release => tasks::ProposalKind::Release,
    }
}

pub fn encode_escalation_outcome(
    value: &tasks::EscalationOutcome,
    _limits: &View,
) -> Result<wire::EscalationOutcome, wire::Problem> {
    Ok(match value {
        tasks::EscalationOutcome::Released => wire::EscalationOutcome::Released,
        tasks::EscalationOutcome::Rejected => wire::EscalationOutcome::Rejected,
        tasks::EscalationOutcome::Passed { revision } => {
            wire::EscalationOutcome::Passed(wire::EscalationOutcomePassed { revision: *revision })
        }
        tasks::EscalationOutcome::Stale => wire::EscalationOutcome::Stale,
        tasks::EscalationOutcome::NoFurther => wire::EscalationOutcome::NoFurther,
        tasks::EscalationOutcome::Limit => wire::EscalationOutcome::Limit,
    })
}
#[must_use]
pub fn decode_escalation_outcome(value: wire::EscalationOutcome) -> tasks::EscalationOutcome {
    match value {
        wire::EscalationOutcome::Released => tasks::EscalationOutcome::Released,
        wire::EscalationOutcome::Rejected => tasks::EscalationOutcome::Rejected,
        wire::EscalationOutcome::Passed(value) => {
            let wire::EscalationOutcomePassed { revision } = value;
            tasks::EscalationOutcome::Passed { revision }
        }
        wire::EscalationOutcome::Stale => tasks::EscalationOutcome::Stale,
        wire::EscalationOutcome::NoFurther => tasks::EscalationOutcome::NoFurther,
        wire::EscalationOutcome::Limit => tasks::EscalationOutcome::Limit,
    }
}

#[must_use]
pub fn encode_proposal_outcome(value: &tasks::ProposalOutcome) -> wire::ProposalOutcome {
    match value {
        tasks::ProposalOutcome::Accepted => wire::ProposalOutcome::Accepted,
        tasks::ProposalOutcome::Rejected => wire::ProposalOutcome::Rejected,
        tasks::ProposalOutcome::Passed => wire::ProposalOutcome::Passed,
        tasks::ProposalOutcome::Withdrawn => wire::ProposalOutcome::Withdrawn,
        tasks::ProposalOutcome::Stale => wire::ProposalOutcome::Stale,
    }
}
#[must_use]
pub fn decode_proposal_outcome(value: wire::ProposalOutcome) -> tasks::ProposalOutcome {
    match value {
        wire::ProposalOutcome::Accepted => tasks::ProposalOutcome::Accepted,
        wire::ProposalOutcome::Rejected => tasks::ProposalOutcome::Rejected,
        wire::ProposalOutcome::Passed => tasks::ProposalOutcome::Passed,
        wire::ProposalOutcome::Withdrawn => tasks::ProposalOutcome::Withdrawn,
        wire::ProposalOutcome::Stale => tasks::ProposalOutcome::Stale,
    }
}

#[must_use]
pub fn encode_source(value: &authority::Source) -> wire::Source {
    match value {
        authority::Source::Task => wire::Source::Task,
        authority::Source::Role => wire::Source::Role,
        authority::Source::Project => wire::Source::Project,
        authority::Source::Deployment => wire::Source::Deployment,
    }
}
#[must_use]
pub fn decode_source(value: wire::Source) -> authority::Source {
    match value {
        wire::Source::Task => authority::Source::Task,
        wire::Source::Role => authority::Source::Role,
        wire::Source::Project => authority::Source::Project,
        wire::Source::Deployment => authority::Source::Deployment,
    }
}

pub fn encode_lacks(value: &authority::Lacks, _limits: &View) -> Result<wire::Lacks, wire::Problem> {
    let authority::Lacks { tools, grants, executors, tasks, depth, spend, deadline, notes, models } = value;
    Ok(wire::Lacks {
        tools: *tools,
        grants: *grants,
        executors: *executors,
        tasks: *tasks,
        depth: *depth,
        spend: *spend,
        deadline: *deadline,
        notes: *notes,
        models: *models,
    })
}
#[must_use]
pub fn decode_lacks(value: wire::Lacks) -> authority::Lacks {
    let wire::Lacks { tools, grants, executors, tasks, depth, spend, deadline, notes, models } = value;
    authority::Lacks { tools, grants, executors, tasks, depth, spend, deadline, notes, models }
}

pub fn encode_judge(value: &authority::Judge, _limits: &View) -> Result<wire::Judge, wire::Problem> {
    let authority::Judge { connector, requirement, parameters } = value;
    Ok(wire::Judge { connector: *connector, requirement: *requirement, parameters: *parameters })
}
#[must_use]
pub fn decode_judge(value: wire::Judge) -> authority::Judge {
    let wire::Judge { connector, requirement, parameters } = value;
    authority::Judge { connector, requirement, parameters }
}

#[must_use]
pub fn encode_answer(value: &authority::Answer) -> wire::Answer {
    match value {
        authority::Answer::Allow => wire::Answer::Allow,
        authority::Answer::Wait => wire::Answer::Wait,
        authority::Answer::Propose => wire::Answer::Propose,
        authority::Answer::Refuse => wire::Answer::Refuse,
    }
}
#[must_use]
pub fn decode_answer(value: wire::Answer) -> authority::Answer {
    match value {
        wire::Answer::Allow => authority::Answer::Allow,
        wire::Answer::Wait => authority::Answer::Wait,
        wire::Answer::Propose => authority::Answer::Propose,
        wire::Answer::Refuse => authority::Answer::Refuse,
    }
}

pub fn encode_finding(value: &authority::Finding, limits: &View) -> Result<wire::Finding, wire::Problem> {
    Ok(match value {
        authority::Finding::Oversized => wire::Finding::Oversized,
        authority::Finding::UnknownProject => wire::Finding::UnknownProject,
        authority::Finding::UnknownRole => wire::Finding::UnknownRole,
        authority::Finding::Authority { source, lacks } => wire::Finding::Authority(wire::FindingAuthority {
            source: encode_source(source),
            lacks: encode_lacks(lacks, limits)?,
        }),
        authority::Finding::Executor { source } => {
            wire::Finding::Executor(wire::FindingExecutor { source: encode_source(source) })
        }
        authority::Finding::Tasks { source } => {
            wire::Finding::Tasks(wire::FindingTasks { source: encode_source(source) })
        }
        authority::Finding::Spend { source } => {
            wire::Finding::Spend(wire::FindingSpend { source: encode_source(source) })
        }
        authority::Finding::Arithmetic => wire::Finding::Arithmetic,
        authority::Finding::RunBudget => wire::Finding::RunBudget,
        authority::Finding::RunCap => wire::Finding::RunCap,
        authority::Finding::Deadline => wire::Finding::Deadline,
        authority::Finding::Account => wire::Finding::Account,
        authority::Finding::Writer => wire::Finding::Writer,
        authority::Finding::Tool => wire::Finding::Tool,
        authority::Finding::Grant { source } => {
            wire::Finding::Grant(wire::FindingGrant { source: encode_source(source) })
        }
        authority::Finding::Price { source } => {
            wire::Finding::Price(wire::FindingPrice { source: encode_source(source) })
        }
        authority::Finding::ResourceAccess => wire::Finding::ResourceAccess,
        authority::Finding::ResourceUnavailable => wire::Finding::ResourceUnavailable,
        authority::Finding::Reference => wire::Finding::Reference,
        authority::Finding::Scope { source } => {
            wire::Finding::Scope(wire::FindingScope { source: encode_source(source) })
        }
        authority::Finding::Required { judge } => {
            wire::Finding::Required(wire::FindingRequired { judge: encode_judge(judge, limits)? })
        }
        authority::Finding::Failed { judge } => {
            wire::Finding::Failed(wire::FindingFailed { judge: encode_judge(judge, limits)? })
        }
        authority::Finding::Unguarded { judge } => {
            wire::Finding::Unguarded(wire::FindingUnguarded { judge: encode_judge(judge, limits)? })
        }
        authority::Finding::Unpermitted => wire::Finding::Unpermitted,
        authority::Finding::Undecidable => wire::Finding::Undecidable,
        authority::Finding::PeriodSpend => wire::Finding::PeriodSpend,
    })
}
#[must_use]
pub fn decode_finding(value: wire::Finding) -> authority::Finding {
    match value {
        wire::Finding::Oversized => authority::Finding::Oversized,
        wire::Finding::UnknownProject => authority::Finding::UnknownProject,
        wire::Finding::UnknownRole => authority::Finding::UnknownRole,
        wire::Finding::Authority(value) => {
            let wire::FindingAuthority { source, lacks } = value;
            authority::Finding::Authority { source: decode_source(source), lacks: decode_lacks(lacks) }
        }
        wire::Finding::Executor(value) => {
            let wire::FindingExecutor { source } = value;
            authority::Finding::Executor { source: decode_source(source) }
        }
        wire::Finding::Tasks(value) => {
            let wire::FindingTasks { source } = value;
            authority::Finding::Tasks { source: decode_source(source) }
        }
        wire::Finding::Spend(value) => {
            let wire::FindingSpend { source } = value;
            authority::Finding::Spend { source: decode_source(source) }
        }
        wire::Finding::Arithmetic => authority::Finding::Arithmetic,
        wire::Finding::RunBudget => authority::Finding::RunBudget,
        wire::Finding::RunCap => authority::Finding::RunCap,
        wire::Finding::Deadline => authority::Finding::Deadline,
        wire::Finding::Account => authority::Finding::Account,
        wire::Finding::Writer => authority::Finding::Writer,
        wire::Finding::Tool => authority::Finding::Tool,
        wire::Finding::Grant(value) => {
            let wire::FindingGrant { source } = value;
            authority::Finding::Grant { source: decode_source(source) }
        }
        wire::Finding::Price(value) => {
            let wire::FindingPrice { source } = value;
            authority::Finding::Price { source: decode_source(source) }
        }
        wire::Finding::ResourceAccess => authority::Finding::ResourceAccess,
        wire::Finding::ResourceUnavailable => authority::Finding::ResourceUnavailable,
        wire::Finding::Reference => authority::Finding::Reference,
        wire::Finding::Scope(value) => {
            let wire::FindingScope { source } = value;
            authority::Finding::Scope { source: decode_source(source) }
        }
        wire::Finding::Required(value) => {
            let wire::FindingRequired { judge } = value;
            authority::Finding::Required { judge: decode_judge(judge) }
        }
        wire::Finding::Failed(value) => {
            let wire::FindingFailed { judge } = value;
            authority::Finding::Failed { judge: decode_judge(judge) }
        }
        wire::Finding::Unguarded(value) => {
            let wire::FindingUnguarded { judge } = value;
            authority::Finding::Unguarded { judge: decode_judge(judge) }
        }
        wire::Finding::Unpermitted => authority::Finding::Unpermitted,
        wire::Finding::Undecidable => authority::Finding::Undecidable,
        wire::Finding::PeriodSpend => authority::Finding::PeriodSpend,
    }
}

#[must_use]
pub fn encode_host_kind(value: &fleet::HostKind) -> wire::HostKind {
    match value {
        fleet::HostKind::Worker => wire::HostKind::Worker,
        fleet::HostKind::Engine => wire::HostKind::Engine,
    }
}
#[must_use]
pub fn decode_host_kind(value: wire::HostKind) -> fleet::HostKind {
    match value {
        wire::HostKind::Worker => fleet::HostKind::Worker,
        wire::HostKind::Engine => fleet::HostKind::Engine,
    }
}

pub fn encode_outbox_outcome(
    value: &domain::effects::connector::OutboxOutcome,
    _limits: &View,
) -> Result<wire::OutboxOutcome, wire::Problem> {
    Ok(match value {
        domain::effects::connector::OutboxOutcome::Made => wire::OutboxOutcome::Made,
        domain::effects::connector::OutboxOutcome::Failed => wire::OutboxOutcome::Failed,
        domain::effects::connector::OutboxOutcome::Withdrawn => wire::OutboxOutcome::Withdrawn,
        domain::effects::connector::OutboxOutcome::NotMade => wire::OutboxOutcome::NotMade,
        domain::effects::connector::OutboxOutcome::Uncertain => wire::OutboxOutcome::Uncertain,
        domain::effects::connector::OutboxOutcome::Held { entry } => {
            wire::OutboxOutcome::Held(wire::OutboxOutcomeHeld { entry: *entry })
        }
    })
}
#[must_use]
pub fn decode_outbox_outcome(value: wire::OutboxOutcome) -> domain::effects::connector::OutboxOutcome {
    match value {
        wire::OutboxOutcome::Made => domain::effects::connector::OutboxOutcome::Made,
        wire::OutboxOutcome::Failed => domain::effects::connector::OutboxOutcome::Failed,
        wire::OutboxOutcome::Withdrawn => domain::effects::connector::OutboxOutcome::Withdrawn,
        wire::OutboxOutcome::NotMade => domain::effects::connector::OutboxOutcome::NotMade,
        wire::OutboxOutcome::Uncertain => domain::effects::connector::OutboxOutcome::Uncertain,
        wire::OutboxOutcome::Held(value) => {
            let wire::OutboxOutcomeHeld { entry } = value;
            domain::effects::connector::OutboxOutcome::Held { entry }
        }
    }
}

#[must_use]
pub fn encode_delivery_outcome(value: &domain::DeliveryOutcome) -> wire::DeliveryOutcome {
    match value {
        domain::DeliveryOutcome::Delivered => wire::DeliveryOutcome::Delivered,
        domain::DeliveryOutcome::Nothing => wire::DeliveryOutcome::Nothing,
        domain::DeliveryOutcome::Stale => wire::DeliveryOutcome::Stale,
        domain::DeliveryOutcome::Refused => wire::DeliveryOutcome::Refused,
        domain::DeliveryOutcome::Invalid => wire::DeliveryOutcome::Invalid,
        domain::DeliveryOutcome::Failed => wire::DeliveryOutcome::Failed,
    }
}
#[must_use]
pub fn decode_delivery_outcome(value: wire::DeliveryOutcome) -> domain::DeliveryOutcome {
    match value {
        wire::DeliveryOutcome::Delivered => domain::DeliveryOutcome::Delivered,
        wire::DeliveryOutcome::Nothing => domain::DeliveryOutcome::Nothing,
        wire::DeliveryOutcome::Stale => domain::DeliveryOutcome::Stale,
        wire::DeliveryOutcome::Refused => domain::DeliveryOutcome::Refused,
        wire::DeliveryOutcome::Invalid => domain::DeliveryOutcome::Invalid,
        wire::DeliveryOutcome::Failed => domain::DeliveryOutcome::Failed,
    }
}

/// The versioned body stored in a transcript frame (protocol/store/transcripts.md, 3).
pub fn turn(value: &domain::TurnRecord, limits: &View) -> Result<Box<[u8]>, wire::Problem> {
    let value = runs::encode_turn_record(value, limits);
    value.encode(&limits.core)
}
/// Decode a transcript frame's versioned core turn.
pub fn read_turn(bytes: &[u8], limits: &View) -> Result<domain::TurnRecord, wire::Problem> {
    Ok(runs::decode_turn_record(wire::TurnRecord::decode(&limits.core, &mut Reader::new(bytes))?))
}
