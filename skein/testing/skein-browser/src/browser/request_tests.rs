//! Opt-in request reports on synthetic CDP events (testing/browser.md, 9).
#![expect(clippy::disallowed_macros, reason = "synthetic CDP events use fixed formatted fixtures")]
use super::{Browser, Phase};
use crate::Limits;
use crate::boundary::{Event, Request, Trouble};
use crate::wire::decode::Document;
use skein_lib::{Env, Time, Token, Wall};

const PERSON: Token = Token::new(2);
const PAGE: Token = Token::new(3);
const URL: &[u8] = b"http://127.0.0.1:9/s?x=1";
struct Peer {
    browser: Browser,
    env: Env<Limits>,
    network: usize,
}
impl Peer {
    fn open(requests: bool) -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: Limits::default() };
        let mut browser = Browser::new(Token::new(1), &env.limits);
        browser.phase = Phase::Ready;
        browser.request(&env, Request::Person { person: PERSON, downloads: None });
        let mut peer = Self { browser, env, network: 0 };
        peer.settle();
        peer.browser.request(&peer.env, Request::Page { person: PERSON, page: PAGE, url: URL.into(), requests });
        peer.settle();
        peer.notify("Page.loadEventFired", "{}");
        let Some(Event::Opened { page: PAGE }) = peer.browser.events.pop_front() else { panic!("Opened") };
        assert!(peer.browser.events.is_empty());
        peer
    }
    fn settle(&mut self) {
        while let Some(command) = self.browser.outgoing.pop_front() {
            let doc = Document::parse(
                command.bytes.strip_suffix(&[0]).expect("synthetic CDP fixture"),
                self.env.limits.command,
            )
            .expect("synthetic CDP fixture");
            let root = doc.root();
            let method = root.get(b"method").expect("synthetic CDP fixture").text().expect("synthetic CDP fixture");
            let result = match method.as_ref() {
                b"Target.createBrowserContext" => r#"{"browserContextId":"ctx"}"#,
                b"Target.createTarget" => r#"{"targetId":"target"}"#,
                b"Target.attachToTarget" => r#"{"sessionId":"session"}"#,
                b"Page.getFrameTree" => r#"{"frameTree":{"frame":{"id":"main"}}}"#,
                b"Network.enable" => {
                    let params = root.get(b"params").expect("Network parameters");
                    for name in [b"maxTotalBufferSize".as_slice(), b"maxResourceBufferSize", b"maxPostDataSize"] {
                        assert_eq!(params.get(name).expect("no body buffering").u64(), Some(0));
                    }
                    self.network = self.network.saturating_add(1);
                    "{}"
                }
                b"Page.navigate" => {
                    assert_eq!(
                        self.network,
                        usize::from(self.browser.page_requests(PAGE)),
                        "Network precedes navigation"
                    );
                    "{}"
                }
                _ => "{}",
            };
            let id = root.get(b"id").expect("command id").u64().expect("integer id");
            self.browser.message(&self.env, format!(r#"{{"id":{id},"result":{result}}}"#).as_bytes());
        }
    }
    fn notify(&mut self, method: &str, params: &str) {
        self.browser.message(
            &self.env,
            format!(r#"{{"method":"{method}","sessionId":"session","params":{params}}}"#).as_bytes(),
        );
    }
    fn sent(&mut self, id: &str, load: &str, frame: &str, kind: &str, redirect: bool) {
        let redirect = if redirect { r#", "redirectResponse":{}"# } else { "" };
        self.notify("Network.requestWillBeSent", &format!(r#"{{"requestId":"{id}","loaderId":"{load}","frameId":"{frame}","type":"{kind}","request":{{"method":"GET","url":"http://127.0.0.1:9/s?x=1"}}{redirect}}}"#));
    }
    fn answer(&mut self, method: &str, id: &str) {
        self.notify(method, &format!(r#"{{"requestId":"{id}"}}"#));
    }
    fn expect_sent(&mut self, expected_id: &[u8], expected_load: &[u8], expected_document: bool, expected_url: &[u8]) {
        let Some(Event::Sent { page, id, load, document, method, url }) = self.browser.events.pop_front() else {
            panic!("Sent")
        };
        assert_eq!(page, PAGE);
        assert_eq!(id.as_ref(), expected_id);
        assert_eq!(load.as_ref(), expected_load);
        assert_eq!(document, expected_document);
        assert_eq!(method.as_ref(), b"GET");
        assert_eq!(url.as_ref(), expected_url);
    }
    fn expect_answer(&mut self, expected: &[u8]) {
        let Some(Event::Answered { page, id }) = self.browser.events.pop_front() else { panic!("Answered") };
        assert_eq!(page, PAGE);
        assert_eq!(id.as_ref(), expected);
    }
}

#[test]
fn requests_are_enabled_before_navigation_only_where_the_page_asks() {
    for requests in [false, true] {
        let mut peer = Peer::open(requests);
        assert_eq!(peer.network, usize::from(requests));
        if !requests {
            peer.sent("7.1", "L1", "main", "Fetch", false);
            peer.answer("Network.dataReceived", "7.1");
            assert!(peer.browser.events.is_empty());
            assert_eq!(peer.browser.requests.capacity(), 0, "opt-out allocates no request table");
        }
    }
}

#[test]
fn only_main_frame_requests_are_sent_in_order_without_redirects() {
    let mut peer = Peer::open(true);
    peer.sent("L1", "L1", "main", "Document", false);
    peer.sent("child", "Lchild", "child-frame", "Fetch", false);
    peer.sent("7.1", "L1", "main", "Fetch", false);
    peer.sent("7.1", "L1", "main", "Fetch", true);
    peer.expect_sent(b"L1", b"L1", true, URL);
    peer.expect_sent(b"7.1", b"L1", false, URL);
    assert!(peer.browser.events.is_empty());
    peer.env.limits.text = 8;
    peer.sent("123456789", "abcdefghi", "main", "Fetch", false);
    peer.expect_sent(b"12345678", b"abcdefgh", false, b"http://1");
}

#[test]
fn only_the_first_body_end_or_failure_answers_a_request() {
    for first in ["Network.dataReceived", "Network.loadingFinished", "Network.loadingFailed"] {
        let mut peer = Peer::open(true);
        peer.answer(first, "unknown");
        peer.sent("7.1", "L1", "main", "Fetch", false);
        peer.expect_sent(b"7.1", b"L1", false, URL);
        peer.answer(first, "7.1");
        peer.expect_answer(b"7.1");
        peer.answer("Network.dataReceived", "7.1");
        peer.answer("Network.dataReceived", "7.1");
        peer.answer("Network.loadingFinished", "7.1");
        peer.answer("Network.loadingFailed", "7.1");
        assert!(peer.browser.events.is_empty());
        assert!(peer.browser.requests.is_empty());
    }
}

#[test]
fn past_the_request_limit_sent_precedes_trouble_and_no_answer_is_followed() {
    let mut peer = Peer::open(true);
    peer.env.limits.requests = 1;
    peer.sent("7.1", "L1", "main", "Fetch", false);
    peer.sent("7.2", "L1", "main", "Fetch", false);
    peer.expect_sent(b"7.1", b"L1", false, URL);
    peer.expect_sent(b"7.2", b"L1", false, URL);
    let Some(Event::Trouble { page, trouble, text, url }) = peer.browser.events.pop_front() else { panic!("Trouble") };
    assert_eq!(page, PAGE);
    assert_eq!(trouble, Trouble::Request);
    assert_eq!(text.as_ref(), b"requests past the limit");
    assert_eq!(url.as_ref(), URL);
    peer.answer("Network.dataReceived", "7.2");
    assert!(peer.browser.events.is_empty());
    peer.answer("Network.loadingFinished", "7.1");
    peer.expect_answer(b"7.1");
    peer.sent("7.3", "L1", "main", "Fetch", false);
    peer.expect_sent(b"7.3", b"L1", false, URL);
    peer.answer("Network.loadingFailed", "7.3");
    peer.expect_answer(b"7.3");
    assert!(peer.browser.events.is_empty());
}

#[test]
fn closing_a_page_drops_its_requests_and_reports_nothing_after_closed() {
    let mut peer = Peer::open(true);
    peer.sent("7.1", "L1", "main", "Fetch", false);
    peer.expect_sent(b"7.1", b"L1", false, URL);
    peer.browser.close_page_local(PAGE);
    let Some(Event::Closed { owner: PAGE }) = peer.browser.events.pop_front() else { panic!("Closed") };
    assert!(peer.browser.requests.is_empty());
    peer.browser.reclaim();
    peer.answer("Network.dataReceived", "7.1");
    peer.sent("7.2", "L1", "main", "Fetch", false);
    peer.answer("Network.loadingFinished", "7.2");
    assert!(peer.browser.events.is_empty());
}
