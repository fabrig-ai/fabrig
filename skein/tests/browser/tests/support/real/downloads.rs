//! Download and physical-letter facts from Chromium.
use super::{Event, Expect, FIND, KEY, Key, PAGE, PERSON, PRESS, Request, Rig, Scenario, Stage, Trouble, query};
impl Rig {
    pub(super) fn download_event(&mut self, event: Event) -> Result<(), Event> {
        match event {
            Event::Opened { page: PAGE } => {
                if matches!(self.scenario, Scenario::Letter) {
                    self.stage = Stage::Keying;
                    self.ask(Request::Key {
                        page: PAGE,
                        op: KEY,
                        key: Key::Letter { letter: b's', alt: true },
                        shift: true,
                    });
                } else {
                    self.stage = Stage::Finding;
                    self.ask(Request::Find { page: PAGE, op: FIND, query: query(b"button", b"Save") });
                }
            }
            Event::Found { op: FIND, seen, more } => {
                assert_eq!(more, 0);
                assert_eq!(seen.len(), 1);
                self.stage = Stage::Pressing;
                self.ask(Request::Press { page: PAGE, op: PRESS, node: seen.get(0).unwrap().node, at: None });
            }
            Event::Done { op: PRESS } => {
                self.stage = Stage::Expecting;
                self.ask(Request::Await {
                    page: PAGE,
                    op: super::EXPECT,
                    query: query(b"heading", b"Saved"),
                    expect: Expect::Present,
                    within: super::Duration::from_secs(2),
                });
            }
            Event::Done { op: KEY } => {
                self.stage = Stage::Expecting;
                self.ask(Request::Await {
                    page: PAGE,
                    op: super::EXPECT,
                    query: query(b"heading", b"KeyS alt=true shift=true text="),
                    expect: Expect::Present,
                    within: super::Duration::from_secs(2),
                });
            }
            Event::Met { op: super::EXPECT, .. } => {
                self.await_met = true;
                if !matches!(self.scenario, Scenario::Download { allow: true }) || self.downloaded.is_some() {
                    self.close_browser();
                }
            }
            Event::Downloaded { person, name, path } => {
                assert_eq!(person, PERSON);
                assert!(matches!(self.scenario, Scenario::Download { allow: true }));
                self.downloaded = Some((name, std::ffi::OsStr::from_bytes(&path).into()));
                if self.await_met {
                    self.close_browser();
                }
            }
            Event::Trouble { trouble: Trouble::Download, .. }
                if matches!(self.scenario, Scenario::Download { allow: false }) =>
            {
                self.download_trouble = true;
            }
            event => return Err(event),
        }
        Ok(())
    }
}
use std::os::unix::ffi::OsStrExt;
