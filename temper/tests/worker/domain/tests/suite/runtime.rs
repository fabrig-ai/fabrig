//! Focused runtime stories and full trace replay.
use temper_worker_domain_world::runtime::{Close, Settings, World};

/// The turns seed 7's script leaves unacknowledged as it shuts down.
const ABANDONED_TURNS: u64 = 2;

// Measured peak: 34,043 bytes; 25% headroom, rounded up to two significant digits.
const RUNTIME_HEAP_CEILING: u64 = 43_000;

fn run(settings: Settings) -> World {
    let mut world = World::new(settings, RUNTIME_HEAP_CEILING);
    world.run();
    world
}

#[test]
fn fenced_turns_survive_busy_reconnect_and_crossed_acknowledgements() {
    let world = run(Settings { seed: 5, turns: 3, byte_slots: 2, merging: false, close: Close::Committed });
    let stats = world.stats();
    assert!(stats.copies > stats.turns);
    assert_eq!(stats.busy, stats.turns);
    assert!(stats.reconnects > stats.turns);
    assert!(stats.peak_retained <= 6);
    assert_eq!(stats.saves, 1);
}

#[test]
fn merge_paths_and_expected_heads_cross_the_worker_and_park_has_no_saved_merge() {
    let world = run(Settings { seed: 19, turns: 2, byte_slots: 1, merging: true, close: Close::Committed });
    assert_eq!(world.stats().commits, 2, "unresolved paths then an explicit resolution");
    assert_eq!(world.stats().saves, 0);
}

#[test]
fn a_worker_shutting_down_past_the_grace_abandons_the_turns_it_keeps() {
    let world = run(Settings { seed: 7, turns: 3, byte_slots: 3, merging: false, close: Close::Abandoned });
    assert_eq!(world.stats().abandoned, ABANDONED_TURNS);
}

#[test]
fn a_worker_shutting_down_past_the_grace_abandons_an_unacknowledged_answer_with_its_turns() {
    let world = run(Settings { seed: 7, turns: 3, byte_slots: 3, merging: false, close: Close::Unacknowledged });
    assert_eq!(world.stats().abandoned, ABANDONED_TURNS + 1);
}

#[test]
fn a_channel_reopened_before_the_run_ends_gets_every_turn_and_nothing_is_abandoned() {
    let world = run(Settings { seed: 7, turns: 3, byte_slots: 3, merging: false, close: Close::Reopened });
    assert_eq!(world.stats().abandoned, 0);
}

#[test]
fn full_boundary_and_fact_traces_replay_exactly() {
    let settings = Settings { seed: 27, turns: 2, byte_slots: 2, merging: true, close: Close::Committed };
    let first = run(settings);
    let second = run(settings);
    assert_eq!(first.stats(), second.stats());
    assert_eq!(first.trace(), second.trace());
}

#[test]
fn facts_can_be_dropped_without_changing_the_runtime_boundaries() {
    let settings = Settings { seed: 13, turns: 3, byte_slots: 2, merging: false, close: Close::Committed };
    let first = run(settings);
    let mut second = World::without_facts(settings, RUNTIME_HEAP_CEILING);
    second.run();
    assert_eq!(first.boundaries(), second.boundaries());
}
