//! The client carries waits whole and never interprets the continuation.
use super::fixtures;
use crate::translate::reads;
use alloc::boxed::Box;
use jig_client_domain::{Readable, contract as c};
use jig_client_protocol_wire::{self as w, Record as _};
use skein_lib::{List, Reader, Wall};

#[test]
fn waits_read_filters_and_opaque_cursor_round_trip() {
    let limits = fixtures::limits();
    for (kind, wire_kind, party, due, cursor) in [
        (None, None, None, None, None),
        (Some(c::WaitKind::SignOff), Some(w::WaitKind::SignOff), Some(17), Some(19), Some([0, 255, 1])),
    ] {
        let after = match cursor {
            Some(bytes) => Some(c::Cursor::new(Box::new(bytes), 3).expect("bounded cursor")),
            None => None,
        };
        let read = c::Read::Waits { project: 11, kind, party, due, after };
        let owned = reads::owned(Readable::Jig(read), limits.wire()).expect("bounded waits request");
        assert_eq!(owned.owner, w::Owner::Jig);
        assert_eq!(owned.version, w::VERSION);
        let wire = w::Read::decode(limits.wire(), &mut Reader::new(&owned.record)).unwrap();
        assert_eq!(
            wire,
            w::Read::Waits(w::ReadWaits {
                project: 11,
                kind: wire_kind,
                party,
                due,
                after: match cursor {
                    Some(value) => Some(w::Cursor { value: Box::new(value) }),
                    None => None,
                },
            })
        );
    }
}

#[test]
fn a_full_waits_page_carries_its_wall_rows_and_cursor() {
    let base = fixtures::limits();
    let count = w::CEILINGS.wait_page_rows;
    let limits =
        crate::Limits::new(jig_client_domain::Limits { rows: count, ..*base.client() }, base.exchanges()).unwrap();
    let mut rows = List::with_capacity(count);
    let mut expected = List::with_capacity(count);
    for i in 0..count {
        let number = u64::from(i) + 11;
        let wait = w::Wait {
            task: 7,
            kind: w::WaitKind::Dependency,
            object: w::WaitObject::Task(w::WaitTask { value: number }),
            since: 3,
            bound: w::WaitBound::At(w::WaitAtBound { at: 17, then: w::AtBound::Hold }),
            hears: Some(w::EscalationHolder::Person(w::EscalationHolderPerson { value: 2 })),
        };
        rows.push(w::WaitRow { chip: fixtures::chip_record(), wait }).unwrap();
        expected
            .push(c::WaitRow {
                chip: fixtures::chip(),
                wait: c::Wait {
                    task: 7,
                    kind: c::WaitKind::Dependency,
                    object: c::WaitObject::Task(number),
                    since: Wall::from_nanos(3),
                    bound: c::waits::Bound::At { at: Wall::from_nanos(17), then: c::AtBound::Hold },
                    hears: Some(c::EscalationHolder::Person(2)),
                },
            })
            .unwrap();
    }
    let after = [9, 0, 255];
    let wire = w::ReadValue::Waits(w::WaitPage { now: 23, rows, after: Some(w::Cursor { value: Box::new(after) }) });
    let bytes = wire.encode(limits.wire()).expect("all rows fit their wire bound");
    let value = reads::value(&bytes, &limits).expect("full page decodes");
    assert_eq!(
        value,
        c::ReadValue::Waits(Box::new(c::WaitPage {
            now: Wall::from_nanos(23),
            rows: expected,
            after: Some(c::Cursor::new(Box::new(after), 3).unwrap()),
        }))
    );
}
