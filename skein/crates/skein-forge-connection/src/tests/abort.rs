//! The owner's abort, a close that does not wait (programming-model.md, 5.2).
use super::fixtures;
use crate::{Answered, Event, Request};
use skein_forge::api::{self, Op, Read};
use skein_lib::{Duration, List, Queue, Token};

/// The entities io is asked to close and to abort, in order, left queued for the harness.
fn ends(io: &Queue<skein_io::Request>) -> (List<Token>, List<Token>) {
    let mut closes = List::with_capacity(8);
    let mut aborts = List::with_capacity(8);
    for request in io {
        match request {
            skein_io::Request::Close { entity } => closes.push(*entity).expect("fixture closes"),
            skein_io::Request::Abort { entity } => aborts.push(*entity).expect("fixture aborts"),
            skein_io::Request::Connect { .. }
            | skein_io::Request::Stream { .. }
            | skein_io::Request::Listen { .. }
            | skein_io::Request::Bind { .. }
            | skein_io::Request::Reject { .. }
            | skein_io::Request::Output { .. }
            | skein_io::Request::Spawn { .. }
            | skein_io::Request::Signal { .. }
            | skein_io::Request::Usage { .. }
            | skein_io::Request::Window { .. } => panic!("an ending asks io only to close: {request:?}"),
        }
    }
    (closes, aborts)
}

#[test]
fn abort_ends_each_call_in_flight_as_its_deadline_would_and_then_says_closed() {
    let mut h = fixtures::known();
    h.run(Duration::from_secs(5));
    h.call(1, fixtures::REPO, Op::Read(Read::Settings));
    h.run(Duration::ZERO);
    h.connected(fixtures::FORGE);
    h.call(2, fixtures::REPO, Op::Read(Read::Settings));
    h.run(Duration::ZERO);
    assert_eq!(h.peers.len(), 2, "one connection per call");
    h.send(Request::Abort);
    let mut ended = List::with_capacity(2);
    for event in &h.up {
        match event {
            Event::Answered(answer) => ended.push(answer.call.raw()).expect("two calls"),
            other @ (Event::Installation { .. }
            | Event::Fact { .. }
            | Event::GitToken { .. }
            | Event::Hint(_)
            | Event::Listening { .. }
            | Event::ListenerLost
            | Event::Connected { .. }
            | Event::Stopped { .. }
            | Event::Closed) => panic!("only the calls' terminals before io settles: {other:?}"),
        }
    }
    assert_eq!(ended.as_slice(), [1, 2], "each call ends at once, once");
    let (closes, aborts) = ends(&h.io);
    assert!(closes.is_empty(), "nothing closes gracefully");
    assert_eq!(aborts.len(), 2, "both connections aborted");
    assert_eq!(h.component.next_deadline(), None, "no deadline while io settles");
    h.run(Duration::ZERO);
    assert!(h.closed, "closed once io settled");
    assert!(h.peers.is_empty(), "no socket owned");
    let written = Answered { call: Token::new(1), cost: 1, free: 0, rate: None, result: Err(api::Error::Timeout) };
    let connecting =
        Answered { call: Token::new(2), cost: 0, free: 0, rate: None, result: Err(api::Error::Unavailable) };
    assert_eq!(h.answered(1), Some(&written), "a written call may have reached the forge");
    assert_eq!(h.answered(2), Some(&connecting), "a connecting call never reached it");
    h.send(Request::Close);
    h.send(Request::Abort);
    h.run(Duration::from_secs(20));
    assert_eq!(h.io.len(), 0, "nothing after closed");
    assert_eq!(h.answers.len(), 2, "one terminal per call");
}

#[test]
fn abort_after_close_turns_the_graceful_close_into_an_abort() {
    let mut h = fixtures::known();
    h.run(Duration::ZERO);
    h.send(Request::Close);
    let (closes, aborts) = ends(&h.io);
    assert_eq!(closes.len(), 1, "the idle start connection closes gracefully");
    assert!(aborts.is_empty(), "nothing in flight");
    assert!(h.up.is_empty(), "close waits for io");
    let graceful = *closes.as_slice().first().expect("a graceful close");
    h.send(Request::Abort);
    let (_, aborts) = ends(&h.io);
    assert_eq!(aborts.as_slice(), [graceful], "the graceful close aborted");
    assert!(h.up.is_empty(), "abort waits for io too");
    h.send(Request::Abort);
    h.send(Request::Close);
    assert_eq!(h.io.len(), 2, "a second abort or close changes nothing");
    h.run(Duration::ZERO);
    assert!(h.closed, "closed once io settled");
    assert_eq!(h.component.next_deadline(), None);
}

#[test]
fn close_after_abort_changes_nothing() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Settings));
    h.run(Duration::ZERO);
    h.send(Request::Abort);
    let events = h.up.len();
    let requests = h.io.len();
    assert_eq!(events, 1, "the call's terminal");
    let (closes, aborts) = ends(&h.io);
    assert!(closes.is_empty(), "nothing closes gracefully");
    assert_eq!(aborts.len(), 1, "the call's connection, the start's it took over");
    h.send(Request::Close);
    assert_eq!(h.up.len(), events, "no event");
    assert_eq!(h.io.len(), requests, "no request to io");
    h.run(Duration::ZERO);
    assert!(h.closed, "closed once io settled");
    assert!(h.answered(1).is_some(), "the call ended once");
    assert_eq!(h.component.next_deadline(), None);
}

#[test]
fn abort_when_idle_is_closed_at_once_once_io_settles() {
    let mut h = fixtures::known();
    h.run(Duration::from_secs(5));
    assert!(h.peers.is_empty(), "the idle connection closed at idle");
    h.send(Request::Abort);
    assert!(h.io.is_empty(), "nothing to abort");
    assert!(matches_closed(h.up.pop()), "closed at once");
    assert!(h.up.is_empty());
    let mut h = fixtures::known();
    h.run(Duration::ZERO);
    h.send(Request::Abort);
    let (closes, aborts) = ends(&h.io);
    assert!(closes.is_empty(), "the idle connection is not closed gracefully");
    assert_eq!(aborts.len(), 1, "the idle connection aborted");
    assert!(h.up.is_empty(), "closed waits for io");
    h.run(Duration::ZERO);
    assert!(h.closed, "closed once io settled");
    assert_eq!(h.component.next_deadline(), None);
}

fn matches_closed(event: Option<Event>) -> bool {
    match event {
        Some(Event::Closed) => true,
        Some(
            Event::Answered(_)
            | Event::Installation { .. }
            | Event::Fact { .. }
            | Event::GitToken { .. }
            | Event::Hint(_)
            | Event::Listening { .. }
            | Event::ListenerLost
            | Event::Connected { .. }
            | Event::Stopped { .. },
        )
        | None => false,
    }
}
