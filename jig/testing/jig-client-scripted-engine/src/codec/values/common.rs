//! Wire projections of common facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, w};
use super::wire_pattern;
pub(in crate::codec) fn wire_proposal_key(
    value: &c::ProposalKey,
    limits: &w::Limits,
) -> Result<w::ProposalKey, w::Problem> {
    Ok(w::ProposalKey { proposer: wire_proposer(&value.proposer, limits)?, number: value.number })
}

pub(in crate::codec) fn wire_proposer(value: &c::Proposer, _limits: &w::Limits) -> Result<w::Proposer, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Proposer::Task(value) => {
            w::Proposer::Task(w::ProposerTask { value: *(value) })
        }
        c::Proposer::Party(value) => {
            w::Proposer::Party(w::ProposerParty { value: *(value) })
        }
    })
}

pub(in crate::codec) fn wire_count(value: &c::Count, _limits: &w::Limits) -> Result<w::Count, w::Problem> {
    Ok(w::Count { value: value.value, limited: value.limited })
}

pub(in crate::codec) fn wire_chip_executor(
    value: &c::ChipExecutor,
    _limits: &w::Limits,
) -> Result<w::ChipExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ChipExecutor::Job { kind } => {
            w::ChipExecutor::Job(w::ChipExecutorJob { kind: *kind })
        }
        c::ChipExecutor::Agent { charter } => w::ChipExecutor::Agent(w::ChipExecutorAgent { charter: *(charter) }),
        c::ChipExecutor::Procedure { connector, kind } => w::ChipExecutor::Procedure(w::ChipExecutorProcedure { connector: *(connector), kind: *(kind) }),
        c::ChipExecutor::Party { party } => {
            w::ChipExecutor::Party(w::ExecutorParty { party: *(party) })
        }
        c::ChipExecutor::Role { project, role } => w::ChipExecutor::Role(w::ExecutorRole { project: *(project), role: *(role) }),
    })
}

pub(in crate::codec) fn wire_revision(value: &c::Revision, _limits: &w::Limits) -> Result<w::Revision, w::Problem> {
    Ok(w::Revision { value: value.value })
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
            w::ObjectKey::Task(w::ObjectKeyTask { value: *(value) })
        }
        c::ObjectKey::Proposal(value) => w::ObjectKey::Proposal(w::ObjectKeyProposal { value: wire_proposal_key(value, limits)? }),
        c::ObjectKey::Escalation(value) => w::ObjectKey::Escalation(w::ObjectKeyEscalation { value: *(value) }),
        c::ObjectKey::Question { task, message } => w::ObjectKey::Question(w::ObjectKeyQuestion { task: *(task), message: *(message) }),
        c::ObjectKey::PersonTask(value) => w::ObjectKey::PersonTask(w::ObjectKeyPersonTask { value: *(value) }),
        c::ObjectKey::Result(value) => {
            w::ObjectKey::Result(w::ObjectKeyResult { value: *(value) })
        }
        c::ObjectKey::Note { project, note } => w::ObjectKey::Note(w::ObjectKeyNote { project: *(project), note: *(note) }),
    })
}

pub(in crate::codec) fn wire_turn_name(value: &c::TurnName, _limits: &w::Limits) -> Result<w::TurnName, w::Problem> {
    Ok(w::TurnName { attempt: value.attempt, number: value.number })
}

pub(in crate::codec) fn wire_note_scope(value: &c::NoteScope, limits: &w::Limits) -> Result<w::NoteScope, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NoteScope::Tasks { project } => {
            w::NoteScope::Tasks(w::NoteScopeTasks { project: *project })
        }
        c::NoteScope::Party { party } => {
            w::NoteScope::Party(w::NoteScopeParty { party: *party })
        }
        c::NoteScope::Deployment => w::NoteScope::Deployment,
        c::NoteScope::Project(value) => {
            w::NoteScope::Project(w::NoteScopeProject { value: *(value) })
        }
        c::NoteScope::Resource { project, connector, pattern } => w::NoteScope::Resource(w::NoteScopeResource {
                project: *(project),
                connector: *(connector),
                pattern: wire_pattern(pattern, limits)?,
            }),
        c::NoteScope::Task { project, task } => w::NoteScope::Task(w::NoteScopeTask { project: *(project), task: *(task) }),
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

pub(in crate::codec) fn wire_actor(value: &c::Actor, _limits: &w::Limits) -> Result<w::Actor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Actor::Party(value) => w::Actor::Party(w::ActorParty { value: *(value) }),
        c::Actor::Task(value) => w::Actor::Task(w::ActorTask { value: *(value) }),
        c::Actor::Deployment => w::Actor::Deployment,
        c::Actor::ProjectDeployment { project } => w::Actor::ProjectDeployment(w::ActorProjectDeployment { project: *(project) }),
    })
}

pub(in crate::codec) fn wire_party(value: &c::Party, limits: &w::Limits) -> Result<w::Party, w::Problem> {
    Ok(w::Party {
            number: value.number,
            kind: wire_party_kind(&value.kind, limits)?,
            name: bytes::copy_of(value.name.bytes()),
        })
}

pub(in crate::codec) fn wire_decision(value: &c::Decision, _limits: &w::Limits) -> Result<w::Decision, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Decision::Accepted => w::Decision::Accepted,
        c::Decision::Rejected { reason } => w::Decision::Rejected(w::DecisionRejected { reason: bytes::copy_of((reason).bytes()) }),
        c::Decision::PassedUp => w::Decision::PassedUp,
        c::Decision::Released { again } => w::Decision::Released(w::DecisionReleased { again: (again).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()? }),
        c::Decision::LeftHeld { reason } => w::Decision::LeftHeld(w::DecisionLeftHeld { reason: bytes::copy_of((reason).bytes()) }),
    })
}

pub(in crate::codec) fn wire_ending_fields(
    value: &c::Ending,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::EndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Ending::Task(value) => w::EndingFields::Task(w::EndingTaskFields { value: facts.chip_ending(value, Some(parent), limits)? }),
        c::Ending::Done => w::EndingFields::Done,
        c::Ending::Failed { reason } => w::EndingFields::Failed(w::EndingFailedFields {
                reason: (reason)
                    .as_ref()
                    .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                    .transpose()?,
            }),
        c::Ending::Cancelled { by, reason, at } => w::EndingFields::Cancelled(w::EndingCancelledFields {
                by: wire_actor(by, limits)?,
                reason: (reason)
                    .as_ref()
                    .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                    .transpose()?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            }),
        c::Ending::Withdrawn { reason } => w::EndingFields::Withdrawn(w::EndingWithdrawnFields { reason: bytes::copy_of((reason).bytes()) }),
        c::Ending::Decided { by, party, how, at } => w::EndingFields::Decided(w::EndingDecidedFields {
                by: *(by),
                party: (party).as_ref().map(|item| wire_party(item, limits)).transpose()?,
                how: (how).as_ref().map(|item| wire_decision(item, limits)).transpose()?,
                at: (at).as_nanos(),
            }),
        c::Ending::Deleted => w::EndingFields::Deleted,
    })
}
