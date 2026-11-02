//! Wire projections of common facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, w};
use super::wire_pattern;
pub(in crate::codec) fn wire_proposal_key(
    value: &c::ProposalKey,
    limits: &w::Limits,
) -> Result<w::ProposalKey, w::Problem> {
    w::ProposalKey::new(
        limits,
        w::ProposalKeyParts { proposer: wire_proposer(&value.proposer, limits)?, number: value.number },
    )
}

pub(in crate::codec) fn wire_proposer(value: &c::Proposer, limits: &w::Limits) -> Result<w::Proposer, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Proposer::Task(value) => {
            w::Proposer::Task(w::ProposerTask::new(limits, w::ProposerTaskParts { value: *(value) })?)
        }
        c::Proposer::Party(value) => {
            w::Proposer::Party(w::ProposerParty::new(limits, w::ProposerPartyParts { value: *(value) })?)
        }
    })
}

pub(in crate::codec) fn wire_count(value: &c::Count, limits: &w::Limits) -> Result<w::Count, w::Problem> {
    w::Count::new(limits, w::CountParts { value: value.value, limited: value.limited })
}

pub(in crate::codec) fn wire_chip_executor(
    value: &c::ChipExecutor,
    limits: &w::Limits,
) -> Result<w::ChipExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ChipExecutor::Job { kind } => {
            w::ChipExecutor::Job(w::ChipExecutorJob::new(limits, w::ChipExecutorJobParts { kind: *kind })?)
        }
        c::ChipExecutor::Agent { charter } => w::ChipExecutor::Agent(w::ChipExecutorAgent::new(
            limits,
            w::ChipExecutorAgentParts { charter: *(charter) },
        )?),
        c::ChipExecutor::Procedure { connector, kind } => w::ChipExecutor::Procedure(w::ChipExecutorProcedure::new(
            limits,
            w::ChipExecutorProcedureParts { connector: *(connector), kind: *(kind) },
        )?),
        c::ChipExecutor::Party { party } => {
            w::ChipExecutor::Party(w::ExecutorParty::new(limits, w::ExecutorPartyParts { party: *(party) })?)
        }
        c::ChipExecutor::Role { project, role } => w::ChipExecutor::Role(w::ExecutorRole::new(
            limits,
            w::ExecutorRoleParts { project: *(project), role: *(role) },
        )?),
    })
}

pub(in crate::codec) fn wire_revision(value: &c::Revision, limits: &w::Limits) -> Result<w::Revision, w::Problem> {
    w::Revision::new(limits, w::RevisionParts { value: value.value })
}

pub(in crate::codec) fn wire_party_kind(value: &c::PartyKind, _limits: &w::Limits) -> Result<w::PartyKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PartyKind::Person => w::PartyKind::Person,
        c::PartyKind::Service => w::PartyKind::Service,
    })
}

pub(in crate::codec) fn wire_object_key(value: &c::ObjectKey, limits: &w::Limits) -> Result<w::ObjectKey, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ObjectKey::Task(value) => {
            w::ObjectKey::Task(w::ObjectKeyTask::new(limits, w::ObjectKeyTaskParts { value: *(value) })?)
        }
        c::ObjectKey::Proposal(value) => w::ObjectKey::Proposal(w::ObjectKeyProposal::new(
            limits,
            w::ObjectKeyProposalParts { value: wire_proposal_key(value, limits)? },
        )?),
        c::ObjectKey::Escalation(value) => w::ObjectKey::Escalation(w::ObjectKeyEscalation::new(
            limits,
            w::ObjectKeyEscalationParts { value: *(value) },
        )?),
        c::ObjectKey::Question { task, message } => w::ObjectKey::Question(w::ObjectKeyQuestion::new(
            limits,
            w::ObjectKeyQuestionParts { task: *(task), message: *(message) },
        )?),
        c::ObjectKey::PersonTask(value) => w::ObjectKey::PersonTask(w::ObjectKeyPersonTask::new(
            limits,
            w::ObjectKeyPersonTaskParts { value: *(value) },
        )?),
        c::ObjectKey::Result(value) => {
            w::ObjectKey::Result(w::ObjectKeyResult::new(limits, w::ObjectKeyResultParts { value: *(value) })?)
        }
        c::ObjectKey::Note { project, note } => w::ObjectKey::Note(w::ObjectKeyNote::new(
            limits,
            w::ObjectKeyNoteParts { project: *(project), note: *(note) },
        )?),
    })
}

pub(in crate::codec) fn wire_turn_name(value: &c::TurnName, limits: &w::Limits) -> Result<w::TurnName, w::Problem> {
    w::TurnName::new(limits, w::TurnNameParts { attempt: value.attempt, number: value.number })
}

pub(in crate::codec) fn wire_note_scope(value: &c::NoteScope, limits: &w::Limits) -> Result<w::NoteScope, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NoteScope::Deployment => w::NoteScope::Deployment,
        c::NoteScope::Project(value) => {
            w::NoteScope::Project(w::NoteScopeProject::new(limits, w::NoteScopeProjectParts { value: *(value) })?)
        }
        c::NoteScope::Resource { project, connector, pattern } => w::NoteScope::Resource(w::NoteScopeResource::new(
            limits,
            w::NoteScopeResourceParts {
                project: *(project),
                connector: *(connector),
                pattern: wire_pattern(pattern, limits)?,
            },
        )?),
        c::NoteScope::Task { project, task } => w::NoteScope::Task(w::NoteScopeTask::new(
            limits,
            w::NoteScopeTaskParts { project: *(project), task: *(task) },
        )?),
    })
}

pub(in crate::codec) fn wire_phase(value: &c::Phase, _limits: &w::Limits) -> Result<w::Phase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Phase::Waiting => w::Phase::Waiting,
        c::Phase::Idle => w::Phase::Idle,
        c::Phase::Due => w::Phase::Due,
        c::Phase::Preparing => w::Phase::Preparing,
        c::Phase::Claimed => w::Phase::Claimed,
        c::Phase::Running => w::Phase::Running,
        c::Phase::BackingOff => w::Phase::BackingOff,
        c::Phase::Active => w::Phase::Active,
        c::Phase::Closing => w::Phase::Closing,
        c::Phase::Held => w::Phase::Held,
        c::Phase::Ended => w::Phase::Ended,
        c::Phase::Done => w::Phase::Done,
        c::Phase::Failed => w::Phase::Failed,
        c::Phase::Cancelled => w::Phase::Cancelled,
    })
}

pub(in crate::codec) fn wire_actor(value: &c::Actor, limits: &w::Limits) -> Result<w::Actor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Actor::Party(value) => w::Actor::Party(w::ActorParty::new(limits, w::ActorPartyParts { value: *(value) })?),
        c::Actor::Task(value) => w::Actor::Task(w::ActorTask::new(limits, w::ActorTaskParts { value: *(value) })?),
        c::Actor::Deployment => w::Actor::Deployment,
        c::Actor::ProjectDeployment { project } => w::Actor::ProjectDeployment(w::ActorProjectDeployment::new(
            limits,
            w::ActorProjectDeploymentParts { project: *(project) },
        )?),
    })
}

pub(in crate::codec) fn wire_party(value: &c::Party, limits: &w::Limits) -> Result<w::Party, w::Problem> {
    w::Party::new(
        limits,
        w::PartyParts {
            number: value.number,
            kind: wire_party_kind(&value.kind, limits)?,
            name: bytes::copy_of(value.name.bytes()),
        },
    )
}

pub(in crate::codec) fn wire_decision(value: &c::Decision, limits: &w::Limits) -> Result<w::Decision, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Decision::Accepted => w::Decision::Accepted,
        c::Decision::Rejected { reason } => w::Decision::Rejected(w::DecisionRejected::new(
            limits,
            w::DecisionRejectedParts { reason: bytes::copy_of((reason).bytes()) },
        )?),
        c::Decision::PassedUp => w::Decision::PassedUp,
        c::Decision::Released { again } => w::Decision::Released(w::DecisionReleased::new(
            limits,
            w::DecisionReleasedParts { again: (again).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()? },
        )?),
        c::Decision::LeftHeld { reason } => w::Decision::LeftHeld(w::DecisionLeftHeld::new(
            limits,
            w::DecisionLeftHeldParts { reason: bytes::copy_of((reason).bytes()) },
        )?),
    })
}

pub(in crate::codec) fn wire_ending_fields(
    value: &c::Ending,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::EndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Ending::Task(value) => w::EndingFields::Task(w::EndingTaskFields::new(
            limits,
            w::EndingTaskFieldsParts { value: facts.chip_ending(value, Some(parent), limits)? },
        )?),
        c::Ending::Done => w::EndingFields::Done,
        c::Ending::Failed { reason } => w::EndingFields::Failed(w::EndingFailedFields::new(
            limits,
            w::EndingFailedFieldsParts {
                reason: (reason)
                    .as_ref()
                    .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                    .transpose()?,
            },
        )?),
        c::Ending::Cancelled { by, reason, at } => w::EndingFields::Cancelled(w::EndingCancelledFields::new(
            limits,
            w::EndingCancelledFieldsParts {
                by: wire_actor(by, limits)?,
                reason: (reason)
                    .as_ref()
                    .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                    .transpose()?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::Ending::Withdrawn { reason } => w::EndingFields::Withdrawn(w::EndingWithdrawnFields::new(
            limits,
            w::EndingWithdrawnFieldsParts { reason: bytes::copy_of((reason).bytes()) },
        )?),
        c::Ending::Decided { by, party, how, at } => w::EndingFields::Decided(w::EndingDecidedFields::new(
            limits,
            w::EndingDecidedFieldsParts {
                by: *(by),
                party: (party).as_ref().map(|item| wire_party(item, limits)).transpose()?,
                how: (how).as_ref().map(|item| wire_decision(item, limits)).transpose()?,
                at: (at).as_nanos(),
            },
        )?),
        c::Ending::Deleted => w::EndingFields::Deleted,
    })
}
