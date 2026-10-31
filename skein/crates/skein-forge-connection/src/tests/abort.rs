//! Component abort transitions (programming-model.md, 5.2; forge-connection.md, 8.5).
use super::*;
use crate::tests::fixtures::{self, Harness};
use skein_io::{Event as IoEvent, Request as IoRequest};
use skein_lib::{Duration, Wall};

fn idle() -> Harness {
    let mut h = Harness::ready();
    h.run(Duration::ZERO);
    assert_eq!(h.component.connections.len(), 1);
    assert!(h.io.is_empty());
    assert!(h.up.is_empty());
    h
}

fn socket(h: &Harness) -> (Token, Token) {
    let (owner, id) = h.component.connections.first().expect("one API connection");
    (*owner, h.component.connection_slab.get(*id).expect("indexed connection").socket.expect("connected"))
}

fn settled(h: &mut Harness, owner: Token) {
    assert!(h.up.is_empty(), "Closed waits for io");
    assert_eq!(h.component.next_deadline(), None, "settling waits on io alone");
    h.component.up(&h.env, IoEvent::Closed { owner }, &mut h.up, &mut h.io);
    let Some(Event::Closed) = h.up.pop() else { unreachable!("Closed follows settlement") };
    assert!(h.up.is_empty());
    h.component.fire(&h.env, &mut h.up, &mut h.io);
    assert!(h.up.is_empty(), "Closed is emitted once");
    assert!(h.io.is_empty());
}

#[test]
fn abort_drops_live_calls_and_held_and_returned_answers() {
    let mut h = idle();
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    assert_eq!(h.component.calls.len(), 1, "the call is in flight");
    h.io = Queue::uncapped();
    let (owner, entity) = socket(&h);
    for (call, returned) in [(2, false), (3, true)] {
        let answer =
            Answered { call: Token::new(call), cost: 1, free: 0, rate: None, result: Err(api::Error::Unavailable) };
        h.send(if returned { Request::Returned(answer) } else { Request::Refused(answer) });
    }
    assert_eq!(h.component.held.len(), 2);
    h.send(Request::Abort);
    assert!(h.component.calls.is_empty());
    assert!(h.component.holders.is_empty());
    assert!(h.component.held.is_empty());
    let Some(IoRequest::Abort { entity: sent }) = h.io.pop() else { unreachable!("expected socket request") };
    assert_eq!(sent, entity);
    assert!(h.io.is_empty());
    h.send(Request::Offer);
    h.component.up(&h.env, IoEvent::Stream { owner, up: skein_lib::stream::Up::End }, &mut h.up, &mut h.io);
    settled(&mut h, owner);
    h.component.reclaim();
    assert_eq!(h.component.waiting.len(), 0);
}

#[test]
fn abort_after_close_escalates_an_idle_socket() {
    let mut h = idle();
    let (owner, entity) = socket(&h);
    h.send(Request::Close);
    let Some(IoRequest::Close { entity: sent }) = h.io.pop() else { unreachable!("expected socket request") };
    assert_eq!(sent, entity);
    h.send(Request::Abort);
    let Some(IoRequest::Abort { entity: sent }) = h.io.pop() else { unreachable!("expected socket request") };
    assert_eq!(sent, entity);
    assert!(h.io.is_empty());
    settled(&mut h, owner);
}

#[test]
fn close_after_abort_changes_nothing() {
    let mut h = idle();
    let (owner, entity) = socket(&h);
    h.send(Request::Abort);
    let Some(IoRequest::Abort { entity: sent }) = h.io.pop() else { unreachable!("expected socket request") };
    assert_eq!(sent, entity);
    h.send(Request::Close);
    assert!(h.io.is_empty());
    settled(&mut h, owner);
}

#[test]
fn a_second_abort_changes_nothing() {
    let mut h = idle();
    let (owner, entity) = socket(&h);
    h.send(Request::Abort);
    let Some(IoRequest::Abort { entity: sent }) = h.io.pop() else { unreachable!("expected socket request") };
    assert_eq!(sent, entity);
    h.send(Request::Abort);
    assert!(h.io.is_empty());
    settled(&mut h, owner);
}

#[test]
fn abort_reaches_a_socket_reported_after_the_request() {
    let mut h = idle();
    h.component = Component::new(fixtures::settings(), &h.env.limits, &fixtures::shared()).expect("valid settings");
    h.component.begin(&h.env, &mut h.up, &mut h.io);
    let Some(IoRequest::Connect { owner, .. }) = h.io.pop() else { unreachable!("start connects") };
    h.send(Request::Abort);
    h.io = Queue::uncapped();
    assert!(h.up.is_empty());
    let entity = Token::new(100);
    h.component.up(&h.env, IoEvent::Connecting { owner, socket: entity }, &mut h.up, &mut h.io);
    let Some(IoRequest::Abort { entity: sent }) = h.io.pop() else { unreachable!("expected socket request") };
    assert_eq!(sent, entity);
    h.component.up(&h.env, IoEvent::Connected { owner }, &mut h.up, &mut h.io);
    assert!(h.io.is_empty());
    settled(&mut h, owner);
}

#[test]
fn abort_escalates_the_listener_and_settles_webhook_sockets() {
    for close_first in [false, true] {
        let limits = fixtures::limits();
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut settings = fixtures::hook_settings();
        settings.forges.get_mut(0).expect("forge").access = None;
        let addr = settings.webhooks.as_ref().expect("listener").listener;
        let mut c = Component::new(settings, &limits, &fixtures::shared()).expect("valid settings");
        let mut up = Queue::uncapped();
        let mut io = Queue::uncapped();
        c.begin(&env, &mut up, &mut io);
        let Some(IoRequest::Listen { owner, .. }) = io.pop() else { unreachable!("listener requested") };
        let listener = Token::new(200);
        c.up(&env, IoEvent::Listening { owner, listener, addr }, &mut up, &mut io);
        let Some(Event::Listening { .. }) = up.pop() else { unreachable!("listener reports its address") };
        let socket = Token::new(201);
        c.up(&env, IoEvent::Accepted { owner, socket, peer: addr }, &mut up, &mut io);
        let Some(IoRequest::Bind { owner: hook, .. }) = io.pop() else { unreachable!("accepted hook bound") };
        io = Queue::uncapped();
        if close_first {
            c.down(&env, Request::Close, &mut up, &mut io);
            let Some(IoRequest::Close { entity }) = io.pop() else { unreachable!("expected shutdown request") };
            assert_eq!(entity, listener);
            let Some(IoRequest::Abort { entity }) = io.pop() else { unreachable!("expected shutdown request") };
            assert_eq!(entity, socket);
        }
        c.down(&env, Request::Abort, &mut up, &mut io);
        let Some(IoRequest::Abort { entity }) = io.pop() else { unreachable!("expected shutdown request") };
        assert_eq!(entity, listener);
        if !close_first {
            let Some(IoRequest::Abort { entity }) = io.pop() else { unreachable!("expected shutdown request") };
            assert_eq!(entity, socket);
        }
        assert!(io.is_empty());
        c.down(&env, Request::Abort, &mut up, &mut io);
        c.down(&env, Request::Close, &mut up, &mut io);
        assert!(io.is_empty());
        c.up(&env, IoEvent::Closed { owner }, &mut up, &mut io);
        assert!(up.is_empty(), "webhook still settles");
        c.up(&env, IoEvent::Closed { owner: hook }, &mut up, &mut io);
        let Some(Event::Closed) = up.pop() else { unreachable!("Closed follows settlement") };
        assert_eq!(c.next_deadline(), None);
        c.fire(&Env { now: env.now.saturating_add(Duration::from_secs(100)), ..env }, &mut up, &mut io);
        assert!(up.is_empty());
    }
}
