//! The browser's own facts about a press's tab and its opener.
use super::{Event, FIND, PAGE, PRESS, Request, Rig, Stage, query};
impl Rig {
    pub fn tab_script(&mut self) {
        self.pages.pages.push(skein_fake_person_browser::origins::Page {
            path: b"/tabs.js".as_slice().into(),
            kind: b"text/javascript; charset=utf-8".as_slice().into(),
            body: br"const heading=document.querySelector('h1');
            document.querySelector('#open').addEventListener('click',()=>{const t=window.open('','_blank');t.document.title='Opening';window.held=t;heading.textContent='Opened';});
            document.querySelector('#cut').addEventListener('click',()=>{window.held.opener=null;heading.textContent='Cut';});
            const unpressed=window.open('','_blank');heading.textContent=unpressed?'Allowed':'Blocked';".as_slice().into(),
            headers: vec![],
        });
    }

    pub(super) fn tabs_event(&mut self, event: Event) -> Result<(), Event> {
        match event {
            Event::Opened { page } => {
                assert_eq!(page, PAGE);
                self.stage = Stage::Finding;
                self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"heading", b"Blocked") });
            }
            Event::Found { op, seen, more } if self.stage == Stage::Finding => {
                assert_eq!(
                    (op, seen.len(), more),
                    (FIND, 1, 0),
                    "opening with no press must be blocked; version {}",
                    String::from_utf8_lossy(&self.version)
                );
                self.ask(Request::Tabs { page: PAGE, op: FIND });
            }
            Event::Tabs { op, tabs, more } if self.stage == Stage::Finding => {
                assert_eq!((op, tabs.len(), more), (FIND, 0, 0), "no tab without a press");
                self.stage = Stage::Capturing;
                self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"button", b"Cut opener") });
            }
            Event::Found { op, seen, more } if self.stage == Stage::Capturing => {
                assert_eq!((op, seen.len(), more), (FIND, 1, 0));
                self.tabs_cut = Some(seen.get(0).expect("cut button").node);
                self.stage = Stage::Pressing;
                self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"button", b"Open") });
            }
            Event::Found { op, seen, more } => {
                assert_eq!((op, seen.len(), more), (FIND, 1, 0));
                self.ask(Request::Press { page: PAGE, op: PRESS, node: seen.get(0).expect("button").node, at: None });
            }
            Event::Done { op } => {
                assert_eq!(op, PRESS);
                self.ask(Request::Tabs { page: PAGE, op: FIND });
            }
            Event::Tabs { op, tabs, more } => {
                assert_eq!((op, tabs.len(), more), (FIND, 1, 0));
                let tab = tabs.get(0).expect("press's tab");
                assert_eq!(tab.title.as_ref(), b"Opening");
                assert_eq!(tab.url.as_ref(), b"about:blank");
                let expected = self.stage == Stage::Pressing;
                eprintln!(
                    "{}: Target.getTargets canAccessOpener={}, expected={expected}",
                    String::from_utf8_lossy(&self.version),
                    tab.opener
                );
                assert_eq!(tab.opener, expected, "browser's word on opener");
                if expected {
                    self.stage = Stage::Keying;
                    self.ask(Request::Press {
                        page: PAGE,
                        op: PRESS,
                        node: self.tabs_cut.expect("cut button found before opening"),
                        at: None,
                    });
                } else {
                    self.close_browser();
                }
            }
            other => return Err(other),
        }
        Ok(())
    }
}
