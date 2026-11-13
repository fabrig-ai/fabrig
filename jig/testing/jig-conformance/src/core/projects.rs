//! Independent configured policies selected by incoming makes, applied only
//! when their durable answers name the made projects (domain/people/acts.md, 2–3).
use super::Store;
use jig_domain_authority as authority;
use jig_domain_core_vocabulary as core;
use jig_domain_people as people;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Makes {
    configured: BTreeMap<u32, authority::Policy>,
    requests: BTreeMap<people::RequestKey, authority::Policy>,
    acts: BTreeMap<people::ActKey, authority::Policy>,
}
impl Makes {
    pub(super) fn new(config: &core::Config) -> Self {
        let mut makes = Self { configured: BTreeMap::new(), requests: BTreeMap::new(), acts: BTreeMap::new() };
        makes.configure(config);
        makes
    }
    pub(super) fn configure(&mut self, config: &core::Config) {
        self.configured =
            config.defaults.policies.iter().map(|(number, seed)| (*number, seed.policy.clone())).collect();
    }
    pub(super) fn ask(&mut self, store: &Store<'_>, event: &core::Event) {
        if let core::Event::People(people::Event::Ask {
            sign_in,
            key,
            ask: people::Ask::MakeProject { policy, .. },
            ..
        }) = event
        {
            let record = store.rows.get(&core::Key::People(people::Key::SignIn(*sign_in)));
            if let Some(core::Record::People(people::Stored::SignIn { person, .. })) = record
                && let Some(policy) = self.configured.get(policy)
            {
                self.requests.insert(people::RequestKey { person: *person, key: *key }, policy.clone());
            }
        }
    }
    pub(super) fn act(&mut self, event: &core::Event) {
        if let core::Event::Act { connector, key, act, .. } = event
            && let people::Act::MakeProject { policy, .. } = &**act
            && let Some(policy) = self.configured.get(policy)
        {
            self.acts.insert(people::ActKey { connector: *connector, key: *key }, policy.clone());
        }
    }
    pub(super) fn durable(&mut self, store: &Store<'_>) -> Vec<(u32, authority::Policy)> {
        let mut projects = Vec::new();
        for row in store.rows.values().copied() {
            let (policy, outcome) = match row {
                core::Record::People(people::Stored::Answer { key, outcome, .. }) => {
                    (self.requests.remove(key), outcome)
                }
                core::Record::People(people::Stored::Acted { key, outcome, .. }) => (self.acts.remove(key), outcome),
                core::Record::Core(_)
                | core::Record::Tasks(_)
                | core::Record::People(_)
                | core::Record::Notes(_)
                | core::Record::Accounts(_) => continue,
            };
            if let Some(policy) = policy
                && let people::Outcome::ProjectMade { project } = outcome
            {
                projects.push((*project, policy));
            }
        }
        projects
    }
}
