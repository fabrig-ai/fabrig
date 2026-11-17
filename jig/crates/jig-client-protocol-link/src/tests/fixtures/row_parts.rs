//! Task-row parts at the wire boundary (client/domain/contract/parts.md, 3).
use super::*;
use skein_lib::List;

pub fn part_query() -> c::TaskQuery {
    let mut parts = List::with_capacity(4);
    for kind in 1..=4 {
        parts.push(c::PartAsk { owner: 19, kind, version: 23 }).unwrap();
    }
    c::TaskQuery {
        project: Some(7),
        requester: None,
        root: None,
        parent: None,
        label: None,
        phase: None,
        executor: None,
        tracked: None,
        order: c::QueryOrder::Number,
        parts,
    }
}

pub fn row_parts() -> List<w::RowPart> {
    let lim = *limits().wire();
    let owner = w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: 19 }).unwrap());
    List::from_boxed(Box::new([
        w::RowPart::Value(
            w::Owned::new(&lim, w::OwnedParts { owner: owner.clone(), version: 23, record: Box::from([255, 0, 1]) })
                .unwrap(),
        ),
        w::RowPart::Unknown,
        w::RowPart::Failed(w::PartFailed::new(&lim, w::PartFailedParts { why: w::PartFailure::Unreadable }).unwrap()),
        w::RowPart::NotServed(
            w::NotServed::new(
                &lim,
                w::NotServedParts {
                    owner: owner.clone(),
                    served: List::from_boxed(Box::new([w::Served::new(
                        &lim,
                        w::ServedParts { owner, first: 24, last: 25 },
                    )
                    .unwrap()])),
                },
            )
            .unwrap(),
        ),
    ]))
}

pub fn expected_row_parts() -> List<c::RowPart> {
    List::from_boxed(Box::new([
        c::RowPart::Value(Family { family: 19, version: 23, bytes: Box::from([255, 0, 1]) }),
        c::RowPart::Unknown,
        c::RowPart::Failed(c::PartFailure::Unreadable),
        c::RowPart::NotServed(c::NotServed {
            owner: c::Owner::Family(19),
            served: List::from_boxed(Box::new([c::Served { owner: c::Owner::Family(19), first: 24, last: 25 }])),
        }),
    ]))
}

pub fn part_rows(parts: List<w::RowPart>) -> Box<[u8]> {
    let lim = *limits().wire();
    let mut row = jig_client_test_records::task_line_record().into_parts();
    row.parts = parts;
    let row = w::TaskLine::new(&lim, row).unwrap();
    let mut bytes = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut bytes).unwrap();
    let bytes = bytes.finish();
    let mut framed = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut framed, &bytes).unwrap();
    framed.finish()
}

pub fn part_page(parts: List<w::RowPart>) -> w::TaskPage {
    let lim = *limits().wire();
    let rows = part_rows(parts);
    w::TaskPage::new(
        &lim,
        w::TaskPageParts {
            live: rows.clone(),
            live_after: None,
            ended_mark: 61,
            ended: w::EndedGroup::Rows(
                w::EndedRows::new(&lim, w::EndedRowsParts { rows, after: None, cut: false }).unwrap(),
            ),
        },
    )
    .unwrap()
}

pub fn part_delivery(name: u64, parts: List<w::RowPart>) -> Box<[u8]> {
    let lim = *limits().wire();
    let tasks = w::Tasks::new(&lim, w::TasksParts { rows: part_rows(parts), more: false, ended_mark: 61 }).unwrap();
    let value = w::DeliveryValue::Tasks(tasks);
    let mut bytes = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut bytes).unwrap();
    let document =
        w::Owned::new(&lim, w::OwnedParts { owner: w::Owner::Jig, version: w::VERSION, record: bytes.finish() })
            .unwrap();
    event_bytes(&w::StreamEvent::Delivery(w::Delivery::new(&lim, w::DeliveryParts { watch: name, document }).unwrap()))
}

pub fn malformed_row_parts() -> [(List<w::RowPart>, u32); 5] {
    let lim = *limits().wire();
    let mut fewer = row_parts();
    fewer.truncate(3);
    let mut another_owner = row_parts();
    let mut another_version = row_parts();
    let mut jig_owner = row_parts();
    for (parts, owner, version) in [
        (&mut another_owner, w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: 20 }).unwrap()), 23),
        (&mut another_version, w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: 19 }).unwrap()), 24),
        (&mut jig_owner, w::Owner::Jig, 23),
    ] {
        *parts.get_mut(0).unwrap() = w::RowPart::Value(
            w::Owned::new(&lim, w::OwnedParts { owner, version, record: Box::from([255, 0, 1]) }).unwrap(),
        );
    }
    [(fewer, 4), (row_parts(), 3), (another_owner, 4), (another_version, 4), (jig_owner, 4)]
}
