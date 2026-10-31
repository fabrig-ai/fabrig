use super::*;

pub fn read_wire_history_page() -> w::HistoryPage {
    jig_client_test_records::read_wire_history_page()
}

pub fn read_wire_note_page() -> w::NotePage {
    jig_client_test_records::read_wire_note_page()
}

pub fn read_wire_owner_ended_page() -> w::OwnerEndedPage {
    jig_client_test_records::read_wire_owner_ended_page()
}

pub fn read_wire_owner_inbox_page() -> w::OwnerInboxPage {
    jig_client_test_records::read_wire_owner_inbox_page()
}

pub fn read_wire_owner_spend_page() -> w::OwnerSpendPage {
    jig_client_test_records::read_wire_owner_spend_page()
}

pub fn read_wire_activity_page() -> w::ActivityPage {
    jig_client_test_records::read_wire_activity_page()
}
