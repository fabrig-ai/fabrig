//! Deleting a project from How work runs (client/pages/settings.md, 1).
use super::{Pages, Request};
use alloc::boxed::Box;
use jig_client::{Client, Key, Outcome, Told, contract as c};
use skein_lib::Queue;
#[derive(Debug)]
pub struct Open {
    pub project: u32,
    pub name: Box<[u8]>,
    pub sent: Option<Key>,
    pub failed: Option<Outcome>,
}
/// The same ownership that offers Rename offers deletion (client/pages/settings.md, 1).
#[must_use]
pub fn offered(me: Option<&c::Me>, project: u32) -> bool {
    super::switcher::owned(me, Some(project)).is_some()
}
pub(super) fn open(pages: &mut Pages, client: &Client, project: u32) {
    if pages.address != (crate::Address::Settings { project }) {
        return;
    }
    if let Some(open) = &pages.deleting
        && open.project == project
    {
        return;
    }
    let Some(row) = super::switcher::owned(pages.me(client), Some(project)) else { return };
    let name = row.name.bytes().into();
    pages.confirm = None;
    pages.deleting = Some(Open { project, name, sent: None, failed: None });
}
pub(super) fn confirm(pages: &mut Pages, client: &Client, out: &mut Queue<Request>) {
    let Some(open) = &pages.deleting else { return };
    if open.sent.is_some() {
        return;
    }
    let ask = match pages.me(client) {
        Some(me) => jig_client::shapes::remove_project(me, open.project).ok(),
        None => None,
    };
    let Some(ask) = ask else {
        pages.deleting = None;
        return;
    };
    let open = pages.deleting.as_mut().expect("open deletion sheet");
    pages.serial = pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: crate::owner::Owner::Pages.asker(), number: pages.serial };
    open.sent = Some(key);
    open.failed = None;
    out.push(Request::Client(jig_client::Event::Send {
        key,
        name: None,
        request: jig_client::Ask::Jig(Box::new(ask)),
    }));
}
pub(super) fn told(pages: &mut Pages, told: &Told, out: &mut Queue<Request>) -> bool {
    let Some(open) = &mut pages.deleting else { return false };
    match told {
        Told::Pending { key } if open.sent == Some(*key) => true,
        Told::Answered { key, outcome } if open.sent == Some(*key) => {
            open.sent = None;
            if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::ProjectRemoved { project },
            )))) = outcome
                && *project == open.project
            {
                pages.deleting = None;
                out.push(Request::Show { address: crate::Address::AllHome });
            } else {
                open.failed = Some(outcome.clone());
            }
            true
        }
        Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Page { .. }
        | Told::Link { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Ended { .. } => false,
    }
}
