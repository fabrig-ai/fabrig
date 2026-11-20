//! Shared activity wire records; original client values remain independent.
use super::w;
use skein_lib::Writer;
use smith_transcript::Record;
/// An archived v6 turn, built from a current turn without supplied system context.
#[must_use]
pub fn full_turn() -> Box<[u8]> {
    use skein_lib::List;
    use smith_transcript as s;
    let replay = s::Replay { dialect: u32::MAX, bytes: Box::new([0, 127, 255]) };
    let text = s::Text { text: b"abc".as_slice().into(), replay: Some(replay) };
    let mut blocks = List::with_capacity(1);
    blocks.push(s::Block::Text(text)).expect("one block");
    let message = s::Message { role: s::Role::User, blocks };
    let mut messages = List::with_capacity(1);
    messages.push(message).expect("one message");
    let affinity = s::Affinity { key: [165; 16], thread: u32::MAX };
    let opening = s::Opening { affinity, account: Some(u32::MAX), backend: s::Backend::Direct };
    let usage = s::Usage {
        input: Some(u64::MAX),
        output: Some(u64::MAX),
        cache_read: Some(u64::MAX),
        cache_write: Some(u64::MAX),
        reasoning: Some(u64::MAX),
    };
    let turn = s::Turn {
        kind: s::TurnKind::Ordinary,
        opening: Some(opening),
        endpoint: b"abc".as_slice().into(),
        model: b"abc".as_slice().into(),
        effort: Some(b"abc".as_slice().into()),
        dialect: u32::MAX,
        place: u32::MAX,
        usage,
        spent: u64::MAX,
        jobs: u32::MAX,
        messages,
        system: None,
    };
    let mut writer = Writer::new(skein_lib::length::usize_of(turn.measure(&s::CEILINGS).expect("bounded transcript")));
    turn.write(&smith_transcript::CEILINGS, &mut writer).expect("measured turn");
    let current = writer.finish();
    let (last, body) = current.split_last().expect("system option suffix");
    assert_eq!(*last, 0, "no system context");
    let mut archived = body.to_vec();
    archived.get_mut(..2).expect("version prefix").copy_from_slice(&[0, 6]);
    archived.into_boxed_slice()
}
const UTF8_TEXTS: [&[u8]; 1] = [b"~\0\x01"];
fn wire_cursor(value: &[u8]) -> w::Cursor {
    w::Cursor { value: Box::from(value) }
}
#[must_use]
pub fn turn_line(position: u64, at: u64, number: u32, bytes: &[u8]) -> w::DecodedActivityLine {
    let source = w::ActivityLine {
        task: 7,
        position,
        at,
        line: w::Activity::Turn(w::ActivityTurn { reply: position, attempt: 44, turn: number, read: None }),
    };
    let decoded = w::ArchivedTurn {
        task: 7,
        name: w::TurnName { attempt: 44, number },
        cumulative: 0,
        read: None,
        at,
        turn: w::TurnBytes::Carried(w::TurnCarried { bytes: Box::from(bytes) }),
    };
    w::DecodedActivityLine { source, decoded: Some(decoded) }
}

#[must_use]
pub fn activity_rows(rows: &[w::DecodedActivityLine]) -> w::ActivityPage {
    let mut length = 0_usize;
    for row in rows {
        length = length
            .checked_add(skein_lib::length::usize_of(row.measure(&crate::limits()).expect("bounded fixture")))
            .expect("literal fixture fits")
            .checked_add(4)
            .expect("literal fixture fits");
    }
    let mut piece = Writer::new(length);
    for row in rows {
        let mut bytes =
            Writer::new(skein_lib::length::usize_of(row.measure(&crate::limits()).expect("bounded fixture")));
        row.write(&crate::limits(), &mut bytes).expect("literal fixture fits");
        skein_codec::piece::write(&mut piece, &bytes.finish()).expect("literal fixture fits");
    }
    w::ActivityPage { rows: piece.finish(), cut: true, before: None, after: Some(wire_cursor(UTF8_TEXTS[0])) }
}

// jig-client-protocol-link/src/tests/fixtures/turns.rs pins these positions, timestamps and turn numbers.
fn first_turn(bytes: &[u8]) -> w::DecodedActivityLine {
    turn_line(10, 20, 1, bytes)
}
fn second_turn(bytes: &[u8]) -> w::DecodedActivityLine {
    turn_line(11, 21, 2, bytes)
}

#[must_use]
pub fn decoded_activity_record() -> w::ActivityPage {
    unread_activity_record(&full_turn())
}

#[must_use]
pub fn unread_activity_record(bytes: &[u8]) -> w::ActivityPage {
    activity_rows(&[first_turn(bytes), second_turn(&full_turn())])
}

#[must_use]
pub fn cut_activity_record() -> w::ActivityPage {
    activity_rows(&[first_turn(&full_turn())])
}

fn owner_turn_source() -> w::ActivityLine {
    w::ActivityLine {
        task: 7,
        position: 88,
        at: 66,
        line: w::Activity::Turn(w::ActivityTurn { reply: 999, attempt: 44, turn: 3, read: Some(77) }),
    }
}

#[must_use]
pub fn archived_activity_record() -> w::ActivityPage {
    let turn = w::TurnCarried { bytes: Box::from([0, 4]) };
    owner_turn_activity_record(Some(w::TurnBytes::Carried(turn)))
}

#[must_use]
pub fn owner_turn_activity_record(turn: Option<w::TurnBytes>) -> w::ActivityPage {
    let source = owner_turn_source();
    let mut decoded = None;
    if let Some(turn) = turn {
        decoded = Some(w::ArchivedTurn {
            task: 7,
            name: w::TurnName { attempt: 44, number: 3 },
            cumulative: 123,
            read: Some(78),
            at: 99,
            turn,
        });
    }
    let row = w::DecodedActivityLine { source, decoded };
    activity_rows(&[row])
}
