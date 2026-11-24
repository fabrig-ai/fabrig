//! A landing's retained read admission is a live wake (engine-checks.md, 2.2).
use skein_lib::{Duration, Wall};
use skein_world::inspect::{Moment, Wake, check_waits};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{World, changes, landings};

#[test]
fn a_landing_waits_on_its_read_admission_retry() {
    let mut world = World::with_branches(73, 32);
    world.adopt();
    assert!(world.project_landing(landings::PROJECT, temper_engine_domain_forge_change::Way::Turn, Box::default()));
    for task in 61..69 {
        world.produce(format!("base{task}").as_bytes());
    }
    world.slow_calls(Duration::from_secs(10));
    // Two calls and four pending reads fit; the next gathers wait in the
    // connector's read line, without an admitted call or a live merge write.
    for task in 61..69 {
        let mut change = changes::change_row();
        change.task = task + 100;
        change.change.task = change.task;
        change.pull = Some(task);
        change.base = format!("base{task}").into_bytes().into();
        change.change.way = temper_engine_domain_forge_change::Way::Turn;
        change.change.state = temper_engine_domain_forge_change::State::Ready {
            head: [1; 32],
            ready_since: Wall::from_nanos(world.now().as_nanos()),
        };
        let mut row = landings::landing_row(&change);
        row.landing.members[0].gates = Box::default();
        world.event(forge::Event::Change { row: change });
        row.landing.task = task;
        row.landing.phase = landing::Phase::Turn { at: Wall::from_nanos(world.now().as_nanos()) };
        row.waiting = Some(landing::Awaited::Merge);
        row.landing.members[0].landed = Some([1; 32]);
        world.event(forge::Event::Landing { row });
        world.event(forge::Event::StepLanding {
            task,
            entry: task,
            heard: forge::LandingHeard {
                job: None,
                outcomes: Box::default(),
                dependencies: Box::default(),
                held: Box::default(),
                failure: Box::default(),
                preparation: Some(landing::Preparation::Idle),
                released: false,
                cancelled: false,
                kept: Box::default(),
            },
        });
    }
    let inspection = world.inspection();
    let waiting = inspection
        .waits()
        .iter()
        .filter(|wait| wait.kind == "landing: merge" && wait.wakes.iter().any(|wake| matches!(wake, Wake::Timer(_))))
        .count();
    assert!(waiting > 0, "at least one gather is waiting for read admission: {inspection:?}");
    check_waits(&[&inspection], Moment { seed: 73, at: world.now(), iteration: 0 })
        .expect("every gather has a live wake");
}
