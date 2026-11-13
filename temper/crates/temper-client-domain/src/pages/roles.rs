//! How work runs, role and rule pages read the forge's view (temper/docs/design/client/pages/README.md and temper/docs/design/client/pages/chat/conversation.md, 3).
use super::Request;
use crate::{Address, forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, contract as c};
use skein_lib::{Decimal, Queue};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
pub struct Roles {
    address: Option<Address>,
    handle: Option<Handle>,
    view: Option<Box<w::View>>,
    pending: bool,
    again: bool,
    failed: bool,
    revision: Option<u32>,
}

#[derive(Debug)]
pub enum Read<'a> {
    Reading,
    Drawn(&'a w::View),
    Unreadable,
}

#[expect(clippy::derivable_impls, reason = "page state starts explicitly without a read or revision")]
impl Default for Roles {
    fn default() -> Self {
        Self { address: None, handle: None, view: None, pending: false, again: false, failed: false, revision: None }
    }
}

impl Roles {
    pub(crate) fn configure(
        &mut self,
        address: &Address,
        client: &Client,
        limits: forge::Limits,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let project = match address {
            Address::Settings { project } | Address::Role { project, .. } | Address::Rule { project, .. } => *project,
            Address::AllHome
            | Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Work { .. }
            | Address::Told
            | Address::Remembered
            | Address::Members
            | Address::Limits
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Chat { .. }
            | Address::Talk { .. }
            | Address::Task { .. }
            | Address::Home { .. }
            | Address::Goals { .. }
            | Address::Changes { .. }
            | Address::Team { .. }
            | Address::Repositories { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Ask { .. }
            | Address::Tell { .. }
            | Address::Adopt { .. } => {
                self.leave(out);
                return;
            }
        };
        if self.address.as_ref() != Some(address) {
            self.leave(out);
            self.address = Some(address.clone());
            self.again = true;
        }
        if let Some(document) = client.document(forge::ways::handle())
            && let Some(Latest::Jig(c::Delivery::Settings(settings))) = document.latest
            && match self.revision {
                Some(old) => settings.newest > old,
                None => true,
            }
        {
            self.revision = Some(settings.newest);
            self.again = true;
        }
        if self.pending || !self.again {
            return;
        }
        self.again = false;
        let Some(bounds) = forge::wire_limits(&limits) else {
            self.failed = true;
            return;
        };
        let read = w::ViewRead::new(&bounds, w::ViewReadParts { project, address: view_address(address) })
            .expect("page address fits view");
        let record = w::Reading::new(&bounds, w::ReadingParts { read: w::Read::View(read) }).expect("view read fits");
        let handle = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        if let Some(old) = self.handle.replace(handle) {
            out.push(Request::Client(jig_client::Event::Unwatch { handle: old }));
        }
        self.pending = true;
        out.push(Request::Client(jig_client::Event::Read {
            handle,
            read: Readable::Family {
                scope: c::Scope::Project(project),
                window: c::Window { rows: 0, after: None },
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::reading(&record),
                },
            },
        }));
    }
    pub(crate) fn leave(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        *self = Self::default();
    }
    pub(super) fn told(&mut self, told: &Told, limits: forge::Limits) -> bool {
        match told {
            Told::Changed { handle } if self.address.is_some() && *handle == forge::ways::handle() => true,
            Told::Page { handle, page } if self.handle == Some(*handle) && self.pending => {
                self.pending = false;
                self.view = match answer(page, limits) {
                    Some(view) => Some(Box::new(view)),
                    None => None,
                };
                self.failed = self.view.is_none();
                true
            }
            Told::Ended { handle, .. } if self.handle == Some(*handle) => {
                self.pending = false;
                self.failed = true;
                self.view = None;
                true
            }
            Told::Page { .. }
            | Told::Ended { .. }
            | Told::Changed { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Link { .. }
            | Told::Text { .. } => false,
        }
    }
    #[must_use]
    pub fn read(&self) -> Read<'_> {
        match self.view.as_deref() {
            Some(w::View::NotDrawn(_)) => Read::Unreadable,
            Some(view) => Read::Drawn(view),
            None if self.failed => Read::Unreadable,
            None => Read::Reading,
        }
    }
}

fn view_address(address: &Address) -> Box<[u8]> {
    match address {
        Address::Settings { .. } => Box::from(b"roles".as_slice()),
        Address::Role { charter, .. } => {
            skein_lib::bytes::concat(&[b"charter ", Decimal::of(u64::from(*charter)).as_bytes()])
        }
        Address::Rule { kind, number, .. } => skein_lib::bytes::concat(&[
            b"rule ",
            Decimal::of(u64::from(*kind)).as_bytes(),
            b"/",
            Decimal::of(u64::from(*number)).as_bytes(),
        ]),
        Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Work { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Repositories { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Ask { .. }
        | Address::Tell { .. }
        | Address::Adopt { .. } => Box::from(b"".as_slice()),
    }
}
fn answer(page: &Readout, limits: forge::Limits) -> Option<w::View> {
    let Readout::Family { family: wire::FORGE, record: Some(bytes) } = page else {
        return None;
    };
    Some(wire::decode_view_answer(bytes, &forge::wire_limits(&limits)?)?.into_parts().view)
}
