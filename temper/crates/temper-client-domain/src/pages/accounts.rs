//! Borrowed account facts and row-bound operations (temper/docs/design/client/pages/README.md and temper/docs/design/client/README.md, 3.6).
use super::Request;
use crate::owner::Owner;
use jig_client::{Client, Handle, Latest, State, contract as c};
use skein_lib::Queue;
mod acts;
pub mod paste;
pub mod request;
pub(crate) mod terminal_login;
pub use paste::{Paste, PasteState};
mod ways;
pub use acts::{Act, Broken, Operation, Pending};
pub use ways::{Way, ways};
/// The carried-value bound (client/README.md, 3.6); the engine catalogue is staged.
pub const KEY_BYTES: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub holder: c::AskAccountHolder,
    pub account: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignInState {
    Unknown,
    Started { source: u32 },
    Signing { source: u32 },
    Ended { source: u32 },
}

/// A fixed global handle outside the increasing page namespace; zero belongs to `me`.
#[must_use]
pub fn handle() -> Handle {
    Handle { asker: Owner::Pages.asker(), number: u64::MAX }
}

/// Missing, behind and ended watches are unknown, rather than false availability.
#[must_use]
pub fn document(client: &Client) -> Option<&c::Accounts> {
    current_me(client)?;
    let doc = client.document(handle())?;
    if doc.state != State::Live {
        return None;
    }
    if let Latest::Jig(c::Delivery::Accounts(accounts)) = doc.latest? { Some(accounts) } else { None }
}

/// A page-local confirmation, never kept across a reload (client/pages/README.md, Ready's place).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Added {
    pub provider: u32,
    pub kind: crate::WayKind,
}

#[derive(Debug)]
pub struct Accounts {
    pub added: Option<Added>,
    /// The standalone connection’s Ready place keeps its automatic trial alive.
    pub added_from_connect: bool,
    /// The row request whose successful end has already supplied its line.
    pub announced: Option<jig_client::Key>,
    pub deployment: Option<Handle>,
    pub key_for: Option<(Row, Option<u32>)>,
    pub request_row: Option<Row>,
    pub request: request::Request,
}

impl Default for Accounts {
    fn default() -> Self {
        Self::new()
    }
}

impl Accounts {
    #[must_use]
    pub fn new() -> Self {
        Self {
            added: None,
            added_from_connect: false,
            announced: None,
            deployment: None,
            key_for: None,
            request_row: None,
            request: request::Request::default(),
        }
    }

    pub(crate) fn configure(&mut self, owner: bool, next: &mut u64, out: &mut Queue<Request>) {
        let deployment_key = match self.key_for {
            Some((row, _)) => row.holder == c::AskAccountHolder::Deployment,
            None => false,
        };
        if !owner && self.deployment.is_some() && deployment_key {
            self.request.clear_key();
            self.request.confirm = None;
            self.key_for = None;
        }
        if owner && self.deployment.is_none() {
            let handle = Handle { asker: Owner::Pages.asker(), number: *next };
            *next = next.checked_add(1).expect("page handles fit");
            self.deployment = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: jig_client::Watchable::Jig(c::Subject::Deployment),
            }));
        } else if !owner && let Some(handle) = self.deployment.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }

    pub(crate) fn leave(&mut self, out: &mut Queue<Request>) {
        self.added = None;
        self.added_from_connect = false;
        self.request.clear_key();
        self.request.confirm = None;
        self.key_for = None;
        if let Some(handle) = self.deployment.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }

    #[must_use]
    pub fn deployment<'a>(&self, client: &'a Client) -> Option<&'a c::Deployment> {
        if current_me(client)?.deployment_role != Some(c::HeldRole::Owner) {
            return None;
        }
        let doc = client.document(self.deployment?)?;
        if doc.state != State::Live {
            return None;
        }
        if let Latest::Jig(c::Delivery::Deployment(deployment)) = doc.latest? { Some(deployment) } else { None }
    }
}

/// Current identity only; old owner facts cannot keep a deployment watch open.
#[must_use]
pub fn current_me(client: &Client) -> Option<&c::Me> {
    let doc = client.document(super::me_handle())?;
    if doc.state != State::Live {
        return None;
    }
    if let Latest::Jig(c::Delivery::Me(me)) = doc.latest? { Some(me) } else { None }
}

/// Only this request's successful end supplies a line; declined usage stays in its panel.
pub(crate) fn signed_in(request: &request::Request, client: &Client, chosen: Option<u32>) -> Option<Added> {
    let SignInState::Ended { source } = request.sign_in_state(client) else { return None };
    let last = document(client)?.last_end.as_ref()?;
    if last.source != source {
        return None;
    }
    let account = match last.end {
        c::AccountEnd::Made { account, declined: false } | c::AccountEnd::Renewed { account, declined: false } => {
            account
        }
        c::AccountEnd::Made { declined: true, .. }
        | c::AccountEnd::Renewed { declined: true, .. }
        | c::AccountEnd::Refused { .. }
        | c::AccountEnd::Failed { .. } => return None,
    };
    if let Some(pending) = &request.pending
        && let c::Ask::SignIn { provider, .. } = &pending.ask
    {
        return Some(Added { provider: *provider, kind: crate::WayKind::SignIn });
    }
    if let Some(provider) = chosen {
        return Some(Added { provider, kind: crate::WayKind::SignIn });
    }
    for row in &document(client)?.accounts {
        if row.number == account {
            return Some(Added { provider: row.provider?, kind: crate::WayKind::SignIn });
        }
    }
    None
}
