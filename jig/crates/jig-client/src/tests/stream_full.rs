use super::{Client, fixtures};
use crate::{Event, Request, StreamBound, StreamFull, Streamed, With};
use skein_lib::{Secret, Time, Token};

const FULL: StreamFull = StreamFull { bound: StreamBound::Component, size: 2 };
const LATEST: StreamFull = StreamFull { bound: StreamBound::Exchanges, size: 8 };

fn refused() -> Client {
    let mut c = Client::new();
    c.step(Event::SignIn { with: With::Held(Secret::new(Box::new([7; 40]))) });
    c.step(Event::Streamed { message: Streamed::Full(FULL) });
    c.step(Event::StreamGone);
    c
}

#[test]
fn a_stream_full_refusal_keeps_its_bound_through_backoff() {
    let mut c = refused();
    assert_eq!(c.machine.link().stream_full, Some(FULL));
    let until = c.machine.next_deadline().expect("dropped watches back off");
    c.fire(Time::from_nanos(until.as_nanos().checked_sub(1).unwrap()));
    assert!(c.requests().is_empty());
    assert_eq!(c.machine.link().stream_full, Some(FULL));
    c.fire(until);
    assert_eq!(c.requests(), &[Request::OpenStream { credential: Some(Secret::new(Box::new([7; 40]))) }]);
    assert_eq!(c.machine.link().stream_full, Some(FULL));
    c.step_at(until, Event::Streamed { message: Streamed::Full(LATEST) });
    c.step_at(until, Event::StreamGone);
    c.fire(c.machine.next_deadline().unwrap());
    assert_eq!(c.machine.link().stream_full, Some(LATEST));
}

#[test]
fn a_hello_clears_the_last_stream_full_refusal() {
    let mut c = refused();
    c.fire(c.machine.next_deadline().unwrap());
    c.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    assert_eq!(c.machine.link().stream_full, None);
}

#[test]
fn a_sign_in_change_clears_the_last_stream_full_refusal() {
    let mut c = refused();
    c.step(Event::SignIn { with: With::Transport });
    assert_eq!(c.machine.link().stream_full, None);
}

#[test]
fn a_stale_exchange_full_terminal_changes_nothing() {
    let mut c = Client::signed_in();
    let exchange = c.read(fixtures::TASK);
    c.step(Event::Full { exchange, full: FULL });
    assert_eq!(c.machine.link().stream_full, Some(FULL));
    c.step(Event::Full { exchange, full: LATEST });
    assert_eq!(c.machine.link().stream_full, Some(FULL));
    assert!(c.requests().is_empty());
    c.step(Event::Full { exchange: Token::new(u64::MAX), full: LATEST });
    assert_eq!(c.machine.link().stream_full, Some(FULL));
    assert!(c.requests().is_empty());
}
