//! Durable-row translations and independent production evidence for jig's referee.
use jig_conformance::referee as r;
use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infra;
use jig_ops_domain_observability as obs;
use r::EffectKey;
use std::collections::BTreeMap;

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

pub fn observer(seed: u64, environment_chat: bool) -> jig_conformance::core::Observer {
    let (configuration, _) = super::config::root_for(seed, environment_chat);
    let mut kinds = BTreeMap::from([
        ((2, 1), r::Kind { recovery: r::Recovery::Keyed, price: 0 }),
        ((2, 2), r::Kind { recovery: r::Recovery::Conditional, price: 0 }),
    ]);
    let mut owned = vec![(2, r::Pattern { base: vec![b"env".to_vec()], terminal: Vec::new(), open: true })];
    if environment_chat {
        kinds.insert((2, 4), r::Kind { recovery: r::Recovery::Keyed, price: 40 });
        kinds.insert((2, 5), r::Kind { recovery: r::Recovery::Conditional, price: 0 });
        owned.push((2, r::Pattern { base: vec![b"pool".to_vec()], terminal: Vec::new(), open: true }));
    }
    let mut observer = jig_conformance::core::Observer::new(
        &configuration.core,
        jig_conformance::core::ConnectorPolicy { kinds, owned },
    );
    observer.record();
    observer
}

pub type Store = jig_fake_store::Store<root::Key, root::Record>;

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
        pending: store
            .pending
            .iter()
            .map(|pending| jig_conformance::core::Commit {
                number: pending.number,
                writes: pending
                    .writes
                    .iter()
                    .map(|write| match write {
                        jig_fake_store::Write::Save { row: root::Record::Core(row), .. } => {
                            jig_conformance::core::Write::Save(row)
                        }
                        jig_fake_store::Write::Erase(root::Key::Core(key)) => {
                            jig_conformance::core::Write::Erase(key.clone())
                        }
                        jig_fake_store::Write::Save { .. } | jig_fake_store::Write::Erase(_) => {
                            jig_conformance::core::Write::Other
                        }
                    })
                    .collect(),
            })
            .collect(),
        effects: effects(store),
        ..jig_conformance::core::Store::default()
    }
}

pub fn effects(store: &Store) -> jig_conformance::core::Effects {
    let mut snapshot = jig_conformance::core::Effects::default();
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
            root::Record::Infrastructure(infra::Record::Proposal { number, effect, .. }) => {
                let (kind, _, _, target) = parts(effect);
                snapshot
                    .proposals
                    .insert(*number, (2, jig_conformance::core::Effect { kind, target, purpose: target }));
            }
            root::Record::Core(_) | root::Record::Infrastructure(_) | root::Record::Observability(_) => {}
        }
    }
    snapshot
}

pub fn snapshot(store: &Store) -> r::Snapshot {
    let view = core_store(store);
    let mut snapshot = jig_conformance::core::snapshot(&view.rows);
    snapshot.receipts.extend(view.effects.receipts);
    snapshot.attempts.extend(view.effects.attempts);
    snapshot.held_effects.extend(view.effects.held_effects);
    snapshot.decisions = view.effects.decisions;
    snapshot
}
