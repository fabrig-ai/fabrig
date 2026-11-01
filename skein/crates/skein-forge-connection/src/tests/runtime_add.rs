//! Runtime admission and starts (forge-connection.md, 3; playground-reset draft, 3).
use super::fixtures::{self, FORGE, Harness};
use crate::{
    Added, Component, Limits, Request, Settings, Shared, Unusable,
    settings::{Forge, Kind, LogOrigin, Transport},
    start::Start,
};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Duration, List, Secret};

fn empty() -> Settings {
    let mut settings = fixtures::settings();
    settings.forges = List::with_capacity(0);
    settings
}
fn forge(kind: Kind, number: u16) -> Forge {
    let settings = match kind {
        Kind::Forgejo => fixtures::settings(),
        Kind::GitHub => fixtures::github_settings(),
    };
    let mut forge = settings.forges.into_boxed().into_iter().next().expect("fixture forge");
    forge.number = number;
    forge.address = skein_io::Addr::Ip(([127, 0, 0, 1], 3000_u16.checked_add(number).expect("fixture port")).into());
    forge
}
fn add(h: &mut Harness, forge: Forge) {
    let mut endpoints = List::with_capacity(h.endpoints.len().checked_add(1).expect("added endpoint room"));
    for endpoint in &h.endpoints {
        endpoints.push(*endpoint).expect("kept endpoints");
    }
    endpoints.push((forge.address, forge.number)).expect("added endpoint");
    h.endpoints = endpoints;
    h.component.add(&h.env, forge, &mut h.up, &mut h.io).expect("valid runtime forge");
    h.run(Duration::ZERO);
}
fn ready(h: &mut Harness, number: u16) {
    h.connected(number);
    h.answer(number, &fixtures::user_ok());
    h.answer(number, &fixtures::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.start(number), Start::Ready { cap: 2 });
}
fn settings_call(h: &mut Harness, number: u16, call: u64) {
    h.call(call, api::Repository { forge: number, repository: 1 }, api::Op::Read(api::Read::Settings));
    h.answer(number, &fixtures::ok(fixtures::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    assert_eq!(
        h.answered(call).expect("settings answer").result,
        Ok(api::Answer::Settings(api::Settings {
            default_branch: Box::from(b"main".as_slice()),
            merge: true,
            squash: false,
            rebase: false,
        }))
    );
    assert_eq!(h.answered(call).expect("settings answer").cost, 1);
}

#[test]
fn an_added_forgejo_forge_answers_calls_as_a_configured_one() {
    let mut h = Harness::new(empty());
    add(&mut h, forge(Kind::Forgejo, FORGE));
    assert_eq!(h.start(FORGE), Start::Checking { next: h.env.limits.start_backoff });
    ready(&mut h, FORGE);
    settings_call(&mut h, FORGE, 1);
    let tables = h.component.forgejo_tables();
    assert!(tables.repository(fixtures::REPO).is_some(), "first dialect has repository room");
    tables.keep_user(FORGE, 2, Box::from(b"fixture".as_slice()));
    assert_eq!(tables.user(FORGE, 2), Some(b"fixture".as_slice()));
}

#[test]
fn a_forge_added_without_access_answers_not_connected_until_its_connect() {
    let mut h = Harness::new(empty());
    let mut forge = forge(Kind::Forgejo, FORGE);
    let access = forge.access.take().expect("fixture access");
    add(&mut h, forge);
    assert_eq!(h.start(FORGE), Start::Unconnected);
    assert_eq!(h.connections_opened(), 0);
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("unconnected answer").result, Err(api::Error::NotConnected));
    assert_eq!(h.answered(1).expect("unconnected answer").cost, 0);
    h.send(Request::Connect { forge: FORGE, access });
    h.run(Duration::ZERO);
    ready(&mut h, FORGE);
    settings_call(&mut h, FORGE, 2);
}

#[test]
fn an_added_github_forge_takes_its_reserved_connection() {
    // All call slots are occupied before the first GitHub forge arrives.
    let mut h = Harness::ready();
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.call(2, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.run(Duration::ZERO);
    h.connected(FORGE);
    assert_eq!(h.connections_for(FORGE), h.env.limits.connections);
    add(&mut h, forge(Kind::GitHub, fixtures::OTHER_FORGE));
    assert_eq!(h.connections_for(fixtures::OTHER_FORGE), 1);
    h.connected(fixtures::OTHER_FORGE);
    h.answer(fixtures::OTHER_FORGE, &fixtures::ok(b"[]"));
    h.run(Duration::ZERO);
    assert_eq!(h.start(fixtures::OTHER_FORGE), Start::Ready { cap: 100 });
    assert_eq!(h.connections_for(FORGE), 2);
    assert!(h.answered(1).is_none());
    assert!(h.answered(2).is_none());
    assert!(!super::github_start::contains(&h.request_bytes(fixtures::OTHER_FORGE), b"/api/v1"));
}

#[test]
fn the_first_added_github_forge_starts_connects_and_answers_calls() {
    use super::github_start::{CONTENTS, EXPIRES, FULL, LIVE, VIEWER, mint};
    let mut h = Harness::new(empty());
    h.git_tokens = Some(List::with_capacity(32));
    h.installation_events = Some(List::with_capacity(32));
    h.facts = Some(List::with_capacity(32));
    let mut forge = forge(Kind::GitHub, FORGE);
    let access = forge.access.take().expect("App access");
    add(&mut h, forge);
    assert_eq!(h.start(FORGE), Start::Unconnected);
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("unconnected").result, Err(api::Error::NotConnected));
    h.send(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    h.send(Request::Connect { forge: FORGE, access });
    h.run(Duration::ZERO);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(LIVE));
    h.answer(FORGE, &mint(FULL, EXPIRES));
    h.answer(FORGE, &mint(CONTENTS, EXPIRES));
    h.answer(FORGE, &fixtures::ok(VIEWER));
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    h.call(2, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.run(Duration::ZERO);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(fixtures::GITHUB_REPOSITORY));
    h.run(Duration::ZERO);
    assert_eq!(
        h.answered(2).expect("GitHub settings").result,
        Ok(api::Answer::Settings(api::Settings {
            default_branch: Box::from(b"main".as_slice()),
            merge: true,
            squash: true,
            rebase: false,
        }))
    );
    let tables = h.component.github_tables();
    assert!(tables.repository(fixtures::REPO).is_some());
    tables.keep_user(FORGE, 9, Box::from(b"test-app[bot]".as_slice()));
    assert_eq!(tables.user(FORGE, 9), Some(b"test-app[bot]".as_slice()));
}

#[test]
fn adding_leaves_the_kept_forges_starts_alone() {
    let limits = Limits { forges: 3, connections: 3, ..fixtures::limits() };
    let shared = Shared { calls: 3, ..fixtures::shared() };
    let mut h = Harness::with(limits, shared, fixtures::two_forges());
    for number in [FORGE, fixtures::OTHER_FORGE] {
        ready(&mut h, number);
    }
    let before = [h.request_bytes(FORGE), h.request_bytes(fixtures::OTHER_FORGE)];
    let mut connections = List::with_capacity(2);
    for (owner, _) in &h.peers {
        connections.push(*owner).expect("kept connections");
    }
    h.known(fixtures::REPO, fixtures::specimen());
    add(&mut h, forge(Kind::Forgejo, 3));
    ready(&mut h, 3);
    for (number, bytes) in [FORGE, fixtures::OTHER_FORGE].into_iter().zip(before) {
        assert_eq!(h.start(number), Start::Ready { cap: 2 });
        assert_eq!(h.connections_for(number), 1);
        assert_eq!(h.request_bytes(number), bytes);
    }
    for owner in connections.into_boxed() {
        assert!(h.peers.contains_key(&owner), "kept socket unchanged");
    }
    assert!(h.component.forgejo_tables().repository(fixtures::REPO).is_some());
    assert_eq!(h.connected_count, 3, "each start connected only once");
}

// Each factory makes both the proposed addition and new's complete settings.
fn refused(limits: Limits, shared: Shared, initial: fn() -> Settings, proposed: fn() -> Forge) -> Unusable {
    let mut combined = initial();
    let mut all = List::with_capacity(combined.forges.len().checked_add(1).expect("proposed forge room"));
    for forge in combined.forges.into_boxed() {
        all.push(forge).expect("kept forges");
    }
    all.push(proposed()).expect("proposed forge");
    combined.forges = all;
    let expected = Component::new(combined, &limits, &shared).expect_err("new refuses");
    let mut h = Harness::with(limits, shared, initial());
    let number = proposed().number;
    let previous = h.component.start(number);
    let opened = h.connections_opened();
    assert!(h.io.is_empty());
    assert!(h.up.is_empty());
    assert_eq!(h.component.add(&h.env, proposed(), &mut h.up, &mut h.io), Err(Added::Unusable(expected.clone())));
    assert_eq!(h.component.start(number), previous);
    assert_eq!(h.connections_opened(), opened);
    assert!(h.io.is_empty());
    assert!(h.up.is_empty());
    expected
}
fn disconnected() -> Settings {
    let mut settings = fixtures::settings();
    settings.forges.get_mut(0).expect("forge").access = None;
    settings
}

#[test]
fn an_add_is_refused_as_new_refuses() {
    let l = fixtures::limits();
    let s = fixtures::shared();
    assert_eq!(refused(l, s, disconnected, || forge(Kind::Forgejo, FORGE)), Unusable::Duplicate { forge: FORGE });
    assert_eq!(
        refused(Limits { forges: 1, ..l }, s, disconnected, || forge(Kind::Forgejo, 2)),
        Unusable::Forges { forges: 1 }
    );
    let cases: [fn() -> Forge; 10] = [
        || {
            let mut f = forge(Kind::Forgejo, 2);
            f.address = skein_io::Addr::InProcess(skein_io::Name::EMPTY);
            f
        },
        || {
            let mut f = forge(Kind::Forgejo, 2);
            f.address = skein_io::Addr::Ip(([192, 0, 2, 1], 80).into());
            f
        },
        || {
            let mut f = forge(Kind::Forgejo, 2);
            f.transport = Transport::Plaintext { name: Some(Box::from(b"bad/name".as_slice())) };
            f
        },
        || {
            let mut f = forge(Kind::Forgejo, 2);
            f.access.as_mut().expect("access").user = 0;
            f
        },
        || {
            let mut f = forge(Kind::Forgejo, 2);
            f.access.as_mut().expect("access").credential =
                skein_forge::credential::Credential::Token(Secret::new(Box::from(b"bad\nvalue".as_slice())));
            f
        },
        || {
            let mut f = forge(Kind::GitHub, 2);
            f.access = forge(Kind::Forgejo, 2).access;
            f
        },
        || {
            let mut f = forge(Kind::GitHub, 2);
            f.access.as_mut().expect("access").credential =
                skein_forge::credential::Credential::App { id: 7, key: Secret::new(Box::from(b"invalid".as_slice())) };
            f
        },
        || {
            let mut f = forge(Kind::Forgejo, 2);
            f.access.as_mut().expect("access").webhook_secret = Some(Secret::new(Box::from(b"artificial".as_slice())));
            f
        },
        || {
            let mut f = forge(Kind::Forgejo, 2);
            f.log_origins = List::with_capacity(1);
            f.log_origins.push(origin()).expect("origin");
            f
        },
        || {
            let mut f = forge(Kind::GitHub, 2);
            let mut origin = origin();
            origin.origin.tls = true;
            f.log_origins = List::with_capacity(1);
            f.log_origins.push(origin).expect("origin");
            f
        },
    ];
    for proposed in cases {
        refused(l, s, disconnected, proposed);
    }
    assert_eq!(
        refused(Limits { key_bytes: 1, ..l }, s, disconnected, || forge(Kind::GitHub, 2)),
        Unusable::KeyBytes { key_bytes: 1 }
    );
    refused(Limits { renew_before: Duration::from_secs(1), ..l }, s, disconnected, || forge(Kind::GitHub, 2));
    refused(
        Limits { deadline: Duration::from_secs(271), ..l },
        Shared { lifetime: Duration::from_secs(271), ..s },
        disconnected,
        || forge(Kind::GitHub, 2),
    );
    refused(
        l,
        s,
        || {
            let mut s = empty();
            s.user_agent = Box::from(b"".as_slice());
            s
        },
        || forge(Kind::Forgejo, 2),
    );
    refused(Limits { log_origins: 1, ..l }, s, disconnected, || {
        let mut f = forge(Kind::GitHub, 2);
        f.log_origins = List::with_capacity(2);
        f.log_origins.push(origin()).expect("first origin");
        f.log_origins.push(origin()).expect("second origin");
        f
    });
}
fn origin() -> LogOrigin {
    LogOrigin {
        origin: skein_forge::github::redirect::Origin {
            tls: false,
            host: Box::from(b"logs.test".as_slice()),
            port: 80,
        },
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 80).into()),
    }
}

#[test]
fn an_add_after_close_is_refused() {
    for request in [Request::Close, Request::Abort] {
        let mut h = Harness::new(fixtures::settings());
        h.send(request);
        let state = h.start(FORGE);
        let io = h.io.len();
        let up = h.up.len();
        assert_eq!(h.component.add(&h.env, forge(Kind::Forgejo, 2), &mut h.up, &mut h.io), Err(Added::Closed));
        assert_eq!(h.component.start(2), None);
        assert_eq!(h.start(FORGE), state);
        assert_eq!((h.io.len(), h.up.len()), (io, up));
        h.run(Duration::ZERO);
        assert!(h.closed);
        assert_eq!(h.component.add(&h.env, forge(Kind::Forgejo, 2), &mut h.up, &mut h.io), Err(Added::Closed));
        assert_eq!(h.component.start(2), None);
        assert!(h.io.is_empty());
        assert!(h.up.is_empty());
    }
}
