//! Runtime membership (skein/docs/design/forge-connection.md, 3.1).
use super::{fixtures as f, github_start as gh};
use crate::{Event, Request, Settings, settings::Forge, start::Start};
use alloc::boxed::Box;
use skein_forge::api::{self, Op, Read};
use skein_lib::{Duration, List, Queue, Token};

fn empty() -> Settings {
    let mut settings = f::settings();
    settings.forges = List::with_capacity(0);
    settings
}

fn forge(settings: Settings) -> Forge {
    settings.forges.into_boxed().into_iter().next().expect("fixture forge")
}

fn harness() -> f::Harness {
    let mut h = f::Harness::new(empty());
    h.endpoints = List::with_capacity(2);
    h.endpoints.push((forge(f::settings()).address, f::FORGE)).expect("fixture endpoint");
    h.git_tokens = Some(List::with_capacity(32));
    h.installation_events = Some(List::with_capacity(32));
    h.facts = Some(List::with_capacity(32));
    h
}

#[test]
fn an_empty_configuration_is_accepted() {
    let mut h = harness();
    h.run(Duration::from_secs(20));
    assert_eq!(h.connections_opened(), 0);
    assert!(h.up.is_empty());
    assert_eq!(crate::settings::owned_bytes(&empty()), Some(12));
}

#[test]
fn an_added_forgejo_starts_with_its_access() {
    let mut h = harness();
    h.send(Request::Add(forge(f::settings())));
    h.run(Duration::ZERO);
    h.connected(f::FORGE);
    assert_eq!(
        f::header(&h.request_bytes(f::FORGE), b"Authorization: "),
        b"token 0123456789abcdef0123456789abcdef01234567"
    );
    h.answer(f::FORGE, &f::user_ok());
    h.answer(f::FORGE, &f::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.start(f::FORGE), Start::Ready { cap: 2 });
    h.known(f::REPO, f::specimen());
    h.call(1, f::REPO, Op::Read(Read::Branch { branch: Box::from(b"main".as_slice()) }));
    h.run(Duration::ZERO);
    h.answer(f::FORGE, &f::ok(f::BRANCH_BODY));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("added forge serves calls").result, Ok(api::Answer::Commit([0; 32])));
}

#[test]
fn an_added_github_mints_and_serves_a_call() {
    let mut h = harness();
    h.send(Request::Add(forge(f::github_settings())));
    gh::accepted(&mut h, &[1]);
    h.run(Duration::ZERO);
    h.connected(f::FORGE);
    h.answer(f::FORGE, &f::ok(gh::LIVE));
    gh::full(&mut h);
    gh::contents(&mut h);
    h.answer(f::FORGE, &f::ok(gh::VIEWER));
    h.run(Duration::ZERO);
    assert_eq!(h.start(f::FORGE), Start::Ready { cap: skein_forge::github::pin::PAGE_CAP });
    assert_eq!(h.git_tokens.as_ref().expect("tokens").len(), 1);
    f::github_repository(&mut h, f::REPO, 22);
    h.call(1, f::REPO, Op::Read(Read::Branch { branch: Box::from(b"main".as_slice()) }));
    h.run(Duration::ZERO);
    h.connected(f::FORGE);
    assert!(gh::contains(&h.request_bytes(f::FORGE), b"Authorization: Bearer artificial-full-api-secret\r\n"));
    h.answer(f::FORGE, &f::ok(f::GITHUB_BRANCH));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("call answered").result, Ok(api::Answer::Commit([0; 32])));
}

#[test]
fn remove_ends_waiting_and_flying_calls_unavailable() {
    let mut h = f::known();
    h.run(Duration::from_secs(5));
    h.call(1, f::REPO, Op::Read(Read::Settings));
    h.run(Duration::ZERO);
    h.connected(f::FORGE);
    h.call(2, f::REPO, Op::Read(Read::Settings));
    h.send(Request::Remove { forge: f::FORGE });
    assert_eq!(h.up.len(), 2, "both terminals emitted in the removal step");
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("flying terminal").cost, 1);
    for call in [1, 2] {
        assert_eq!(h.answered(call).expect("terminal").result, Err(api::Error::Unavailable));
    }
    assert!(h.peers.is_empty(), "every connection aborted");
    h.run(Duration::from_secs(20));
    assert_eq!(h.answers.len(), 2, "one terminal per call");

    let mut h = f::Harness::new(f::settings());
    h.call(1, f::REPO, Op::Read(Read::Settings));
    assert!(h.answered(1).is_none(), "waiting for the start");
    h.send(Request::Remove { forge: f::FORGE });
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("waiting terminal").result, Err(api::Error::Unavailable));
}

#[test]
fn a_removed_forge_names_no_event_after() {
    let mut h = harness();
    h.send(Request::Add(forge(f::github_settings())));
    h.run(Duration::ZERO);
    let owner = *h.peers.first().expect("starting connection").0;
    h.send(Request::Remove { forge: f::FORGE });
    h.run(Duration::ZERO);
    assert_eq!(h.component.start(f::FORGE), None);
    for event in [
        skein_io::Event::Connected { owner },
        skein_io::Event::Failed { owner, error: skein_io::Error::Refused },
        skein_io::Event::Closed { owner },
    ] {
        h.component.up(&h.env, event, &mut h.up, &mut h.io);
    }
    h.run(Duration::from_secs(3600));
    assert!(h.up.is_empty());
    assert!(h.git_tokens.as_ref().expect("tokens").is_empty());
    assert!(h.installation_events.as_ref().expect("installations").is_empty());
    assert!(h.facts.as_ref().expect("facts").is_empty());
    assert!(h.hints.is_empty());
    assert_eq!(h.connected_count, 0);
    assert_eq!(h.stopped, None);
}

#[test]
fn an_add_past_the_bound_is_ignored() {
    let mut h = harness();
    for number in [1, 2, 3] {
        let mut forge = forge(f::settings());
        forge.number = number;
        forge.access = None;
        h.send(Request::Add(forge));
    }
    assert_eq!(h.component.start(1), Some(Start::Unconnected));
    assert_eq!(h.component.start(2), Some(Start::Unconnected));
    assert_eq!(h.component.start(3), None);
    h.send(Request::Remove { forge: 99 });
    h.send(Request::Remove { forge: 1 });
    assert_eq!(h.component.start(2), Some(Start::Unconnected), "swap keeps the other forge");
    let mut third = forge(f::settings());
    third.number = 3;
    third.access = None;
    h.send(Request::Add(third));
    assert_eq!(h.component.start(3), Some(Start::Unconnected), "removal frees admission room");
    h.run(Duration::ZERO);
    assert_eq!(h.connections_opened(), 0);
}

#[test]
fn removal_aborts_a_connection_whose_socket_arrives_later() {
    let mut h = harness();
    h.component.down(&h.env, Request::Add(forge(f::settings())), &mut h.up, &mut h.io);
    let Some(skein_io::Request::Connect { owner, .. }) = h.io.pop() else { panic!("connect expected") };
    h.send(Request::Remove { forge: f::FORGE });
    let socket = Token::new(100);
    h.component.up(&h.env, skein_io::Event::Connecting { owner, socket }, &mut h.up, &mut h.io);
    let Some(skein_io::Request::Abort { entity }) = h.io.pop() else { panic!("abort expected") };
    assert_eq!(entity, socket);
    assert!(h.up.is_empty());
}

#[test]
fn removal_preserves_the_other_forges_live_start() {
    let mut h = f::Harness::new(f::two_forges());
    h.send(Request::Remove { forge: f::FORGE });
    h.run(Duration::ZERO);
    h.connected(f::OTHER_FORGE);
    h.answer(f::OTHER_FORGE, &f::user_ok());
    h.answer(f::OTHER_FORGE, &f::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.start(f::OTHER_FORGE), Start::Ready { cap: 2 });
    h.known(f::OTHER_REPO, f::specimen());
    h.call(1, f::OTHER_REPO, Op::Read(Read::Branch { branch: Box::from(b"main".as_slice()) }));
    h.run(Duration::ZERO);
    h.answer(f::OTHER_FORGE, &f::ok(f::BRANCH_BODY));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("remaining forge serves calls").result, Ok(api::Answer::Commit([0; 32])));
}

#[test]
fn removal_discards_queued_notifications_and_listener_hints() {
    let mut h = harness();
    let mut added = forge(f::settings());
    added.access = None;
    h.send(Request::Add(added));
    h.up.push(Event::Connected { forge: f::FORGE });
    h.up.push(Event::Connected { forge: f::OTHER_FORGE });
    h.send(Request::Remove { forge: f::FORGE });
    let Some(Event::Connected { forge }) = h.up.pop() else { panic!("other forge's notification expected") };
    assert_eq!(forge, f::OTHER_FORGE);
    assert!(h.up.is_empty());

    let mut hook = crate::hooks::connection::Hooked::new(Token::new(100), &h.env);
    hook.refused(api::Hint {
        repository: f::REPO,
        change: api::Change::Branch(Box::from(b"main".as_slice())),
        key: None,
    });
    hook.offer();
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    hook.pump(Token::new(101), &h.env, &empty(), &mut up, &mut io);
    assert!(up.is_empty(), "listener drops a removed forge's hint when popped");
}

#[test]
#[should_panic(expected = "a forge number is configured once")]
fn a_duplicate_number_asserts_only_in_tests() {
    let mut h = harness();
    let mut added = forge(f::settings());
    added.access = None;
    h.send(Request::Add(added));
    h.send(Request::Add(forge(f::settings())));
}
