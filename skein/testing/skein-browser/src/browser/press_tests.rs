//! Refusals before mouse input (skein/docs/design/browser.md, 6).
use super::{Browser, OpKind, Operation, PendingKind, Step};
use crate::boundary::{Event, Refusal};
use crate::{Limits, wire::decode::Document};
use skein_lib::{Env, Time, Token, Wall};

fn refused(step: Step, error: &[u8]) -> Refusal {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: Limits::default() };
    let mut browser = Browser::new(Token::new(1), &env.limits);
    let op = Token::new(4);
    assert!(browser.insert_op(
        Operation {
            token: op,
            page: Token::new(3),
            kind: OpKind::Press { node: 7, point: None, at: None, hit: None },
            due: None,
            warned: false,
        },
        env.limits.ops,
    ));
    let document = Document::parse(error, env.limits.message).expect("CDP error");
    browser.command_error(&env, b"DOM.scrollIntoViewIfNeeded", PendingKind::Step(op, step), document.root());
    let Some(Event::Refused { op: got, why, .. }) = browser.events.pop_front() else { panic!("press refused") };
    assert_eq!(got, op);
    assert!(browser.op(op).is_none());
    assert!(browser.events.is_empty(), "one terminal");
    assert!(browser.outgoing.is_empty(), "no mouse input after refusal");
    why
}

#[test]
fn a_detached_node_at_scroll_is_gone_before_mouse_input() {
    let error = br#"{"code":-32000,"message":"Node is detached from document"}"#;
    assert_eq!(refused(Step::Scroll, error), Refusal::Gone);
}

#[test]
fn a_press_keeps_protocol_errors_and_never_retries_mouse_input() {
    let error = br#"{"code":-32602,"message":"Invalid parameters"}"#;
    for step in [
        Step::Scroll,
        Step::Accessibility,
        Step::Quads,
        Step::Metrics,
        Step::Hit,
        Step::Describe,
        Step::MouseMove,
        Step::MouseDown,
        Step::MouseUp,
    ] {
        assert_eq!(refused(step, error), Refusal::Protocol, "{step:?}");
    }
    let error = br#"{"code":-32000,"message":"No node found for given backend id"}"#;
    for step in [Step::MouseMove, Step::MouseDown, Step::MouseUp] {
        assert_eq!(refused(step, error), Refusal::Protocol, "{step:?}");
    }
}
