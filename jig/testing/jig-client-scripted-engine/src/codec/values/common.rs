//! Wire projections of common facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, w};
use super::wire_pattern;
pub(in crate::codec) fn wire_proposal_key(value: &c::ProposalKey) -> w::ProposalKey {
    w::ProposalKey { proposer: wire_proposer(&value.proposer), number: value.number }
}

pub(in crate::codec) fn wire_proposer(value: &c::Proposer) -> w::Proposer {
    match value {
        c::Proposer::Task(value) => w::Proposer::Task(w::ProposerTask { value: *(value) }),
        c::Proposer::Party(value) => w::Proposer::Party(w::ProposerParty { value: *(value) }),
    }
}

pub(in crate::codec) fn wire_count(value: &c::Count) -> w::Count {
    w::Count { value: value.value, limited: value.limited }
}

pub(in crate::codec) fn wire_chip_executor(value: &c::ChipExecutor) -> w::ChipExecutor {
    match value {
        c::ChipExecutor::Job { kind } => w::ChipExecutor::Job(w::ChipExecutorJob { kind: *kind }),
        c::ChipExecutor::Agent { charter } => w::ChipExecutor::Agent(w::ChipExecutorAgent { charter: *(charter) }),
        c::ChipExecutor::Procedure { connector, kind } => {
            w::ChipExecutor::Procedure(w::ChipExecutorProcedure { connector: *(connector), kind: *(kind) })
        }
        c::ChipExecutor::Party { party } => w::ChipExecutor::Party(w::ExecutorParty { party: *(party) }),
        c::ChipExecutor::Role { project, role } => {
            w::ChipExecutor::Role(w::ExecutorRole { project: *(project), role: *(role) })
        }
    }
}

pub(in crate::codec) fn wire_revision(value: &c::Revision) -> w::Revision {
    w::Revision { value: value.value }
}

pub(in crate::codec) fn wire_party_kind(value: &c::PartyKind) -> w::PartyKind {
    match value {
        c::PartyKind::Person => w::PartyKind::Person,
        c::PartyKind::Service => w::PartyKind::Service,
    }
}

pub(in crate::codec) fn wire_object_key(value: &c::ObjectKey) -> w::ObjectKey {
    match value {
        c::ObjectKey::Task(value) => w::ObjectKey::Task(w::ObjectKeyTask { value: *(value) }),
        c::ObjectKey::Proposal(value) => {
            w::ObjectKey::Proposal(w::ObjectKeyProposal { value: wire_proposal_key(value) })
        }
        c::ObjectKey::Escalation(value) => w::ObjectKey::Escalation(w::ObjectKeyEscalation { value: *(value) }),
        c::ObjectKey::Question { task, message } => {
            w::ObjectKey::Question(w::ObjectKeyQuestion { task: *(task), message: *(message) })
        }
        c::ObjectKey::PersonTask(value) => w::ObjectKey::PersonTask(w::ObjectKeyPersonTask { value: *(value) }),
        c::ObjectKey::Result(value) => w::ObjectKey::Result(w::ObjectKeyResult { value: *(value) }),
        c::ObjectKey::Note { project, note } => {
            w::ObjectKey::Note(w::ObjectKeyNote { project: *(project), note: *(note) })
        }
    }
}

pub(in crate::codec) fn wire_turn_name(value: &c::TurnName) -> w::TurnName {
    w::TurnName { attempt: value.attempt, number: value.number }
}

pub(in crate::codec) fn wire_note_scope(value: &c::NoteScope) -> w::NoteScope {
    match value {
        c::NoteScope::Tasks { project } => w::NoteScope::Tasks(w::NoteScopeTasks { project: *project }),
        c::NoteScope::Party { party } => w::NoteScope::Party(w::NoteScopeParty { party: *party }),
        c::NoteScope::Deployment => w::NoteScope::Deployment,
        c::NoteScope::Project(value) => w::NoteScope::Project(w::NoteScopeProject { value: *(value) }),
        c::NoteScope::Resource { project, connector, pattern } => w::NoteScope::Resource(w::NoteScopeResource {
            project: *(project),
            connector: *(connector),
            pattern: wire_pattern(pattern),
        }),
        c::NoteScope::Task { project, task } => {
            w::NoteScope::Task(w::NoteScopeTask { project: *(project), task: *(task) })
        }
    }
}

pub(in crate::codec) fn wire_phase(value: &c::Phase) -> w::Phase {
    match value {
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
    }
}

pub(in crate::codec) fn wire_actor(value: &c::Actor) -> w::Actor {
    match value {
        c::Actor::Party(value) => w::Actor::Party(w::ActorParty { value: *(value) }),
        c::Actor::Task(value) => w::Actor::Task(w::ActorTask { value: *(value) }),
        c::Actor::Deployment => w::Actor::Deployment,
        c::Actor::ProjectDeployment { project } => {
            w::Actor::ProjectDeployment(w::ActorProjectDeployment { project: *(project) })
        }
    }
}

pub(in crate::codec) fn wire_party(value: &c::Party) -> w::Party {
    w::Party { number: value.number, kind: wire_party_kind(&value.kind), name: bytes::copy_of(value.name.bytes()) }
}

pub(in crate::codec) fn wire_decision(value: &c::Decision) -> w::Decision {
    match value {
        c::Decision::Accepted => w::Decision::Accepted,
        c::Decision::Rejected { reason } => {
            w::Decision::Rejected(w::DecisionRejected { reason: bytes::copy_of((reason).bytes()) })
        }
        c::Decision::PassedUp => w::Decision::PassedUp,
        c::Decision::Released { again } => w::Decision::Released(w::DecisionReleased { again: *again }),
        c::Decision::LeftHeld { reason } => {
            w::Decision::LeftHeld(w::DecisionLeftHeld { reason: bytes::copy_of((reason).bytes()) })
        }
    }
}

pub(in crate::codec) fn wire_ending_fields(
    value: &c::Ending,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::EndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Ending::Task(value) => {
            w::EndingFields::Task(w::EndingTaskFields { value: facts.chip_ending(value, Some(parent), limits)? })
        }
        c::Ending::Done => w::EndingFields::Done,
        c::Ending::Failed { reason } => w::EndingFields::Failed(w::EndingFailedFields {
            reason: (reason).as_ref().map(|item| bytes::copy_of((item).bytes())),
        }),
        c::Ending::Cancelled { by, reason, at } => w::EndingFields::Cancelled(w::EndingCancelledFields {
            by: wire_actor(by),
            reason: (reason).as_ref().map(|item| bytes::copy_of((item).bytes())),
            at: (at).as_ref().map(|item| (item).as_nanos()),
        }),
        c::Ending::Withdrawn { reason } => {
            w::EndingFields::Withdrawn(w::EndingWithdrawnFields { reason: bytes::copy_of((reason).bytes()) })
        }
        c::Ending::Decided { by, party, how, at } => w::EndingFields::Decided(w::EndingDecidedFields {
            by: *(by),
            party: (party).as_ref().map(wire_party),
            how: (how).as_ref().map(wire_decision),
            at: (at).as_nanos(),
        }),
        c::Ending::Deleted => w::EndingFields::Deleted,
    })
}
