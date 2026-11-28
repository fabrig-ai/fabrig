//! The worker's host child domain at random: many rough worlds, each settled,
//! every ending reached among them.

use std::{collections::BTreeSet, convert::Infallible};

use jig_domain_host::Limits;
use jig_host_world::engine::ENDINGS;
use jig_host_world::{Settings, Span, World};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};

// With decoded starts, the 2000-seed inventory stops growing at 1496: 20 endings and 23 paths.
// Keep twice that exact prefix, including the rare agent-start cancellation.
const COVERAGE_PREFIX: u64 = 1496;
const SWEEP_SEEDS: u64 = 2 * COVERAGE_PREFIX;

const ITERATIONS: u32 = 200_000;

fn run(settings: &Settings) -> World {
    let mut world = World::new(*settings);
    world.run(ITERATIONS);
    world
}

/// A rough world, varied by seed: some shut down, some lose contact for
/// longer than the grace, some let runs make several calls at once and hold
/// several inbound events.
fn rough(seed: u64) -> Settings {
    let rough = Settings::rough(seed);
    let shutdown = if seed.is_multiple_of(4) { Some(Span::millis(20_000, 120_000)) } else { None };
    let host = if seed.is_multiple_of(2) { rough.host } else { Limits { run_calls: 3, held: 3, ..rough.host } };
    Settings { shutdown, host, ..rough }
}

#[test]
fn random_worlds_settle_and_reach_every_ending() {
    let check_coverage = gate();
    let mut reached = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut last_growth = 0;
    cases(this_test!("random_worlds_settle_and_reach_every_ending"), 0..SWEEP_SEEDS, |seed| {
        let world = run(&rough(seed));
        if !check_coverage {
            return Ok(());
        }
        let previous = (reached.len(), seen.len());
        let stats = world.stats();
        for ending in stats.endings.keys() {
            reached.insert(*ending);
        }
        let paths = [
            ("busy calls", stats.busy),
            ("calls unavailable", stats.unavailable),
            ("late calls", stats.parent.late_calls),
            ("relays unanswered", stats.engine.unanswered),
            ("events too large", stats.engine.too_large),
            ("events past the hold", stats.engine.full),
            ("events to a run ending", stats.engine.ending),
            ("stale messages", stats.stale),
            ("reports", stats.reports),
            ("cancels of every run", stats.cancel_alls),
            ("runs forgotten on reconnecting", stats.engine.forgotten),
            ("requests to agents gone", stats.parent.dropped),
            ("saves with a workspace changed", stats.parent.saves_stale),
            ("saves with a delivery failed", stats.parent.saves_failed),
            ("prepares aborted", stats.parent.aborts),
        ];
        for (path, count) in paths {
            if count > 0 {
                seen.insert(path);
            }
        }
        for path in stats.paths.keys() {
            seen.insert(path);
        }
        if previous != (reached.len(), seen.len()) {
            last_growth = seed + 1;
        }
        Ok::<(), Infallible>(())
    })
    .expect("world checks assert");
    if !check_coverage {
        return;
    }
    assert!(last_growth <= COVERAGE_PREFIX, "coverage prefix grew to {last_growth}");
    let missing: Vec<&str> = ENDINGS.iter().copied().filter(|ending| !reached.contains(ending)).collect();
    assert!(missing.is_empty(), "every ending is reached: {missing:?} are not");
    let paths = [
        "duplicate assignments",
        "oversized outcomes",
        "endings said during a stop",
        "deliveries settled during a stop",
        "cancels as a workspace was prepared",
        "cancels as a workspace failed to prepare",
        "cancels as an agent failed to start",
        "cancels as an agent started",
    ];
    let missing: Vec<&str> = paths.iter().copied().filter(|path| !seen.contains(path)).collect();
    assert!(missing.is_empty(), "every path is taken: {missing:?} are not");
    assert_eq!(seen.len(), 15 + paths.len(), "every path is taken: only {seen:?} are");
}

#[test]
fn random_link_losses_and_hellos_preserve_which_withdrawals_are_unsent() {
    cases(this_test!("random_link_losses_and_hellos_preserve_which_withdrawals_are_unsent"), 0..128, |seed| {
        let mut rng = skein_lib::Rng::new(seed);
        let mut world = jig_host_world::turn_world::World::new();
        world.assign(false);
        world.hold_relays();
        let mut up = false;
        let mut hello = 0;
        let mut pending = None;
        let mut unsent = 0;
        let mut withdrawn = 0;
        for _ in 0..256 {
            match rng.below(4) {
                0 => {
                    world.lose_contact();
                    up = false;
                }
                1 => {
                    world.reconnect();
                    up = true;
                    hello += 1;
                }
                2 if pending.is_none() => {
                    world.call(false);
                    pending = Some((up, hello));
                }
                _ => {
                    if let Some((sent_up, hello_at_relay)) = pending.take() {
                        world.withdraw_call();
                        let expected = if sent_up || hello != hello_at_relay {
                            withdrawn += 1;
                            jig_domain_host::CallAnswer::Withdrawn
                        } else {
                            unsent += 1;
                            jig_domain_host::CallAnswer::Unsent
                        };
                        assert_eq!(world.reply(), Some(&expected), "seed {seed}");
                    }
                }
            }
        }
        if gate() {
            assert!(unsent > 0 && withdrawn > 0, "both withdrawal paths: seed {seed}");
        }
        if pending.is_some() {
            world.withdraw_call();
        }
        world.shutdown();
        world.reconnect();
        world.acknowledge_answer();
        world.assert_settled();
        Ok::<(), Infallible>(())
    })
    .expect("world checks assert");
}
