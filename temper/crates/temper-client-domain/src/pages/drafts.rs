//! Work's landed drafts watch (temper/docs/design/client/pages/work.md, 1).
use super::Request;
use crate::{forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Told};
use skein_lib::Queue;
use temper_client_protocol_wire::v1 as w;

#[derive(Debug)]
pub enum State {
    Loading,
    Shown(Box<w::DraftsPanel>),
    Ended,
}
#[derive(Debug)]
pub struct Drafts {
    project: u32,
    handle: Option<Handle>,
    state: State,
}
impl Drafts {
    pub(super) fn new(project: u32, serial: &mut u64, limits: forge::Limits, out: &mut Queue<Request>) -> Self {
        let handle = Handle { asker: Owner::Pages.asker(), number: *serial };
        *serial = serial.checked_add(1).expect("page handles fit");
        let mut page = Self { project, handle: None, state: State::Ended };
        if let Some(subject) = forge::drafts::watch(project, &limits) {
            page.handle = Some(handle);
            page.state = State::Loading;
            out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
        }
        page
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }
    pub(super) fn told(&mut self, told: &Told, client: &Client, limits: forge::Limits, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if Some(*handle) == self.handle => {
                let panel = match client.document(*handle) {
                    Some(document) if document.state == jig_client::State::Ended => None,
                    Some(document) => match document.latest {
                        Some(Latest::Family(record)) => forge::drafts::panel(record, limits),
                        Some(Latest::Jig(_)) => None,
                        None => return true,
                    },
                    None => return true,
                };
                if let Some(panel) = panel && panel.project() == self.project {
                    self.state = State::Shown(Box::new(panel));
                } else {
                    self.end(out);
                }
                true
            }
            Told::Ended { handle, .. } if Some(*handle) == self.handle => {
                self.end(out);
                true
            }
            Told::Link { .. } | Told::Changed { .. } | Told::Ended { .. } | Told::Page { .. }
            | Told::Text { .. } | Told::Pending { .. } | Told::Answered { .. } | Told::Kept { .. } | Told::Dropped { .. } => false,
        }
    }
    fn end(&mut self, out: &mut Queue<Request>) {
        self.leave(out);
        self.handle = None;
        self.state = State::Ended;
    }
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }
}
