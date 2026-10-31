use super::*;
use skein_lib::{List, Wall};
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

pub fn read_value_owner_wake_policy() -> c::OwnerWakePolicy {
    c::OwnerWakePolicy {
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

pub fn read_value_owner_authority() -> c::OwnerAuthority {
    c::OwnerAuthority {
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

pub fn read_value_owner_delegation() -> c::OwnerDelegation {
    c::OwnerDelegation { kinds: List::with_capacity(0), tasks: 7, depth: 7 }
}

pub fn read_value_impact() -> c::Impact {
    c::Impact { tasks: read_value_phase_counts(), live_runs: 7, proposals: 7 }
}

pub fn read_value_phase_counts() -> c::PhaseCounts {
    c::PhaseCounts { ended: 7, waiting: 7, running: 7, held: 7, closing: 7, done: 7, failed: 7, cancelled: 7 }
}

pub fn read_value_owner_escalation() -> c::OwnerEscalation {
    c::OwnerEscalation { acts: List::with_capacity(0), task: Box::new(task()) }
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
