//! Each seed draws every io scenario (`skein/docs/design/testing/sweeps.md`, 4.2–4.5).
//! Calm cases inject no faults. Across the gate's chaotic cases, every socket
//! fault falls and every answer comes from a cancel of each operation io cancels
//! (`skein/docs/design/runtime/kernel.md`, 5).

use std::collections::BTreeSet;
use std::convert::Infallible;

use skein_io_world::census::{Answer, Target, cancels, faults};
use skein_io_world::scenarios::SCENARIOS;
use skein_sim::{Config, Fault};
use skein_world::sweep::{cases, gate};
use skein_world::this_test;

/// Seeds per scenario, calm and chaotic.
const SEEDS: u64 = 150;

#[test]
fn scenario_draws_accept_full_width_seeds() {
    for (name, scenario) in SCENARIOS {
        for config in [Config::calm(), Config::chaos()] {
            let world = scenario(u64::MAX, config);
            assert_eq!(world.sim.seed(), u64::MAX, "{name}: the simulator keeps the full seed");
        }
    }
}

#[test]
fn every_scenario_settles_calm_over_many_seeds() {
    let test = this_test!("every_scenario_settles_calm_over_many_seeds");
    cases::<_, Infallible, _>(test, 0..SEEDS, |seed| {
        for (name, scenario) in SCENARIOS {
            let outcome = scenario(seed, Config::calm()).run();
            assert!(faults(&outcome.trace).is_empty(), "{name}, seed {seed}: a calm world injects no fault");
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn every_scenario_settles_under_chaos_and_every_fault_falls() {
    let mut fell = BTreeSet::new();
    let mut answered = BTreeSet::new();
    let at_gate = gate();
    let test = this_test!("every_scenario_settles_under_chaos_and_every_fault_falls");
    cases::<_, Infallible, _>(test, 0..SEEDS, |seed| {
        for (_, scenario) in SCENARIOS {
            let outcome = scenario(seed, Config::chaos()).run();
            let cancellations = cancels(&outcome.trace);
            if at_gate {
                fell.extend(faults(&outcome.trace));
                answered.extend(cancellations);
            }
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if !at_gate {
        return;
    }
    for fault in [
        Fault::Latency,
        Fault::ShortRecv,
        Fault::ShortSend,
        Fault::Reset,
        Fault::Refuse,
        Fault::NoBuffer,
        Fault::TimedOut,
        Fault::CancelRace,
        Fault::CancelUnsubmitted,
        Fault::LateReset,
    ] {
        assert!(fell.contains(&fault), "{fault:?} fell in some seed: {fell:?}");
    }
    for target in [Target::Accept, Target::Connect, Target::Recv, Target::Send] {
        for answer in [Answer::Stopped, Answer::TooLate, Answer::Unsubmitted] {
            assert!(answered.contains(&(target, answer)), "a cancel of a {target:?} was {answer:?}: {answered:?}");
        }
    }
}
