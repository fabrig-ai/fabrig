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
