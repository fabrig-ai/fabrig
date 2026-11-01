//! The first-run state (temper/docs/design/draft/playground-reset.md, 6.3–6.4).
use super::accounts;
use crate::Step;
use crate::{
    Domain,
    forge::{self, forges::State as Forges},
};
use jig_client::{Latest, State as DocumentState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Empty,
    Unfinished(Step),
    Whole,
}
#[derive(Debug)]
pub struct Welcome {
    pub decided: bool,
}
impl Welcome {
    pub(crate) const fn new() -> Self {
        Self { decided: false }
    }
}
/// None while facts are missing; otherwise the deployment's first-run state.
#[must_use]
pub fn state(root: &Domain) -> Option<State> {
    let me = accounts::current_me(root.client())?;
    if !root.demo_answered() {
        return None;
    }
    let model = model_usable(root)?;
    if me.deployment_role != Some(jig_client::contract::HeldRole::Owner) {
        return Some(if model || root.forge().demo_running() { State::Whole } else { State::Empty });
    }
    let mut begun = false;
    let mut whole = false;
    match root.forges() {
        Forges::Unknown => return None,
        Forges::Ended => {}
        Forges::Rows(rows) => {
            for row in rows {
                if forge::listed::personal(row, root.forge().demo()) {
                    begun |= forge::connect::begun(row);
                    whole |= forge::connect::whole(row);
                }
            }
        }
    }
    Some(if root.forge().demo_running() || whole && model {
        State::Whole
    } else if whole {
        State::Unfinished(Step::Model { way: None })
    } else if begun || real_account(root) {
        State::Unfinished(Step::Forge { forge: None })
    } else {
        State::Empty
    })
}
#[must_use]
pub fn decision(root: &Domain) -> Option<bool> {
    Some(state(root)? == State::Whole)
}
fn real_account(root: &Domain) -> bool {
    let provider = root.forge().demo_provider();
    if let Some(document) = root.client().document(accounts::handle())
        && document.state != DocumentState::Ended
        && let Some(Latest::Jig(jig_client::contract::Delivery::OwnerAccounts(accounts))) = document.latest
    {
        for account in &accounts.accounts {
            if account.provider.is_some() && account.provider != provider {
                return true;
            }
        }
    }
    if let Some(deployment) = forge::demo::owner_deployment(root.forge(), root.client()) {
        for account in &deployment.accounts {
            if account.provider.is_some() && account.provider != provider {
                return true;
            }
        }
    }
    false
}

/// The welcome and setup strip use the same model facts, including an ended watch.
#[must_use]
pub fn model_usable(root: &Domain) -> Option<bool> {
    let document = root.client().document(accounts::handle())?;
    let accounts = if document.state == DocumentState::Ended {
        None
    } else if let Some(Latest::Jig(jig_client::contract::Delivery::OwnerAccounts(accounts))) = document.latest {
        Some(accounts.as_ref())
    } else {
        return None;
    };
    let Some(accounts) = accounts else { return Some(false) };
    let Some(provider) = root.forge().demo_provider() else { return Some(accounts.usable) };
    Some(forge::demo::other_model_usable(
        accounts,
        forge::demo::owner_deployment(root.forge(), root.client()),
        provider,
    ))
}
