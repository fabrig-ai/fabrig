//! Restart predicates from jig/docs/design/domain/connectors.md, section 14.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use jig_ops_domain_observability as obs;
use jig_ops_fake_production as production;
use jig_ops_world::{World, limits};
use skein_lib::Token;

use super::obs_key;

// One past observability's 80-request carrier; subscriptions can hold this live set.
const LIVE_SERVICES: u32 = 81;
const HELD_SERVICE: u32 = 0;
const HEALTHY_REPLICAS: u32 = 3; // The fake services start with three healthy replicas.
const FIRST_SUBSCRIBER: u64 = 100;
const EFFECT_OWNER: u64 = 7; // The owner in obs_key().
const EFFECT_ENTRY: u64 = 3; // The outbox number used by the connector stories.
const PROJECT: u32 = 1;

fn service(number: u32) -> obs::Service {
    obs::Service::new(Box::from(*b"production"), format!("service-{number}").into_bytes().into_boxed_slice())
}

fn loaded_world() -> World {
    let mut bounds = limits();
    bounds.services = LIVE_SERVICES;
    bounds.subscriptions = LIVE_SERVICES;
    let mut world = World::with_limits(0, bounds);
    for number in 0..LIVE_SERVICES {
        let name = format!("service-{number}");
        world.production.add_service(production::ServiceName::new("production", &name), "v1", HEALTHY_REPLICAS);
        world.event(obs::Event::Subscribe {
            task: FIRST_SUBSCRIBER + u64::from(number),
            topic: obs::Topic::Health(service(number)),
            wake_at: 7,
            keep_at: 1,
        });
    }
    let token = Token::new(EFFECT_ENTRY);
    world
        .event(obs::Event::Describe { token, effect: obs::Effect { rule: Box::from(*b"checkout-errors"), until: 90 } });
    world.event(obs::Event::Keep { project: Some(PROJECT), entry: EFFECT_ENTRY, token, key: obs_key() });
    let sent = world.event(obs::Event::Make { key: obs_key() });
    for request in sent {
        if let obs::Request::System(call) = request {
            // The system makes the effect; its reply is lost before the cold restart.
            world.system(call);
        }
    }
    world.restore();
    world.seen.clear();
    world
}

fn enqueue(requests: Vec<obs::Request>, pending: &mut VecDeque<obs::SystemRequest>) {
    for request in requests {
        if let obs::Request::System(call) = request {
            pending.push_back(call);
        }
    }
}

fn restarts(world: &World) -> Vec<obs::RestartStage> {
    let mut stages = Vec::new();
    for request in &world.seen {
        if let obs::Request::Restarted { stage } = request {
            stages.push(*stage);
        }
    }
    stages
}

fn assert_results(world: &World) {
    let mut refreshes = BTreeMap::new();
    let mut closed = BTreeSet::new();
    let mut outcomes = Vec::new();
    for request in &world.seen {
        if let obs::Request::System(obs::SystemRequest::Facts { service }) = request {
            *refreshes.entry(service.clone()).or_insert(0) += 1;
        }
        if let obs::Request::Closed { task } = request {
            assert!(closed.insert(*task), "one close per task");
        }
        if let obs::Request::Outcome { entry, key, outcome } = request {
            outcomes.push((*entry, *key, *outcome));
        }
    }
    assert_eq!(refreshes.len(), usize::try_from(LIVE_SERVICES).expect("fixture count fits"));
    for number in 0..LIVE_SERVICES {
        assert_eq!(refreshes.get(&service(number)), Some(&1));
        assert!(closed.remove(&(FIRST_SUBSCRIBER + u64::from(number))));
    }
    assert_eq!(closed, BTreeSet::from([EFFECT_OWNER]));
    assert_eq!(outcomes, [(EFFECT_ENTRY, obs_key(), obs::Outcome::Made)]);
    assert_eq!(restarts(world), [obs::RestartStage::ReadAfresh, obs::RestartStage::Outbox]);
}

/// Pins `read_afresh_complete` and `outbox_settled`: 81 live refreshes exceed
/// the 80-slot carrier concurrently; 81 ready closes run across separate steps.
/// One fact and the effect owner's lookup reply stay held until their phase ends.
#[test]
fn a_restart_reports_only_after_every_refresh_settles() {
    let mut world = loaded_world();
    let mut pending = VecDeque::new();
    enqueue(world.event(obs::Event::ReadAfresh), &mut pending);
    enqueue(world.event(obs::Event::Close { task: EFFECT_OWNER }), &mut pending);
    for number in 0..LIVE_SERVICES {
        let task = FIRST_SUBSCRIBER + u64::from(number);
        enqueue(world.event(obs::Event::Close { task }), &mut pending);
    }
    let mut held = None;
    while let Some(call) = pending.pop_front() {
        let reply = world.system(call);
        if matches!(&reply, obs::SystemEvent::Fact { service: name, .. } if *name == service(HELD_SERVICE)) {
            held = Some(reply);
        } else {
            enqueue(world.event(obs::Event::System(reply)), &mut pending);
        }
        assert!(restarts(&world).is_empty(), "a held fact prevents read-afresh completion");
        assert!(!world.seen.contains(&obs::Request::Closed { task: EFFECT_OWNER }));
    }
    let fact = held.expect("the first live service's fact is held");
    world.event(obs::Event::System(fact.clone()));
    assert_eq!(restarts(&world), [obs::RestartStage::ReadAfresh]);

    let calls = world.event(obs::Event::Restart);
    let mut looked = None;
    for request in calls {
        if let obs::Request::System(call) = request {
            looked = Some(world.system(call));
        }
    }
    // A duplicate fact advances a real step while the outbox lookup remains held.
    world.event(obs::Event::System(fact));
    assert_eq!(restarts(&world), [obs::RestartStage::ReadAfresh]);
    assert!(!world.seen.contains(&obs::Request::Closed { task: EFFECT_OWNER }));
    let answer = looked.expect("the sent silence is looked up on restart");
    world.event(obs::Event::System(answer.clone()));
    world.event(obs::Event::System(answer));
    assert_results(&world);
}
