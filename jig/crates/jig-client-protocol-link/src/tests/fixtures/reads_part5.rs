use super::*;
use skein_lib::{List, Wall};
pub fn read_value_proposal_key() -> c::ProposalKey {
    c::ProposalKey { proposer: read_value_proposer(), number: 7 }
}

pub fn read_value_proposer() -> c::Proposer {
    c::Proposer::Task(7)
}

pub fn read_value_actor() -> c::Actor {
    c::Actor::Party(7)
}

pub fn read_value_recipient() -> c::Recipient {
    c::Recipient::Task(7)
}

pub fn read_value_owner_proposal() -> c::OwnerProposal {
    c::OwnerProposal {
        key: read_value_proposal_key(),
        project: 7,
        reason: read_value_document_text(),
        action: read_value_owner_proposal_action(),
        as_holder: false,
        waiting: read_value_owner_proposal_waiting(),
        funding: List::with_capacity(0),
        acts: List::with_capacity(0),
    }
}

pub fn read_value_document_text() -> c::DocumentText {
    c::DocumentText { bytes: text(b""), cut: false }
}

pub fn read_value_owner_proposal_action() -> c::OwnerProposalAction {
    c::OwnerProposalAction::Batch(List::with_capacity(0))
}

pub fn read_value_owner_proposal_waiting() -> c::OwnerProposalWaiting {
    c::OwnerProposalWaiting { holder: read_value_owner_history_holder(), since: Wall::from_nanos(7) }
}

pub fn read_value_owner_history_holder() -> c::OwnerHistoryHolder {
    c::OwnerHistoryHolder::Task(7)
}

pub fn read_value_hold() -> c::Hold {
    c::Hold {
        placed: None,
        revision: read_value_revision(),
        reason: c::HoldReason::Tries { failures: List::with_capacity(0) },
        before: read_value_phase(),
        since: Wall::from_nanos(7),
        result: None,
        spend: List::with_capacity(0),
    }
}

pub fn read_value_revision() -> c::Revision {
    c::Revision { value: 7 }
}
