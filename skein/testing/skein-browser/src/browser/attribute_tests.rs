//! A fake CDP peer for an attribute read whose element a render replaces.
use super::super::cdp_tests::{PAGE, Peer, ROOT};
use crate::Limits;
use crate::boundary::{Event, Read, Refusal, Request};
use skein_lib::Token;

const READ: Token = Token::new(4);
const SELECTOR: &[u8] = br#"{"nodeId":1,"selector":"button"}"#;
const MISSING: &str = r#"{"code":-32000,"message":"Could not find node with given id"}"#;

fn start() -> Peer {
    let mut peer = Peer::opened(Limits::default());
    peer.ask(Request::Attribute {
        page: PAGE,
        op: READ,
        selector: b"button".as_slice().into(),
        name: b"aria-pressed".as_slice().into(),
    });
    peer
}

fn selected(peer: &mut Peer) {
    peer.reply(b"DOM.getDocument", ROOT, "result", r#"{"root":{"nodeId":1}}"#);
    peer.reply(b"DOM.querySelector", SELECTOR, "result", r#"{"nodeId":7}"#);
}

fn replaced(peer: &mut Peer) {
    selected(peer);
    peer.reply(b"DOM.getAttributes", br#"{"nodeId":7}"#, "error", MISSING);
    assert!(peer.browser.events.is_empty(), "a replaced element is no terminal");
}

fn poll(peer: &mut Peer) {
    peer.env.now = peer.env.now.saturating_add(peer.env.limits.poll);
    peer.tick();
}

#[test]
fn a_replaced_element_is_read_again_from_the_document() {
    let mut peer = start();
    replaced(&mut peer);
    assert!(peer.browser.sendable().is_none(), "the read waits a poll");
    poll(&mut peer);
    selected(&mut peer);
    peer.reply(b"DOM.getAttributes", br#"{"nodeId":7}"#, "result", r#"{"attributes":["aria-pressed","true"]}"#);
    let Some(Event::Attribute { op: READ, read: Read::Value(value) }) = peer.browser.events.pop_front() else {
        panic!("the read's value")
    };
    assert_eq!(value.as_ref(), b"true");
}

#[test]
fn a_document_replaced_before_the_selector_is_read_again() {
    let mut peer = start();
    peer.reply(b"DOM.getDocument", ROOT, "result", r#"{"root":{"nodeId":1}}"#);
    peer.reply(b"DOM.querySelector", SELECTOR, "error", MISSING);
    assert!(peer.browser.events.is_empty(), "a replaced document is no terminal");
    poll(&mut peer);
    selected(&mut peer);
    peer.reply(b"DOM.getAttributes", br#"{"nodeId":7}"#, "result", r#"{"attributes":[]}"#);
    let Some(Event::Attribute { op: READ, read: Read::NoAttribute }) = peer.browser.events.pop_front() else {
        panic!("the read's answer")
    };
}

#[test]
fn an_element_replaced_past_the_deadline_is_gone() {
    let mut peer = start();
    selected(&mut peer);
    peer.env.now = peer.env.now.saturating_add(peer.env.limits.answer);
    peer.reply(b"DOM.getAttributes", br#"{"nodeId":7}"#, "error", MISSING);
    let Some(Event::Refused { op: READ, why: Refusal::Gone, .. }) = peer.browser.events.pop_front() else {
        panic!("the read's refusal")
    };
    assert!(peer.browser.op(READ).is_none());
}
