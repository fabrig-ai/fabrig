//! Credential-bearing test application configuration.
use super::root;
use crate::fixtures::WORKSPACE_CREDENTIAL;
use jig_domain_tasks::{HoldKind, ResourceKind, Taken};

#[must_use]
pub fn with_workspace_credential(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, limits) = crate::charters::two_charters(seed);
    config.resource_kinds[0] = Box::new([ResourceKind { kind: 1, hold: HoldKind::Exclusive { taken: Taken::Waits } }]);
    config.first.resources[0].credential = Some(WORKSPACE_CREDENTIAL);
    (config, limits)
}
