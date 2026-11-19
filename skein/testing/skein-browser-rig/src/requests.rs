//! Chromium's first streamed bytes and leaving document's loader (browser.md, 9).
use super::{Event, KEY, PAGE, Request, Rig, Stage};
use skein_browser::boundary::Go;

#[expect(clippy::struct_excessive_bools, reason = "the story waits for independent browser observations")]
#[derive(Default)]
pub(super) struct State {
    first_load: Option<Box<[u8]>>,
    next_load: Option<Box<[u8]>>,
    stream: Option<Box<[u8]>>,
    opened: bool,
    streamed: bool,
    keepalive: bool,
    reloaded: bool,
}

impl Rig {
    pub(super) fn request_event(&mut self, event: Event) -> Result<(), Event> {
        match event {
            Event::Sent { page, id, load, document, method, url } => {
                assert_eq!(page, PAGE);
                assert_eq!(method.as_ref(), b"GET");
                if document {
                    assert!(url.ends_with(b"/requests"));
                    assert_eq!(id, load, "a navigation's request id is its loader id");
                    if let Some(first) = &self.requests.first_load {
                        assert_ne!(&load, first, "reload has a new document load");
                        assert!(self.requests.next_load.is_none(), "only two navigations");
                        self.requests.next_load = Some(load);
                    } else {
                        self.requests.first_load = Some(load);
                    }
                } else if url.ends_with(b"/request-stream") {
                    if self.requests.next_load.is_none() {
                        assert_eq!(Some(&load), self.requests.first_load.as_ref(), "fetch belongs to its document");
                        self.requests.stream = Some(id);
                    } else {
                        assert_eq!(Some(&load), self.requests.next_load.as_ref(), "reloaded page's fetch");
                    }
                } else if url.ends_with(b"/request-leave") {
                    assert!(self.requests.next_load.is_some(), "pagehide's request follows the new document's Sent");
                    assert_eq!(Some(&load), self.requests.first_load.as_ref(), "keepalive keeps the leaving load");
                    self.requests.keepalive = true;
                } else {
                    assert!(url.ends_with(b"/favicon.ico"), "unexpected request: {}", String::from_utf8_lossy(&url));
                }
            }
            Event::Answered { page, id } => {
                assert_eq!(page, PAGE);
                if self.requests.stream.as_ref() == Some(&id) {
                    assert!(!self.requests.streamed, "first answer only");
                    assert!(self.pages.open_streams() > 0, "Answered comes while the server keeps its stream open");
                    self.requests.streamed = true;
                }
            }
            Event::Opened { page: PAGE } => self.requests.opened = true,
            Event::Done { op: KEY } => self.requests.reloaded = true,
            event => return Err(event),
        }
        if self.requests.opened && self.requests.streamed && self.stage == Stage::Opening {
            self.stage = Stage::Reloading;
            self.ask(Request::Go { page: PAGE, op: KEY, to: Go::Reload });
        }
        if self.requests.reloaded && self.requests.keepalive && self.stage != Stage::Closing {
            self.close_browser();
        }
        Ok(())
    }
}
