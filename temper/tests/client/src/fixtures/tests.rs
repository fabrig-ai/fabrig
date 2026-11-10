//! Joined-world replay (temper/docs/design/testing/checks.md, 3).
use super::{PARTY, chats_page, faults, world};
use jig_client_kit::harness::{Outcome, Until};
use skein_world::domain::Verdict;

fn recovery(seed: u64) -> Outcome {
    let mut world = world(seed);
    world.randomness(12, 0, 0);
    let person = world.client(PARTY, chats_page());
    world.random_person(person);
    for (at, fault) in faults(seed) {
        world.fault_at(at, fault);
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
    assert!(outcome.judged.0 > 0, "seed {seed}: the referee judged the run");
    outcome
}

#[test]
fn seed_five_replays_with_cold_then_warm_phrasebooks() {
    let first = recovery(5);
    let replay = recovery(5);
    assert!(!first.trace.is_empty(), "replay compares the observed run");
    assert_eq!(replay.trace, first.trace);
    assert_eq!(replay.judged, first.judged);
    assert_eq!(replay.now, first.now);
    assert_eq!(replay.iterations, first.iterations);
    assert_eq!(replay.callbacks, first.callbacks);
    assert_eq!(replay.faults, first.faults);
    assert_eq!(replay.dropped, first.dropped);
}
