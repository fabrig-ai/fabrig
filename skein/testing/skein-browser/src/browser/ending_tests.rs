//! Browser-wide failures at the CDP pipe and command boundaries (browser.md, 3).

#![expect(clippy::disallowed_macros, reason = "synthetic CDP replies are formatted from fixture values")]

use super::{Browser, Download, Page, Person, Phase, document, down, fire, render, up};
use crate::boundary::{Below, Down, Ending, Event, Refusal, Request};
use crate::{Limits, wire::decode::Document};
use skein_lib::stream::{self, Fault, Read, Up};
use skein_lib::{Env, Intake, Queue, Time, Token, Wall};

const BROWSER: Token = Token::new(1);
const PERSON: Token = Token::new(2);
const PAGE: Token = Token::new(3);
const IMAGE: Token = Token::new(4);

struct Peer {
    browser: Browser,
    env: Env<Limits>,
    above: Queue<Event>,
    below: Queue<Down>,
}
impl Peer {
    fn start(limits: Limits) -> Self {
        let mut peer = Self {
            browser: Browser::new(BROWSER, &limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::uncapped(),
            below: Queue::uncapped(),
        };
        peer.fire();
        peer
    }

    fn ready(limits: Limits) -> Self {
        let mut peer = Self::start(limits);
        let id = peer.command(b"Browser.getVersion");
        peer.reply(id, r#""result":{"product":"fixture"}"#);
        let Some(Event::Ready { .. }) = peer.above.pop() else { panic!("version reply makes the browser ready") };
        peer
    }

    fn fire(&mut self) {
        fire(&mut self.browser, &self.env, self.env.now, &mut self.above, &mut self.below);
    }

    fn up(&mut self, input: Below) {
        up(&mut self.browser, &self.env, input, &mut self.above, &mut self.below);
    }

    fn send(&mut self, frame: &[u8]) {
        self.up(Below::Replies(Up::Bytes(frame.into())));
    }

    fn ask(&mut self, request: Request) {
        down(&mut self.browser, &self.env, request, &mut self.above, &mut self.below);
    }

    fn command(&mut self, method: &[u8]) -> u64 {
        self.up(Below::Commands(Up::Room));
        while let Some(down) = self.below.pop() {
            if let Down::Commands(stream::Down::Send(bytes)) = down {
                let json = bytes.strip_suffix(b"\0").expect("framed command");
                let document = Document::parse(json, self.env.limits.command).expect("JSON command");
                let root = document.root();
                assert_eq!(root.get(b"method").expect("method").text().as_deref(), Some(method));
                return root.get(b"id").expect("id").u64().expect("numeric id");
            }
        }
        panic!("missing command {method:?}");
    }

    fn reply(&mut self, id: u64, member: &str) {
        self.send(format!("{{\"id\":{id},{member}}}\0").as_bytes());
    }

    fn person(&mut self) {
        assert_eq!(self.browser.phase, Phase::Ready);
        assert!(self.browser.insert_person(
            Person {
                token: PERSON,
                context: Some(b"context".as_slice().into()),
                downloads: Some(b"/downloads".as_slice().into()),
                ready: true,
                closing: false,
                closed: false,
            },
            self.env.limits.persons,
        ));
    }

    fn page(&mut self) {
        self.person();
        assert!(self.browser.insert_page(
            Page {
                token: PAGE,
                person: PERSON,
                target: Some(b"target".as_slice().into()),
                session: Some(b"session".as_slice().into()),
                initial_url: b"about:blank".as_slice().into(),
                requests: false,
                files: false,
                css: render::Css::Disabled,
                frame: None,
                document: document::Root::Empty,
                navigating: false,
                opening: false,
                load: None,
                load_due: None,
                closed: false,
            },
            self.env.limits.pages,
        ));
    }

    fn download(&mut self) {
        self.person();
        self.browser.downloads.push(Download {
            guid: b"download".as_slice().into(),
            person: PERSON,
            page: PAGE,
            name: b"file".as_slice().into(),
        });
    }

    fn ended(&mut self, expected: Ending, words: &[u8]) {
        self.fire();
        let event = self.above.pop();
        let Some(Event::Ended { why, detail }) = event else { panic!("expected {expected:?}, got {event:?}") };
        assert_eq!(why, expected);
        assert_eq!(detail.as_ref(), words, "{why:?}");
    }

    fn closed(&mut self, entity: Token) {
        self.fire();
        let event = self.above.pop();
        let Some(Event::Closed { owner }) = event else { panic!("close {entity:?}: {event:?}") };
        assert_eq!(owner, entity);
    }

    fn nothing_after_close(&mut self) {
        self.up(Below::Replies(Up::End));
        self.fire();
        assert!(self.above.is_empty(), "another pipe event cannot announce a second ending");
        assert!(!self.browser.work_pending());
        assert_eq!(self.browser.next_deadline(), None);
    }
}

#[test]
fn broken_pipes_name_the_pipe_and_fault_once() {
    for (input, detail) in [
        (Below::Replies(Up::End), b"replies pipe: end".as_slice()),
        (Below::Replies(Up::Failed(Fault::Reset)), b"replies pipe: reset"),
        (Below::Replies(Up::Room), b"replies pipe: unexpected room"),
        (Below::Commands(Up::End), b"commands pipe: end"),
        (Below::Commands(Up::Failed(Fault::Invalid)), b"commands pipe: invalid"),
        (Below::Commands(Up::Bytes(b"data".as_slice().into())), b"commands pipe: unexpected bytes"),
    ] {
        let mut peer = Peer::ready(Limits::default());
        peer.up(input);
        peer.ended(Ending::Pipe, detail);
        peer.closed(BROWSER);
        peer.nothing_after_close();
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "repetition and concatenation build the deep and long test frames")]
fn bad_frames_name_the_violated_wire_rule() {
    const MESSAGE: u32 = 160;
    const PAST_MAX_DEPTH: usize = 65;
    let long = [b" ".repeat(usize::try_from(MESSAGE).expect("fixture bound")), b"0\0".to_vec()].concat();
    let deep = [b"[".repeat(PAST_MAX_DEPTH), b"0".to_vec(), b"]".repeat(PAST_MAX_DEPTH), b"\0".to_vec()].concat();
    for (frame, detail) in [
        (b"{}".as_slice(), b"missing NUL delimiter".as_slice()),
        (&long, b"past the message limit"),
        (&deep, b"nested too deep"),
        (b"{\0", b"not JSON"),
    ] {
        let mut peer = Peer::ready(Limits { message: MESSAGE, ..Limits::default() });
        peer.send(frame);
        peer.ended(Ending::Message, detail);
        peer.closed(BROWSER);
        peer.nothing_after_close();
    }
}

#[test]
fn a_reply_to_an_unsent_id_names_the_id() {
    // Id 1 has been allocated but not yet sent, since the pipe has not granted room.
    for (id, detail) in [(0, b"0".as_slice()), (1, b"1"), (1234, b"1234")] {
        let mut peer = Peer::start(Limits::default());
        peer.reply(id, r#""result":{}"#);
        peer.ended(Ending::Unsent, detail);
        peer.closed(BROWSER);
        peer.nothing_after_close();
    }
}

#[test]
fn a_document_that_is_not_a_reply_or_event_ends_the_browser() {
    let mut peer = Peer::ready(Limits::default());
    peer.send(b"{}\0");
    peer.ended(Ending::Neither, b"neither a reply nor an event");
    peer.closed(BROWSER);
    peer.nothing_after_close();
}

#[test]
fn a_refused_or_unanswered_browser_command_names_its_method() {
    for (method, refused, cancel) in [
        (b"Browser.getVersion".as_slice(), b"Browser.getVersion: refused by peer".as_slice(), false),
        (b"Browser.cancelDownload", b"Browser.cancelDownload: refused by peer", true),
        (b"Browser.close", b"Browser.close: refused by peer", false),
    ] {
        for answer in [false, true] {
            let mut peer = if method == b"Browser.getVersion" {
                Peer::start(Limits::default())
            } else {
                let mut peer = Peer::ready(Limits::default());
                if cancel {
                    peer.download();
                }
                peer.ask(Request::Close { entity: BROWSER });
                peer
            };
            let id = peer.command(method);
            if answer {
                peer.reply(id, r#""error":{"message":"refused by peer"}"#);
                peer.ended(Ending::Refused, refused);
            } else {
                let due = peer.env.now.saturating_add(peer.env.limits.answer);
                peer.env.now = Time::from_nanos(due.as_nanos().checked_sub(1).expect("positive answer deadline"));
                peer.fire();
                assert!(peer.above.is_empty(), "{method:?} still has time to answer");
                peer.env.now = due;
                peer.ended(Ending::Unanswered, method);
            }
            if cancel {
                peer.closed(PERSON);
            }
            peer.closed(BROWSER);
            peer.nothing_after_close();
        }
    }
}

#[test]
fn ending_detail_is_bounded_by_text_not_message() {
    for (text, expected) in [(0, b"".as_slice()), (22, b"Browser.getVersion: lo")] {
        let mut peer = Peer::start(Limits { text, ..Limits::default() });
        let id = peer.command(b"Browser.getVersion");
        peer.reply(id, r#""error":{"message":"long version error"}"#);
        peer.ended(Ending::Refused, expected);
        peer.closed(BROWSER);
    }
}

#[test]
fn a_requested_close_that_is_answered_has_no_ending() {
    let mut peer = Peer::ready(Limits::default());
    peer.download();
    peer.ask(Request::Close { entity: BROWSER });
    for method in [b"Browser.cancelDownload".as_slice(), b"Browser.close"] {
        let id = peer.command(method);
        peer.reply(id, r#""result":{}"#);
    }
    peer.closed(PERSON);
    peer.closed(BROWSER);
    peer.nothing_after_close();
}

#[test]
fn an_oversize_screenshot_reply_crashes_the_operation_before_ending() {
    // Small wire limits test the cut without constructing a multi-megabyte image.
    const MESSAGE: u32 = 64;
    for past in [false, true] {
        let mut peer = Peer::ready(Limits { message: MESSAGE, ..Limits::default() });
        peer.page();
        peer.ask(Request::Screenshot { page: PAGE, op: IMAGE });
        let id = peer.command(b"Page.bringToFront");
        peer.reply(id, r#""result":{}"#);
        let mut scan = None;
        for down in &peer.below {
            if let Down::Replies(stream::Down::Demand { read, .. }) = down {
                scan = Some(*read);
            }
        }
        let read = scan.expect("reply scan");
        let Read::Scan { max, .. } = read else { panic!("reply scan, got {read:?}") };
        assert_eq!(max, MESSAGE.saturating_add(1));
        let id = peer.command(b"Page.captureScreenshot");
        let json = format!(r#"{{"id":{id},"result":{{"data":"iVBORw0KGgo="}}}}"#);
        let mut frame = json.into_bytes();
        let len = usize::try_from(MESSAGE).expect("fixture bound");
        frame.resize(len.saturating_add(usize::from(past)), b' ');
        frame.push(0);
        let mut intake = Intake::with_capacity(max);
        let read_len = usize::try_from(max).expect("fixture bound");
        intake.append(frame.get(..read_len).expect("first read")).expect("the first read fills the scan");
        let bytes = intake.meet(read).expect("the scan delivers at its limit");
        peer.up(Below::Replies(Up::Bytes(bytes)));
        let event = peer.above.pop();
        if past {
            let Some(Event::Refused { op: IMAGE, why: Refusal::Crashed, .. }) = event else {
                panic!("expected the screenshot to crash first, got {event:?}");
            };
            peer.ended(Ending::Message, b"past the message limit");
            peer.closed(PAGE);
            peer.closed(PERSON);
            peer.closed(BROWSER);
            peer.nothing_after_close();
        } else {
            let Some(Event::Screenshot { op: IMAGE, png }) = event else {
                panic!("screenshot at bound, got {event:?}")
            };
            assert_eq!(png.as_ref(), b"\x89PNG\r\n\x1a\n");
            assert!(peer.above.is_empty());
        }
    }
}
