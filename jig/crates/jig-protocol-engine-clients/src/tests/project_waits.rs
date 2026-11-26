//! Waits read exchanges pair the original query with its opaque continuation.
use super::fixtures;
use crate::translate::{cursor, reads as translate};
use jig_client_domain::contract as c;
use jig_client_protocol_wire::{self as w, Record as _};
use jig_domain_core_vocabulary::{reads, waits};
use skein_lib::{List, Reader, Wall};

const NOW: Wall = Wall::from_nanos(83);
const QUERY: reads::WaitQuery = reads::WaitQuery { project: 7, kind: None, party: None, due: None };
const FILTERED: reads::WaitQuery =
    reads::WaitQuery { kind: Some(waits::WaitKind::Dependency), party: Some(3), due: Some(17), ..QUERY };

fn after(query: reads::WaitQuery, position: reads::WaitPosition) -> reads::Cursor {
    reads::Cursor::Waits { query, now: NOW, after: position }
}

fn row(order: u32) -> reads::WaitRow {
    let chip = fixtures::chip();
    let wait = waits::Wait {
        task: chip.number,
        kind: waits::WaitKind::Dependency,
        object: waits::WaitObject::Task(u64::from(order) + 11),
        since: Wall::from_nanos(3),
        bound: waits::Bound::At { at: Wall::from_nanos(97), then: waits::AtBound::Hold },
        hears: Some(jig_domain_tasks::EscalationHolder::Person(3)),
    };
    reads::WaitRow { chip, position: reads::WaitPosition::of(&wait, NOW, order), wait }
}

fn wire_after(cursor: &reads::Cursor) -> w::Cursor {
    w::Cursor { value: cursor::encode(cursor, w::CEILINGS.cursor_value).unwrap() }
}

#[test]
fn waits_read_filters_and_continuation_round_trip_to_the_core() {
    let limits = fixtures::wire_limits();
    for (kind, party, due, query, after) in [
        (None, None, None, QUERY, None),
        (Some(w::WaitKind::Dependency), Some(3), Some(17), FILTERED, Some(after(FILTERED, row(1).position))),
    ] {
        let read = w::Read::Waits(w::ReadWaits {
            project: 7,
            kind,
            party,
            due,
            after: match &after {
                Some(cursor) => Some(wire_after(cursor)),
                None => None,
            },
        });
        assert_eq!(translate::decode(&fixtures::encoded_read(&read), &limits), Ok(reads::Read::Waits { query, after }));
    }
    let malformed = w::Read::Waits(w::ReadWaits {
        project: 7,
        kind: None,
        party: None,
        due: None,
        after: Some(w::Cursor { value: Box::new([0]) }),
    });
    assert_eq!(translate::decode(&fixtures::encoded_read(&malformed), &limits), Err(reads::Why::Malformed));
}

#[test]
fn a_full_waits_page_round_trips_through_the_read_exchange() {
    let mut exchange = fixtures::read_exchange();
    let count = w::CEILINGS.wait_page_rows;
    exchange.env.limits.wire.owned_record = 65_536;
    let limits = exchange.env.limits.wire;
    let read = w::Read::Waits(w::ReadWaits { project: 7, kind: None, party: None, due: None, after: None });
    let (owner, reply, core_read) = exchange.read(&read);
    assert_eq!(core_read, reads::Read::Waits { query: QUERY, after: None });
    assert!(exchange.read_unanswered(owner));
    let mut rows = List::with_capacity(count);
    for order in 0..count {
        rows.push(row(order)).unwrap();
    }
    let next = after(QUERY, rows.last().unwrap().position);
    let source = reads::WaitPage { now: NOW, rows, after: Some(next.clone()) };
    exchange.release_read(reply, reads::Answer::Waits(Box::new(source.clone())));
    let w::ReadAnswer::Value(answer) = exchange.read_answer(owner) else { panic!("bounded waits page") };
    let w::ReadValue::Waits(page) = w::ReadValue::decode(&limits, &mut Reader::new(&answer.record)).unwrap() else {
        panic!("waits page")
    };
    assert_eq!(page.after, Some(wire_after(&next)));
    assert_eq!(cursor::decode(&page.after.unwrap().value), Some(next));
    let base = fixtures::client_limits();
    let client =
        jig_client_protocol_link::Limits::new(jig_client_domain::Limits { rows: count, ..*base.client() }, 4).unwrap();
    let c::ReadValue::Waits(client_page) =
        jig_client_protocol_link::translate::reads::value(&answer.record, &client).unwrap()
    else {
        panic!("client waits page")
    };
    assert_eq!((client_page.now, client_page.rows.len()), (source.now, count));
    for (decoded, row) in client_page.rows.iter().zip(&source.rows) {
        assert_eq!(decoded.chip.task, row.chip.number);
        assert_eq!(
            decoded.wait,
            c::Wait {
                task: row.wait.task,
                kind: c::WaitKind::Dependency,
                object: c::WaitObject::Task(u64::from(row.position.order) + 11),
                since: row.wait.since,
                bound: c::waits::Bound::At { at: Wall::from_nanos(97), then: c::AtBound::Hold },
                hears: Some(c::EscalationHolder::Person(3)),
            }
        );
    }
}

#[test]
fn a_wire_byte_cut_resumes_after_the_last_row_sent() {
    let limits = fixtures::wire_limits();
    let first = row(0);
    let after = after(FILTERED, first.position);
    let one =
        reads::WaitPage { now: NOW, rows: List::from_boxed(Box::new([first.clone()])), after: Some(after.clone()) };
    let w::ReadValue::Waits(one) = wire_page(one, &limits) else { panic!("waits page") };
    let bytes = w::ReadValue::Waits(one.clone()).measure(&limits).unwrap();
    let limits = w::Limits { owned_record: bytes, ..limits };
    let both = reads::WaitPage { now: NOW, rows: List::from_boxed(Box::new([first, row(1)])), after: None };
    let wire = wire_page(both, &limits);
    assert_eq!(wire, w::ReadValue::Waits(one));
    assert_eq!(wire.measure(&limits).unwrap(), bytes);
    let next = w::Read::Waits(w::ReadWaits {
        project: 7,
        kind: Some(w::WaitKind::Dependency),
        party: Some(3),
        due: Some(17),
        after: Some(wire_after(&after)),
    });
    assert_eq!(
        translate::decode(&fixtures::encoded_read(&next), &limits),
        Ok(reads::Read::Waits { query: FILTERED, after: Some(after) })
    );
}

fn wire_page(page: reads::WaitPage, limits: &w::Limits) -> w::ReadValue {
    let w::ReadAnswer::Value(answer) = translate::answer(
        &reads::Read::Waits { query: FILTERED, after: None },
        &reads::Answer::Waits(Box::new(page)),
        &fixtures::models(),
        &crate::Private::none(),
        &fixtures::settings(),
        limits,
    )
    .expect("bounded answer") else {
        panic!("a waits value")
    };
    w::ReadValue::decode(limits, &mut Reader::new(&answer.record)).unwrap()
}
