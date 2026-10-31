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
pub fn historical_task_record() -> w::OwnerHistorical {
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
    let mut historical = read_wire_owner_historical().into_parts();
    historical.at = Some(u64::MAX);
    historical.value = w::OwnerHistoricalValue::Task(
        w::OwnerHistoricalValueTask::new(&lim, w::OwnerHistoricalValueTaskParts { document: task_record(), result })
            .unwrap(),
    );
    w::OwnerHistorical::new(&lim, historical).unwrap()
}
pub fn expected_historical_task() -> c::OwnerHistorical {
    let mut result = read_value_result();
    result.ending = c::Ending::Failed { reason: Some(text(b"failed")) };
    c::OwnerHistorical {
        key: c::ObjectKey::Task(7),
        value: c::OwnerHistoricalValue::Task { document: Box::new(task()), result: Box::new(result) },
        at: Some(skein_lib::Wall::from_nanos(u64::MAX)),
        by: None,
    }
}

const UTF8_TEXTS: [&[u8]; 1] = [&[b'~'; 48]];

pub fn ended_delegate_record() -> w::OwnerEndedDelegate {
    let lim = limits().wire;
    let mut facts = List::with_capacity(1);
    facts
        .push(w::Fact::OwnerChipEnding(
            w::OwnerChipEndingRow::new(
                &lim,
                w::OwnerChipEndingRowParts {
                    parent: None,
                    value: w::OwnerChipEndingFields::Failed(
                        w::OwnerChipEndingFailedFields::new(
                            &lim,
                            w::OwnerChipEndingFailedFieldsParts {
                                value: jig_client_test_records::wire_text(b"failed"),
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ))
        .unwrap();
    let rows = jig_client_test_records::chip_from_rows(&facts.into_boxed()).into_parts().rows;
    w::OwnerEndedDelegate::new(
        &lim,
        w::OwnerEndedDelegateParts {
            chip: chip_record(),
            ended: w::OwnerChipEnding::new(&lim, w::OwnerChipEndingParts { rows }).unwrap(),
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
pub fn owner_ended_page_piece(rows: Box<[u8]>) -> w::OwnerEndedPage {
    w::OwnerEndedPage::new(&limits().wire, w::OwnerEndedPageParts { rows, before: None, after: None, cut: false })
        .unwrap()
}
pub fn expected_owner_ended_page(count: u32) -> c::OwnerEndedPage {
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(c::OwnerEndedDelegate {
            chip: chip(),
            ended: c::OwnerChipEnding::Failed(document_text(b"failed")),
            at: None,
        })
        .unwrap();
    }
    c::OwnerEndedPage { rows, before: None, after: None, cut: false }
}
