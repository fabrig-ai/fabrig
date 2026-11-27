use super::fixtures as f;
use crate::forge::{self, deployment};
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn page(welcome: bool, owned: bool) -> Readout {
    Readout::Family {
        family: wire::FORGE,
        record: Some(wire::deployment_answer(&w::DeploymentAnswer { push: None, welcome, owned, program_login: None })),
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
    assert_eq!(&wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read, &w::Read::Deployment);
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
        let expected = Some(deployment::Answer { push: None, welcome, owned, program_login: None });
        tell(&mut side, Told::Link { link: Link::Offline }, &mut out);
        assert_eq!(side.deployment(), expected.as_ref());
        let next = read(&mut side);
        assert!(tell(
            &mut side,
            Told::Page { handle: next, page: Readout::Family { family: wire::FORGE, record: None } },
            &mut out
        ));
        assert_eq!(side.deployment(), expected.as_ref());
        assert_eq!(forge::shown(&side), shown, "an unchanged answer does not publish");
        tell(&mut side, Told::Link { link: Link::Offline }, &mut out);
        let next = read(&mut side);
        assert!(tell(&mut side, Told::Page { handle: next, page: page(!welcome, !owned) }, &mut out));
        assert_eq!(
            side.deployment(),
            Some(&deployment::Answer { push: None, welcome: !welcome, owned: !owned, program_login: None })
        );
        assert_ne!(forge::shown(&side), shown, "a changed answer publishes");
        assert!(out.is_empty(), "answers send no reads");
    }
}
#[test]
fn a_read_that_ends_with_no_record_reads_as_no_welcome() {
    for page in [
        Readout::Family { family: wire::FORGE, record: None },
        Readout::NotServed { family: Some(wire::FORGE) },
        Readout::Limit,
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
        Readout::Family { family: wire::FORGE, record: Some(Box::new([255])) },
        Readout::Family { family: 99, record: None },
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let handle = read(&mut side);
        let mut out = Queue::uncapped();
        assert!(tell(&mut side, Told::Page { handle, page }, &mut out));
        assert_eq!(
            side.deployment(),
            Some(&deployment::Answer { push: None, welcome: false, owned: false, program_login: None })
        );
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

fn link(root: &mut crate::Domain, link: Link) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Link { link }, &mut out);
    out
}

fn openings(out: &Queue<crate::Request>, expected: &jig_client::Watchable) -> List<Token> {
    let mut found = List::uncapped();
    for request in out {
        if let crate::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = request
            && subject == expected
        {
            found.push(*watch).unwrap();
        }
    }
    found
}

fn reads(out: &Queue<crate::Request>, expected: &w::Read) -> List<Token> {
    let mut found = List::uncapped();
    for request in out {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: Readable::Family { scope, record, .. } },
                },
        } = request
            && record.family == wire::FORGE
            && &wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read == expected
        {
            assert_eq!(*scope, c::Scope::Deployment);
            found.push(*read).unwrap();
        }
    }
    found
}

fn up(root: &mut crate::Domain, up: jig_client::Up) {
    crate::step(
        root,
        &f::env(&f::limits()),
        crate::Event::Client { event: jig_client::Event::Up { up } },
        &mut Queue::uncapped(),
    );
}

#[test]
fn partyless_watches_and_reads_leave_in_the_live_step() {
    use jig_client::{Down, UnlinkedRequest, Up, Watchable};
    for end in [Link::SignedOut, Link::OutOfDate] {
        let mut root = crate::unlinked(&f::limits(), f::SEED);
        f::start(&mut root, List::uncapped());
        crate::step(
            &mut root,
            &f::env(&f::limits()),
            crate::Event::Went { place: crate::Place::Address(crate::Address::Told) },
            &mut Queue::uncapped(),
        );
        let subjects = [
            forge::dismissed::watch(&f::limits().forge).unwrap(),
            forge::away::watch(&f::limits().forge).unwrap(),
            forge::trials::watch(&f::limits().forge).unwrap(),
            forge::approvals::watch(&f::limits().forge).unwrap(),
            forge::forges::watch(&f::limits().forge).unwrap(),
            Watchable::Jig(c::Subject::Settings),
            forge::devices::watch(f::limits().forge).unwrap(),
        ];
        let starting = link(&mut root, Link::Starting);
        for subject in &subjects {
            assert!(openings(&starting, subject).is_empty(), "starting: {subject:?}");
        }
        for read in [w::Read::Deployment, w::Read::Playground] {
            assert!(reads(&starting, &read).is_empty());
        }
        assert!(root.pages().me(root.client()).is_none());
        let live = link(&mut root, Link::Live);
        let mut watches = List::uncapped();
        for subject in &subjects {
            let opened = openings(&live, subject);
            assert_eq!(opened.len(), 1, "live without me: {subject:?}");
            let watch = *opened.get(0).unwrap();
            watches.push(watch).unwrap();
            up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
        }
        for read in [w::Read::Deployment, w::Read::Playground] {
            assert_eq!(reads(&live, &read).len(), 1, "live without me: {read:?}");
        }
        for (state, count) in [(Link::Live, 0), (Link::Offline, 0), (Link::Live, 1)] {
            let out = link(&mut root, state);
            for subject in &subjects {
                assert!(openings(&out, subject).is_empty(), "retained across {state:?}: {subject:?}");
            }
            for read in [w::Read::Deployment, w::Read::Playground] {
                assert_eq!(reads(&out, &read).len(), count, "reads at {state:?}: {read:?}");
            }
        }
        for watch in &watches {
            up(&mut root, Up::WatchEnded { watch: *watch, end: c::WatchEnd::Dropped });
        }
        link(&mut root, Link::Offline);
        link(&mut root, Link::Live);
        let mut env = f::env(&f::limits());
        env.now = skein_lib::Time::from_nanos(1_000_000_000);
        let mut retried = Queue::uncapped();
        crate::fire(&mut root, &env, &mut retried);
        watches.clear();
        for subject in &subjects {
            let opened = openings(&retried, subject);
            assert_eq!(opened.len(), 1, "dropped then live: {subject:?}");
            let watch = *opened.get(0).unwrap();
            watches.push(watch).unwrap();
            up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
        }
        let out = link(&mut root, end);
        for watch in &watches {
            let mut closed = false;
            for request in &out {
                if let crate::Request::Unlinked {
                    request: UnlinkedRequest::Down { down: Down::Close { watch: value } },
                } = request
                {
                    closed |= value == watch;
                }
            }
            assert!(closed, "{end:?} closes {watch:?}");
        }
        for subject in &subjects {
            assert!(openings(&out, subject).is_empty(), "nothing opens at {end:?}");
        }
    }
}

#[test]
fn the_deployment_command_survives_a_gap_and_changes_with_the_answer() {
    let mut side = forge::Side::new(&f::limits().forge, f::SEED);
    let mut out = Queue::uncapped();
    for line in [Some(b"'/box/temper' program-login".as_slice()), None] {
        let handle = read(&mut side);
        let answer = w::DeploymentAnswer {
            push: None,
            welcome: true,
            owned: false,
            program_login: match line {
                Some(line) => Some(line.into()),
                None => None,
            },
        };
        let before = forge::shown(&side);
        tell(
            &mut side,
            Told::Page {
                handle,
                page: Readout::Family { family: wire::FORGE, record: Some(wire::deployment_answer(&answer)) },
            },
            &mut out,
        );
        assert_eq!(side.deployment().unwrap().program_login.as_deref(), line);
        assert_ne!(forge::shown(&side), before);
        tell(&mut side, Told::Link { link: Link::Offline }, &mut out);
        let handle = read(&mut side);
        tell(&mut side, Told::Page { handle, page: Readout::Limit }, &mut out);
        assert_eq!(side.deployment().unwrap().program_login.as_deref(), line);
        tell(&mut side, Told::Link { link: Link::Offline }, &mut out);
    }
}
