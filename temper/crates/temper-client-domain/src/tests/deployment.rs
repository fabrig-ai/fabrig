use super::fixtures as f;
use crate::forge::{self, deployment};
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn page(welcome: bool, owned: bool) -> Readout {
    Readout::Family {
        family: wire::FORGE,
        record: Some(wire::deployment_answer(
            &w::DeploymentAnswer::new(&w::CEILINGS, w::DeploymentAnswerParts { welcome, owned }).unwrap(),
        )),
    }
}
fn tell(side: &mut forge::Side, told: Told, out: &mut Queue<forge::Request>) -> bool {
    deployment::told(side, &told, f::limits().forge, out)
}
fn read(side: &mut forge::Side) -> Handle {
    let mut out = Queue::uncapped();
    assert!(!tell(side, Told::Link { link: Link::Live }, &mut out));
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, read } }) = out.pop() else {
        panic!("deployment read")
    };
    let Readable::Family { scope, window, record } = read else { panic!("family read") };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read(), &w::Read::Deployment);
    assert_ne!(handle.number, 0);
    assert!(out.is_empty());
    handle
}
#[test]
fn the_read_goes_out_once_a_link_is_live() {
    let mut side = forge::Side::new(&f::limits().forge, f::SEED);
    let mut out = Queue::uncapped();
    for link in [Link::Starting, Link::Offline, Link::SignedOut] {
        assert!(!tell(&mut side, Told::Link { link }, &mut out));
        assert!(out.is_empty());
    }
    let mut first = read(&mut side);
    assert_eq!(side.deployment(), None);
    for gap in [Link::Offline, Link::SignedOut, Link::Starting] {
        assert!(!tell(&mut side, Told::Link { link: Link::Live }, &mut out));
        assert!(out.is_empty(), "no second read while live");
        assert!(!tell(&mut side, Told::Link { link: gap }, &mut out));
        assert!(out.is_empty(), "no read during a gap");
        let next = read(&mut side);
        assert_ne!(first, next);
        assert!(!tell(&mut side, Told::Page { handle: first, page: page(false, false) }, &mut out));
        assert_eq!(side.deployment(), None, "a replaced read changes nothing");
        first = next;
    }
    assert!(tell(&mut side, Told::Page { handle: first, page: page(true, true) }, &mut out));
    let shown = forge::shown(&side);
    assert!(!tell(&mut side, Told::Page { handle: first, page: page(false, false) }, &mut out));
    assert_eq!(forge::shown(&side), shown, "a duplicate page changes nothing");
    assert!(!tell(&mut side, Told::Link { link: Link::Live }, &mut out));
    assert!(out.is_empty(), "a completed read does not poll");
}
#[test]
fn the_answer_is_kept_across_a_gap_and_a_read_with_no_record() {
    for (welcome, owned) in [(true, true), (false, true), (true, false), (false, false)] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let first = read(&mut side);
        let mut out = Queue::uncapped();
        let before = forge::shown(&side);
        assert!(tell(&mut side, Told::Page { handle: first, page: page(welcome, owned) }, &mut out));
        assert_ne!(forge::shown(&side), before, "a new answer publishes");
        let shown = forge::shown(&side);
        let expected = Some(deployment::Answer { welcome, owned });
        tell(&mut side, Told::Link { link: Link::Offline }, &mut out);
        assert_eq!(side.deployment(), expected);
        let next = read(&mut side);
        assert!(tell(
            &mut side,
            Told::Page { handle: next, page: Readout::Family { family: wire::FORGE, record: None } },
            &mut out
        ));
        assert_eq!(side.deployment(), expected);
        assert_eq!(forge::shown(&side), shown, "an unchanged answer does not publish");
        tell(&mut side, Told::Link { link: Link::Offline }, &mut out);
        let next = read(&mut side);
        assert!(tell(&mut side, Told::Page { handle: next, page: page(!welcome, !owned) }, &mut out));
        assert_eq!(side.deployment(), Some(deployment::Answer { welcome: !welcome, owned: !owned }));
        assert_ne!(forge::shown(&side), shown, "a changed answer publishes");
        assert!(out.is_empty(), "answers send no reads");
    }
}
#[test]
fn a_read_that_ends_with_no_record_reads_as_no_welcome() {
    for page in [
        Readout::Family { family: wire::FORGE, record: None },
        Readout::NotServed { family: Some(wire::FORGE), served: None },
        Readout::Limit,
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
        Readout::Family { family: wire::FORGE, record: Some(Box::new([255])) },
        Readout::Family { family: 99, record: None },
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let handle = read(&mut side);
        let mut out = Queue::uncapped();
        assert!(tell(&mut side, Told::Page { handle, page }, &mut out));
        assert_eq!(side.deployment(), Some(deployment::Answer { welcome: false, owned: false }));
        assert!(out.is_empty());
    }
}

pub(super) fn deliver_deployment(root: &mut crate::Domain, read: Token, welcome: bool, owned: bool) {
    crate::step(
        root,
        &f::env(&f::limits()),
        crate::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::ReadDone { read, result: page(welcome, owned) } },
        },
        &mut Queue::uncapped(),
    );
}
pub(super) fn answer(root: &mut crate::Domain, welcome: bool, owned: bool) {
    let mut child = Queue::uncapped();
    tell(&mut root.forge, Told::Link { link: Link::Offline }, &mut child);
    tell(&mut root.forge, Told::Link { link: Link::Live }, &mut child);
    let Some(forge::Request::Client { asked }) = child.pop() else { panic!("deployment read") };
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), crate::Event::Client { event: asked }, &mut out);
    while let Some(request) = out.pop() {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: jig_client::Down::Read { read, what: Readable::Family { .. } } },
        } = request
        {
            deliver_deployment(root, read, welcome, owned);
            return;
        }
    }
    panic!("deployment read reaches the world");
}
