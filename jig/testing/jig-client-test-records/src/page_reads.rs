//! Existing client page wire literals, shared without changing their values.
use super::w;
use skein_lib::List;

#[must_use]
pub fn read_wire_history_page() -> w::HistoryPage {
    w::HistoryPage { cut: false, rows: Box::from([]), before: None, after: None }
}

#[must_use]
pub fn read_wire_note_page() -> w::NotePage {
    w::NotePage { cut: false, rows: Box::from([]), before: None, after: None }
}

#[must_use]
pub fn read_wire_owner_ended_page() -> w::EndedPage {
    w::EndedPage { rows: Box::from([]), before: None, after: None, cut: false }
}

#[must_use]
pub fn read_wire_owner_inbox_page() -> w::InboxPage {
    w::InboxPage { selection: None, cut: false, rows: Box::from([]), before: None, after: None }
}

#[must_use]
pub fn read_wire_owner_spend_page() -> w::SpendPage {
    w::SpendPage { rows: List::with_capacity(0), before: None, after: None, cut: false }
}

#[must_use]
pub fn read_wire_activity_page() -> w::ActivityPage {
    w::ActivityPage { cut: false, rows: Box::from([]), before: None, after: None }
}
