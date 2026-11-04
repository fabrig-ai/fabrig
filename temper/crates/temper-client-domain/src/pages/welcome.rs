//! First-run facts (temper/docs/design/client/README.md, 3.5).
use super::accounts;
use crate::{
    Domain,
    forge::{
        self,
        forges::{self, State as Forges},
    },
};
use jig_client::{Latest, State};
#[derive(Debug)]
pub struct Welcome {
    pub decided: bool,
}
impl Welcome {
    pub(crate) const fn new() -> Self {
        Self { decided: false }
    }
}
/// The store's first-run state (temper/docs/design/client/README.md, 3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstRun {
    Empty,
    Unfinished { step: crate::Step },
    Whole,
}
/// None while the session's playground, forge or account facts are missing.
#[must_use]
pub fn decision(root: &Domain) -> Option<FirstRun> {
    let me = accounts::current_me(root.client())?;
    if !root.forge().playground_answered() {
        return None;
    }
    let model = model_usable(root)?;
    let rows = match root.forges() {
        Forges::Unknown => return None,
        Forges::Ended => None,
        Forges::Rows(rows) => Some(rows),
    };
    let mut kept = false;
    let mut connected = false;
    if let Some(rows) = rows {
        for row in rows {
            if row.mode != forges::Mode::Fake {
                kept = true;
                connected |= forge_connected(row);
            }
        }
    }
    Some(if connected && model && !me.projects.is_empty() {
        FirstRun::Whole
    } else if !kept && !model {
        FirstRun::Empty
    } else if !connected {
        FirstRun::Unfinished { step: crate::Step::Forge { forge: None } }
    } else if !model {
        FirstRun::Unfinished { step: crate::Step::Model { way: None } }
    } else {
        FirstRun::Unfinished { step: crate::Step::Ready }
    })
}
/// A real connected forge, including GitHub's accepted installation (3.5).
#[must_use]
pub fn forge_connected(row: &forges::Row) -> bool {
    let connected = match row.connection {
        forges::Connection::Connected { .. } => true,
        forges::Connection::NotConnected
        | forges::Connection::NotSetUp
        | forges::Connection::SettingUp { .. }
        | forges::Connection::SetUpFailed { .. }
        | forges::Connection::Connecting
        | forges::Connection::Stopped { .. } => false,
    };
    row.mode != forges::Mode::Fake && connected && (row.kind != forges::Kind::GitHub || forges::accepted(row))
}
/// Ready's request and last ending survive navigation (3.5).
#[derive(Debug)]
pub struct DefaultProject {
    pub making: Option<jig_client::Key>,
    pub attempt: u64,
    pub ended: Option<Ended>,
    pub project: Option<u32>,
}
impl DefaultProject {
    pub(crate) const fn new() -> Self {
        Self { making: None, attempt: 0, ended: None, project: None }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Ended {
    Refused(jig_client::contract::Refusal),
    Unknown,
}
/// A real account alone keeps the playground offered (3.5).
#[must_use]
pub fn playground_offered(root: &Domain) -> Option<bool> {
    if !root.forge().playground_answered() {
        return None;
    }
    match root.forges() {
        Forges::Unknown => None,
        Forges::Ended => Some(true),
        Forges::Rows(rows) => {
            for row in rows {
                if row.mode != forges::Mode::Fake {
                    return Some(false);
                }
            }
            Some(true)
        }
    }
}

/// The welcome reads model facts, including an ended watch.
#[must_use]
pub fn model_usable(root: &Domain) -> Option<bool> {
    let document = root.client().document(accounts::handle())?;
    let accounts = if document.state == State::Ended {
        None
    } else if let Some(Latest::Jig(jig_client::contract::Delivery::Accounts(accounts))) = document.latest {
        Some(accounts.as_ref())
    } else {
        return None;
    };
    let Some(accounts) = accounts else { return Some(false) };
    let Some(provider) = root.forge().playground_provider() else { return Some(accounts.usable) };
    Some(forge::model::other_model_usable(
        accounts,
        forge::model::owner_deployment(root.forge(), root.client()),
        provider,
    ))
}
