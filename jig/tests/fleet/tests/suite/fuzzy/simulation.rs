//! The engine's fleet child domain at random: many random worlds, each settled,
//! every ending reached among them.

use std::{collections::BTreeMap, convert::Infallible};

use jig_fleet_world::{ENDINGS, Settings, World};
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};

const ITERATIONS: u32 = 400_000;

fn run(settings: &Settings) -> World {
    let mut world = World::new(*settings);
    world.run(ITERATIONS);
    world
}

#[test]
fn random_worlds_settle_and_reach_every_ending() -> Result<(), Case<Infallible>> {
    let at_gate = gate();
    let mut endings: BTreeMap<&str, u32> = BTreeMap::new();
    cases(this_test!("random_worlds_settle_and_reach_every_ending"), 0..300, |seed| {
        let world = run(&Settings::random(seed));
        if at_gate {
            for (ending, count) in world.stats().endings {
                *endings.entry(ending).or_insert(0) += count;
            }
        }
        Ok(())
    })?;
    if at_gate {
        for ending in ENDINGS {
            assert!(endings.get(ending).copied().unwrap_or(0) > 0, "{ending} reached in some world: {endings:?}");
        }
    }
    Ok(())
}

#[test]
fn random_kept_words_never_lose_or_replace_a_held_claim_early() -> Result<(), Case<Infallible>> {
    let at_gate = gate();
    let mut checked = (0, 0, 0);
    cases(this_test!("random_kept_words_never_lose_or_replace_a_held_claim_early"), 0..100, |seed| {
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
        if at_gate {
            let (adopted, lost, placed) = world.restart_checks();
            checked.0 += adopted;
            checked.1 += lost;
            checked.2 += placed;
        }
        Ok(())
    })?;
    if at_gate {
        assert!(
            checked.0 > 0 && checked.1 > 0 && checked.2 > 0,
            "held adoptions, losses and placements exercised: {checked:?}"
        );
    }
    Ok(())
}
