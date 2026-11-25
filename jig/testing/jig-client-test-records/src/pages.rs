//! Shared page wire literals; neither side derives its value from a translator.
use super::{inbox_both_groups_record, w, wire_text};
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Writer};
fn rows<T, const N: usize>(values: [T; N]) -> List<T> {
    List::from_boxed(Box::new(values))
}
fn pieces<const N: usize>(values: [Box<[u8]>; N]) -> Box<[u8]> {
    let size = values.iter().map(|v| v.len().checked_add(4).expect("literal piece size")).sum();
    let mut out = Writer::new(size);
    for value in values {
        skein_codec::piece::write(&mut out, &value).expect("literal piece");
    }
    out.finish()
}
fn activity_row(position: u64, attempt: u64, number: u32, decoded: Option<w::ArchivedTurn>) -> w::DecodedActivityLine {
    w::DecodedActivityLine {
        source: w::ActivityLine {
            task: 7,
            position,
            at: 17,
            line: w::Activity::Turn(w::ActivityTurn { reply: 61, attempt, turn: number, read: Some(0) }),
        },
        decoded,
    }
}
fn archive(
    attempt: u64,
    number: u32,
    cumulative: u64,
    read: Option<u64>,
    at: u64,
    turn: w::TurnBytes,
) -> w::ArchivedTurn {
    w::ArchivedTurn { task: 7, name: w::TurnName { attempt, number }, cumulative, read, at, turn }
}
#[must_use]
pub fn carried_activity_page(cursor_layout: &[u8; 32]) -> w::ActivityPage {
    let version = [0, 2];
    let form =
        [0, 1, 10, 0, 0, 0, 0, 0, 0, 0, 7, 1, 0, 0, 0, 0, 19, 1, 1, 11, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7];
    let cursor = [version.as_slice(), cursor_layout.as_slice(), form.as_slice()].concat().into_boxed_slice();
    let first = activity_row(
        13,
        31,
        37,
        Some(archive(31, 37, 19, Some(0), 23, w::TurnBytes::Carried(w::TurnCarried { bytes: Box::from([255, 0, 1]) }))),
    );
    let second = activity_row(
        11,
        41,
        43,
        Some(archive(41, 43, 29, None, 47, w::TurnBytes::Carried(w::TurnCarried { bytes: Box::from([128, 254]) }))),
    );
    let third = activity_row(7, 53, 59, None);
    w::ActivityPage {
        cut: true,
        rows: pieces([
            first.encode(&crate::limits()).expect("literal record encodes"),
            second.encode(&crate::limits()).expect("literal record encodes"),
            third.encode(&crate::limits()).expect("literal record encodes"),
        ]),
        before: Some(w::Cursor { value: cursor }),
        after: None,
    }
}
#[must_use]
pub fn oversized_activity_page() -> w::ActivityPage {
    let row = activity_row(
        13,
        31,
        37,
        Some(archive(31, 37, 19, Some(0), 23, w::TurnBytes::TooLarge(w::TurnTooLarge { bytes: 8_388_609 }))),
    );
    w::ActivityPage {
        cut: true,
        rows: pieces([row.encode(&crate::limits()).expect("literal record encodes")]),
        before: None,
        after: None,
    }
}
#[must_use]
pub fn mixed_history_page() -> w::HistoryPage {
    let mut reason = wire_text("é".as_bytes());
    reason.cut = true;
    let change = w::HistoryRow::Change(Box::new(w::HistoryRowChange {
        value: w::HistoryLine {
            task: 7,
            revision: 19,
            by: w::Requester::Person(w::RequesterPerson { value: 23 }),
            change: w::HistoryChange::Amended,
            reason,
            proposal: None,
            amendment: None,
            at: None,
        },
    }));
    let attempt = w::HistoryRow::Attempt(w::HistoryRowAttempt {
        value: w::AttemptLine {
            task: 7,
            attempt: 2,
            claimed: Some(14),
            started: Some(15),
            ended: Some(19),
            end: Some(w::RunEnd::Finished),
            phases: Some(3),
        },
    });
    let hold = w::HistoryRow::Hold(w::HistoryRowHold {
        value: w::HoldLine {
            task: 7,
            position: 31,
            at: 37,
            before: w::PriorPhase::Waiting,
            why: w::HoldCategory::Deadline,
            source: None,
            ending: None,
        },
    });
    w::HistoryPage {
        cut: true,
        rows: pieces([
            change.encode(&crate::limits()).expect("literal record encodes"),
            attempt.encode(&crate::limits()).expect("literal record encodes"),
            hold.encode(&crate::limits()).expect("literal record encodes"),
        ]),
        before: None,
        after: None,
    }
}
#[must_use]
pub fn selected_inbox_pages() -> [w::InboxPage; 2] {
    let source = inbox_both_groups_record();
    [source.needs, source.updates].map(|rows| w::InboxPage {
        cut: true,
        rows,
        before: None,
        after: None,
        selection: source.selection.clone(),
    })
}
#[must_use]
pub fn scoped_notes_page() -> w::NotePage {
    let pattern = w::Pattern {
        segments: rows([Box::<[u8]>::from([255, b'/', 0])]),
        last: w::PatternEnd::Open(w::PatternEndOpen { value: Box::from([128, 0]) }),
    };
    let mut body = wire_text(b"body");
    body.cut = true;
    let row = w::Note {
        number: 7,
        revision: w::Revision { value: 11 },
        scope: w::NoteScope::Resource(w::NoteScopeResource { project: 13, connector: 17, pattern }),
        description: wire_text("é".as_bytes()),
        body,
        writer: w::NoteAuthor::Task(w::NoteAuthorTask { task: 19, attempt: 23 }),
        tasks: rows([29, 31]),
        changed: Some(37),
        acts: Box::new([]),
        cut: true,
    };
    w::NotePage {
        cut: true,
        rows: pieces([row.encode(&crate::limits()).expect("literal record encodes")]),
        before: None,
        after: None,
    }
}
