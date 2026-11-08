//! Decisions retain their bounded wire facts (client/README.md, 4.6).
use super::fixtures as f;
use crate::forge::decided;
use jig_client::{Family, Watchable, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) fn decision(number: u32, at: u64) -> w::Decision {
    w::Decision::new(
        &w::CEILINGS,
        w::DecisionParts {
            number,
            task: u64::from(number),
            at,
            question: b"Which approach?".as_slice().into(),
            took: b"The smaller change".as_slice().into(),
            why: b"It fits the goal.".as_slice().into(),
            overturned: false,
            reach: w::Reach::new(
                &w::CEILINGS,
                w::ReachParts {
                    kind: w::ReachKind::Wording,
                    class: w::ReachClass::Decide,
                    reason: w::ReachReason::Kind,
                    under: 0,
                    sets: 0,
                    matched: 0,
                    away: false,
                },
            )
            .unwrap(),
        },
    )
    .unwrap()
}
pub(super) fn record(values: &[w::Decision]) -> Family {
    let mut rows = List::with_capacity(64);
    for value in values {
        rows.push(value.clone()).unwrap();
    }
    let document = w::Decisions::new(
        &w::CEILINGS,
        w::DecisionsParts { project: f::PROJECT, rows, asked: List::with_capacity(0), cut: false },
    )
    .unwrap();
    Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::decisions(&document) }
}
#[test]
fn a_document_decodes() {
    let value = decision(3, 100);
    let valid = record(core::slice::from_ref(&value));
    let document = decided::document(&valid, f::limits().forge).unwrap();
    assert_eq!(document.project(), f::PROJECT);
    assert_eq!(document.rows().as_slice(), &[value]);
    let valid_document = valid.clone();
    for invalid in [
        Family { family: wire::FORGE + 1, ..valid.clone() },
        Family { version: wire::CLIENT_VERSION + 1, ..valid.clone() },
        Family { bytes: Box::default(), ..valid.clone() },
        Family { bytes: skein_lib::bytes::concat(&[valid.bytes.as_ref(), &[0]]), ..valid },
    ] {
        assert_eq!(decided::document(&invalid, f::limits().forge), None);
    }
    let Watchable::Family { scope, window, record } = decided::watch(f::PROJECT, &f::limits().forge).unwrap() else {
        panic!("family watch")
    };
    assert_eq!(scope, c::Scope::Project(f::PROJECT));
    assert_eq!(window, c::Window { rows: 0, after: None });
    let w::Watch::Decided(subject) = wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().into_parts().watch
    else {
        panic!("decided watch")
    };
    assert_eq!(subject.project(), f::PROJECT);
    let mut limits = f::limits().forge;
    limits.field_bytes = u32::MAX;
    assert!(decided::watch(f::PROJECT, &limits).is_none());
    assert!(decided::document(&valid_document, limits).is_none());
}

#[test]
fn a_refused_or_ended_watch_is_no_rows() {
    use crate::{Address, Event, Place, Request};
    use jig_client::{Down, Up};
    use skein_lib::{Queue, Wall};
    for (delivered, end) in [(false, c::WatchEnd::NotAllowed), (true, c::WatchEnd::SubjectEnded)] {
        let limits = f::limits();
        let env = f::env(&limits);
        let mut root = crate::unlinked(&limits, f::SEED);
        let me = f::start(&mut root, List::with_capacity(0));
        f::confirm(&mut root, me, f::PARTY, &mut Queue::uncapped());
        let mut out = Queue::uncapped();
        crate::step(
            &mut root,
            &env,
            Event::Went { place: Place::Address(Address::Home { project: f::PROJECT }) },
            &mut out,
        );
        let subject = decided::watch(f::PROJECT, &limits.forge).unwrap();
        let mut watch = None;
        for request in &out {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch: token, subject: found } },
            } = request
                && found == &subject
            {
                watch = Some(*token);
            }
        }
        let watch = watch.unwrap();
        let record = record(&[decision(1, 100)]);
        if delivered {
            crate::step(
                &mut root,
                &env,
                Event::Client {
                    event: jig_client::Event::Up { up: Up::FamilyDelivered { watch, record: record.clone() } },
                },
                &mut Queue::uncapped(),
            );
            assert!(crate::pages::home::facts(&root, f::PROJECT, Wall::EPOCH).decided.is_some());
        }
        crate::step(
            &mut root,
            &env,
            Event::Client { event: jig_client::Event::Up { up: Up::WatchEnded { watch, end } } },
            &mut Queue::uncapped(),
        );
        assert!(crate::pages::home::facts(&root, f::PROJECT, Wall::EPOCH).decided.is_none());
        crate::step(
            &mut root,
            &env,
            Event::Client { event: jig_client::Event::Up { up: Up::FamilyDelivered { watch, record } } },
            &mut Queue::uncapped(),
        );
        assert!(
            crate::pages::home::facts(&root, f::PROJECT, Wall::EPOCH).decided.is_none(),
            "late delivery stays ended"
        );
    }
}
