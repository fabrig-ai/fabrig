use super::*;
use skein_lib::List;

#[path = "reads_wire.rs"]
mod wire;
pub use wire::*;
#[path = "reads_values.rs"]
mod values;
pub use values::*;

pub fn raw_cursor(value: &[u8]) -> c::Cursor {
    c::Cursor::new(Box::from(value), 256).unwrap()
}
pub fn wire_cursor(value: &[u8]) -> w::Cursor {
    w::Cursor { value: Box::from(value) }
}
pub fn note_page_with_cursors() -> w::NotePage {
    let mut parts = read_wire_note_page();
    parts.before = Some(wire_cursor(UTF8_TEXTS[0]));
    parts.after = Some(wire_cursor(&[0; 48]));
    parts.cut = true;
    parts
}
pub fn expected_note_page_with_cursors() -> c::NotePage {
    c::NotePage {
        rows: List::with_capacity(0),
        before: Some(raw_cursor(UTF8_TEXTS[0])),
        after: Some(raw_cursor(&[0; 48])),
        cut: true,
    }
}
pub fn ended_task_record() -> w::Ended {
    let ending = w::Fact::Ending(w::EndingRow {
        parent: None,
        value: w::EndingFields::Failed(w::EndingFailedFields { reason: Some(Box::from(b"failed".as_slice())) }),
    });
    let mut result = read_wire_result();
    result.ending = w::Ending { rows: fact_bytes(&ending) };
    let result = result;
    let mut ended = read_wire_owner_ended();
    ended.at = Some(u64::MAX);
    ended.value = w::EndedValue::Task(w::EndedValueTask { document: task_record(), result });
    ended
}
pub fn expected_ended_task() -> c::Ended {
    let mut result = read_value_result();
    result.ending = c::Ending::Failed { reason: Some(text(b"failed")) };
    c::Ended {
        key: c::ObjectKey::Task(7),
        value: c::EndedValue::Task { document: Box::new(task()), result: Box::new(result) },
        at: Some(skein_lib::Wall::from_nanos(u64::MAX)),
        by: None,
    }
}

const UTF8_TEXTS: [&[u8]; 1] = [&[b'~'; 48]];

pub fn ended_delegate_record() -> w::EndedDelegate {
    let mut facts = List::with_capacity(1);
    facts
        .push(w::Fact::ChipEnding(w::ChipEndingRow {
            parent: None,
            value: w::ChipEndingFields::Failed(w::ChipEndingFailedFields {
                value: jig_client_test_records::wire_text(b"failed"),
            }),
        }))
        .unwrap();
    let rows = jig_client_test_records::chip_from_rows(&facts.into_boxed()).rows;
    w::EndedDelegate { chip: chip_record(), ended: w::ChipEnding { rows }, at: None }
}
pub fn ended_rows(count: u32) -> Box<[u8]> {
    let row = ended_delegate_record();
    let mut bytes = Writer::new(
        usize::try_from(row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    row.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
    let bytes = bytes.finish();
    let mut rows =
        Writer::new(bytes.len().checked_add(4).unwrap().checked_mul(usize::try_from(count).unwrap()).unwrap());
    for _ in 0..count {
        skein_codec::piece::write(&mut rows, &bytes).unwrap();
    }
    rows.finish()
}
pub fn trailing_ended_row() -> Box<[u8]> {
    let row = ended_delegate_record();
    let mut bytes = Writer::new(
        usize::try_from(row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .unwrap()
            .checked_add(1)
            .unwrap(),
    );
    row.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
    bytes.put(&[0]).unwrap();
    let bytes = bytes.finish();
    let mut rows = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut rows, &bytes).unwrap();
    rows.finish()
}
pub fn ended_page_piece(rows: Box<[u8]>) -> w::EndedPage {
    w::EndedPage { rows, before: None, after: None, cut: false }
}
pub fn expected_owner_ended_page(count: u32) -> c::EndedPage {
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(c::EndedDelegate { chip: chip(), ended: c::ChipEnding::Failed(document_text(b"failed")), at: None })
            .unwrap();
    }
    c::EndedPage { rows, before: None, after: None, cut: false }
}
