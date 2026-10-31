use super::*;
use crate::tests::fixtures;

// forge-connection.md 6.1 reserves the listener's owner prefix.
const LISTENER_OWNER: u64 = 1_u64 << 59_u32;
const LAST_API_OWNER: u64 = LISTENER_OWNER - 1;
fn token_fixture() -> (Component, Env<Limits>, Queue<skein_io::Request>) {
    let limits = fixtures::limits();
    let c = Component::new(fixtures::settings(), &limits, &fixtures::shared()).expect("valid");
    let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    (c, env, Queue::uncapped())
}
#[test]
#[should_panic(expected = "connection token prefix")]
fn api_tokens_stop_before_the_listener_range() {
    let (mut c, env, mut io) = token_fixture();
    c.next_token = LISTENER_OWNER;
    c.open_own(0, &env, &mut io);
}
#[test]
fn the_last_api_token_is_still_issued() {
    let (mut c, env, mut io) = token_fixture();
    c.next_token = LAST_API_OWNER;
    assert_eq!(c.open_own(0, &env, &mut io).raw(), LAST_API_OWNER);
}

#[test]
fn forgejo_starts_leave_the_github_reserved_slot_free() {
    // Three Forgejo starts contend for the two call slots; GitHub has its own third slot.
    const FORGEJO_FORGES: u32 = 3;
    const GITHUB_AT: u32 = FORGEJO_FORGES;
    const FORGES: u32 = FORGEJO_FORGES + 1;
    let limits = Limits { forges: FORGES, ..fixtures::limits() };
    let mut settings = fixtures::settings();
    let mut forges = List::with_capacity(FORGES);
    for number in 1..=FORGEJO_FORGES {
        let mut forge = fixtures::settings().forges.into_boxed().into_iter().next().expect("fixture forge");
        forge.number = u16::try_from(number).expect("forge number");
        forges.push(forge).expect("forge room");
    }
    let mut github = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
    github.number = u16::try_from(FORGES).expect("forge number");
    forges.push(github).expect("forge room");
    settings.forges = forges;
    let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut c = Component::new(settings, &limits, &fixtures::shared()).expect("fixture limits");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    c.begin(&env, &mut up, &mut io);
    assert_eq!(c.call_connections(), limits.connections);
    assert!(c.own_tokens.get(GITHUB_AT).expect("GitHub slot").is_some());
    assert_eq!(c.connections.len(), 3);
}

use crate::tests::github_renewal::{ready as github_ready, renewed_contents, renewed_full, renewing};

fn refuse(h: &mut fixtures::Harness, revision: u64) {
    h.component.github_token_refused(0, 1, revision, &h.env);
}
fn revision(h: &fixtures::Harness) -> u64 {
    h.component.test_github().rows.get(&1).expect("row").token_revision
}
#[test]
fn call_401_coalesces_one_renewal() {
    let mut h = github_ready();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(row.full.pending && row.contents.pending);
    h.run(skein_lib::Duration::ZERO);
    h.connected(fixtures::FORGE);
    renewed_full(&mut h);
    let gh = h.component.test_github();
    assert_eq!(gh.mint, Some((1, skein_forge::github::request::Scope::Contents)));
    let row = gh.rows.get(&1).expect("row");
    assert!(!row.full.pending && !row.contents.pending);
    renewed_contents(&mut h);
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(h.git_tokens.as_ref().expect("token history").len(), 2);
    match h.component.test_github().work {
        crate::installations::Work::Idle => {}
        crate::installations::Work::Minting { .. }
        | crate::installations::Work::Identity { .. }
        | crate::installations::Work::Waiting { .. } => unreachable!("coalesced scopes settled"),
    }
}
#[test]
fn call_401_coalesces_an_already_running_full_renewal() {
    let mut h = renewing();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    assert!(!h.component.test_github().rows.get(&1).expect("row").full.pending);
    renewed_full(&mut h);
    let gh = h.component.test_github();
    assert_eq!(gh.mint, Some((1, skein_forge::github::request::Scope::Contents)));
    assert!(!gh.rows.get(&1).expect("row").full.pending);
}
#[test]
fn stale_call_401_cannot_renew_a_replaced_full() {
    let mut h = renewing();
    let old = revision(&h);
    renewed_full(&mut h);
    refuse(&mut h, old);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(!row.full.pending && !row.contents.pending);
    assert_eq!(row.refused_revision, None);
}
#[test]
fn call_401_cannot_revive_gone() {
    let mut h = github_ready();
    let old = revision(&h);
    h.send(Request::Installations { forge: fixtures::FORGE, accepted: Box::from([]) });
    refuse(&mut h, old);
    let row = h.component.test_github().rows.get(&1).expect("Gone row");
    assert!(!row.full.pending && !row.contents.pending);
    assert_eq!(row.refused_revision, None);
}
#[test]
fn call_401_cannot_revive_a_replaced_connect() {
    let mut h = github_ready();
    let old = revision(&h);
    h.send(Request::Connect {
        forge: fixtures::FORGE,
        access: fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("public key"), 7, 9),
    });
    refuse(&mut h, old);
    let row = h.component.test_github().rows.get(&1).expect("replacement row");
    assert!(!row.full.pending && !row.contents.pending);
    assert_eq!(row.refused_revision, None);
}
#[test]
fn call_401_during_backoff_keeps_one_retry() {
    let mut h = renewing();
    h.answer(fixtures::FORGE, &fixtures::status(503));
    let until = h.component.test_github().deadline();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    assert_eq!(h.component.test_github().deadline(), until);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(!row.full.pending && row.contents.pending);
}

#[test]
fn call_401_coalesces_an_already_running_contents_renewal() {
    let mut h = crate::tests::github_renewal::contents_renewing();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(row.full.pending);
    assert!(!row.contents.pending);
    renewed_contents(&mut h);
    renewed_full(&mut h);
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(h.git_tokens.as_ref().expect("token history").len(), 2);
    assert!(h.component.test_github().mint.is_none());
}

#[test]
fn renewal_local_signing_refusal_stops_invalid_key() {
    let mut h = github_ready();
    h.component.settings.forges.get_mut(0).expect("forge").access.as_mut().expect("access").credential =
        skein_forge::credential::Credential::App { id: 7, key: skein_lib::Secret::new(Box::from([0; 64])) };
    h.component.test_github_mut().jwt.forget();
    let revision = revision(&h);
    refuse(&mut h, revision);
    h.run(skein_lib::Duration::ZERO);
    h.connected(fixtures::FORGE);
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(h.stopped(), Some((fixtures::FORGE, crate::Stopped::Invalid { value: crate::Value::Key })));
    assert_eq!(h.connected_count, 1);
}

fn abort_calls() -> fixtures::Harness {
    let mut h = fixtures::known();
    h.run(skein_lib::Duration::ZERO);
    for call in [1, 2] {
        h.call(call, fixtures::REPO, api::Op::Read(api::Read::Settings));
    }
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(h.component.calls.len(), 2);
    assert!(h.up.is_empty());
    assert!(h.io.is_empty());
    h
}

#[test]
fn abort_cancels_each_call_once_and_waits_for_aborted_sockets() {
    let mut h = abort_calls();
    let mut sockets = List::with_capacity(2);
    for (owner, id) in &h.component.connections {
        let socket = h.component.connection_slab.get(*id).expect("connection").socket.expect("socket");
        sockets.push((*owner, socket)).expect("two connections");
    }
    assert_eq!(sockets.len(), 2);
    h.component.down(&h.env, Request::Abort, &mut h.up, &mut h.io);
    for call in [1, 2] {
        let Some(Event::Cancelled { call: token }) = h.up.pop() else { panic!("cancelled call") };
        assert_eq!(token, Token::new(call));
    }
    assert!(h.up.is_empty());
    for (_, socket) in &sockets {
        assert_eq!(h.io.pop(), Some(skein_io::Request::Abort { entity: *socket }));
    }
    assert!(h.io.is_empty());
    for request in [Request::Abort, Request::Close] {
        h.component.down(&h.env, request, &mut h.up, &mut h.io);
        assert!(h.up.is_empty());
        assert!(h.io.is_empty());
    }
    h.component.down(
        &h.env,
        Request::Call { call: Token::new(3), repository: fixtures::REPO, op: api::Op::Read(api::Read::Settings) },
        &mut h.up,
        &mut h.io,
    );
    let Some(Event::Cancelled { call }) = h.up.pop() else { panic!("refused call") };
    assert_eq!(call, Token::new(3));
    let mut unsettled = sockets.len();
    for (owner, _) in &sockets {
        h.component.up(&h.env, skein_io::Event::Closed { owner: *owner }, &mut h.up, &mut h.io);
        unsettled = unsettled.checked_sub(1).expect("one settlement per socket");
        if unsettled > 0 {
            assert!(h.up.is_empty(), "Closed waits for every socket");
        }
    }
    let Some(Event::Closed) = h.up.pop() else { panic!("closed") };
    h.component.fire(&h.env, &mut h.up, &mut h.io);
    assert!(h.up.is_empty());
    assert!(h.io.is_empty());
    assert_eq!(h.component.next_deadline(), None);
}

#[test]
fn abort_after_close_upgrades_every_gracefully_closing_socket() {
    let mut h = fixtures::known();
    h.run(skein_lib::Duration::ZERO);
    let (owner, id) = h.component.connections.first().expect("idle connection");
    let (owner, id) = (*owner, *id);
    let socket = h.component.connection_slab.get(id).expect("connection").socket.expect("socket");
    h.component.down(&h.env, Request::Close, &mut h.up, &mut h.io);
    assert_eq!(h.io.pop(), Some(skein_io::Request::Close { entity: socket }));
    h.component.down(&h.env, Request::Abort, &mut h.up, &mut h.io);
    assert_eq!(h.io.pop(), Some(skein_io::Request::Abort { entity: socket }));
    h.component.down(&h.env, Request::Abort, &mut h.up, &mut h.io);
    assert!(h.up.is_empty());
    assert!(h.io.is_empty());
    h.component.up(&h.env, skein_io::Event::Closed { owner }, &mut h.up, &mut h.io);
    let Some(Event::Closed) = h.up.pop() else { panic!("closed") };
    assert!(h.up.is_empty());
}

#[test]
fn abort_before_connecting_aborts_the_late_socket() {
    let (mut c, env, mut io) = token_fixture();
    let mut up = Queue::uncapped();
    c.begin(&env, &mut up, &mut io);
    let Some(skein_io::Request::Connect { owner, .. }) = io.pop() else { panic!("start connects") };
    c.down(&env, Request::Abort, &mut up, &mut io);
    assert!(up.is_empty());
    assert!(io.is_empty());
    let socket = Token::new(99);
    c.up(&env, skein_io::Event::Connecting { owner, socket }, &mut up, &mut io);
    assert_eq!(io.pop(), Some(skein_io::Request::Abort { entity: socket }));
    c.up(&env, skein_io::Event::Closed { owner }, &mut up, &mut io);
    let Some(Event::Closed) = up.pop() else { panic!("closed") };
    assert!(up.is_empty());
    assert_eq!(c.next_deadline(), None);
}
