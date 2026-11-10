//! A page stays usable beside its own tab (skein/docs/design/testing/browser.md, 6).
use super::{Event, FIND, IMAGE, PAGE, PRESS, Request, Rig, SNAPSHOT, Stage, query};

impl Rig {
    pub fn beside_page(&mut self) {
        self.fixed_page(br#"<!doctype html><button id="open">Open</button><button id="point">Point tab</button><h1>Before</h1><script>
        const heading=document.querySelector('h1');
        function visibility(){document.body.setAttribute('data-visibility',document.visibilityState);}
        document.addEventListener('visibilitychange',visibility);visibility();
        document.querySelector('#open').onclick=()=>{window.held=window.open('','_blank');heading.textContent='Opened';visibility();};
        document.querySelector('#point').onclick=()=>{window.held.opener=null;window.held.location.replace('/tab-loaded');heading.textContent='Pointed '+document.visibilityState;};
        </script>"#.to_vec());
        self.pages.pages.push(skein_fake_person_browser::origins::Page {
            path: b"/tab-loaded".as_slice().into(),
            kind: b"text/html; charset=utf-8".as_slice().into(),
            body: br"<!doctype html><h1>Tab loaded</h1><script>document.title='Loaded '+document.visibilityState;</script>".as_slice().into(),
            headers: vec![],
        });
    }

    pub(super) fn beside_event(&mut self, event: Event) -> Result<(), Event> {
        match event {
            Event::Opened { page: PAGE } => {
                self.stage = Stage::Finding;
                self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"button", b"Open") });
            }
            Event::Found { op: FIND, seen, more: 0 } if self.stage == Stage::Finding => {
                self.stage = Stage::Pressing;
                self.ask(Request::Press {
                    page: PAGE,
                    op: PRESS,
                    node: seen.get(0).expect("open button").node,
                    at: None,
                });
            }
            Event::Done { op: PRESS } if self.stage == Stage::Pressing => {
                self.stage = Stage::Expecting;
                self.ask(Request::Attribute {
                    page: PAGE,
                    op: FIND,
                    selector: b"body".as_slice().into(),
                    name: b"data-visibility".as_slice().into(),
                });
            }
            Event::Attribute { op: FIND, read } => {
                assert_eq!(
                    read,
                    skein_browser::boundary::Read::Value(b"hidden".as_slice().into()),
                    "page is hidden until the person looks again"
                );
                eprintln!("{}: page reports hidden after opening", String::from_utf8_lossy(&self.version));
                self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"heading", b"Opened") });
            }
            Event::Found { op: FIND, seen, more } if self.stage == Stage::Expecting => {
                eprintln!("fresh heading: {} matches, {more} more", seen.len());
                if seen.len() != 1 || more != 0 {
                    self.stage = Stage::Reloading;
                    self.ask(Request::Snapshot { page: PAGE, op: SNAPSHOT });
                } else {
                    self.stage = Stage::Typing;
                    self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"button", b"Point tab") });
                }
            }
            Event::Refused { op: FIND, why, detail } if self.stage == Stage::Expecting => {
                eprintln!("fresh heading refused: {why:?} {}", String::from_utf8_lossy(&detail));
                self.stage = Stage::Reloading;
                self.ask(Request::Snapshot { page: PAGE, op: SNAPSHOT });
            }
            Event::Snapshot { op: SNAPSHOT, text } => panic!(
                "fresh find missed heading written after opening; AX snapshot: {}",
                String::from_utf8_lossy(&text)
            ),
            Event::Found { op: FIND, seen, more: 0 } if self.stage == Stage::Typing => {
                assert_eq!(seen.len(), 1, "fresh button beside tab");
                self.stage = Stage::Keying;
                self.ask(Request::Press {
                    page: PAGE,
                    op: PRESS,
                    node: seen.get(0).expect("point button").node,
                    at: None,
                });
            }
            Event::Done { op: PRESS } if self.stage == Stage::Keying => {
                self.stage = Stage::Opening;
                self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"heading", b"Pointed visible") });
            }
            Event::Found { op: FIND, seen, more } if self.stage == Stage::Opening => {
                assert_eq!((seen.len(), more), (1, 0), "fresh press lands; page itself sees visible");
                self.stage = Stage::Capturing;
                self.ask(Request::Screenshot { page: PAGE, op: IMAGE });
            }
            Event::Screenshot { op: IMAGE, png } => {
                assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "viewport PNG");
                self.stage = Stage::Closing;
                self.ask(Request::Tabs { page: PAGE, op: FIND });
            }
            Event::Tabs { op: FIND, tabs, more } => {
                assert_eq!((tabs.len(), more), (1, 0));
                let tab = tabs.get(0).expect("pointed tab");
                if tab.title.starts_with(b"Loaded ") {
                    assert_eq!(tab.title.as_ref(), b"Loaded hidden", "new tab loads while hidden");
                    let expected = format!("http://{}/tab-loaded", self.pages.addr.expect("origin"));
                    assert_eq!(tab.url.as_ref(), expected.as_bytes(), "tab loads behind page");
                    assert!(!tab.opener, "pointing cuts opener");
                    self.close_browser();
                } else {
                    self.tab_due = Some(self.browser_env.now.saturating_add(self.browser_env.limits.poll));
                }
            }
            other => return Err(other),
        }
        Ok(())
    }
}
