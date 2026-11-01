//! The welcome's store states (temper/docs/design/draft/playground-reset.md, 6.3–6.4).
use super::accounts;
use crate::{
    Domain,
    forge::{self, forges::State as Forges},
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
/// The store as the welcome reads it (temper/docs/design/draft/playground-reset.md, 6.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Empty,
    Unfinished(crate::Step),
    Whole,
}

/// None until the session's welcome facts have answered.
#[must_use]
pub fn setup(root: &Domain) -> Option<Setup> {
    let me = accounts::current_me(root.client())?;
    if !root.demo_answered() {
        return None;
    }
    let model = model_usable(root)?;
    if me.deployment_role != Some(jig_client::contract::HeldRole::Owner) {
        return Some(if model { Setup::Whole } else { Setup::Empty });
    }
    let mut kept = false;
    let mut finished = false;
    match root.forges() {
        Forges::Unknown => return None,
        Forges::Ended => {}
        Forges::Rows(rows) => {
            for row in rows {
                if !row.playground {
                    kept = true;
                    finished |= row.finished;
                }
            }
        }
    }
    Some(if finished && model {
        Setup::Whole
    } else if !kept && !model {
        Setup::Empty
    } else if !finished {
        Setup::Unfinished(crate::Step::Forge { forge: None })
    } else {
        Setup::Unfinished(crate::Step::Model { way: None })
    })
}

/// A real forge kept in the store withdraws the playground offer.
#[must_use]
pub fn playground_offered(root: &Domain) -> bool {
    let Some(playground) = root.forge().demo() else { return false };
    if !root.demo_answered() || playground.state != forge::demo::DemoState::NotStarted {
        return false;
    }
    match root.forges() {
        Forges::Unknown => false,
        Forges::Ended => true,
        Forges::Rows(rows) => {
            for row in rows {
                if !row.playground {
                    return false;
                }
            }
            true
        }
    }
}

/// None while facts are missing; otherwise whether the person's first run is over.
#[must_use]
pub fn decision(root: &Domain) -> Option<bool> {
    let setup = setup(root)?;
    Some(root.forge().demo_running() || root.forge().demo_resetting() || setup == Setup::Whole)
}

/// The welcome and setup strip use the same model facts, including an ended watch.
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
    let Some(provider) = root.forge().demo_provider() else { return Some(accounts.usable) };
    for line in &accounts.accounts {
        if forge::demo::usable_account(line, Some(provider)) {
            return Some(true);
        }
    }
    Some(false)
}
