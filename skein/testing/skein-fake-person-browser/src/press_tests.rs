//! Semantic presses recover a replaced control (skein/docs/design/ui-web.md, 14.2).
use crate::BrowserFace;
use crate::tests::{face, found, seen};
use skein_browser::boundary::{Event, Refusal, Request};
use skein_fake_person::{Doing, Face, Find, Found};
use skein_lib::{Time, Token};
use skein_ui::{Act, NodeId, Role};

fn pressing() -> (BrowserFace<'static>, Token) {
    let mut face = face();
    let find = Find::exactly_named(Role::Button, b"Sign out");
    face.find(&find);
    found(&mut face, Some(seen(b"")));
    assert_eq!(face.find(&find), Found::Node(NodeId(0)));
    face.perform(Doing::Act { node: NodeId(0), act: Act::Press }).expect("press");
    let Request::Press { op, .. } = face.requests().next().expect("press") else { panic!("press request") };
    (face, op)
}

#[test]
fn a_press_finds_a_replaced_control_again_before_sending_input() {
    let (mut face, op) = pressing();
    face.event(&Event::Refused { op, why: Refusal::Gone, detail: Box::default() });
    assert!(face.pending(), "the semantic press still waits for its control");
    assert!(face.failure().is_none());
    face.tick(Time::ZERO.saturating_add(skein_browser::Limits::default().poll));
    let Request::Find { op: find_op, query, .. } = face.requests().next().expect("fresh find") else {
        panic!("find request")
    };
    assert_eq!(query.role.as_ref(), b"button");
    assert_eq!(query.name.as_ref(), b"Sign out");
    assert_eq!(query.matching, skein_browser::boundary::Match::Exact);
    let mut nodes = skein_lib::List::with_capacity(1);
    nodes.push(skein_browser::boundary::Seen { node: 42, ..seen(b"") }).expect("room");
    face.event(&Event::Found { op: find_op, seen: nodes, more: 0 });
    let Request::Press { op: next, node, .. } = face.requests().next().expect("replacement press") else {
        panic!("press request")
    };
    assert_eq!(node, 42);
    assert_ne!(next, op);
    face.event(&Event::Done { op });
    assert!(face.pending(), "a stale terminal cannot complete the replacement");
    face.event(&Event::Done { op: next });
    assert!(!face.pending());
    assert!(face.failure().is_none());
}

#[test]
fn a_press_keeps_its_deadline_across_repeated_replacements() {
    // Four poll intervals fit within the original answer deadline
    // (skein/docs/design/browser.md, 6).
    const REPLACEMENTS: u64 = 4;
    let (mut face, mut op) = pressing();
    let limits = skein_browser::Limits::default();
    for replacement in 1..=REPLACEMENTS {
        face.event(&Event::Refused { op, why: Refusal::Gone, detail: Box::default() });
        face.tick(Time::ZERO.saturating_add(limits.poll.saturating_mul(replacement)));
        assert!(face.pending());
        found(&mut face, Some(seen(b"")));
        let Request::Press { op: next, .. } = face.requests().next().expect("retry press") else {
            panic!("press request")
        };
        op = next;
    }
    face.event(&Event::Refused { op, why: Refusal::Gone, detail: Box::default() });
    face.tick(Time::ZERO.saturating_add(limits.answer));
    assert!(!face.pending());
    assert_eq!(face.refusal(), Some(Refusal::Gone));
    assert_eq!(face.failure().as_deref(), Some("browser refused act: Gone (press deadline passed)"));
    assert!(face.requests().next().is_none());
}

#[test]
fn a_press_keeps_other_refusals_and_a_missing_replacement() {
    for why in [Refusal::Protocol, Refusal::Covered, Refusal::Hidden, Refusal::Disabled, Refusal::Crashed] {
        let (mut face, op) = pressing();
        face.event(&Event::Refused { op, why, detail: Box::default() });
        assert!(!face.pending());
        assert_eq!(face.refusal(), Some(why));
        assert!(face.failure().is_some());
        assert!(face.requests().next().is_none());
    }
    let (mut face, op) = pressing();
    face.event(&Event::Refused { op, why: Refusal::Gone, detail: Box::default() });
    face.tick(Time::ZERO.saturating_add(skein_browser::Limits::default().poll));
    found(&mut face, None);
    assert!(face.pending(), "a missing replacement waits until the original deadline");
    face.tick(Time::ZERO.saturating_add(skein_browser::Limits::default().answer));
    assert!(!face.pending());
    assert_eq!(face.refusal(), Some(Refusal::Gone));
    assert!(face.failure().expect("missing control").contains("Gone"));
    assert!(face.requests().next().is_none());
    let (mut face, op) = pressing();
    face.event(&Event::Refused { op, why: Refusal::Gone, detail: Box::default() });
    face.tick(Time::ZERO.saturating_add(skein_browser::Limits::default().poll));
    let Request::Find { op, .. } = face.requests().next().expect("retry query") else { panic!("find") };
    face.event(&Event::Refused { op, why: Refusal::Protocol, detail: Box::default() });
    assert!(!face.pending());
    assert_eq!(face.refusal(), Some(Refusal::Protocol));
    assert!(face.failure().expect("query failed").contains("Protocol"));
}

#[test]
fn a_replacement_search_keeps_the_scope_and_match_number() {
    let mut face = face();
    let scope = skein_fake_person::Target::ExactRole { role: Role::Region, name: b"Accounts".as_slice().into() };
    let find = Find::exactly_named(Role::Button, b"Sign out").within(Box::new([scope])).nth(1);
    face.find(&find);
    found(&mut face, Some(seen(b"")));
    let Request::Find { op, .. } = face.requests().next().expect("scoped find") else { panic!("find") };
    let mut nodes = skein_lib::List::with_capacity(2);
    for node in [10, 11] {
        nodes.push(skein_browser::boundary::Seen { node, ..seen(b"") }).expect("room");
    }
    face.event(&Event::Found { op, seen: nodes, more: 0 });
    let Found::Node(node) = face.find(&find) else { panic!("second control") };
    face.perform(Doing::Act { node, act: Act::Press }).expect("press");
    let Request::Press { op, node, .. } = face.requests().next().expect("press") else { panic!("press") };
    assert_eq!(node, 11);
    face.event(&Event::Refused { op, why: Refusal::Gone, detail: Box::default() });
    face.tick(Time::ZERO.saturating_add(skein_browser::Limits::default().poll));
    let Request::Find { op, query, .. } = face.requests().next().expect("new scope") else { panic!("find") };
    assert_eq!(query.name.as_ref(), b"Accounts");
    assert_eq!(query.within, None);
    let mut nodes = skein_lib::List::with_capacity(1);
    nodes.push(skein_browser::boundary::Seen { node: 20, ..seen(b"") }).expect("room");
    face.event(&Event::Found { op, seen: nodes, more: 0 });
    let Request::Find { op, query, .. } = face.requests().next().expect("new scoped find") else { panic!("find") };
    assert_eq!(query.within, Some(20));
    assert_eq!(query.name.as_ref(), b"Sign out");
    let mut nodes = skein_lib::List::with_capacity(2);
    for node in [21, 22] {
        nodes.push(skein_browser::boundary::Seen { node, ..seen(b"") }).expect("room");
    }
    face.event(&Event::Found { op, seen: nodes, more: 0 });
    let Request::Press { node, .. } = face.requests().next().expect("second replacement") else { panic!("press") };
    assert_eq!(node, 22);
}
