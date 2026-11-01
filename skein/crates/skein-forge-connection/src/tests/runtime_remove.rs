//! Runtime removal ends one forge's work and preserves its neighbours (8.5; draft 4).
use super::fixtures::{self as f, FORGE, Harness, OTHER_FORGE, REPO};
use crate::{Component, Event, Request, start::Start};
use alloc::boxed::Box;
use skein_forge::api::{self, Error, Op, Read};
use skein_lib::{Duration, List, Queue, Token, stream::Up};

fn settings_answer() -> api::Answer {
    api::Answer::Settings(api::Settings {
        default_branch: Box::from(b"main".as_slice()),
        merge: true,
        squash: false,
        rebase: false,
    })
}
fn remove(h: &mut Harness, forge: u16) {
    h.component.remove(&h.env, forge, &mut h.up, &mut h.io);
}
fn ready(h: &mut Harness, forge: u16) {
    h.connected(forge);
    h.answer(forge, &f::user_ok());
    h.answer(forge, &f::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.start(forge), Start::Ready { cap: 2 });
}
fn two_ready() -> Harness {
    let mut h = Harness::new(f::two_forges());
    for forge in [FORGE, OTHER_FORGE] {
        ready(&mut h, forge);
        h.known(api::Repository { forge, repository: 1 }, f::specimen());
    }
    h
}
fn aborts(io: &Queue<skein_io::Request>) -> List<Token> {
    let mut aborted = List::with_capacity(8);
    for request in io {
        if let skein_io::Request::Abort { entity } = request {
            aborted.push(*entity).expect("fixture aborts");
        }
    }
    aborted
}
fn owner(h: &Harness) -> Token {
    *h.peers.first().expect("one connection").0
}
fn event(h: &mut Harness, event: skein_io::Event) {
    h.component.up(&h.env, event, &mut h.up, &mut h.io);
}

#[test]
fn removing_a_forge_answers_its_open_calls_not_connected() {
    let mut h = two_ready();
    h.call(1, REPO, Op::Read(Read::Settings));
    h.call(2, f::OTHER_REPO, Op::Read(Read::Settings));
    h.run(Duration::ZERO);
    let kept_requests = h.request_bytes(OTHER_FORGE);
    remove(&mut h, FORGE);
    assert_eq!(h.component.start(FORGE), None);
    assert_eq!(h.start(OTHER_FORGE), Start::Ready { cap: 2 });
    let Some(Event::Answered(answer)) = h.up.pop() else {
        panic!("removed call's terminal at once");
    };
    assert_eq!((answer.call, &answer.result, answer.cost), (Token::new(1), &Err(Error::NotConnected), 1));
    h.send(Request::Refused(answer));
    assert!(h.up.is_empty());
    assert_eq!(aborts(&h.io).len(), 1);
    h.send(Request::Offer);
    h.run(Duration::ZERO);
    assert_eq!(h.request_bytes(OTHER_FORGE), kept_requests);
    assert_eq!(h.connections_for(OTHER_FORGE), 1);
    h.answer(OTHER_FORGE, &f::ok(f::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(2).expect("kept call").result, Ok(settings_answer()));
    assert_eq!(h.answered(1).expect("removed call").result, Err(Error::NotConnected));
    h.send(Request::Close);
    h.run(Duration::ZERO);
    assert!(h.closed);
}

#[test]
fn a_removed_forges_late_responses_change_nothing() {
    let mut h = f::known();
    h.call(1, REPO, Op::Read(Read::Settings));
    h.run(Duration::ZERO);
    let owner = owner(&h);
    remove(&mut h, FORGE);
    assert_eq!(aborts(&h.io).len(), 1);
    event(&mut h, skein_io::Event::Stream { owner, up: Up::Bytes(f::ok(f::REPOSITORY_BODY)) });
    event(&mut h, skein_io::Event::Connected { owner });
    event(&mut h, skein_io::Event::Failed { owner, error: skein_io::Error::Refused });
    assert_eq!(h.up.len(), 1, "one terminal before settlement");
    assert_eq!(h.io.len(), 1, "only the abort");
    h.send(Request::Close);
    assert!(!h.closed);
    assert_eq!(h.up.len(), 1, "close still waits for io");
    h.run(Duration::ZERO);
    assert!(h.closed);
    assert!(h.peers.is_empty());
    event(&mut h, skein_io::Event::Closed { owner });
    event(&mut h, skein_io::Event::Stream { owner, up: Up::Bytes(f::ok(f::REPOSITORY_BODY)) });
    h.run(Duration::ZERO);
    assert_eq!(h.answers.len(), 1);
    assert_eq!(h.answered(1).expect("terminal").result, Err(Error::NotConnected));
    assert_eq!(h.component.next_deadline(), None);
}

#[test]
fn a_call_on_a_removed_forge_is_answered_as_an_unknown_forges() {
    let mut h = f::known();
    remove(&mut h, FORGE);
    h.run(Duration::ZERO);
    for (call, forge) in [(1, FORGE), (2, 99)] {
        h.call(call, api::Repository { forge, repository: 1 }, Op::Read(Read::Settings));
        h.run(Duration::ZERO);
        let answer = h.answered(call).expect("unknown forge terminal");
        assert_eq!((&answer.result, answer.cost), (&Err(Error::Unavailable), 0));
    }
    let opened = h.connections_opened();
    let access = f::settings().forges.get_mut(0).expect("fixture forge").access.take().expect("access");
    h.send(Request::Connect { forge: FORGE, access });
    h.send(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    h.run(Duration::ZERO);
    assert_eq!(h.component.start(FORGE), None);
    assert_eq!(h.connections_opened(), opened);
    assert!(h.io.is_empty());
}

#[test]
fn a_removed_github_forge_drops_its_installations_and_log_work() {
    use super::github_start::{CONTENTS, EXPIRES, FULL, LIVE, VIEWER, mint};
    let limits = crate::Limits { log_poll: Duration::from_secs(60), ..f::limits() };
    let mut h = Harness::with(limits, f::shared(), f::github_settings());
    h.send(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    h.connected(FORGE);
    h.answer(FORGE, &f::ok(LIVE));
    h.answer(FORGE, &mint(FULL, EXPIRES));
    h.answer(FORGE, &mint(CONTENTS, EXPIRES));
    h.answer(FORGE, &f::ok(VIEWER));
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert!(h.component.test_github().rows.contains_key(&1), "installation kept");
    assert!(!h.git_tokens.as_ref().expect("token output").is_empty());
    h.run(h.env.limits.log_poll);
    h.connected(FORGE);
    assert!(super::github_start::contains(&h.request_bytes(FORGE), b"/app/hook/deliveries"));
    let owner = owner(&h);
    let installations = h.installation_events.as_ref().expect("installation output").len();
    let tokens = h.git_tokens.as_ref().expect("token output").len();
    let opened = h.connections_opened();
    remove(&mut h, FORGE);
    assert_eq!(aborts(&h.io).len(), 1);
    event(&mut h, skein_io::Event::Stream { owner, up: Up::Bytes(f::ok(b"[]")) });
    assert_eq!(h.io.len(), 1, "no further log request");
    h.run(h.env.limits.log_poll.saturating_add(h.env.limits.deadline));
    h.send(Request::Offer);
    h.run(Duration::ZERO);
    assert_eq!(h.installation_events.as_ref().expect("installation output").len(), installations);
    assert_eq!(h.git_tokens.as_ref().expect("token output").len(), tokens);
    assert_eq!(h.connections_opened(), opened);
    assert!(h.hints.is_empty());
    assert_eq!(h.component.start(FORGE), None);
    assert_eq!(h.component.next_deadline(), None);
    h.send(Request::Close);
    h.run(Duration::ZERO);
    assert!(h.closed);
}

#[test]
fn a_removed_forges_webhook_path_answers_not_found() {
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: f::limits() };
    let mut component = Component::new(f::hook_settings(), &env.limits, &f::shared()).expect("hook settings");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    component.remove(&env, FORGE, &mut up, &mut io);
    component.begin(&env, &mut up, &mut io);
    let Some(skein_io::Request::Listen { owner: listener, .. }) = io.pop() else {
        panic!("listener");
    };
    component.up(
        &env,
        skein_io::Event::Accepted {
            owner: listener,
            socket: Token::new(100),
            peer: skein_io::Addr::Ip(([127, 0, 0, 1], 12345).into()),
        },
        &mut up,
        &mut io,
    );
    let Some(skein_io::Request::Bind { owner, .. }) = io.pop() else {
        panic!("hook bind");
    };
    let mut wire = skein_world::stream::Wire::new(1);
    wire.write(&f::signed_head(f::PULL_PAYLOAD, Some(b"pull_request")));
    wire.write(f::PULL_PAYLOAD);
    for _ in 0..1024_u32 {
        for _ in 0..io.len() {
            match io.pop().expect("hook request") {
                skein_io::Request::Stream { down, .. } => wire.take(down),
                skein_io::Request::Close { .. } => {}
                other @ (skein_io::Request::Listen { .. }
                | skein_io::Request::Connect { .. }
                | skein_io::Request::Bind { .. }
                | skein_io::Request::Reject { .. }
                | skein_io::Request::Output { .. }
                | skein_io::Request::Spawn { .. }
                | skein_io::Request::Signal { .. }
                | skein_io::Request::Usage { .. }
                | skein_io::Request::Window { .. }
                | skein_io::Request::Abort { .. }) => panic!("webhook output: {other:?}"),
            }
        }
        if let Some(event) = wire.answer() {
            component.up(&env, skein_io::Event::Stream { owner, up: event }, &mut up, &mut io);
        }
    }
    assert!(wire.received.starts_with(b"HTTP/1.1 404"));
    assert!(up.is_empty(), "no hint from the removed forge");
}

#[test]
fn held_answers_outlive_their_forges_removal() {
    for refused in [true, false] {
        let mut h = f::known();
        h.call(1, REPO, Op::Read(Read::Settings));
        h.answer(FORGE, &f::ok(f::REPOSITORY_BODY));
        h.run(Duration::ZERO);
        let answer = h.answers.remove(&1).expect("answered call");
        let expected = answer.clone();
        h.send(if refused { Request::Refused(answer) } else { Request::Returned(answer) });
        remove(&mut h, FORGE);
        h.run(Duration::ZERO);
        assert!(h.answered(1).is_none(), "held until offered");
        h.send(Request::Offer);
        h.run(Duration::ZERO);
        assert_eq!(h.answered(1), if refused { Some(&expected) } else { None });
    }
}

#[test]
fn a_forge_added_after_a_removal_starts() {
    let mut h = two_ready();
    remove(&mut h, FORGE);
    h.run(Duration::ZERO);
    let mut forge = f::settings().forges.into_boxed().into_iter().next().expect("fixture forge");
    forge.number = 3;
    forge.address = skein_io::Addr::Ip(([127, 0, 0, 1], 3002).into());
    let mut endpoints = List::with_capacity(3);
    for endpoint in &h.endpoints {
        endpoints.push(*endpoint).expect("kept endpoint");
    }
    endpoints.push((forge.address, forge.number)).expect("new endpoint");
    h.endpoints = endpoints;
    h.component.add(&h.env, forge, &mut h.up, &mut h.io).expect("new forge");
    h.run(Duration::ZERO);
    // With one call slot held by forge 2's idle start, the new start uses the other.
    ready(&mut h, 3);
    h.call(1, api::Repository { forge: 3, repository: 1 }, Op::Read(Read::Settings));
    h.answer(3, &f::ok(f::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("new forge call").result, Ok(settings_answer()));
    assert_eq!(h.start(OTHER_FORGE), Start::Ready { cap: 2 });
    assert_eq!(h.component.start(FORGE), None);
}

#[test]
fn removing_an_unknown_forge_changes_nothing() {
    let mut h = two_ready();
    h.call(1, REPO, Op::Read(Read::Settings));
    h.run(Duration::ZERO);
    let bytes = h.all_request_bytes(FORGE);
    let opened = h.connections_opened();
    remove(&mut h, 99);
    assert!(h.up.is_empty());
    assert!(h.io.is_empty());
    assert_eq!(h.all_request_bytes(FORGE), bytes);
    assert_eq!(h.connections_opened(), opened);
    for forge in [FORGE, OTHER_FORGE] {
        assert_eq!(h.start(forge), Start::Ready { cap: 2 });
    }
    h.answer(FORGE, &f::ok(f::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("unaffected call").result, Ok(settings_answer()));
}

#[test]
fn a_socket_named_after_removal_is_aborted_and_settled() {
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: f::limits() };
    let mut component = Component::new(f::settings(), &env.limits, &f::shared()).expect("settings");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    component.begin(&env, &mut up, &mut io);
    let Some(skein_io::Request::Connect { owner, .. }) = io.pop() else {
        panic!("start connect");
    };
    component.remove(&env, FORGE, &mut up, &mut io);
    assert!(io.is_empty(), "io has not named a socket yet");
    component.up(&env, skein_io::Event::Connected { owner }, &mut up, &mut io);
    assert!(io.is_empty(), "late connected cannot revive the exchange");
    let socket = Token::new(100);
    component.up(&env, skein_io::Event::Connecting { owner, socket }, &mut up, &mut io);
    assert_eq!(aborts(&io).as_slice(), &[socket]);
    component.down(&env, Request::Close, &mut up, &mut io);
    assert!(up.is_empty(), "close waits for the removed socket");
    component.up(&env, skein_io::Event::Closed { owner }, &mut up, &mut io);
    let Some(Event::Closed) = up.pop() else {
        panic!("closed after settlement");
    };
    assert!(up.is_empty());
}

#[test]
fn removal_reports_spent_cost_and_ends_calls_waiting_for_start() {
    let mut h = Harness::new(f::settings());
    h.call(1, REPO, Op::Read(Read::Settings));
    remove(&mut h, FORGE);
    h.run(Duration::ZERO);
    let answer = h.answered(1).expect("waiting call ended");
    assert_eq!((&answer.result, answer.cost), (&Err(Error::NotConnected), 0));

    let mut h = Harness::ready();
    h.call(1, REPO, Op::Read(Read::Branch { branch: Box::from(b"main".as_slice()) }));
    h.answer(FORGE, &f::ok(f::REPOSITORY_BODY));
    h.run(Duration::ZERO);
    assert!(h.answered(1).is_none(), "branch exchange still open after resolution");
    remove(&mut h, FORGE);
    h.run(Duration::ZERO);
    let answer = h.answered(1).expect("written call ended");
    assert_eq!((&answer.result, answer.cost), (&Err(Error::NotConnected), 2));
}

#[test]
fn removal_leaves_a_shifted_forges_start_in_flight() {
    let mut h = Harness::new(f::two_forges());
    h.connected(OTHER_FORGE);
    let requests = h.request_bytes(OTHER_FORGE);
    remove(&mut h, FORGE);
    h.run(Duration::ZERO);
    assert_eq!(h.request_bytes(OTHER_FORGE), requests);
    h.answer(OTHER_FORGE, &f::user_ok());
    h.answer(OTHER_FORGE, &f::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.start(OTHER_FORGE), Start::Ready { cap: 2 });
    assert_eq!(h.connected_count, 1);
    assert_eq!(h.component.start(FORGE), None);
}

#[test]
fn removal_aborts_a_github_mint_and_drops_its_late_token() {
    use super::github_start::{CONTENTS, EXPIRES, FULL, LIVE, mint};
    let mut h = Harness::new(f::github_settings());
    h.send(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    h.connected(FORGE);
    h.answer(FORGE, &f::ok(LIVE));
    h.answer(FORGE, &mint(FULL, EXPIRES));
    assert!(super::github_start::contains(&h.request_bytes(FORGE), b"access_tokens"));
    let owner = owner(&h);
    remove(&mut h, FORGE);
    assert_eq!(aborts(&h.io).len(), 1);
    event(&mut h, skein_io::Event::Stream { owner, up: Up::Bytes(mint(CONTENTS, EXPIRES)) });
    h.run(Duration::ZERO);
    assert!(h.git_tokens.as_ref().expect("tokens").is_empty());
    assert!(h.installation_events.as_ref().expect("installations").is_empty());
    assert!(h.up.is_empty());
    assert!(h.io.is_empty());
    h.send(Request::Close);
    h.run(Duration::ZERO);
    assert!(h.closed);
}

#[test]
fn removal_aborts_a_webhook_body_and_drops_its_delivery_state() {
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: f::limits() };
    let mut component = Component::new(f::hook_settings(), &env.limits, &f::shared()).expect("hook settings");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    component.begin(&env, &mut up, &mut io);
    let Some(skein_io::Request::Listen { owner: listener, .. }) = io.pop() else {
        panic!("listen");
    };
    let Some(skein_io::Request::Connect { .. }) = io.pop() else {
        panic!("start connection");
    };
    let socket = Token::new(100);
    component.up(
        &env,
        skein_io::Event::Accepted { owner: listener, socket, peer: skein_io::Addr::Ip(([127, 0, 0, 1], 12345).into()) },
        &mut up,
        &mut io,
    );
    let Some(skein_io::Request::Bind { owner, .. }) = io.pop() else {
        panic!("bind");
    };
    let mut wire = skein_world::stream::Wire::new(1);
    wire.write(&f::signed_head(f::PULL_PAYLOAD, Some(b"pull_request")));
    for _ in 0..1024_u32 {
        for _ in 0..io.len() {
            let Some(skein_io::Request::Stream { down, .. }) = io.pop() else {
                panic!("body demand");
            };
            wire.take(down);
        }
        if let Some(event) = wire.answer() {
            component.up(&env, skein_io::Event::Stream { owner, up: event }, &mut up, &mut io);
        }
    }
    assert!(wire.received.is_empty(), "signed head waits for the body");
    component.remove(&env, FORGE, &mut up, &mut io);
    assert_eq!(aborts(&io).as_slice(), &[socket]);
    component.up(&env, skein_io::Event::Stream { owner, up: Up::Bytes(Box::from(f::PULL_PAYLOAD)) }, &mut up, &mut io);
    component.down(&env, Request::Offer, &mut up, &mut io);
    assert!(up.is_empty(), "no delivery hint");
    assert_eq!(io.len(), 1, "no further body or response work");
}
