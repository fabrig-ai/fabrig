//! Downloads and letter keys at the CDP boundary.
#![expect(clippy::disallowed_macros, reason = "synthetic CDP transcripts are formatted from the test's fixed fixtures")]
use super::{Browser, Phase};
use crate::boundary::{Event, Key, Refusal, Request, Trouble};
use crate::{Limits, wire::decode::Document};
use alloc::{boxed::Box, vec::Vec};
use skein_lib::{Env, Time, Token, Wall};

const PERSON: Token = Token::new(2);
const PAGE: Token = Token::new(3);
const OP: Token = Token::new(4);
struct Peer {
    browser: Browser,
    env: Env<Limits>,
}
impl Peer {
    fn new(directory: Option<&[u8]>) -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: Limits::default() };
        let mut browser = Browser::new(Token::new(1), &env.limits);
        browser.phase = Phase::Ready;
        browser.request(
            &env,
            Request::Person {
                person: PERSON,
                downloads: match directory {
                    Some(directory) => Some(Box::from(directory)),
                    None => None,
                },
            },
        );
        Self { browser, env }
    }
    fn reply(&mut self, method: &[u8], expected: &[u8], result: &[u8]) {
        let command = self.browser.outgoing.pop_front().expect("command");
        let document =
            Document::parse(command.bytes.strip_suffix(&[0]).expect("NUL"), self.env.limits.command).expect("CDP");
        assert_eq!(document.root().get(b"method").unwrap().text().unwrap().as_ref(), method);
        assert_eq!(document.root().get(b"params").unwrap().raw(), expected);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"{\"id\":");
        bytes.extend_from_slice(document.root().get(b"id").unwrap().raw());
        bytes.extend_from_slice(b",\"result\":");
        bytes.extend_from_slice(result);
        bytes.push(b'}');
        self.browser.message(&self.env, &bytes);
    }
    fn context(&mut self, params: &[u8]) {
        self.reply(b"Target.createBrowserContext", b"{}", br#"{"browserContextId":"ctx"}"#);
        self.reply(b"Browser.setDownloadBehavior", params, b"{}");
    }
    fn open() -> Self {
        let mut peer = Self::new(Some(b"/downloads"));
        peer.context(
            br#"{"browserContextId":"ctx","behavior":"allowAndName","eventsEnabled":true,"downloadPath":"/downloads"}"#,
        );
        peer.browser.request(
            &peer.env,
            Request::Page { person: PERSON, page: PAGE, url: Box::from(b"about:blank".as_slice()) },
        );
        peer.reply(
            b"Target.createTarget",
            br#"{"url":"about:blank","browserContextId":"ctx"}"#,
            br#"{"targetId":"target"}"#,
        );
        peer.reply(b"Target.attachToTarget", br#"{"targetId":"target","flatten":true}"#, br#"{"sessionId":"session"}"#);
        // These tests start with an attached page and exercise only download/key commands.
        peer.browser.outgoing.clear();
        peer.browser.pending.clear();
        peer.browser.page_mut(PAGE).unwrap().opening = false;
        peer
    }
    fn downloaded(&mut self, expected_name: &[u8], expected_path: &[u8]) {
        let Some(Event::Downloaded { person, name, path }) = self.browser.events.pop_front() else {
            panic!("completed download")
        };
        assert_eq!(person, PERSON);
        assert_eq!(name.as_ref(), expected_name);
        assert_eq!(path.as_ref(), expected_path);
    }
    fn trouble(&mut self, expected: &[u8]) {
        let Some(Event::Trouble { page, trouble, text }) = self.browser.events.pop_front() else {
            panic!("download trouble")
        };
        assert_eq!(page, PAGE);
        assert_eq!(trouble, Trouble::Download);
        assert_eq!(text.as_ref(), expected);
    }
    fn closed(&mut self, expected: Token) {
        let Some(Event::Closed { owner }) = self.browser.events.pop_front() else { panic!("closed") };
        assert_eq!(owner, expected);
    }
    fn begin(&mut self, frame: &str, guid: &str, name: &str) {
        self.browser.message(&self.env, format!(r#"{{"method":"Browser.downloadWillBegin","params":{{"frameId":"{frame}","guid":"{guid}","suggestedFilename":"{name}"}}}}"#).as_bytes());
    }
    fn progress(&mut self, guid: &str, state: &str) {
        self.browser.message(
            &self.env,
            format!(r#"{{"method":"Browser.downloadProgress","params":{{"guid":"{guid}","state":"{state}"}}}}"#)
                .as_bytes(),
        );
    }
}
#[test]
fn persons_set_their_download_policy_before_becoming_ready() {
    for (directory, params) in [
        (
            Some(b"/downloads".as_slice()),
            br#"{"browserContextId":"ctx","behavior":"allowAndName","eventsEnabled":true,"downloadPath":"/downloads"}"#
                .as_slice(),
        ),
        (None, br#"{"browserContextId":"ctx","behavior":"deny","eventsEnabled":true}"#),
    ] {
        let mut peer = Peer::new(directory);
        peer.reply(b"Target.createBrowserContext", b"{}", br#"{"browserContextId":"ctx"}"#);
        assert!(!peer.browser.person(PERSON).unwrap().ready);
        peer.browser
            .request(&peer.env, Request::Page { person: PERSON, page: PAGE, url: b"about:blank".as_slice().into() });
        let policy = peer.browser.outgoing.pop_front().expect("download policy");
        assert!(peer.browser.sendable().is_none(), "target waits for the policy reply");
        peer.browser.outgoing.push_front(policy);
        peer.reply(b"Browser.setDownloadBehavior", params, b"{}");
        assert!(peer.browser.person(PERSON).unwrap().ready);
    }
}
#[test]
fn letter_keys_carry_alt_shift_and_text_on_both_events() {
    for (alt, shift, expected_key, text, modifiers) in [
        (false, false, "s", "s", ""),
        (false, true, "S", "S", ",\"modifiers\":8"),
        (true, false, "s", "", ",\"modifiers\":1"),
        (true, true, "S", "", ",\"modifiers\":9"),
    ] {
        let mut peer = Peer::open();
        peer.browser
            .request(&peer.env, Request::Key { page: PAGE, op: OP, key: Key::Letter { letter: b's', alt }, shift });
        for kind in ["keyDown", "keyUp"] {
            let params = format!(
                r#"{{"type":"{kind}","key":"{expected_key}","code":"KeyS","windowsVirtualKeyCode":83,"text":"{text}"{modifiers}}}"#
            );
            peer.reply(b"Input.dispatchKeyEvent", params.as_bytes(), b"{}");
        }
        let Some(Event::Done { op: OP }) = peer.browser.events.pop_front() else { panic!("key done") };
    }
}
#[test]
fn invalid_letters_are_refused_before_any_command() {
    for letter in [b'A', b'`', b'{', 0, 255] {
        let mut peer = Peer::open();
        peer.browser.request(
            &peer.env,
            Request::Key { page: PAGE, op: OP, key: Key::Letter { letter, alt: true }, shift: true },
        );
        assert!(peer.browser.outgoing.is_empty());
        let Some(Event::Refused { op: OP, why: Refusal::Protocol }) = peer.browser.events.pop_front() else {
            panic!("invalid letter refused")
        };
    }
}
#[test]
fn a_completed_download_names_its_person_name_and_path() {
    let mut peer = Peer::open();
    peer.begin("target", "guid", "recording.json");
    peer.progress("guid", "inProgress");
    peer.progress("guid", "completed");
    peer.downloaded(b"recording.json", b"/downloads/guid");
    peer.progress("guid", "completed");
    assert!(peer.browser.events.is_empty());
    assert!(peer.browser.downloads.is_empty());
}
#[test]
fn a_cancelled_download_is_its_pages_trouble() {
    let mut peer = Peer::open();
    peer.begin("target", "guid", "recording.json");
    peer.progress("guid", "canceled");
    peer.trouble(b"recording.json");
    assert!(peer.browser.downloads.is_empty());
}
#[test]
fn one_past_the_download_limit_is_cancelled() {
    let mut peer = Peer::open();
    peer.env.limits.downloads = 1;
    peer.begin("target", "first", "first.json");
    peer.begin("target", "second", "second.json");
    assert_eq!(peer.browser.downloads.len(), 1);
    peer.reply(b"Browser.cancelDownload", br#"{"browserContextId":"ctx","guid":"second"}"#, b"{}");
    peer.trouble(b"second.json");
}
#[test]
fn unknown_targets_and_child_frames_are_not_kept_or_reported() {
    let mut peer = Peer::open();
    for frame in ["unknown", "child-frame"] {
        peer.begin(frame, "guid", "recording.json");
        peer.progress("guid", "completed");
    }
    assert!(peer.browser.events.is_empty());
    assert!(peer.browser.downloads.is_empty());
}
#[test]
fn a_download_outlives_its_pages_close() {
    let mut peer = Peer::open();
    peer.begin("target", "guid", "recording.json");
    peer.browser.close_page_local(PAGE);
    peer.browser.reclaim();
    peer.closed(PAGE);
    peer.progress("guid", "completed");
    peer.downloaded(b"recording.json", b"/downloads/guid");
}
#[test]
fn a_person_cancels_downloads_before_closed_and_reports_none_after() {
    let mut peer = Peer::open();
    peer.begin("target", "guid", "recording.json");
    peer.browser.request(&peer.env, Request::Close { entity: PERSON });
    peer.reply(b"Browser.cancelDownload", br#"{"browserContextId":"ctx","guid":"guid"}"#, b"{}");
    peer.reply(b"Target.disposeBrowserContext", br#"{"browserContextId":"ctx"}"#, b"{}");
    peer.closed(PAGE);
    peer.closed(PERSON);
    peer.progress("guid", "completed");
    assert!(peer.browser.events.is_empty());
}
#[test]
fn a_download_name_past_text_is_trouble_and_cancelled() {
    let mut peer = Peer::open();
    peer.env.limits.text = 4;
    peer.begin("target", "guid", "12345");
    peer.reply(b"Browser.cancelDownload", br#"{"browserContextId":"ctx","guid":"guid"}"#, b"{}");
    peer.trouble(b"1234");
}
#[test]
fn a_download_path_past_text_reports_trouble_instead_of_a_path() {
    let mut peer = Peer::open();
    peer.env.limits.text = 14; // /downloads/guid is fifteen bytes.
    peer.begin("target", "guid", "name");
    peer.progress("guid", "completed");
    peer.trouble(b"name");
}

#[test]
fn names_and_paths_at_the_text_bound_are_kept_whole() {
    let mut peer = Peer::open();
    peer.env.limits.text = 15; // Both the suggested name and /downloads/guid have fifteen bytes.
    peer.begin("target", "guid", "123456789012345");
    peer.progress("guid", "completed");
    peer.downloaded(b"123456789012345", b"/downloads/guid");
}
#[test]
fn closing_the_browser_cancels_downloads_before_its_close_command() {
    let mut peer = Peer::open();
    peer.begin("target", "guid", "recording.json");
    peer.browser.request(&peer.env, Request::Close { entity: Token::new(1) });
    peer.reply(b"Browser.cancelDownload", br#"{"browserContextId":"ctx","guid":"guid"}"#, b"{}");
    peer.reply(b"Browser.close", b"{}", b"{}");
    peer.progress("guid", "completed");
    peer.closed(PAGE);
    peer.closed(PERSON);
    peer.closed(Token::new(1));
    assert!(peer.browser.events.is_empty());
}
