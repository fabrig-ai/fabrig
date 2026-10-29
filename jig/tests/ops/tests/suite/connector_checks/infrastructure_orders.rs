//! Restart and task closure from jig/docs/design/domain/connectors.md, sections 4 and 14.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use jig_ops_domain_infrastructure as infra;
use jig_ops_fake_production as production;
use jig_ops_world::{InfrastructureWorld, infrastructure_limits};
use skein_lib::Token;

// Five tasks naming thirteen services each keep 65 refreshes live, past the 64-slot carrier.
const LIVE_SERVICES: u32 = 65;
const HELD_SERVICE: u32 = 0;
const HEALTHY_REPLICAS: u32 = 3; // The fake services start with three healthy replicas.
const RESOURCE_OWNERS: u32 = 5;
const SERVICES_PER_OWNER: u32 = 13;
const FIRST_RESOURCE_OWNER: u64 = 100;
const FIRST_CLOSING_OWNER: u64 = 1000;
const EFFECT_OWNER: u64 = 7;
const FIRST_EFFECT_ENTRY: u64 = 10;
const SECOND_EFFECT_ENTRY: u64 = 11;
const EFFECT_ENTRIES: [u64; 2] = [FIRST_EFFECT_ENTRY, SECOND_EFFECT_ENTRY]; // Two unsettled effects make the last-result fence observable.
const PROJECT: u32 = 1;

fn service(number: u32) -> infra::Service {
    infra::Service::new(Box::from(*b"production"), format!("service-{number}").into_bytes().into_boxed_slice())
}

fn key(entry: u64) -> infra::Key {
    infra::Key::procedure([1; 16], EFFECT_OWNER, entry)
}

fn loaded_world() -> (InfrastructureWorld, Vec<infra::SystemEvent>) {
    let mut bounds = infrastructure_limits();
    bounds.services = LIVE_SERVICES;
    bounds.tasks = RESOURCE_OWNERS + 1; // One task slot is reused by the ready-close batches.
    bounds.resources_per_task = SERVICES_PER_OWNER;
    let mut world = InfrastructureWorld::with_limits(0, infra::Backend::OperationIds, bounds);
    for owner in 0..RESOURCE_OWNERS {
        let mut resources = Vec::new();
        for local in 0..SERVICES_PER_OWNER {
            let number = owner * SERVICES_PER_OWNER + local;
            let name = format!("service-{number}");
            world.production.add_service(production::ServiceName::new("production", &name), "v1", HEALTHY_REPLICAS);
            resources.push(infra::Resource::Service(service(number)));
        }
        world.event(infra::Event::Names {
            task: FIRST_RESOURCE_OWNER + u64::from(owner),
            project: PROJECT,
            resources: resources.into_boxed_slice(),
        });
    }
    let mut replies = Vec::new();
    for (number, entry) in EFFECT_ENTRIES.into_iter().enumerate() {
        let token = Token::new(entry);
        world.event(infra::Event::Describe {
            token,
            effect: infra::Effect::Restart {
                service: service(u32::try_from(number).expect("effect index fits")),
                operation: entry,
            },
        });
        world.event(infra::Event::Keep { project: Some(PROJECT), entry, token, key: key(entry) });
        for request in world.event(infra::Event::Make { key: key(entry) }) {
            if let infra::Request::System(call) = request {
                replies.push(world.system(call));
            }
        }
    }
    world.restore();
    world.seen.clear();
    (world, replies)
}

fn enqueue(requests: Vec<infra::Request>, pending: &mut VecDeque<infra::SystemRequest>) {
    for request in requests {
        if let infra::Request::System(call) = request {
            pending.push_back(call);
        }
    }
}

fn restarts(world: &InfrastructureWorld) -> Vec<infra::RestartStage> {
    let mut stages = Vec::new();
    for request in &world.seen {
        if let infra::Request::Restarted { stage } = request {
            stages.push(*stage);
        }
    }
    stages
}

fn hold_refresh_under_load(world: &mut InfrastructureWorld) -> infra::SystemEvent {
    let mut pending = VecDeque::new();
    enqueue(world.event(infra::Event::ReadAfresh), &mut pending);
    enqueue(world.event(infra::Event::Close { task: EFFECT_OWNER }), &mut pending);
    for number in 0..LIVE_SERVICES {
        let task = FIRST_CLOSING_OWNER + u64::from(number);
        enqueue(
            world.event(infra::Event::Names {
                task,
                project: PROJECT,
                resources: Box::from([infra::Resource::Service(service(HELD_SERVICE))]),
            }),
            &mut pending,
        );
        enqueue(world.event(infra::Event::Close { task }), &mut pending);
        enqueue(world.event(infra::Event::Release { task }), &mut pending);
    }
    let mut held = None;
    while let Some(call) = pending.pop_front() {
        let reply = world.system(call);
        if matches!(&reply, infra::SystemEvent::Service { service: name, .. } if *name == service(HELD_SERVICE)) {
            held = Some(reply);
        } else {
            enqueue(world.event(infra::Event::System(reply)), &mut pending);
        }
        assert!(restarts(world).is_empty(), "a held fact prevents read-afresh completion");
        assert!(!world.seen.contains(&infra::Request::Closed { task: EFFECT_OWNER }));
    }
    held.expect("the first live service's fact is held")
}

fn assert_results(world: &InfrastructureWorld, stages: &[infra::RestartStage]) {
    let mut refreshes = BTreeMap::new();
    let mut closed = BTreeSet::new();
    let mut outcomes = Vec::new();
    for request in &world.seen {
        if let infra::Request::System(infra::SystemRequest::Service { service }) = request {
            *refreshes.entry(service.clone()).or_insert(0) += 1;
        }
        if let infra::Request::Closed { task } = request {
            assert!(closed.insert(*task), "one close per task");
        }
        if let infra::Request::Outcome { entry, key, outcome } = request {
            outcomes.push((*entry, *key, *outcome));
        }
    }
    assert_eq!(refreshes.len(), usize::try_from(LIVE_SERVICES).expect("fixture count fits"));
    for number in 0..LIVE_SERVICES {
        assert_eq!(refreshes.get(&service(number)), Some(&1));
        assert!(closed.remove(&(FIRST_CLOSING_OWNER + u64::from(number))));
    }
    assert_eq!(closed, BTreeSet::from([EFFECT_OWNER]));
    assert_eq!(
        outcomes,
        [
            (FIRST_EFFECT_ENTRY, key(FIRST_EFFECT_ENTRY), infra::Outcome::Made),
            (SECOND_EFFECT_ENTRY, key(SECOND_EFFECT_ENTRY), infra::Outcome::Made)
        ]
    );
    assert_eq!(restarts(world), stages);
}

/// Pins `read_afresh_complete` and `outbox_settled`: 65 refreshes are live
/// concurrently, past the 64-slot carrier; 65 ready closes run in single-task
/// batches because the closing table cannot hold 65. A fact and two lookups wait.
#[test]
fn a_restart_reports_only_after_every_refresh_settles() {
    let (mut world, _) = loaded_world();
    let fact = hold_refresh_under_load(&mut world);
    world.event(infra::Event::System(fact.clone()));
    assert_eq!(restarts(&world), [infra::RestartStage::ReadAfresh]);
    let mut replies = VecDeque::new();
    for request in world.event(infra::Event::Restart) {
        if let infra::Request::System(call) = request {
            replies.push_back(world.system(call));
        }
    }
    world.event(infra::Event::System(fact));
    assert_eq!(restarts(&world), [infra::RestartStage::ReadAfresh]);
    assert!(!world.seen.contains(&infra::Request::Closed { task: EFFECT_OWNER }));
    let first = replies.pop_front().expect("first effect lookup is held");
    world.event(infra::Event::System(first.clone()));
    assert_eq!(restarts(&world), [infra::RestartStage::ReadAfresh]);
    assert!(!world.seen.contains(&infra::Request::Closed { task: EFFECT_OWNER }));
    for reply in replies {
        world.event(infra::Event::System(reply.clone()));
        world.event(infra::Event::System(reply));
    }
    world.event(infra::Event::System(first));
    assert_results(&world, &[infra::RestartStage::ReadAfresh, infra::RestartStage::Outbox]);
}

/// Pins `task_effects_settled`: 65 concurrent refreshes and 65 ready closes
/// across single-task batches exceed the 64-slot carrier. The task has two
/// sent effects; settling the first must not close it while the second is held.
#[test]
fn a_task_closes_after_its_effects_settle_under_load() {
    let (mut world, replies) = loaded_world();
    let fact = hold_refresh_under_load(&mut world);
    world.event(infra::Event::System(fact));
    let mut pending = VecDeque::from(replies);
    let first = pending.pop_front().expect("first effect result is held");
    world.event(infra::Event::System(first.clone()));
    assert!(!world.seen.contains(&infra::Request::Closed { task: EFFECT_OWNER }));
    for reply in pending {
        world.event(infra::Event::System(reply.clone()));
        world.event(infra::Event::System(reply));
    }
    world.event(infra::Event::System(first));
    assert_results(&world, &[infra::RestartStage::ReadAfresh]);
}
