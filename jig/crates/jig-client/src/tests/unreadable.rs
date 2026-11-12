use super::fixtures::{self, Client, STOP, TASK};
use crate::*;
use jig_client_protocol_wire::v20 as w;
use skein_lib::{Duration, Reader, Time, Writer};

fn starting() -> Client {
    let mut c = Client::new();
    c.step(Event::SignIn { with: With::Held(skein_lib::Secret::new(Box::new([7; 40]))) });
    c
}

fn no_sends(c: &Client) {
    for request in c.requests() {
        if let Request::OpenStream { .. } | Request::Exchange { .. } = request {
            panic!("out of date sends nothing: {request:?}");
        }
    }
}

fn unreadable(c: &mut Client) {
    assert_eq!(c.machine.link().state, Link::OutOfDate);
    assert!(c.machine.link().unreadable);
    let mut told = false;
    let mut closed = false;
    for request in c.requests() {
        if let Request::Told { told: Told::Link { link: Link::OutOfDate } } = request {
            told = true;
        }
        if let Request::CloseStream = request {
            closed = true;
        }
    }
    assert!(told);
    assert!(closed);
    no_sends(c);
    c.step(Event::Send { name: None, key: STOP, request: fixtures::stop() });
    assert!(c.machine.pending(STOP).is_some());
    assert!(!fixtures::saved(c.requests()).is_empty(), "the request is kept for a new client");
    no_sends(c);
    c.step(Event::Stored);
    no_sends(c);
    c.step(Event::Watch { handle: TASK, subject: fixtures::task() });
    no_sends(c);
    let past_backoffs = Time::ZERO.saturating_add(fixtures::limits().domain.backoff.most.saturating_mul(4));
    c.fire(past_backoffs);
    no_sends(c);
    assert!(c.machine.pending(STOP).is_some());
    assert_eq!(c.machine.link().state, Link::OutOfDate);
    assert!(c.machine.link().unreadable);
    assert_eq!(c.machine.next_deadline(), None);
}

#[test]
fn a_me_delivery_whose_document_is_cut_short_leaves_the_client_unreadable_sending_nothing() {
    let mut c = starting();
    c.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    let watch = c.opened_watch();
    c.stream(fixtures::frame(watch));
    c.step(Event::Stored);
    let bytes = fixtures::frame(watch);
    let w::StreamEvent::Delivery(delivery) = w::StreamEvent::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap()
    else {
        panic!("me delivery fixture")
    };
    let mut parts = delivery.into_parts();
    let mut document = parts.document.into_parts();
    document.record = Box::from(document.record.split_last().unwrap().1);
    parts.document = w::Owned::new(&w::CEILINGS, document).unwrap();
    c.stream(fixtures::event(&w::StreamEvent::Delivery(w::Delivery::new(&w::CEILINGS, parts).unwrap())));
    unreadable(&mut c);
}

#[test]
fn an_unknown_stream_event_leaves_the_client_unreadable() {
    let mut c = Client::signed_in();
    c.stream(Box::new([255]));
    unreadable(&mut c);
}

#[test]
fn a_hello_cut_short_leaves_the_client_unreadable() {
    let mut c = Client::signed_in();
    c.step(Event::StreamGone);
    c.fire(c.machine.next_deadline().unwrap());
    let bytes = fixtures::hello(jig_client_protocol_wire::VERSION);
    c.stream(Box::from(bytes.split_last().unwrap().1));
    unreadable(&mut c);
}

#[test]
fn an_undecodable_watch_opening_answer_leaves_the_client_unreadable() {
    let mut c = Client::signed_in();
    c.step(Event::Watch { handle: TASK, subject: fixtures::task() });
    c.step(Event::Reply { exchange: c.exchange(), message: Box::new([255]) });
    unreadable(&mut c);
}

#[test]
fn an_undecodable_watch_closing_answer_leaves_the_client_unreadable() {
    let mut c = Client::signed_in();
    c.step(Event::Watch { handle: TASK, subject: fixtures::task() });
    let exchange = c.exchange();
    let answer = w::OpenAnswer::Admitted;
    let mut bytes = Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut bytes).unwrap();
    c.step(Event::Reply { exchange, message: bytes.finish() });
    c.step(Event::Unwatch { handle: TASK });
    c.step(Event::Reply { exchange: c.exchange(), message: Box::new([255]) });
    unreadable(&mut c);
}

#[test]
fn silence_past_twice_the_heartbeat_is_offline_not_unreadable_and_reopens() {
    let mut c = Client::signed_in();
    let past_heartbeat = Time::ZERO
        .saturating_add(fixtures::limits().domain.heartbeat.saturating_mul(2))
        .saturating_add(Duration::from_nanos(1));
    c.fire(past_heartbeat);
    assert_eq!(c.machine.link().state, Link::Offline);
    assert!(!c.machine.link().unreadable);
    c.fire(past_heartbeat.saturating_add(fixtures::limits().domain.backoff.most));
    let mut opened = false;
    for request in c.requests() {
        if let Request::OpenStream { .. } = request {
            opened = true;
        }
    }
    assert!(opened);
}

#[test]
fn an_undecodable_read_answer_stays_malformed_not_unreadable() {
    let mut c = Client::signed_in();
    let exchange = c.read(TASK);
    c.step(Event::Reply { exchange, message: Box::new([255]) });
    assert_eq!(
        fixtures::pages(c.requests()).as_slice(),
        &[(TASK, Readout::Jig(contract::ReadResult::None(contract::ReadNone::Malformed)))]
    );
    assert_eq!(c.machine.link().state, Link::Live);
    assert!(!c.machine.link().unreadable);
}
