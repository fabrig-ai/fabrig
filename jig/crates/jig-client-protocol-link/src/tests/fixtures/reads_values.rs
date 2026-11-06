use super::*;
use skein_lib::{List, Wall};

pub fn read_value_history_page() -> c::HistoryPage {
    c::HistoryPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_note_page() -> c::NotePage {
    c::NotePage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_owner_ended_page() -> c::EndedPage {
    c::EndedPage { rows: List::with_capacity(0), before: None, after: None, cut: false }
}

pub fn read_value_owner_inbox_page() -> c::InboxPage {
    c::InboxPage { selection: None, cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_owner_spend_page() -> c::SpendPage {
    c::SpendPage { rows: List::with_capacity(0), before: None, after: None, cut: false }
}

pub fn read_value_activity_page() -> c::ActivityPage {
    c::ActivityPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_proposal_key() -> c::ProposalKey {
    c::ProposalKey { proposer: read_value_proposer(), number: 7 }
}

pub fn read_value_proposer() -> c::Proposer {
    c::Proposer::Task(7)
}

pub fn read_value_owner_proposal() -> c::Proposal {
    c::Proposal {
        projects: List::with_capacity(0),
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

pub fn read_value_owner_proposal_action() -> c::ProposalAction {
    c::ProposalAction::Batch(List::with_capacity(0))
}

pub fn read_value_owner_proposal_waiting() -> c::ProposalWaiting {
    c::ProposalWaiting { holder: read_value_owner_history_holder(), since: Wall::from_nanos(7) }
}

pub fn read_value_owner_history_holder() -> c::HistoryHolder {
    c::HistoryHolder::Task(7)
}

pub fn read_value_revision() -> c::Revision {
    c::Revision { value: 7 }
}

pub fn read_value_specification_document() -> c::SpecificationDocument {
    c::SpecificationDocument { specification: read_value_document_specification(), cut: false }
}

pub fn read_value_document_specification() -> c::DocumentSpecification {
    c::DocumentSpecification {
        asks: false,
        words: read_value_document_text(),
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    }
}

pub fn read_value_owner_wake_policy() -> c::WakePolicy {
    c::WakePolicy {
        words: read_value_wake_rule(),
        notices: read_value_wake_rule(),
        news: read_value_wake_rule(),
        results: read_value_results_wake(),
        questions: false,
        answers: false,
        timers: false,
    }
}

pub fn read_value_wake_rule() -> c::WakeRule {
    c::WakeRule::Never
}

pub fn read_value_results_wake() -> c::ResultsWake {
    c::ResultsWake::Never
}

pub fn read_value_owner_authority() -> c::Authority {
    c::Authority {
        models: Some(c::AuthorityModels::All),
        tools: 7,
        grants: List::with_capacity(0),
        delegation: read_value_owner_delegation(),
        spend: 7,
        deadline: None,
        notes: 7,
        note_resources: List::with_capacity(0),
    }
}

pub fn read_value_owner_delegation() -> c::Delegation {
    c::Delegation { kinds: List::with_capacity(0), tasks: 7, depth: 7 }
}

pub fn read_value_impact() -> c::Impact {
    c::Impact { tasks: read_value_phase_counts(), live_runs: 7, proposals: 7 }
}

pub fn read_value_phase_counts() -> c::ImpactCounts {
    c::ImpactCounts { ended: 7, waiting: 7, running: 7, held: 7, closing: 7, done: 7, failed: 7, cancelled: 7 }
}

pub fn read_value_owner_escalation() -> c::Escalation {
    c::Escalation { acts: List::with_capacity(0), task: Box::new(task()) }
}

pub fn read_value_question() -> c::Question {
    c::Question {
        task: chip(),
        from: None,
        message: 7,
        words: jig_client_domain::contract::DocumentText::complete(text(b"")),
        since: Wall::from_nanos(7),
        acts: List::with_capacity(0),
        ending: None,
    }
}

pub fn read_value_owner_person_task() -> c::PersonTask {
    c::PersonTask {
        task: chip(),
        revision: read_value_revision(),
        requester: read_value_owner_requester(),
        spec: read_value_specification_document(),
        address: read_value_person_address(),
        taken_by: None,
        since: None,
        stall: None,
        deadline: None,
        contract: read_value_task_contract_document(),
        results: List::with_capacity(0),
        cut: false,
        acts: List::with_capacity(0),
    }
}

pub fn read_value_owner_requester() -> c::Requester {
    c::Requester::Task(7)
}

pub fn read_value_person_address() -> c::PersonAddress {
    c::PersonAddress::Party(7)
}

pub fn read_value_task_contract_document() -> c::TaskContractDocument {
    c::TaskContractDocument::Approval { state: Box::from([]), part: read_value_owner_part() }
}

pub fn read_value_owner_part() -> c::Part {
    c::Part { connector: 7, kind: read_value_owner_part_kind(), key: read_value_owner_part_key() }
}

pub fn read_value_owner_part_kind() -> c::PartKind {
    c::PartKind::Resource
}

pub fn read_value_owner_part_key() -> c::PartKey {
    c::PartKey::Path(List::with_capacity(0))
}

pub fn read_value_result() -> c::Result {
    c::Result {
        task: chip(),
        ending: c::Ending::Done,
        value: None,
        at: None,
        details: None,
        acts: List::with_capacity(0),
    }
}

pub fn read_value_owner_ended() -> c::Ended {
    c::Ended { key: read_value_object_key(), value: read_value_owner_ended_value(), at: None, by: None }
}

pub fn read_value_object_key() -> c::ObjectKey {
    c::ObjectKey::Task(7)
}

pub fn read_value_owner_ended_value() -> c::EndedValue {
    c::EndedValue::Task { document: Box::new(task()), result: Box::new(read_value_result()) }
}
