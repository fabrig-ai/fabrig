//! A fake CDP peer for a tab list.
use super::super::{Browser, Page, Person, Phase};
use crate::boundary::{Event, Refusal, Request};
use crate::{Limits, wire::decode::Document};
use alloc::boxed::Box;
use skein_lib::{Env, Time, Token, Wall};

const PAGE: Token = Token::new(3);
const READ: Token = Token::new(4);
struct Peer {
    browser: Browser,
    env: Env<Limits>,
}
impl Peer {
    fn new() -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: Limits::default() };
        let mut browser = Browser::new(Token::new(1), &env.limits);
        browser.phase = Phase::Ready;
        for (token, target) in [(PAGE, b"target".as_slice()), (Token::new(5), b"owned")] {
            assert!(browser.insert_page(
                Page {
                    token,
                    person: Token::new(2),
                    target: Some(Box::from(target)),
                    session: Some(Box::from(b"session".as_slice())),
                    initial_url: Box::from(b"about:blank".as_slice()),
                    frame: Some(Box::from(b"main".as_slice())),
                    navigating: false,
                    opening: false,
                    load: None,
                    load_due: None,
                    closed: false,
                },
                env.limits.pages
            ));
        }
        browser.persons.push(Some(Person {
            token: Token::new(2),
            context: Some(b"context".as_slice().into()),
            downloads: None,
            ready: true,
            closing: false,
            closed: false,
        }));
        let read = Request::Tabs { page: PAGE, op: READ };
        browser.request(&env, read);
        Self { browser, env }
    }
    fn reply(&mut self, method: &[u8], params: &[u8], field: &[u8], result: &[u8]) {
        let index = self.browser.sendable().expect("next command is sendable");
        let command = self.browser.outgoing.remove(index).expect("next command");
        let bytes = command.bytes.strip_suffix(&[0]).expect("CDP delimiter");
        let document = Document::parse(bytes, self.env.limits.command).expect("encoded command");
        assert_eq!(document.root().get(b"method").expect("method").text().expect("method text").as_ref(), method);
        assert!(document.root().get(b"sessionId").is_none(), "browser-level command");
        assert_eq!(document.root().get(b"params").expect("parameters").raw(), params);
        let mut reply = Vec::new();
        reply.extend_from_slice(b"{\"id\":");
        reply.extend_from_slice(document.root().get(b"id").expect("id").raw());
        reply.extend_from_slice(b",\"");
        reply.extend_from_slice(field);
        reply.extend_from_slice(b"\":");
        reply.extend_from_slice(result);
        reply.extend_from_slice(b"}");
        self.browser.message(&self.env, &reply);
    }
}
fn reply(peer: &mut Peer, result: &[u8]) {
    peer.reply(b"Target.getTargets", b"{}", b"result", result);
}
#[test]
fn tabs_drop_other_persons_workers_and_every_owned_page() {
    let mut peer = Peer::new();
    reply(&mut peer, br#"{"targetInfos":[
        {"targetId":"other","type":"page","browserContextId":"other"},
        {"targetId":"worker","type":"service_worker","browserContextId":"context"},
        {"targetId":"target","type":"page","browserContextId":"context"},
        {"targetId":"owned","type":"page","browserContextId":"context"},
        {"targetId":"tab","type":"page","browserContextId":"context","url":"about:blank","title":"Opening","canAccessOpener":true},
        {"targetId":"cut","type":"page","browserContextId":"context","url":"https://example.test/","title":"Cut"}
    ]}"#);
    let Some(Event::Tabs { op: READ, tabs, more: 0 }) = peer.browser.events.pop_front() else { panic!("tab list") };
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs.get(0).expect("first").url.as_ref(), b"about:blank");
    assert_eq!(tabs.get(0).expect("first").title.as_ref(), b"Opening");
    assert!(tabs.get(0).expect("first").opener);
    assert!(!tabs.get(1).expect("second").opener, "absent canAccessOpener is false");
    assert!(peer.browser.op(READ).is_none());
}
#[test]
fn malformed_tabs_finish_protocol() {
    for result in [br"{}".as_slice(), br#"{"targetInfos":{}}"#, br#"{"targetInfos":[{"targetId":"tab","type":"page","browserContextId":"context","title":"Opening"}]}"#, br#"{"targetInfos":[{"targetId":"tab","type":"page","browserContextId":"context","url":"x","title":"Opening","canAccessOpener":1}]}"#] {
        let mut peer = Peer::new();
        reply(&mut peer, result);
        let Some(Event::Refused { op: READ, why: Refusal::Protocol, detail }) = peer.browser.events.pop_front() else { panic!("protocol refusal") };
        assert_eq!(detail.as_ref(), b"Target.getTargets: malformed reply");
        assert!(peer.browser.op(READ).is_none());
    }
}

fn tab_reply(url: &str, title: &str) -> Vec<u8> {
    let mut reply = Vec::new();
    reply.extend_from_slice(br#"{"targetInfos":[{"targetId":"tab","type":"page","browserContextId":"context","url":""#);
    reply.extend_from_slice(url.as_bytes());
    reply.extend_from_slice(br#"","title":""#);
    reply.extend_from_slice(title.as_bytes());
    reply.extend_from_slice(br#"","canAccessOpener":false}]}"#);
    reply
}
#[test]
fn tabs_keep_text_at_the_bound_and_refuse_either_field_past_it() {
    for (url, title, refused) in [("123456", "123456", false), ("1234567", "123456", true), ("123456", "1234567", true)]
    {
        let mut peer = Peer::new();
        peer.env.limits.text = 6;
        reply(&mut peer, &tab_reply(url, title));
        if refused {
            let Some(Event::Refused { why: Refusal::Limit, .. }) = peer.browser.events.pop_front() else {
                panic!("overlong tab text is refused")
            };
        } else {
            let Some(Event::Tabs { tabs, more: 0, .. }) = peer.browser.events.pop_front() else {
                panic!("whole tab at text bound")
            };
            assert_eq!(tabs.get(0).unwrap().url.as_ref(), url.as_bytes());
            assert_eq!(tabs.get(0).unwrap().title.as_ref(), title.as_bytes());
        }
    }
}
#[test]
fn tabs_count_the_remainder_after_the_match_bound() {
    for kept in [0, 1, 2] {
        let mut peer = Peer::new();
        peer.env.limits.matches = kept;
        reply(
            &mut peer,
            br#"{"targetInfos":[
            {"targetId":"first","type":"page","browserContextId":"context","url":"one","title":"First"},
            {"targetId":"second","type":"page","browserContextId":"context","url":"two","title":"Second"}
        ]}"#,
        );
        let Some(Event::Tabs { tabs, more, .. }) = peer.browser.events.pop_front() else { panic!("bounded tabs") };
        assert_eq!((tabs.len(), more), (kept, 2 - kept));
        if kept > 0 {
            assert_eq!(tabs.get(0).unwrap().title.as_ref(), b"First");
        }
    }
}
