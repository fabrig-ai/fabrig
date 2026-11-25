//! A changed viewport reaches CSS layout (skein/docs/design/testing/browser.md, 5).
use super::{Event, FIND, PAGE, Request, Rig, Stage, query};

mod rendering;
pub(super) use rendering::Rendering;

impl Rig {
    pub(super) fn viewport_event(&mut self, event: Event) -> Result<(), Event> {
        match event {
            Event::Opened { page: PAGE } => {
                self.stage = Stage::Finding;
                self.ask(Request::Viewport { page: PAGE, op: FIND, width: 420, height: 900 });
            }
            Event::Done { op: FIND } => {
                let mut query = query(b"heading", b"Viewport");
                query.boxes = true;
                self.ask(Request::Find { page: PAGE, op: FIND, query });
            }
            Event::Found { op: FIND, seen, more } => {
                assert_eq!((seen.len(), more), (1, 0));
                let rect = seen.get(0).expect("heading").rect.expect("CSS viewport box");
                if self.stage == Stage::Finding {
                    assert_eq!(rect.width, 420);
                    self.stage = Stage::Expecting;
                    self.ask(Request::Viewport { page: PAGE, op: FIND, width: 1440, height: 900 });
                } else {
                    assert_eq!(rect.width, 1440);
                    self.begin_file_reads();
                }
            }
            event => return self.file_read_event(event),
        }
        Ok(())
    }
}
