//! Account facts for the playground (temper/docs/design/client/README.md, 3.5).
use super::{Request, Side};
use jig_client::{Client, Handle, Latest, contract as c};
use skein_lib::Queue;
pub(crate) fn progress(side: &mut Side, client: &Client, out: &mut Queue<Request>) {
    watch_deployment(side, client, out);
}
/// A usable source of a provider other than the playground's, or any provider without a playground.
#[must_use]
pub fn usable_account(line: &c::AccountLine, playground_provider: Option<u32>) -> bool {
    if playground_provider.is_some() && line.provider == playground_provider {
        return false;
    }
    usable_sources(&line.sources)
}

fn usable_sources(sources: &skein_lib::List<c::AccountSource>) -> bool {
    for source in sources {
        if source.usable {
            return true;
        }
    }
    false
}

/// A usable model of a provider other than the playground's: one of the person's `accounts`
/// lines, or, for an owner, a deployment-held account of its delivered, uncut
/// `deployment` document (client/README.md, 3.5 and 4.8).
#[must_use]
pub fn other_model_usable(
    accounts: &c::Accounts,
    deployment: Option<&c::Deployment>,
    playground_provider: u32,
) -> bool {
    for line in &accounts.accounts {
        if usable_account(line, Some(playground_provider)) {
            return true;
        }
    }
    match deployment {
        Some(deployment) => deployment_model(deployment, playground_provider).is_some(),
        None => false,
    }
}

/// The first deployment-held account of another provider than the playground's that is usable
/// and uncut, in an owner's delivered, uncut `deployment` (client/README.md, 3.5, 4.8).
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses bounded loops without closures")]
pub fn deployment_model(deployment: &c::Deployment, playground_provider: u32) -> Option<&c::Account> {
    if deployment.cut {
        return None;
    }
    for account in &deployment.accounts {
        if !account.cut
            && account.holder == c::AccountHolder::Deployment
            && account.provider != Some(playground_provider)
            && usable_sources(&account.sources)
        {
            return Some(account);
        }
    }
    None
}

/// The owner's delivered, uncut deployment, through the playground's one live watch.
#[must_use]
pub fn owner_deployment<'a>(side: &Side, client: &'a Client) -> Option<&'a c::Deployment> {
    if !owner(client) {
        return None;
    }
    let (handle, _) = side.playground.deployment?;
    let document = client.document(handle)?;
    if document.state != jig_client::State::Live {
        return None;
    }
    let Some(Latest::Jig(c::Delivery::Deployment(deployment))) = document.latest else { return None };
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
