use std::{
    collections::{BTreeMap, VecDeque},
    convert::Infallible,
};

use jig_test_connector::{Effect, Event, Outcome, Record, RecordKey, Request, ResourceRole, SystemEvent};
use jig_test_connector_world::{World, kind, path};
use jig_test_system::{Fault, Observed};
use skein_lib::Token;
use skein_world::{
    sweep::{Case, cases},
    this_test,
};

fn run(seed: u64) -> (Vec<Request>, Vec<Observed>, BTreeMap<RecordKey, Record>, u64) {
    // Keep the fixture names and sizes on one byte; draw the schedule from the full seed.
    let fixture = seed.to_le_bytes()[0];
    let mut world = World::new(fixture);
    world.system.preload_history(u32::from(fixture) * 10);
    let local = u16::try_from(seed % 4 + 1).expect("local effect kind is in 1..=4");
    let target = u64::from(fixture) + 5;
    let condition = if local == 2 { Some(1) } else { None };
    if condition.is_some() {
        world.system.other_hand(&path(fixture, 1), 1);
    }
    let token = Token::new(1);
    let effect = Effect {
        kind: kind(fixture, local),
        resources: Box::from([path(fixture, 1)]),
        purpose: 10,
        condition,
        target,
        state: target,
    };
    assert!(matches!(world.event(Event::Describe { token, effect }).as_slice(), [Request::Described { .. }]));
    let kept = world.event(Event::Keep { token, entry: 1, task: 4, key: world.key(4, 10) });
    if seed.is_multiple_of(5) {
        world.cold_restart();
        world.settle_restart(&mut VecDeque::new());
    } else {
        let fault = match seed % 4 {
            0 => Fault::None,
            1 => Fault::BeforeSend,
            2 => Fault::AfterApply,
            3 => Fault::Late,
            _ => unreachable!(),
        };
        world.release(&kept, &mut VecDeque::from([fault]));
        if seed.is_multiple_of(3) {
            world.cold_restart();
            world.settle_restart(&mut VecDeque::new());
        }
    }
    world.wall = 7;
    for _ in 0..3 {
        let due = world.fire();
        world.release(&due, &mut VecDeque::new());
    }
    if let Some(late) = world.system.deliver_late() {
        world.event(Event::System(late));
    }
    world.event(Event::Adopt { project: 1, resource: path(fixture, 2), role: ResourceRole::Owned });
    world.event(Event::Names { task: 2, project: 1, resources: Box::from([path(fixture, 2)]) });
    world.event(Event::Subscribe { task: 2, topic: kind(fixture, 1), wake_at: 5, keep_at: 2 });
    world.event(Event::System(SystemEvent::Pool {
        path: path(fixture, 2),
        slots: u32::try_from(seed % 3).expect("pool slots are in 0..=2"),
        lost: Box::from([2]),
    }));
    world.event(Event::System(SystemEvent::News {
        topic: kind(fixture, 1),
        importance: u16::try_from(seed % 7).expect("news importance is in 0..=6"),
        origin: jig_test_connector::Origin::Other,
    }));
    assert!(
        world
            .trace()
            .iter()
            .filter(|request| matches!(request, Request::Outcome { outcome: Outcome::Made { .. }, .. }))
            .count()
            <= 1
    );
    assert!(world.system.calls() < 20);
    (world.seen, world.system.observed().to_vec(), world.records, world.system.calls())
}

#[test]
fn random_faults_restarts_and_news_replay_with_the_same_committed_records() -> Result<(), Case<Infallible>> {
    cases(this_test!("random_faults_restarts_and_news_replay_with_the_same_committed_records"), 1..=64, |seed| {
        assert_eq!(run(seed), run(seed));
        Ok(())
    })
}
