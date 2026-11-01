//! Runtime forge admission and release (forge-connection.md, sections 3.1 and 3.5).
use super::fixtures::{self, FORGE, Harness, OTHER_FORGE};
use crate::{
    Component, Request, Settings, Unusable,
    settings::{Forge, Transport},
    start::Start,
};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Duration, List};

fn forge() -> Forge {
    fixtures::settings().forges.into_boxed().into_iter().next().expect("fixture forge")
}
fn empty() -> Settings {
    Settings { forges: List::with_capacity(0), ..fixtures::settings() }
}
const PAST_FORGES: u16 = 3; // The fixture holds two forge numbers.

fn inert(number: u16) -> Forge {
    Forge { number, access: None, ..forge() }
}
fn add(h: &mut Harness, forge: Forge) {
    h.send(Request::Add { forge: forge.number, settings: forge });
}

#[test]
fn an_add_past_forges_or_of_a_held_number_is_refused_and_changes_nothing() {
    let mut h = Harness::new(empty());
    add(&mut h, inert(FORGE));
    assert_eq!(h.component.admits(&h.env.limits, &forge()), Err(Unusable::Duplicate { forge: FORGE }));
    add(&mut h, forge());
    h.send(Request::Add { forge: OTHER_FORGE, settings: forge() });
    assert_eq!(h.component.start(FORGE), Some(Start::Unconnected));
    assert_eq!(h.component.start(OTHER_FORGE), None);
    add(&mut h, inert(OTHER_FORGE));
    assert_eq!(
        h.component.admits(&h.env.limits, &inert(PAST_FORGES)),
        Err(Unusable::Forges { forges: h.env.limits.forges })
    );
    add(&mut h, inert(PAST_FORGES));
    assert_eq!(h.component.start(PAST_FORGES), None);
    assert!(h.io.is_empty());
    assert!(h.up.is_empty());
    h.send(Request::Close);
    h.send(Request::Remove { forge: FORGE });
    add(&mut h, inert(PAST_FORGES));
    assert_eq!(h.component.start(FORGE), Some(Start::Unconnected));
    assert_eq!(h.component.start(PAST_FORGES), None);
}

#[test]
fn admits_names_the_same_refusal_as_start_for_a_bad_address_and_a_bad_plaintext_name() {
    let h = Harness::new(empty());
    for (bad, expected) in [
        (
            Forge { address: skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into()), ..forge() },
            Unusable::Address { forge: FORGE },
        ),
        (
            Forge { transport: Transport::Plaintext { name: Some(Box::from(b"bad/name".as_slice())) }, ..forge() },
            Unusable::PlaintextName { forge: FORGE },
        ),
    ] {
        assert_eq!(h.component.admits(&h.env.limits, &bad), Err(expected.clone()));
        let mut settings = empty();
        settings.forges = List::with_capacity(h.env.limits.forges);
        settings.forges.push(bad).expect("one forge");
        fixtures::admission(settings, &h.env.limits, Err(expected), b"runtime refusal");
    }
}

#[test]
fn an_add_of_a_github_forge_checks_the_github_limits() {
    let base = fixtures::limits();
    for (limits, expected) in [
        (
            crate::Limits { deadline: Duration::from_secs(300), renew_before: Duration::from_secs(600), ..base },
            Unusable::AppDeadline { deadline: Duration::from_secs(300) },
        ),
        (
            crate::Limits { renew_before: Duration::from_secs(1), ..base },
            Unusable::RenewBefore { renew_before: Duration::from_secs(1), deadline: base.deadline },
        ),
        (crate::Limits { key_bytes: 1, ..base }, Unusable::KeyBytes { key_bytes: 1 }),
    ] {
        let shared = crate::Shared { lifetime: limits.deadline, ..fixtures::shared() };
        let mut component = Component::new(empty(), &limits, &shared).expect("no GitHub forge yet");
        let mut candidate = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
        candidate.access = None;
        assert_eq!(component.admits(&limits, &candidate), Err(expected));
        let env = skein_lib::Env { limits, now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH };
        let mut up = skein_lib::Queue::with_capacity(0);
        let mut io = skein_lib::Queue::with_capacity(0);
        component.down(&env, Request::Add { forge: FORGE, settings: candidate }, &mut up, &mut io);
        assert_eq!(component.start(FORGE), None);
        assert!(up.is_empty() && io.is_empty());
    }
}

#[test]
fn an_add_before_begin_starts_once_and_says_connected_once() {
    let mut h = Harness::new(empty());
    h.component = Component::new(empty(), &h.env.limits, &fixtures::shared()).expect("empty component");
    h.endpoints = List::with_capacity(h.env.limits.forges);
    h.endpoints.push((forge().address, FORGE)).expect("fixture endpoint");
    add(&mut h, forge());
    h.run(Duration::ZERO);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::user_ok());
    h.component.begin(&h.env, &mut h.up, &mut h.io);
    h.answer(FORGE, &fixtures::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.connections_opened(), 1);
    assert_eq!(h.connected_count, 1);
    assert_eq!(h.component.start(FORGE), Some(Start::Ready { cap: 2 }));
}

#[test]
fn a_remove_drops_every_per_forge_slot_and_the_others_keep_theirs() {
    let mut h = fixtures::ready_two(fixtures::limits(), fixtures::shared());
    let second = api::Repository { forge: OTHER_FORGE, repository: fixtures::REPO.repository };
    h.call(1, second, api::Op::Read(api::Read::Branch { branch: Box::from(b"main".as_slice()) }));
    h.run(Duration::ZERO);
    h.connected(OTHER_FORGE);
    let start = h.component.start(OTHER_FORGE);
    h.component.forgejo_tables().keep_repository(fixtures::REPO, fixtures::specimen());
    h.component.forgejo_tables().keep_user(FORGE, fixtures::WRITER, Box::from(b"writer".as_slice()));
    h.send(Request::Remove { forge: FORGE });
    assert_eq!(h.component.start(FORGE), None);
    assert_eq!(h.component.start(OTHER_FORGE), start);
    assert!(h.component.forgejo_tables().repository(fixtures::REPO).is_none());
    assert!(h.component.forgejo_tables().user(FORGE, fixtures::WRITER).is_none());
    fixtures::branch_answers(&mut h, OTHER_FORGE);
    h.answered(1).expect("second forge answered").result.as_ref().expect("second forge succeeds");
    add(&mut h, inert(FORGE));
    assert_eq!(h.component.start(FORGE), Some(Start::Unconnected));
    h.send(Request::Remove { forge: FORGE });
    h.send(Request::Remove { forge: OTHER_FORGE });
    assert_eq!(h.component.start(OTHER_FORGE), None);
}
