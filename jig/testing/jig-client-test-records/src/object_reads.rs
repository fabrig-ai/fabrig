//! Existing single-read wire literals shared by both protocol sides.
use super::{List, chip_record, fact_bytes, task_record, w};

#[must_use]
pub fn read_ending_record() -> w::Ending {
    let row = w::Fact::Ending(
        w::EndingRow { parent: None, value: w::EndingFields::Done },
    );
    w::Ending { rows: fact_bytes(&row) }
}

#[must_use]
pub fn read_wire_document_specification() -> w::DocumentSpecification {
    w::DocumentSpecification {
            asks: false,
            words: read_wire_document_text(),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        }
}

#[must_use]
pub fn read_wire_document_text() -> w::DocumentText {
    w::DocumentText { bytes: Box::from([]), cut: false }
}

#[must_use]
pub fn read_wire_object_key() -> w::ObjectKey {
    w::ObjectKey::Task(read_wire_object_key_task())
}

#[must_use]
pub fn read_wire_object_key_task() -> w::ObjectKeyTask {
    w::ObjectKeyTask { value: 7 }
}

#[must_use]
pub fn read_wire_owner_escalation() -> w::Escalation {
    w::Escalation { acts: Box::from([]), task: task_record() }
}

#[must_use]
pub fn read_wire_owner_ended() -> w::Ended {
    w::Ended { key: read_wire_object_key(), value: read_wire_owner_ended_value(), at: None, by: None }
}

#[must_use]
pub fn read_wire_owner_ended_value() -> w::EndedValue {
    w::EndedValue::Task(read_wire_owner_ended_value_task())
}

#[must_use]
pub fn read_wire_owner_ended_value_task() -> w::EndedValueTask {
    w::EndedValueTask { document: task_record(), result: read_wire_result() }
}

#[must_use]
pub fn read_wire_owner_history_holder() -> w::HistoryHolder {
    w::HistoryHolder::Task(read_wire_owner_history_holder_task())
}

#[must_use]
pub fn read_wire_owner_history_holder_task() -> w::HistoryHolderTask {
    w::HistoryHolderTask { value: 7 }
}

#[must_use]
pub fn read_wire_owner_part() -> w::Part {
    w::Part { connector: 7, kind: read_wire_owner_part_kind(), key: read_wire_owner_part_key() }
}

#[must_use]
pub fn read_wire_owner_part_key() -> w::PartKey {
    w::PartKey::Path(read_wire_owner_part_key_path())
}

#[must_use]
pub fn read_wire_owner_part_key_path() -> w::PartKeyPath {
    w::PartKeyPath { value: List::with_capacity(0) }
}

#[must_use]
pub fn read_wire_owner_part_kind() -> w::PartKind {
    w::PartKind::Resource
}

#[must_use]
pub fn read_wire_owner_person_task() -> w::PersonTask {
    w::PersonTask {
            task: chip_record(),
            revision: read_wire_revision(),
            requester: read_wire_owner_requester(),
            spec: read_wire_specification_document(),
            address: read_wire_person_address(),
            taken_by: None,
            since: None,
            stall: None,
            deadline: None,
            contract: read_wire_task_contract_document(),
            results: Box::from([]),
            cut: false,
            acts: Box::from([]),
        }
}

#[must_use]
pub fn read_wire_owner_proposal() -> w::Proposal {
    w::Proposal {
            projects: List::with_capacity(0),
            key: read_wire_proposal_key(),
            project: 7,
            reason: read_wire_document_text(),
            action: read_wire_owner_proposal_action(),
            as_holder: false,
            waiting: read_wire_owner_proposal_waiting(),
            funding: List::with_capacity(0),
            acts: Box::from([]),
        }
}

#[must_use]
pub fn read_wire_owner_proposal_action() -> w::ProposalAction {
    w::ProposalAction::Batch(read_wire_owner_proposal_action_batch())
}

#[must_use]
pub fn read_wire_owner_proposal_action_batch() -> w::ProposalActionBatch {
    w::ProposalActionBatch { value: Box::from([]) }
}

#[must_use]
pub fn read_wire_owner_proposal_waiting() -> w::ProposalWaiting {
    w::ProposalWaiting { holder: read_wire_owner_history_holder(), since: 7 }
}

#[must_use]
pub fn read_wire_owner_requester() -> w::Requester {
    w::Requester::Task(read_wire_owner_requester_task())
}

#[must_use]
pub fn read_wire_owner_requester_task() -> w::RequesterTask {
    w::RequesterTask { value: 7 }
}

#[must_use]
pub fn read_wire_person_address() -> w::PersonAddress {
    w::PersonAddress::Party(read_wire_person_address_party())
}

#[must_use]
pub fn read_wire_person_address_party() -> w::PersonAddressParty {
    w::PersonAddressParty { value: 7 }
}

#[must_use]
pub fn read_wire_proposal_key() -> w::ProposalKey {
    w::ProposalKey { proposer: read_wire_proposer(), number: 7 }
}

#[must_use]
pub fn read_wire_proposer() -> w::Proposer {
    w::Proposer::Task(read_wire_proposer_task())
}

#[must_use]
pub fn read_wire_proposer_task() -> w::ProposerTask {
    w::ProposerTask { value: 7 }
}

#[must_use]
pub fn read_wire_question() -> w::Question {
    w::Question {
            task: chip_record(),
            from: None,
            message: 7,
            words: w::DocumentText { bytes: Box::from([]), cut: false },
            since: 7,
            acts: Box::from([]),
            ending: None,
        }
}

#[must_use]
pub fn read_wire_result() -> w::ResultDocument {
    w::ResultDocument {
            task: chip_record(),
            ending: read_ending_record(),
            value: None,
            at: None,
            details: None,
            acts: Box::from([]),
        }
}

#[must_use]
pub fn read_wire_revision() -> w::Revision {
    w::Revision { value: 7 }
}

#[must_use]
pub fn read_wire_specification_document() -> w::SpecificationDocument {
    w::SpecificationDocument { specification: read_wire_document_specification(), cut: false }
}

#[must_use]
pub fn read_wire_task_contract_document() -> w::TaskContractDocument {
    w::TaskContractDocument::Approval(read_wire_task_contract_document_approval())
}

#[must_use]
pub fn read_wire_task_contract_document_approval() -> w::TaskContractDocumentApproval {
    w::TaskContractDocumentApproval { state: Box::from([]), part: read_wire_owner_part() }
}
