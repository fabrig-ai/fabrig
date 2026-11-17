//! The engine's fleet child domain at random: many random worlds, each settled,
//! every ending reached among them.

use std::collections::BTreeMap;

use jig_fleet_world::{ENDINGS, Settings, World};

const ITERATIONS: u32 = 400_000;

fn run(settings: &Settings) -> World {
    let mut world = World::new(*settings);
    world.run(ITERATIONS);
    world
}

#[test]
fn random_worlds_settle_and_reach_every_ending() {
    let mut endings: BTreeMap<&str, u32> = BTreeMap::new();
    for seed in 0..300 {
        let world = run(&Settings::random(seed));
        for (ending, count) in world.stats().endings {
            *endings.entry(ending).or_insert(0) += count;
        }
    }
    for ending in ENDINGS {
        assert!(endings.get(ending).copied().unwrap_or(0) > 0, "{ending} reached in some world: {endings:?}");
    }
}

#[test]
fn random_kept_words_never_lose_or_replace_a_held_claim_early() {
    let mut checked = (0, 0, 0);
    for seed in 0..100 {
        let settings = Settings {
            restart_words: 700,
            restarts: 1,
            read: jig_fleet_world::Span::millis(1, 1000),
            limits: jig_domain_fleet::Limits {
                restart_wait: skein_lib::Duration::from_secs(40),
                ..Settings::random(seed).limits
            },
            ..Settings::random(seed)
        };
        let world = run(&settings);
        let (adopted, lost, placed) = world.restart_checks();
        checked.0 += adopted;
        checked.1 += lost;
        checked.2 += placed;
    }
    assert!(
        checked.0 > 0 && checked.1 > 0 && checked.2 > 0,
        "held adoptions, losses and placements exercised: {checked:?}"
    );
}
