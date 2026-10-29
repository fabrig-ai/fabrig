//! The scripted root's restart order, jig/docs/design/domain/connectors.md, section 14.

use std::collections::{BTreeMap, BTreeSet};

use jig_test_connector::{Effect, Event, Hold, HoldMode, Outcome, Request, ResourceRole, ResourceSpec, RestartStep};
use jig_test_connector_world::{LIMITS, World, config, kind, path};
use jig_test_system::Fault;
use skein_lib::Token;

// Ten live resources exceed the reference connector's nine-request carrier.
const LIVE_RESOURCES: u8 = 10;
const EFFECT_RESOURCE: u8 = 1;
const KEYED_KIND: u16 = 1; // config() assigns keyed creation to local kind one.
const SEED: u8 = 44;
const PROJECT: u32 = 1;
const EFFECT_OWNER: u64 = 4;
const EFFECT_ENTRY: u64 = 1;
const EFFECT_PURPOSE: u64 = 10;
const EFFECT_TARGET: u64 = 7;
const FIRST_CLOSING_OWNER: u64 = 100;

fn loaded_world() -> World {
    let mut bounds = LIMITS;
    bounds.resources = u32::from(LIVE_RESOURCES);
    bounds.facts = u32::from(LIVE_RESOURCES);
    bounds.adoptions = u32::from(LIVE_RESOURCES);
    let mut configuration = config(SEED);
    let mut resources = Vec::new();
    for number in 1..=LIVE_RESOURCES {
        resources.push(ResourceSpec {
            credential: None,
            path: path(SEED, number),
            hold: Hold::Exclusive { mode: HoldMode::Wait },
            writable: true,
        });
    }
    configuration.resources = resources.into_boxed_slice();
    let mut world = World::with_config(SEED, configuration, bounds);
    for number in 1..=LIVE_RESOURCES {
        world.event(Event::Adopt { project: PROJECT, resource: path(SEED, number), role: ResourceRole::Owned });
    }
    let token = Token::new(EFFECT_ENTRY);
    world.event(Event::Describe {
        token,
        effect: Effect {
            kind: kind(SEED, KEYED_KIND),
            resources: Box::from([path(SEED, EFFECT_RESOURCE)]),
            purpose: EFFECT_PURPOSE,
            condition: None,
            target: EFFECT_TARGET,
            state: EFFECT_TARGET,
        },
    });
    world.event(Event::Keep {
        token,
        entry: EFFECT_ENTRY,
        task: EFFECT_OWNER,
        key: world.key(EFFECT_OWNER, EFFECT_PURPOSE),
    });
    for request in world.event(Event::Make { entry: EFFECT_ENTRY }) {
        if let Request::System(call) = request {
            // The fake makes the effect, but the connector loses its answer before restart.
            world.system.answer(call, Fault::None);
        }
    }
    world.seen.clear();
    world
}

fn assert_results(world: &World) {
    let mut refreshes = BTreeMap::new();
    let mut closed = BTreeSet::new();
    let mut outcomes = Vec::new();
    let mut restarted = 0;
    for request in world.trace() {
        if let Request::System(jig_test_connector::SystemRequest::ReadFact { resource, .. }) = request {
            *refreshes.entry(resource.clone()).or_insert(0) += 1;
        }
        if let Request::Closed { task } = request {
            assert!(closed.insert(*task), "one close per task");
        }
        if let Request::Outcome { entry, outcome } = request {
            outcomes.push((*entry, *outcome));
        }
        if *request == Request::RestartDone {
            restarted += 1;
        }
    }
    assert_eq!(refreshes.len(), usize::from(LIVE_RESOURCES));
    for number in 1..=LIVE_RESOURCES {
        assert_eq!(refreshes.get(&path(SEED, number)), Some(&1));
        assert!(closed.remove(&(FIRST_CLOSING_OWNER + u64::from(number))));
    }
    assert_eq!(closed, BTreeSet::from([EFFECT_OWNER]));
    assert_eq!(outcomes, [(EFFECT_ENTRY, Outcome::Made { state: EFFECT_TARGET, found: true })]);
    assert_eq!(restarted, 1);
}

/// Pins `read_afresh_complete` in the scripted root's order and `outbox_settled`
/// in the connector: ten fact replies are outstanding concurrently, past the
/// nine-slot carrier; ten ready closes run in separate batches. The root holds
/// a fact before starting outbox settlement, then holds the effect lookup reply.
#[test]
fn a_restart_reports_only_after_every_refresh_settles() {
    let mut world = loaded_world();
    let reads = world.begin_restart();
    world.event(Event::Close { task: EFFECT_OWNER });
    let mut held = None;
    for request in reads {
        if let Request::System(call) = request {
            let reply = world.system.answer(call, Fault::None);
            if matches!(&reply, jig_test_connector::SystemEvent::Fact { resource, .. } if *resource == path(SEED, EFFECT_RESOURCE))
            {
                held = Some(reply);
            } else {
                world.event(Event::System(reply));
            }
        }
        assert!(!world.trace().contains(&Request::RestartDone));
        assert!(!world.trace().contains(&Request::Closed { task: EFFECT_OWNER }));
    }
    for number in 1..=LIVE_RESOURCES {
        let task = FIRST_CLOSING_OWNER + u64::from(number);
        world.event(Event::Names { task, project: PROJECT, resources: Box::from([path(SEED, number)]) });
        world.event(Event::Close { task });
        world.event(Event::Unnamed { task });
        assert!(!world.trace().contains(&Request::RestartDone));
        assert!(!world.trace().contains(&Request::Closed { task: EFFECT_OWNER }));
    }
    // The scripted root completes every fresh read before asking the outbox to settle.
    let fact = held.expect("the effect resource's fact is held");
    world.event(Event::System(fact.clone()));
    let calls = world.event(Event::Restart(RestartStep::SettleOutbox));
    let mut looked = None;
    for request in calls {
        if let Request::System(call) = request {
            looked = Some(world.system.answer(call, Fault::None));
        }
    }
    world.event(Event::System(fact));
    assert!(!world.trace().contains(&Request::RestartDone));
    assert!(!world.trace().contains(&Request::Closed { task: EFFECT_OWNER }));
    let reply = looked.expect("the restored effect is looked up after the fresh reads");
    world.event(Event::System(reply.clone()));
    world.event(Event::System(reply));
    world.fire();
    assert_results(&world);
}
