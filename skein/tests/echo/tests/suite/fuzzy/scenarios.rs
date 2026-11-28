//! Each seed case runs all the echo's scenarios (skein/docs/design/testing/sweeps.md, 4).
//! Every world runs its referee and contracts, including memory checks at each
//! iteration. The chaotic sweep checks fault and outcome coverage at the gate.

use std::collections::BTreeSet;

use skein_echo_world::census::{FAULTS, OUTCOMES, faults, outcomes};
use skein_echo_world::proc::Proc;
use skein_echo_world::scenarios::SCENARIOS;
use skein_sim::Config;
use skein_world::Outcome;
use skein_world::sweep::{cases, gate};
use skein_world::this_test;

/// Gate seeds per sweep, calm and chaotic.
const SEEDS: u64 = 300;

fn run_case(seed: u64, config: Config, mut observe: impl FnMut(&str, &Outcome<Proc>)) -> Result<(), String> {
    for (name, scenario) in SCENARIOS {
        let outcome = scenario(seed, config).try_run().map_err(|finding| format!("{name}: {finding}"))?;
        observe(name, &outcome);
    }
    Ok(())
}

#[test]
fn every_scenario_holds_calm_over_many_seeds() {
    let test = this_test!("every_scenario_holds_calm_over_many_seeds");
    cases(test, 0..SEEDS, |seed| {
        run_case(seed, Config::calm(), |name, outcome| {
            assert!(faults(&outcome.trace).is_empty(), "{name}, seed {seed}: a calm world injects no fault");
        })
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

/// Under chaos a scenario expects only that every connection finishes; so
/// that the sweep still shows the echo doing what it should, every outcome
/// a client can see must appear over it: served, refused at either
/// entrance, broken, too long, idled out.
#[test]
fn every_scenario_settles_under_chaos_and_every_fault_and_outcome_appears() {
    let at_gate = gate();
    let mut fell = BTreeSet::new();
    let mut came = BTreeSet::new();
    let test = this_test!("every_scenario_settles_under_chaos_and_every_fault_and_outcome_appears");
    cases(test, 0..SEEDS, |seed| {
        run_case(seed, Config::chaos(), |_, outcome| {
            if at_gate {
                fell.extend(faults(&outcome.trace));
                came.extend(outcomes(&outcome.procs));
            }
        })
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        for fault in FAULTS {
            assert!(fell.contains(&fault), "{fault:?} fell in some seed: {fell:?}");
        }
        for outcome in OUTCOMES {
            assert!(came.contains(&outcome), "{outcome:?} came of some connection: {came:?}");
        }
    }
}

#[test]
fn every_scenario_accepts_full_width_seeds() {
    for config in [Config::calm(), Config::chaos()] {
        run_case(u64::MAX, config, |name, outcome| {
            assert_eq!(outcome.seed, u64::MAX, "{name}: the world takes the full case seed");
        })
        .unwrap_or_else(|finding| panic!("{finding}"));
    }
}
