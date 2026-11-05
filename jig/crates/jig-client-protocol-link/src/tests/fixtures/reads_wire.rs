use super::*;

pub fn read_wire_history_page() -> w::HistoryPage {
    jig_client_test_records::read_wire_history_page()
}

pub fn read_wire_note_page() -> w::NotePage {
    jig_client_test_records::read_wire_note_page()
}

pub fn read_wire_owner_ended_page() -> w::EndedPage {
    jig_client_test_records::read_wire_owner_ended_page()
}

pub fn read_wire_owner_inbox_page() -> w::InboxPage {
    jig_client_test_records::read_wire_owner_inbox_page()
}

pub fn read_wire_owner_spend_page() -> w::SpendPage {
    jig_client_test_records::read_wire_owner_spend_page()
}

pub fn read_wire_activity_page() -> w::ActivityPage {
    jig_client_test_records::read_wire_activity_page()
}

pub fn read_wire_owner_proposal() -> w::Proposal {
    jig_client_test_records::read_wire_owner_proposal()
}

pub fn read_wire_impact() -> w::Impact {
    let lim = w::CEILINGS;
    w::Impact::new(&lim, w::ImpactParts { tasks: read_wire_phase_counts(), live_runs: 7, proposals: 7 }).unwrap()
}

pub fn read_wire_phase_counts() -> w::ImpactCounts {
    let lim = w::CEILINGS;
    w::ImpactCounts::new(
        &lim,
        w::ImpactCountsParts {
            ended: 7,
            waiting: 7,
            running: 7,
            held: 7,
            closing: 7,
            done: 7,
            failed: 7,
            cancelled: 7,
        },
    )
    .unwrap()
}

pub fn read_wire_owner_escalation() -> w::Escalation {
    jig_client_test_records::read_wire_owner_escalation()
}

pub fn read_wire_question() -> w::Question {
    jig_client_test_records::read_wire_question()
}

pub fn read_wire_owner_person_task() -> w::PersonTask {
    jig_client_test_records::read_wire_owner_person_task()
}

pub fn read_wire_result() -> w::ResultDocument {
    jig_client_test_records::read_wire_result()
}

pub fn read_wire_owner_ended() -> w::Ended {
    jig_client_test_records::read_wire_owner_ended()
}
