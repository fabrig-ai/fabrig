//! Durable-row translations and independent production evidence for jig's referee.
use jig_conformance::referee as r;
use jig_domain_people as people;
use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infra;
use jig_ops_domain_observability as obs;
use r::EffectKey;
use std::collections::{BTreeMap, BTreeSet};

pub fn fact(observed: &mut Vec<r::Observed>, event: &obs::SystemEvent) {
    if let obs::SystemEvent::Fact { service, fact } = event {
        let target = if service.environment.as_ref() == b"production" { 77 } else { 1 };
        let met = if target == 77 {
            fact.healthy_replicas >= 2
        } else {
            fact.load_percent < 20
                && fact.load.first().is_some_and(|point| point.at <= fact.observed.saturating_sub(1800))
                && fact.observed >= 1800
                && fact.load.iter().all(|point| point.percent < 20)
        };
        observed.push(r::Observed::Read {
            resource: r::Name {
                connector: 1,
                path: vec![b"env".to_vec(), service.environment.to_vec(), b"service".to_vec(), service.name.to_vec()],
            },
            state: met.then_some(target),
            at: fact.observed * 1_000_000_000,
        });
    }
}

pub fn key(value: infra::Key) -> EffectKey {
    let origin = match value.origin {
        infra::Purpose::Call { attempt, completion, position } => {
            r::EffectOrigin::Call { attempt, completion, position }
        }
        infra::Purpose::Procedure { .. } => r::EffectOrigin::Procedure,
        infra::Purpose::Projection { .. } => r::EffectOrigin::Projection,
    };
    EffectKey { deployment: value.deployment, task: value.task, purpose: value.purpose, origin }
}

pub fn parts(effect: &infra::Effect) -> (u16, Vec<r::Name>, Option<u64>, u64) {
    match effect {
        infra::Effect::Restart { service, operation } => (1, vec![resource(service, 2)], None, *operation),
        infra::Effect::Scale { service, from, to } => {
            (2, vec![resource(service, 2)], Some(u64::from(*from)), u64::from(*to))
        }
        // This system write creates the environment. Its pool is an authority
        // and quota dependency; used_slots is derived from existing environments,
        // not a pool value overwritten with this environment's lease end.
        infra::Effect::CreateEnvironment { environment, until, .. } => (
            4,
            vec![r::Name {
                connector: 2,
                path: infra::Resource::Environment(environment.clone()).segments().iter().map(|v| v.to_vec()).collect(),
            }],
            None,
            *until,
        ),
        infra::Effect::TearDown { environment, .. } => (
            5,
            vec![r::Name {
                connector: 2,
                path: infra::Resource::Environment(environment.clone())
                    .segments()
                    .iter()
                    .map(|part| part.to_vec())
                    .collect(),
            }],
            Some(1),
            0,
        ),
        infra::Effect::Rollback { .. } => {
            panic!("the two stories use restart and scale")
        }
    }
}

pub fn resource(service: &infra::Service, connector: u16) -> r::Name {
    r::Name {
        connector,
        path: vec![b"env".to_vec(), service.environment.to_vec(), b"service".to_vec(), service.name.to_vec()],
    }
}

pub fn policy(seed: u64, environment_chat: bool) -> r::Policy {
    let (configuration, _) = super::config::root_for(seed, environment_chat);
    let rules = &configuration.core.defaults.rules;
    let ceiling = jig_conformance::core::scope(&rules.ceiling);
    let mut policy = r::Policy {
        deployment: ceiling.clone(),
        projects: BTreeMap::from([(1, ceiling)]),
        projections: BTreeMap::new(),
        people: BTreeMap::from([(
            (1, 1),
            jig_conformance::core::scope(&if environment_chat {
                super::config::environment_authority(500, true)
            } else {
                super::config::authority(500, true)
            }),
        )]),
        effect_accepters: BTreeSet::from([(1, 1)]),
        requirements: Vec::new(),
        implications: BTreeSet::new(),
        kinds: BTreeMap::from([
            ((2, 1), r::Kind { recovery: r::Recovery::Keyed, price: 0 }),
            ((2, 2), r::Kind { recovery: r::Recovery::Conditional, price: 0 }),
        ]),
        owned: vec![(2, r::Pattern { base: vec![b"env".to_vec()], terminal: Vec::new(), open: true })],
        participating: BTreeSet::new(),
        mechanics: BTreeSet::new(),
        delivery_bound: 60_000_000_000,
        uncertainty_bound: 60_000_000_000,
        story_steps: 2000,
    };
    if environment_chat {
        policy.kinds.insert((2, 4), r::Kind { recovery: r::Recovery::Keyed, price: 40 });
        policy.kinds.insert((2, 5), r::Kind { recovery: r::Recovery::Conditional, price: 0 });
        policy.owned.push((2, r::Pattern { base: vec![b"pool".to_vec()], terminal: Vec::new(), open: true }));
    }
    for kind in [1, 2] {
        policy.requirements.push(r::Requirement {
            project: None,
            connector: 2,
            kind,
            pattern: r::Pattern { base: vec![b"env".to_vec()], terminal: Vec::new(), open: true },
            judge: 1,
            freshness: Some(30_000_000_000),
        });
    }
    policy
}

pub use super::store::Store;

pub(super) fn core_store(store: &Store) -> jig_conformance::core::Store<'_> {
    jig_conformance::core::Store {
        applied: store.applied,
        rows: store
            .rows
            .values()
            .filter_map(|record| match record {
                root::Record::Core(record) => Some((record.key(), record)),
                root::Record::Infrastructure(_) | root::Record::Observability(_) => None,
            })
            .collect(),
        ..jig_conformance::core::Store::default()
    }
}

pub fn inbound(
    intents: &mut jig_conformance::core::intents::Intents,
    store: &Store,
    event: &mut root::Event,
) -> Vec<r::Observed> {
    if let root::Event::Core(event) = event {
        return intents.inbound_core(&core_store(store), event);
    }
    if let root::Event::Effect { key, proposal: Some(_), .. } = event {
        return intents.propose(*key);
    }
    Vec::new()
}

pub fn intents(intents: &mut jig_conformance::core::intents::Intents, store: &Store) -> Vec<r::Observed> {
    intents.durable(&core_store(store))
}

pub fn snapshot(store: &Store) -> r::Snapshot {
    let records = core_store(store).rows;
    let mut snapshot = jig_conformance::core::snapshot(&records);
    let decisions = jig_conformance::core::proposal_decisions(&records);
    for record in store.rows.values() {
        match record {
            root::Record::Infrastructure(infra::Record::Outbox(row)) => {
                let key = key(row.key);
                let (kind, resources, condition, target) = parts(&row.effect);
                if row.attempt > 0 {
                    snapshot.receipts.insert(r::Receipt::Outbox(2, key, row.attempt));
                    snapshot.attempts.insert((2, key), (row.attempt, row.deadline * 1_000_000_000));
                }
                if row.phase == infra::Phase::Held {
                    snapshot.held_effects.insert((2, key));
                }
                snapshot.decisions.push(r::Decision {
                    projection: false,
                    connector: 2,
                    key,
                    kind,
                    resources,
                    condition,
                    target,
                    judged_state: target,
                    accepted_by: None,
                });
            }
            root::Record::Core(_) | root::Record::Infrastructure(_) | root::Record::Observability(_) => {}
        }
    }
    for accepted in decisions {
        for decision in &mut snapshot.decisions {
            if let people::Proposer::Task(task) = accepted.proposer
                && decision.key.task == task
            {
                decision.accepted_by = Some(r::Source::Person(accepted.by));
                snapshot.acceptances.insert(
                    (2, decision.key),
                    r::Acceptance { proposal: accepted.proposal, by: r::Source::Person(accepted.by) },
                );
            }
        }
    }
    snapshot
}
