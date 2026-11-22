//! Open lagging-view scenarios (draft/failure-worlds.md, 7).
use skein_fake_forge_domain::{self as forge, Lag};
use skein_lib::Duration;
use skein_world::streams::Streams;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, fake_config,
    landings::{self, LANDING, Trains},
};

#[test]
#[ignore = "landing train: one owner for a landing's members, and a landing that can always move"]
fn a_late_manual_merge_record_finishes_the_landed_batch() {
    let mut world = landings::turn_in(World::batch_turn_in(&Streams::new(1), None, fake_config()));
    world.batch(2);
    for task in 51..=52 {
        let branch = format!("temper/c{task}");
        world.produce_file(branch.as_bytes(), format!("file{task}").as_bytes(), b"member");
        landings::ready_change(&mut world, task, branch.as_bytes());
    }
    Trains::default().judge(&mut world);
    let mut armed = false;
    let mut finished = false;
    for entry in 100..140 {
        if !armed && world.landing(LANDING).landing.members.iter().any(|m| m.landed.is_some() && !m.recorded) {
            // Only the record is late; the top's fast-forward is already visible.
            forge::lag(
                world.forge_mut(),
                Lag { chance: 1000, min: Duration::from_secs(3), max: Duration::from_secs(3) },
            );
            armed = true;
        }
        let decision = landings::step_landing_raw(&mut world, LANDING, entry);
        if matches!(decision, landing::Decision::Prepare { .. }) {
            world.prepare_landing(LANDING, true);
        }
        if decision == landing::Decision::Finish {
            finished = true;
            break;
        }
    }
    assert!(armed, "the landed squash needed its manual merge record");
    assert!(world.ledger().entries().iter().any(|e| e.kind.family == "lagging" && e.kind.name == "merge_late"));
    assert!(
        finished,
        "late record must reconcile without a person's release: {:?}",
        world.landing(LANDING).landing.phase
    );
}
