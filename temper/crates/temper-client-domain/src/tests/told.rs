//! Device watch lifecycle through the root (client/domain-crates.md, 4.1).
#![expect(clippy::disallowed_methods, reason = "ordinary page tests inspect requests and table values")]
use super::fixtures as f;
use crate::forge::devices;
use jig_client::{Down, Family, UnlinkedRequest, Up};
use skein_lib::{List, Queue, Token, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};
const ENDPOINT: &[u8] = b"https://fcm.googleapis.com/subscription/private";
fn row() -> devices::Row {
    devices::Row {
        number: 7,
        label: b"Safari on iPhone".as_slice().into(),
        at: Wall::from_nanos(10),
        endpoint: ENDPOINT.into(),
    }
}
fn event(root: &mut crate::Domain, event: crate::Event) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut crate::Domain, up: Up) -> Queue<crate::Request> {
    event(root, crate::Event::Client { event: jig_client::Event::Up { up } })
}
#[test]
fn devices_are_own_scope_watched_only_while_the_page_is_open() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::uncapped());
    f::confirm(&mut root, me, f::PARTY, &mut Queue::uncapped());
    let open = crate::Event::Went { place: crate::Place::Address(crate::Address::Told) };
    let out = event(&mut root, open);
    let watch: Token = out
        .iter()
        .find_map(|request| {
            if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
                request
                && *subject == devices::watch(f::limits().forge).unwrap()
            {
                Some(*watch)
            } else {
                None
            }
        })
        .expect("devices watch");
    up(&mut root, Up::Watched { watch, event: jig_client::contract::WatchEvent::Opened });
    let mut rows = List::with_capacity(w::CEILINGS.devices_rows);
    rows.push(
        w::DeviceRow::new(
            &w::CEILINGS,
            w::DeviceRowParts {
                number: 7,
                label: b"Safari on iPhone".as_slice().into(),
                at: 10,
                endpoint: ENDPOINT.into(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    let document = w::Devices::new(&w::CEILINGS, w::DevicesParts { rows }).unwrap();
    up(
        &mut root,
        Up::FamilyDelivered {
            watch,
            record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::devices(&document) },
        },
    );
    assert_eq!(root.pages().told().unwrap().rows(), Some([row()].as_slice()));
    let same = event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Told) });
    assert!(!same.iter().any(|request| {
        if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { subject, .. } } } =
            request
        {
            *subject == devices::watch(f::limits().forge).unwrap()
        } else {
            false
        }
    }));
    let out = event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Accounts) });
    assert!(out.iter().any(|request| {
        if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Close { watch: closed } } } =
            request
        {
            *closed == watch
        } else {
            false
        }
    }));
    assert!(root.pages().told().is_none());
    event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Told) });
    let left = event(&mut root, crate::Event::Went { place: crate::Place::Missing });
    assert!(root.pages().told().is_none());
    assert!(!left.iter().any(|request| {
        if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { subject, .. } } } =
            request
        {
            *subject == devices::watch(f::limits().forge).unwrap()
        } else {
            false
        }
    }));
    assert_eq!(&*crate::address::print(&crate::Address::Told), b"/told");
    assert_eq!(crate::address::parse(b"/told"), crate::Place::Address(crate::Address::Told));
}
