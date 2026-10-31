use super::*;
use skein_lib::List;

#[path = "reads_part0.rs"]
mod part0;
pub use part0::*;
#[path = "reads_part1.rs"]
mod part1;
pub use part1::*;
#[path = "reads_part2.rs"]
mod part2;
pub use part2::*;
#[path = "reads_part3.rs"]
mod part3;
pub use part3::*;
#[path = "reads_part4.rs"]
mod part4;
pub use part4::*;
#[path = "reads_part5.rs"]
mod part5;
pub use part5::*;
#[path = "reads_part6.rs"]
mod part6;
pub use part6::*;
#[path = "reads_part7.rs"]
mod part7;
pub use part7::*;

pub fn raw_cursor(value: &[u8]) -> c::Cursor {
    c::Cursor::new(Box::from(value), 256).unwrap()
}
pub fn wire_cursor(value: &[u8]) -> w::Cursor {
    w::Cursor::new(&w::CEILINGS, w::CursorParts { value: Box::from(value) }).unwrap()
}
pub fn note_page_with_cursors() -> w::NotePage {
    let mut parts = read_wire_note_page().into_parts();
    parts.before = Some(wire_cursor(UTF8_TEXTS[0]));
    parts.after = Some(wire_cursor(&[0; 48]));
    parts.cut = true;
    w::NotePage::new(&w::CEILINGS, parts).unwrap()
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
    let lim = w::CEILINGS;
    let ending = w::Fact::Ending(
        w::EndingRow::new(
            &lim,
            w::EndingRowParts {
                parent: None,
                value: w::EndingFields::Failed(
                    w::EndingFailedFields::new(
                        &lim,
                        w::EndingFailedFieldsParts { reason: Some(Box::from(b"failed".as_slice())) },
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap(),
    );
    let mut result = read_wire_result().into_parts();
    result.ending = w::Ending::new(&lim, w::EndingParts { rows: fact_bytes(&ending) }).unwrap();
    let result = w::ResultDocument::new(&lim, result).unwrap();
    let mut ended = read_wire_owner_ended().into_parts();
    ended.at = Some(u64::MAX);
    ended.value = w::EndedValue::Task(
        w::EndedValueTask::new(&lim, w::EndedValueTaskParts { document: task_record(), result }).unwrap(),
    );
    w::Ended::new(&lim, ended).unwrap()
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
    let lim = *limits().wire();
    let mut facts = List::with_capacity(1);
    facts
        .push(w::Fact::ChipEnding(
            w::ChipEndingRow::new(
                &lim,
                w::ChipEndingRowParts {
                    parent: None,
                    value: w::ChipEndingFields::Failed(
                        w::ChipEndingFailedFields::new(
                            &lim,
                            w::ChipEndingFailedFieldsParts { value: jig_client_test_records::wire_text(b"failed") },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ))
        .unwrap();
    let rows = jig_client_test_records::chip_from_rows(&facts.into_boxed()).into_parts().rows;
    w::EndedDelegate::new(
        &lim,
        w::EndedDelegateParts {
            chip: chip_record(),
            ended: w::ChipEnding::new(&lim, w::ChipEndingParts { rows }).unwrap(),
            at: None,
        },
    )
    .unwrap()
}
pub fn ended_rows(count: u32) -> Box<[u8]> {
    let row = ended_delegate_record();
    let mut bytes = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut bytes).unwrap();
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
    let mut bytes = Writer::new(usize::try_from(row.measure()).unwrap().checked_add(1).unwrap());
    row.encode(&mut bytes).unwrap();
    bytes.put(&[0]).unwrap();
    let bytes = bytes.finish();
    let mut rows = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut rows, &bytes).unwrap();
    rows.finish()
}
pub fn ended_page_piece(rows: Box<[u8]>) -> w::EndedPage {
    w::EndedPage::new(limits().wire(), w::EndedPageParts { rows, before: None, after: None, cut: false }).unwrap()
}
pub fn expected_owner_ended_page(count: u32) -> c::EndedPage {
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(c::EndedDelegate { chip: chip(), ended: c::ChipEnding::Failed(document_text(b"failed")), at: None })
            .unwrap();
    }
    c::EndedPage { rows, before: None, after: None, cut: false }
}
