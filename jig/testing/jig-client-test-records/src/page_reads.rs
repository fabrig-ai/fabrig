//! Existing client page wire literals, shared without changing their values.
use super::w;
use skein_lib::List;

#[must_use]
pub fn read_wire_history_page() -> w::HistoryPage {
    let lim = w::CEILINGS;
    w::HistoryPage::new(&lim, w::HistoryPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_note_page() -> w::NotePage {
    let lim = w::CEILINGS;
    w::NotePage::new(&lim, w::NotePageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_owner_ended_page() -> w::EndedPage {
    let lim = w::CEILINGS;
    w::EndedPage::new(&lim, w::EndedPageParts { rows: Box::from([]), before: None, after: None, cut: false })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_owner_inbox_page() -> w::InboxPage {
    let lim = w::CEILINGS;
    w::InboxPage::new(
        &lim,
        w::InboxPageParts { selection: None, cut: false, rows: Box::from([]), before: None, after: None },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_owner_spend_page() -> w::SpendPage {
    let lim = w::CEILINGS;
    w::SpendPage::new(&lim, w::SpendPageParts { rows: List::with_capacity(0), before: None, after: None, cut: false })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_activity_page() -> w::ActivityPage {
    let lim = w::CEILINGS;
    w::ActivityPage::new(&lim, w::ActivityPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}
