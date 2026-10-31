//! Runtime forge records (temper/docs/design/draft/playground-reset.md, 3 and 4).
use super::fixtures::{self, FORGE, Harness};
use crate::{
    AddRefused, Event, Request, Settings,
    settings::{Forge, Kind, LogOrigin, Transport},
    start::Start,
};
use alloc::boxed::Box;
use skein_forge::api::{self, Op, Read};
use skein_lib::{Duration, List, Token};

fn forge(number: u16, access: bool) -> Forge {
    let mut forge = fixtures::settings().forges.into_boxed().into_iter().next().expect("fixture forge");
    forge.number = number;
    forge.address = skein_io::Addr::Ip(([127, 0, 0, 1], 3000_u16.checked_add(number).expect("fixture port")).into());
    if !access {
        forge.access = None;
    }
    forge
}
fn empty() -> Settings {
    let mut settings = fixtures::settings();
    // No owner-provided capacity may limit runtime additions.
    settings.forges = List::with_capacity(0);
    settings
}
fn call(h: &mut Harness, token: u64, forge: u16) {
    h.call(token, api::Repository { forge, repository: 1 }, Op::Read(Read::Settings));
}
fn not_connected(h: &Harness, token: u64) {
    assert_eq!(h.answered(token).expect("one call terminal").result, Err(api::Error::NotConnected));
}
fn settings_answer(h: &Harness, token: u64) {
    assert_eq!(
        h.answered(token).expect("settings answer").result,
        Ok(api::Answer::Settings(api::Settings {
            default_branch: Box::from(b"main".as_slice()),
            merge: true,
            squash: false,
            rebase: false,
        }))
    );
}
fn removed(event: Option<&Event>, number: u16) {
    let Some(Event::Removed { forge }) = event else { panic!("Removed: {event:?}") };
    assert_eq!(*forge, number);
}
fn refused(h: &Harness, number: u16, expected: AddRefused) {
    assert_eq!(h.lifecycle.len(), 1, "one refusal");
    let Some(Event::Refused { forge, why }) = h.lifecycle.get(0) else { panic!("refusal") };
    assert_eq!((*forge, *why), (number, expected));
}
fn ready(h: &mut Harness, number: u16) {
    h.run(Duration::ZERO);
    h.connected(number);
    h.answer(number, &fixtures::user_ok());
    h.answer(number, &fixtures::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.start(number), Start::Ready { cap: 2 });
}

#[test]
fn add_with_access_starts_and_serves_a_waiting_call() {
    let mut h = Harness::new(empty());
    h.send(Request::Add { forge: forge(FORGE, true) });
    call(&mut h, 1, FORGE);
    assert!(h.answered(1).is_none());
    ready(&mut h, FORGE);
    h.answer(FORGE, &fixtures::ok(fixtures::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    settings_answer(&h, 1);
    assert_eq!(h.connected_count, 1);
    assert!(h.lifecycle.is_empty());
}
#[test]
fn add_without_access_is_not_connected_until_connect() {
    let mut h = Harness::new(empty());
    h.send(Request::Add { forge: forge(FORGE, false) });
    call(&mut h, 1, FORGE);
    h.run(Duration::ZERO);
    not_connected(&h, 1);
    assert_eq!(h.connections_opened(), 0);
    let access = forge(FORGE, true).access.expect("fixture access");
    h.send(Request::Connect { forge: FORGE, access });
    ready(&mut h, FORGE);
    call(&mut h, 2, FORGE);
    h.answer(FORGE, &fixtures::ok(fixtures::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    settings_answer(&h, 2);
}
#[test]
fn add_refuses_a_served_number_without_replacing_it() {
    let mut h = Harness::new(empty());
    h.send(Request::Add { forge: forge(FORGE, false) });
    h.send(Request::Add { forge: forge(FORGE, true) });
    h.run(Duration::ZERO);
    refused(&h, FORGE, AddRefused::Served);
    assert_eq!(h.start(FORGE), Start::Unconnected);
    assert_eq!(h.connections_opened(), 0);
}
#[test]
fn add_refuses_full_and_reuses_the_room_after_remove() {
    let mut h = Harness::with(crate::Limits { forges: 1, ..fixtures::limits() }, fixtures::shared(), empty());
    h.send(Request::Add { forge: forge(1, false) });
    h.send(Request::Add { forge: forge(2, false) });
    h.run(Duration::ZERO);
    refused(&h, 2, AddRefused::Full);
    h.lifecycle.clear();
    h.send(Request::Remove { forge: 1 });
    h.run(Duration::ZERO);
    removed(h.lifecycle.get(0), 1);
    h.send(Request::Add { forge: forge(2, true) });
    ready(&mut h, 2);
    assert_eq!(h.lifecycle.len(), 1);
    assert_eq!(h.component.start(1), None);
}

fn origin(host: &[u8]) -> LogOrigin {
    LogOrigin {
        origin: skein_forge::github::redirect::Origin { tls: false, host: Box::from(host), port: 3000 },
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()),
    }
}
#[test]
fn add_invalid_runs_the_same_checks_as_construction_and_changes_nothing() {
    const INVALID_CASES: u32 = 12;
    for case in 0..INVALID_CASES {
        let mut added = forge(FORGE, false);
        match case {
            0 => added.address = skein_io::Addr::InProcess(skein_io::Name::EMPTY),
            1 => added.address = skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into()),
            2 => added.transport = Transport::Plaintext { name: Some(Box::from(b"bad/name".as_slice())) },
            3 => {
                added.access = forge(FORGE, true).access;
                added.access.as_mut().expect("access").user = 0;
            }
            4 => {
                added.log_origins = List::with_capacity(1);
                added.log_origins.push(origin(b"logs.test")).expect("one origin");
            }
            5 => {
                added.kind = Kind::GitHub;
                added.log_origins = List::with_capacity(3);
                for _ in 0_u32..3 {
                    added.log_origins.push(origin(b"logs.test")).expect("three origins");
                }
            }
            6 => {
                added.kind = Kind::GitHub;
                added.log_origins = List::with_capacity(1);
                added.log_origins.push(origin(b"127.0.0.1")).expect("one origin");
            }
            7 => {
                added.kind = Kind::GitHub;
                let mut origin = origin(b"logs.test");
                origin.origin.tls = true;
                added.log_origins = List::with_capacity(1);
                added.log_origins.push(origin).expect("one origin");
            }
            8 => {
                added.access = forge(FORGE, true).access;
                added.access.as_mut().expect("access").credential = skein_forge::credential::Credential::Token(
                    skein_lib::Secret::new(Box::from(b"unsafe\r\n".as_slice())),
                );
            }
            9 => {
                added.kind = Kind::GitHub;
                added.access = forge(FORGE, true).access;
            }
            10 => {
                added.access = forge(FORGE, true).access;
                added.access.as_mut().expect("access").webhook_secret =
                    Some(skein_lib::Secret::new(Box::from(b"secret".as_slice())));
            }
            11 => {
                added.kind = Kind::GitHub;
                added.access = Some(fixtures::app_access(&[0; 64], 7, 9));
            }
            12..=u32::MAX => unreachable!("bounded fixture cases"),
        }
        let mut h = Harness::new(empty());
        let mut settings = empty();
        settings.forges = List::with_capacity(1);
        settings.forges.push(added).expect("one forge");
        assert!(crate::limits::check(&h.env.limits, &fixtures::shared(), &settings).is_err(), "case {case}");
        let added = settings.forges.into_boxed().into_iter().next().expect("one forge");
        h.send(Request::Add { forge: added });
        h.run(Duration::ZERO);
        refused(&h, FORGE, AddRefused::Invalid);
        assert_eq!(h.component.start(FORGE), None);
        assert_eq!(h.connections_opened(), 0);
        h.send(Request::Add { forge: forge(FORGE, false) });
        assert_eq!(h.component.start(FORGE), Some(Start::Unconnected), "refusal kept the slot free");
    }
}
#[test]
fn add_github_checks_limits_even_when_no_github_was_configured() {
    let limits = crate::Limits { key_bytes: 1, ..fixtures::limits() };
    let mut h = Harness::with(limits, fixtures::shared(), empty());
    let mut added = forge(FORGE, false);
    added.kind = Kind::GitHub;
    h.send(Request::Add { forge: added });
    h.run(Duration::ZERO);
    refused(&h, FORGE, AddRefused::Invalid);
}
#[test]
fn remove_ends_in_flight_and_waiting_calls_once_then_the_number_can_be_added_again() {
    let mut h = fixtures::known();
    call(&mut h, 1, FORGE);
    h.run(Duration::ZERO);
    assert!(h.answered(1).is_none());
    h.send(Request::Remove { forge: FORGE });
    assert_eq!(h.up.len(), 1, "the call ends before io settles");
    let Some(Event::Answered(answer)) = h.up.pop() else { panic!("call terminal") };
    assert_eq!(answer.result, Err(api::Error::NotConnected));
    assert_eq!(answer.cost, 1, "written cost preserved");
    h.up.push(Event::Answered(answer));
    h.send(Request::Remove { forge: FORGE });
    h.send(Request::Connect { forge: FORGE, access: forge(FORGE, true).access.expect("access") });
    h.send(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    h.run(Duration::ZERO);
    removed(h.lifecycle.get(0), FORGE);
    assert_eq!(h.lifecycle.len(), 1);
    assert!(h.peers.is_empty());
    let opened = h.connections_opened();
    call(&mut h, 2, FORGE);
    h.run(Duration::from_secs(100));
    not_connected(&h, 2);
    assert_eq!(h.connections_opened(), opened, "removed work never restarts");
    h.send(Request::Add { forge: forge(FORGE, true) });
    ready(&mut h, FORGE);
    call(&mut h, 3, FORGE);
    h.answer(FORGE, &fixtures::ok(fixtures::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    settings_answer(&h, 3);
    assert_eq!(h.lifecycle.len(), 1);
}
#[test]
fn remove_during_start_answers_waiting_calls_and_cancels_late_connects() {
    let mut h = Harness::new(empty());
    h.send(Request::Add { forge: forge(FORGE, true) });
    call(&mut h, 1, FORGE);
    h.send(Request::Remove { forge: FORGE });
    // Connecting has not supplied a socket yet: the queued Connect must still settle.
    assert_eq!(h.up.len(), 1);
    h.run(Duration::ZERO);
    not_connected(&h, 1);
    removed(h.lifecycle.get(0), FORGE);
    h.component.up(&h.env, skein_io::Event::Connected { owner: Token::new(1) }, &mut h.up, &mut h.io);
    h.run(Duration::from_secs(100));
    assert_eq!(h.lifecycle.len(), 1);
    assert_eq!(h.connected_count, 0);
    assert_eq!(h.connections_opened(), 1);
}
#[test]
fn remove_unknown_is_answered_at_once_and_calls_are_not_connected() {
    let mut h = Harness::new(empty());
    h.send(Request::Remove { forge: FORGE });
    let Some(Event::Removed { forge }) = h.up.pop() else { panic!("immediate terminal") };
    assert_eq!(forge, FORGE);
    call(&mut h, 1, FORGE);
    h.run(Duration::ZERO);
    not_connected(&h, 1);
    assert_eq!(h.connections_opened(), 0);
}
#[test]
fn add_github_with_access_starts_from_empty_and_uses_its_reserved_connection() {
    let mut h = Harness::new(empty());
    let added = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
    h.send(Request::Add { forge: added });
    h.run(Duration::ZERO);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(b"[]"));
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert_eq!(h.connected_count, 1);
    assert_eq!(h.connections_opened(), 1);
}
#[test]
fn remove_github_drops_installation_tokens_and_stops_renewal_and_calls() {
    let mut h = super::github_renewal::renewing();
    assert!(h.component.test_github().rows.get(&1).expect("installation").api.is_some());
    assert_ne!(h.component.test_github().rows.get(&1).expect("installation").git_expires, skein_lib::Wall::EPOCH);
    call(&mut h, 1, FORGE);
    h.run(Duration::ZERO);
    h.send(Request::Remove { forge: FORGE });
    h.run(Duration::ZERO);
    not_connected(&h, 1);
    removed(h.lifecycle.get(0), FORGE);
    let tokens = h.git_tokens.as_ref().expect("GitHub history").len();
    let opened = h.connections_opened();
    h.send(Request::Offer);
    h.run(Duration::from_secs(10_000));
    assert_eq!(h.git_tokens.as_ref().expect("GitHub history").len(), tokens);
    assert_eq!(h.connections_opened(), opened);
    assert!(h.peers.is_empty());
    assert_eq!(h.component.start(FORGE), None);
    assert_eq!(h.lifecycle.len(), 1);
}

#[test]
fn removing_one_forge_keeps_another_forges_call_running() {
    let mut h = fixtures::ready_two(fixtures::limits(), fixtures::shared());
    call(&mut h, 1, FORGE);
    call(&mut h, 2, fixtures::OTHER_FORGE);
    h.run(Duration::ZERO);
    h.connected(fixtures::OTHER_FORGE);
    h.send(Request::Remove { forge: FORGE });
    h.run(Duration::ZERO);
    not_connected(&h, 1);
    assert!(h.answered(2).is_none());
    h.answer(fixtures::OTHER_FORGE, &fixtures::ok(fixtures::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    settings_answer(&h, 2);
    assert_eq!(h.lifecycle.len(), 1);
}

#[test]
fn add_checks_request_head_room_even_when_initial_settings_had_no_forges() {
    let mut settings = empty();
    settings.user_agent = fixtures::filled(fixtures::PLAIN_REQUEST_ROOM, b'a');
    let mut h = Harness::new(settings);
    h.send(Request::Add { forge: forge(FORGE, false) });
    h.run(Duration::ZERO);
    refused(&h, FORGE, AddRefused::Invalid);
    assert_eq!(h.connections_opened(), 0);
}
#[test]
fn add_github_can_start_while_forgejo_occupies_every_call_connection() {
    let mut h = fixtures::known();
    call(&mut h, 1, FORGE);
    call(&mut h, 2, FORGE);
    h.run(Duration::ZERO);
    assert_eq!(h.peers.len(), h.env.limits.connections);
    let mut added = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
    added.number = fixtures::OTHER_FORGE;
    added.address = skein_io::Addr::Ip(([127, 0, 0, 1], 3001).into());
    h.send(Request::Add { forge: added });
    h.run(Duration::ZERO);
    assert_eq!(h.peers.len(), h.env.limits.connections.checked_add(1).expect("reserved connection"));
    h.connected(fixtures::OTHER_FORGE);
    h.answer(fixtures::OTHER_FORGE, &fixtures::ok(b"[]"));
    h.run(Duration::ZERO);
    assert_eq!(h.start(fixtures::OTHER_FORGE), Start::Ready { cap: 100 });
    assert!(h.answered(1).is_none() && h.answered(2).is_none());
}
