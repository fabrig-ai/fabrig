//! Independent policy values shared until either observer edits one
//! (`jig/docs/design/domain/testing.md`, 6).
use super::Policy;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// A complete scenario policy. Sharing is immutable: mutable access first
/// detaches the value, so an adapter cannot edit the referee's retained policy.
#[derive(Clone, PartialEq, Eq)]
pub struct PolicySnapshot(Arc<Policy>);

impl From<Policy> for PolicySnapshot {
    fn from(policy: Policy) -> Self {
        Self(Arc::new(policy))
    }
}

impl std::fmt::Debug for PolicySnapshot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Deref for PolicySnapshot {
    type Target = Policy;
    fn deref(&self) -> &Policy {
        &self.0
    }
}

impl DerefMut for PolicySnapshot {
    fn deref_mut(&mut self) -> &mut Policy {
        Arc::make_mut(&mut self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::referee::Scope;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn policy_snapshots_preserve_the_referees_send_and_sync_contract() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PolicySnapshot>();
        assert_send_sync::<crate::referee::Referee>();
    }

    #[test]
    fn policy_snapshots_keep_both_observers_edits_independent() {
        let original: PolicySnapshot = Policy {
            deployment: Scope::default(),
            projects: BTreeMap::new(),
            projections: BTreeMap::new(),
            people: BTreeMap::new(),
            effect_accepters: BTreeSet::new(),
            requirements: Vec::new(),
            implications: BTreeSet::new(),
            kinds: BTreeMap::new(),
            owned: Vec::new(),
            participating: BTreeSet::new(),
            mechanics: BTreeSet::new(),
            delivery_bound: 20,
            uncertainty_bound: 30,
            story_steps: 10,
        }.into();
        let mut adapter = original.clone();
        let mut referee = original.clone();
        adapter.deployment.budget = 7;
        assert_eq!(referee.deployment.budget, 0);
        referee.delivery_bound = 40;
        assert_eq!(adapter.delivery_bound, 20);
        assert_eq!(original.deployment.budget, 0);
        assert_eq!(original.delivery_bound, 20);
        referee.clone_from(&adapter);
        assert_eq!(referee, adapter);
        adapter.projects.insert(1, Scope { budget: 9, ..Scope::default() });
        assert!(referee.projects.is_empty());
    }
}
