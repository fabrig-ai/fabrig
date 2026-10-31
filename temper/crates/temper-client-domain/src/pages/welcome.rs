//! First-run facts (temper/docs/design/draft/playground-reset.md, 6.1, 6.3–6.4).
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
/// The store's first-run state (temper/docs/design/draft/playground-reset.md, 6.3–6.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstRun {
    Empty,
    Unfinished { step: crate::Step },
    Whole,
}
/// None while the session's playground, forge or account facts are missing.
#[must_use]
pub fn decision(root: &Domain) -> Option<FirstRun> {
    accounts::current_me(root.client())?;
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
                connected |= first_adopted(row);
            }
        }
    }
    Some(if connected && model {
        FirstRun::Whole
    } else if !kept && !model {
        FirstRun::Empty
    } else if !connected {
        FirstRun::Unfinished { step: crate::Step::Forge { forge: None } }
    } else {
        FirstRun::Unfinished { step: crate::Step::Model { way: None } }
    })
}
fn first_adopted(row: &forges::Row) -> bool {
    match row.connection {
        forges::Connection::Connected { .. } => {}
        forges::Connection::NotConnected
        | forges::Connection::NotSetUp
        | forges::Connection::SettingUp { .. }
        | forges::Connection::SetUpFailed { .. }
        | forges::Connection::Connecting
        | forges::Connection::Stopped { .. } => return false,
    }
    let Some(first) = &row.first else { return false };
    match first.state {
        forges::FirstState::Made { .. } => {}
        forges::FirstState::Making | forges::FirstState::Failed { .. } => return false,
    }
    for repository in &row.repositories {
        if repository.owner == first.org && repository.name == first.repository {
            return true;
        }
    }
    false
}
/// A real account alone keeps the playground offered (6.1 and 6.3).
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
    } else if let Some(Latest::Jig(jig_client::contract::Delivery::OwnerAccounts(accounts))) = document.latest {
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
