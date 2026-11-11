//! The session's approvals watch (temper/docs/design/client/pages/README.md, temper/docs/design/client/forge/records.md).
use super::fixtures as f;
use crate::{Domain, Event, Request, forge::approvals};
use jig_client::{Down, Family, Link, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token, Wall, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) const CHANGE: u64 = 81;
pub(super) const OTHER_CHANGE: u64 = 82;
pub(super) const GOAL: u64 = 9;
// The fixture has one approval row and two rows past the bound.
const ONE: u32 = 1;
const MORE: u32 = 2;
pub(super) fn row(task: u64, project: u32, goal: Option<u64>) -> w::ApprovalRow {
    w::ApprovalRow::new(
        &w::CEILINGS,
        w::ApprovalRowParts {
            task,
            project,
            goal,
            forge: 3,
            kind: w::Kind::Github,
            owner: Box::from(*b"acme"),
            name: Box::from(*b"widget"),
            pull: 24,
            address: Box::from(*b"https://github.com/acme/widget/pull/24/files"),
            title: Box::from(*b"A change"),
            since: 1_000_000_000,
            scope: project,
            rule: 4,
            role: 5,
            required: 2,
            counted: 1,
            sets: List::from_boxed(Box::new([w::ApprovalSet::new(
                &w::CEILINGS,
                w::ApprovalSetParts { number: 1, name: b"vsr's protocol core".as_slice().into() },
            )
            .unwrap()])),
            whole: true,
            yours: w::ApprovalYours::Counts,
        },
    )
    .unwrap()
}
pub(super) fn opening(out: &Queue<Request>) -> Option<Token> {
    let expected = approvals::watch(&f::limits().forge).unwrap();
    let mut found = None;
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && *subject == expected
        {
            assert!(found.is_none(), "one approvals watch");
            found = Some(*watch);
        }
    }
    found
}
fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
pub(super) fn deliver(root: &mut Domain, watch: Token, rows: &[w::ApprovalRow], more: u32) -> Queue<Request> {
    let document =
        w::Approvals::new(&w::CEILINGS, w::ApprovalsParts { rows: List::from_boxed(rows.into()), more }).unwrap();
    bytes(root, watch, wire::approvals(&document))
}
fn bytes(root: &mut Domain, watch: Token, bytes: Box<[u8]>) -> Queue<Request> {
    up(
        root,
        Up::FamilyDelivered { watch, record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes } },
    )
}
fn link(root: &mut Domain, link: Link) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Link { link }, &mut out);
    out
}
fn closes(out: &Queue<Request>, expected: Token) -> u32 {
    let mut count = 0_u32;
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
            && *watch == expected
        {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
#[test]
fn approvals_are_watched_once_for_the_session_and_end_with_it() {
    for (ending, expected_closes) in
        [(None, ONE), (Some(c::WatchEnd::NotAllowed), 0), (Some(c::WatchEnd::SubjectEnded), 0)]
    {
        let mut root = crate::unlinked(&f::limits(), f::SEED);
        let me = f::start(&mut root, List::with_capacity(0));
        assert!(root.forge().approvals().rows().is_empty());
        assert_eq!(root.forge().approvals().more(), 0);
        let mut out = Queue::uncapped();
        f::confirm(&mut root, me, f::PARTY, &mut out);
        let watch = opening(&out).expect("approvals watch after me");
        let Watchable::Family { scope, window, record } = approvals::watch(&f::limits().forge).unwrap() else {
            panic!("family watch")
        };
        assert_eq!(scope, c::Scope::Own);
        assert_eq!(window, c::Window { rows: 0, after: None });
        assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
        assert_eq!(*wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch(), w::Watch::Approvals);
        deliver(&mut root, watch, &[row(CHANGE, f::PROJECT, Some(GOAL))], MORE);
        assert_eq!(
            root.forge().approvals().rows(),
            [approvals::Row {
                task: CHANGE,
                project: f::PROJECT,
                goal: Some(GOAL),
                forge: 3,
                kind: crate::forge::forges::Kind::GitHub,
                owner: Box::from(*b"acme"),
                name: Box::from(*b"widget"),
                pull: 24,
                address: Box::from(*b"https://github.com/acme/widget/pull/24/files"),
                title: Box::from(*b"A change"),
                since: Wall::from_nanos(1_000_000_000),
                scope: f::PROJECT,
                rule: 4,
                role: 5,
                required: 2,
                counted: 1,
                sets: Box::new([approvals::ApprovalSet { number: 1, name: b"vsr's protocol core".as_slice().into() }]),
                whole: true,
                yours: approvals::Yours::Counts,
            }]
        );
        assert_eq!(root.forge().approvals().more(), MORE);
        let document =
            w::Approvals::new(&w::CEILINGS, w::ApprovalsParts { rows: List::with_capacity(0), more: 0 }).unwrap();
        let bytes = wire::approvals(&document);
        let mut trailing = Writer::new(bytes.len().checked_add(1).unwrap());
        trailing.put(&bytes).unwrap();
        trailing.put(&[0]).unwrap();
        let out = match ending {
            None => self::bytes(&mut root, watch, trailing.finish()),
            Some(end) => up(&mut root, Up::WatchEnded { watch, end }),
        };
        assert_eq!(closes(&out, watch), expected_closes);
        assert!(root.forge().approvals().rows().is_empty());
        assert_eq!(root.forge().approvals().more(), 0);
        let mut out = Queue::uncapped();
        f::confirm(&mut root, me, f::PARTY, &mut out);
        assert_eq!(opening(&out), None);
        assert_eq!(opening(&link(&mut root, Link::Offline)), None);
        assert_eq!(opening(&link(&mut root, Link::Live)), None);
        deliver(&mut root, watch, &[row(CHANGE, f::PROJECT, None)], 0);
        assert!(root.forge().approvals().rows().is_empty());
        up(&mut root, Up::SignedOut);
        let signed_in = up(&mut root, Up::SignedIn);
        let mut me = None;
        for request in &signed_in {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) },
                    },
            } = request
            {
                me = Some(*watch);
            }
        }
        let me = me.expect("next sign-in opens me");
        let mut out = Queue::uncapped();
        f::confirm(&mut root, me, f::PARTY, &mut out);
        let watch = opening(&out).expect("next sign-in opens approvals");
        deliver(&mut root, watch, &[row(CHANGE, f::PROJECT, None)], 0);
        assert_eq!(closes(&link(&mut root, Link::SignedOut), watch), ONE);
        assert!(root.forge().approvals().rows().is_empty());
    }
}
