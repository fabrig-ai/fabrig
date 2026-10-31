//! Shared activity wire records; original client values remain independent.
use super::w;
use skein_lib::Writer;
pub const FULL_TURN: &[u8] = include_bytes!("../../../crates/jig-protocol-transcript/tests/data/record_turn_full.bin");
const UTF8_TEXTS: [&[u8]; 1] = [b"~\0\x01"];
fn wire_cursor(value: &[u8]) -> w::Cursor {
    w::Cursor::new(&w::CEILINGS, w::CursorParts { value: Box::from(value) }).expect("literal fixture fits")
}
#[must_use]
pub fn turn_line(position: u64, at: u64, number: u32, bytes: &[u8]) -> w::DecodedActivityLine {
    let lim = w::CEILINGS;
    let source = w::ActivityLine::new(
        &lim,
        w::ActivityLineParts {
            task: 7,
            position,
            at,
            line: w::Activity::Turn(
                w::ActivityTurn::new(
                    &lim,
                    w::ActivityTurnParts { reply: position, attempt: 44, turn: number, read: None },
                )
                .expect("literal turn"),
            ),
        },
    )
    .expect("literal source");
    let decoded = w::ArchivedTurn::new(
        &lim,
        w::ArchivedTurnParts {
            task: 7,
            name: w::TurnName::new(&lim, w::TurnNameParts { attempt: 44, number }).expect("literal name"),
            cumulative: 0,
            read: None,
            at,
            turn: w::TurnBytes::Carried(
                w::TurnCarried::new(&lim, w::TurnCarriedParts { bytes: Box::from(bytes) }).expect("literal turn bytes"),
            ),
        },
    )
    .expect("literal archive");
    w::DecodedActivityLine::new(&lim, w::DecodedActivityLineParts { source, decoded: Some(decoded) })
        .expect("literal line")
}

#[must_use]
pub fn activity_rows(rows: &[w::DecodedActivityLine]) -> w::ActivityPage {
    let mut length = 0_usize;
    for row in rows {
        length = length
            .checked_add(skein_lib::length::usize_of(row.measure()))
            .expect("literal fixture fits")
            .checked_add(4)
            .expect("literal fixture fits");
    }
    let mut piece = Writer::new(length);
    for row in rows {
        let mut bytes = Writer::new(skein_lib::length::usize_of(row.measure()));
        row.encode(&mut bytes).expect("literal fixture fits");
        skein_codec::piece::write(&mut piece, &bytes.finish()).expect("literal fixture fits");
    }
    w::ActivityPage::new(
        &w::CEILINGS,
        w::ActivityPageParts { rows: piece.finish(), cut: true, before: None, after: Some(wire_cursor(UTF8_TEXTS[0])) },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn decoded_activity_record() -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, FULL_TURN), turn_line(11, 21, 2, FULL_TURN)])
}

#[must_use]
pub fn unread_activity_record(bytes: &[u8]) -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, bytes), turn_line(11, 21, 2, FULL_TURN)])
}

#[must_use]
pub fn cut_activity_record() -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, FULL_TURN)])
}

fn owner_turn_source() -> w::ActivityLine {
    let lim = w::CEILINGS;
    w::ActivityLine::new(
        &lim,
        w::ActivityLineParts {
            task: 7,
            position: 88,
            at: 66,
            line: w::Activity::Turn(
                w::ActivityTurn::new(&lim, w::ActivityTurnParts { reply: 999, attempt: 44, turn: 3, read: Some(77) })
                    .expect("literal fixture fits"),
            ),
        },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn archived_activity_record() -> w::ActivityPage {
    let lim = w::CEILINGS;
    let turn =
        w::TurnCarried::new(&lim, w::TurnCarriedParts { bytes: Box::from([0, 4]) }).expect("literal fixture fits");
    owner_turn_activity_record(Some(w::TurnBytes::Carried(turn)))
}

#[must_use]
pub fn owner_turn_activity_record(turn: Option<w::TurnBytes>) -> w::ActivityPage {
    let lim = w::CEILINGS;
    let source = owner_turn_source();
    let mut decoded = None;
    if let Some(turn) = turn {
        decoded = Some(
            w::ArchivedTurn::new(
                &lim,
                w::ArchivedTurnParts {
                    task: 7,
                    name: w::TurnName::new(&lim, w::TurnNameParts { attempt: 44, number: 3 })
                        .expect("literal fixture fits"),
                    cumulative: 123,
                    read: Some(78),
                    at: 99,
                    turn,
                },
            )
            .expect("literal fixture fits"),
        );
    }
    let row = w::DecodedActivityLine::new(&lim, w::DecodedActivityLineParts { source, decoded })
        .expect("literal fixture fits");
    activity_rows(&[row])
}
