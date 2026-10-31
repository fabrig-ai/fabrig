//! Reader bridges; t-view-drop removes them.
pub(crate) use super::playground::told;
pub use super::playground::{StartAnswer, StartFailure, answer, ask};
use super::{Request, Side};
use jig_client::{Client, Handle, Latest, contract as c};
use skein_lib::{Queue, Token};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoState {
    NotStarted,
    Running,
    Ended,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetiredDemo {
    pub forge: u16,
    pub provider: u32,
    pub project: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Halves {
    pub model: bool,
    pub forge: bool,
}
pub(crate) fn current(side: &Side, _accounts: Option<&c::OwnerAccounts>, _client: &Client) -> Halves {
    let running = side.playground().is_some();
    Halves { model: running, forge: running }
}

impl Side {
    #[must_use]
    pub fn serves_pages(&self, forge: u16) -> bool {
        match self.playground() {
            Some(playground) => playground.forge != forge,
            None => true,
        }
    }
    #[must_use]
    pub fn demo_request(&self) -> Option<Token> {
        Some(self.playground.start.as_ref()?.request)
    }
    #[must_use]
    pub const fn retired_demo(&self) -> Option<RetiredDemo> {
        None
    }
    #[must_use]
    pub fn demo_provider(&self) -> Option<u32> {
        self.playground_provider()
    }
}
pub(crate) fn progress(
    side: &mut Side,
    _accounts: Option<&c::OwnerAccounts>,
    client: &Client,
    out: &mut Queue<Request>,
) {
    watch_deployment(side, client, out);
}
/// A usable source of a provider other than the demo's, or any provider without a demo.
#[must_use]
pub fn usable_account(line: &c::OwnerAccountLine, demo_provider: Option<u32>) -> bool {
    if demo_provider.is_some() && line.provider == demo_provider {
        return false;
    }
    usable_sources(&line.sources)
}

fn usable_sources(sources: &skein_lib::List<c::OwnerAccountSource>) -> bool {
    for source in sources {
        if source.usable {
            return true;
        }
    }
    false
}

/// A usable model of a provider other than the demo's: one of the person's `accounts`
/// lines, or, for an owner, a deployment-held account of its delivered, uncut
/// `deployment` document (client/README.md, 3.5 and 4.8).
#[must_use]
pub fn other_model_usable(
    accounts: &c::OwnerAccounts,
    deployment: Option<&c::OwnerDeployment>,
    demo_provider: u32,
) -> bool {
    for line in &accounts.accounts {
        if usable_account(line, Some(demo_provider)) {
            return true;
        }
    }
    match deployment {
        Some(deployment) => deployment_model(deployment, demo_provider).is_some(),
        None => false,
    }
}

/// The first deployment-held account of another provider than the demo's that is usable
/// and uncut, in an owner's delivered, uncut `deployment` (client/README.md, 3.5, 4.8).
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses bounded loops without closures")]
pub fn deployment_model(deployment: &c::OwnerDeployment, demo_provider: u32) -> Option<&c::OwnerAccount> {
    if deployment.cut {
        return None;
    }
    for account in &deployment.accounts {
        if !account.cut
            && account.holder == c::OwnerAccountHolder::Deployment
            && account.provider != Some(demo_provider)
            && usable_sources(&account.sources)
        {
            return Some(account);
        }
    }
    None
}

/// The owner's delivered, uncut deployment, through the demo's one live watch.
#[must_use]
pub fn owner_deployment<'a>(side: &Side, client: &'a Client) -> Option<&'a c::OwnerDeployment> {
    if !owner(client) {
        return None;
    }
    let (handle, _) = side.playground.deployment?;
    let document = client.document(handle)?;
    if document.state != jig_client::State::Live {
        return None;
    }
    let Some(Latest::Jig(c::Delivery::OwnerDeployment(deployment))) = document.latest else { return None };
    if deployment.cut {
        return None;
    }
    Some(deployment)
}

fn watch_deployment(side: &mut Side, client: &Client, out: &mut Queue<Request>) {
    let needed = owner(client) && side.playground().is_some();
    if !needed {
        if let Some((handle, _)) = side.playground.deployment.take() {
            out.push(Request::Client { asked: jig_client::Event::Unwatch { handle } });
        }
        return;
    }
    if side.playground.deployment.is_some() {
        return;
    }
    let Some(provider) = side.playground_provider() else { return };
    side.serial = side.serial.checked_add(1).expect("forge handles fit");
    let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    side.playground.deployment = Some((handle, provider));
    out.push(Request::Client {
        asked: jig_client::Event::Watch { handle, subject: jig_client::Watchable::Jig(c::Subject::Deployment) },
    });
}

fn owner(client: &Client) -> bool {
    let Some(document) = client.document(super::me::handle()) else { return false };
    if document.state != jig_client::State::Live {
        return false;
    }
    let Some(Latest::Jig(c::Delivery::Me(me))) = document.latest else { return false };
    me.deployment_role == Some(c::HeldRole::Owner)
}
