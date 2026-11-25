//! Synthetic DOM and CSS replies for the rendering reads (skein/docs/design/testing/browser.md, 9).
use super::super::cdp_tests::{PAGE, Peer, ROOT};
use crate::Limits;
use crate::boundary::{Event, Node, Property, Rect, Refusal, Request};
use skein_lib::{List, Token};

const READ: Token = Token::new(4);
const AGAIN: Token = Token::new(5);
const LEAF: Node = 106;
const TREE: &[u8] = br#"{"depth":-1,"pierce":false}"#;
const SELECTOR: &[u8] = br#"{"nodeId":1,"selector":"div, span"}"#;
const FULL_DOCUMENT: &str = r#"{"root":{"nodeType":9,"nodeId":1,"children":[
  {"nodeType":1,"nodeId":2,"backendNodeId":102,"localName":"html","children":[
    {"nodeType":1,"nodeId":3,"backendNodeId":103,"localName":"head"},
    {"nodeType":8,"nodeId":7,"backendNodeId":107,"localName":""},
    {"nodeType":1,"nodeId":4,"backendNodeId":104,"localName":"body","children":[
      {"nodeType":1,"nodeId":5,"backendNodeId":105,"localName":"div","children":[
        {"nodeType":3,"nodeId":8,"backendNodeId":108,"localName":""},
        {"nodeType":1,"nodeId":6,"backendNodeId":106,"localName":"span"}]}
    ]}
  ]}
]}}"#;
const ROOT_REPLY: &str = r#"{"root":{"nodeId":1}}"#;
const REVERSE_IDS: &str = r#"{"nodeIds":[6,5]}"#;
const DOCUMENT_ORDER: &[(Node, &[u8], u32)] =
    &[(102, b"html", 0), (103, b"head", 1), (104, b"body", 1), (105, b"div", 2), (106, b"span", 3)];
const KEPT_ELEMENTS: u32 = 2;
const OMITTED_ELEMENTS: u32 = 3; // The fixture holds five elements but only two fit.
const PROPERTIES: &str =
    r#"{"computedStyle":[{"name":"color","value":"rgb(1, 2, 3)"},{"name":"cursor","value":"wait"}]}"#;
const BORDER: &str = r#"{"model":{"content":[0,0,0,0,0,0,0,0],"border":[21.1,9.6,-4.9,9.2,-2.5,30.3,22.8,30.2]}}"#;
const BACKEND: &[u8] = br#"{"backendNodeId":106}"#;

fn listing(limits: Limits, selector: Option<&[u8]>) -> Peer {
    let mut peer = Peer::opened(limits);
    let selector = match selector {
        Some(text) => Some(text.into()),
        None => None,
    };
    peer.ask(Request::Elements { page: PAGE, op: READ, selector });
    peer.reply(b"DOM.getDocument", TREE, "result", FULL_DOCUMENT);
    peer
}

fn assert_elements(peer: &mut Peer, expected: &[(Node, &[u8], u32)], beyond: u32) {
    let Some(Event::Elements { op: READ, elements, more }) = peer.browser.events.pop_front() else {
        panic!("the elements read must answer once with its elements");
    };
    assert_eq!(elements.iter().len(), expected.len());
    for (row, &expected) in elements.iter().zip(expected) {
        assert_eq!((row.node, row.name.as_ref(), row.depth), expected);
    }
    assert_eq!(more, beyond);
    assert!(peer.browser.events.is_empty() && peer.browser.op(READ).is_none());
}

#[test]
fn elements_follow_document_order_and_skip_non_elements() {
    let mut peer = listing(Limits::default(), None);
    assert_elements(&mut peer, DOCUMENT_ORDER, 0);
}

#[test]
fn selected_elements_keep_document_order_and_original_depths() {
    let mut peer = listing(Limits::default(), Some(b"div, span"));
    peer.reply(b"DOM.querySelectorAll", SELECTOR, "result", REVERSE_IDS);
    assert_elements(&mut peer, &[(105, b"div", 2), (LEAF, b"span", 3)], 0);
}

#[test]
fn elements_beyond_the_limit_are_counted_not_kept() {
    let mut peer = listing(Limits { elements: KEPT_ELEMENTS, ..Limits::default() }, None);
    let kept = DOCUMENT_ORDER.get(..usize::try_from(KEPT_ELEMENTS).expect("fixture's limit fits")).expect("prefix");
    assert_elements(&mut peer, kept, OMITTED_ELEMENTS);
}

#[test]
fn a_bad_selector_reports_chromiums_protocol_detail() {
    let mut peer = listing(Limits::default(), Some(b"["));
    peer.reply(
        b"DOM.querySelectorAll",
        br#"{"nodeId":1,"selector":"["}"#,
        "error",
        r#"{"code":-32000,"message":"DOM Error while querying"}"#,
    );
    let Some(Event::Refused { op: READ, why: Refusal::Protocol, detail }) = peer.browser.events.pop_front() else {
        panic!("a bad selector is a protocol refusal");
    };
    assert_eq!(detail.as_ref(), b"DOM.querySelectorAll: DOM Error while querying");
}

fn start_style(peer: &mut Peer, op: Token) {
    peer.ask(Request::Style { page: PAGE, op, node: LEAF });
    peer.reply(b"DOM.getDocument", ROOT, "result", ROOT_REPLY);
}

fn reply_style(peer: &mut Peer, op: Token) -> List<Property> {
    peer.reply(b"DOM.pushNodesByBackendIdsToFrontend", br#"{"backendNodeIds":[106]}"#, "result", r#"{"nodeIds":[6]}"#);
    peer.reply(b"CSS.getComputedStyleForNode", br#"{"nodeId":6}"#, "result", PROPERTIES);
    let Some(Event::Style { op: answered, style }) = peer.browser.events.pop_front() else { panic!("computed style") };
    assert_eq!(op, answered);
    style
}

fn assert_style(style: &List<Property>, expected: &[(&[u8], &[u8])]) {
    assert_eq!(style.iter().len(), expected.len());
    for (property, &expected) in style.iter().zip(expected) {
        assert_eq!((property.name.as_ref(), property.value.as_ref()), expected);
    }
}

#[test]
fn style_enables_css_once_and_keeps_property_order() {
    let mut peer = Peer::opened(Limits::default());
    start_style(&mut peer, READ);
    peer.reply(b"CSS.enable", b"{}", "result", "{}");
    let first = reply_style(&mut peer, READ);
    start_style(&mut peer, AGAIN);
    let second = reply_style(&mut peer, AGAIN);
    let expected: &[(&[u8], &[u8])] = &[(b"color", b"rgb(1, 2, 3)"), (b"cursor", b"wait")];
    assert_style(&first, expected);
    assert_style(&second, expected);
}

#[test]
fn a_long_computed_value_is_cut_to_text_limit() {
    const TEXT_LIMIT: u32 = 6; // Fits both property names, but not the rgb value.
    let mut peer = Peer::opened(Limits { text: TEXT_LIMIT, ..Limits::default() });
    start_style(&mut peer, READ);
    peer.reply(b"CSS.enable", b"{}", "result", "{}");
    let style = reply_style(&mut peer, READ);
    assert_style(&style, &[(b"color", b"rgb(1,"), (b"cursor", b"wait")]);
}

fn read_rect(field: &str, json: &str) -> Event {
    let mut peer = Peer::opened(Limits::default());
    peer.ask(Request::Rect { page: PAGE, op: READ, node: LEAF });
    peer.reply(b"DOM.getBoxModel", BACKEND, field, json);
    peer.browser.events.pop_front().expect("the rect read's terminal")
}

#[test]
fn rect_bounds_all_four_border_points_in_whole_pixels() {
    let Event::Rect { op: READ, rect } = read_rect("result", BORDER) else { panic!("border rect") };
    assert_eq!(rect, Some(Rect { x: -4, y: 9, width: 26, height: 21 }));
}

#[test]
fn a_node_without_a_box_has_no_rect() {
    let event = read_rect("error", r#"{"code":-32000,"message":"Could not compute box model."}"#);
    let Event::Rect { op: READ, rect: None } = event else { panic!("no box: {event:?}") };
}

#[test]
fn rect_refuses_a_gone_node() {
    let event = read_rect("error", r#"{"code":-32000,"message":"No node with given id found"}"#);
    let Event::Refused { op: READ, why: Refusal::Gone, .. } = event else { panic!("stale node: {event:?}") };
}

#[test]
fn a_render_read_restarts_after_its_page_navigates() {
    let mut peer = listing(Limits::default(), Some(b"div, span"));
    let old = peer.command(b"DOM.querySelectorAll", SELECTOR);
    peer.notify(br#"{"sessionId":"session","method":"Page.frameStartedLoading","params":{"frameId":"main"}}"#);
    peer.tick();
    assert!(peer.browser.sendable().is_none(), "DOM reads wait for commit");
    peer.answer(old, "result", REVERSE_IDS);
    assert!(peer.browser.events.is_empty(), "the old document cannot answer the read");
    peer.notify(br#"{"sessionId":"session","method":"Page.loadEventFired","params":{}}"#);
    peer.reply(b"DOM.getDocument", TREE, "result", FULL_DOCUMENT);
    peer.reply(b"DOM.querySelectorAll", SELECTOR, "result", r#"{"nodeIds":[6]}"#);
    assert_elements(&mut peer, &[(LEAF, b"span", 3)], 0);
}
