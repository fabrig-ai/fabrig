use super::*;
use skein_lib::List;
pub fn read_wire_owner_ended() -> w::Ended {
    jig_client_test_records::read_wire_owner_ended()
}

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
