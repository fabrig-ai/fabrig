//! Complete composed draws and restart comparisons (workloads.md, 5 and 7).
use skein_world::{
    sweep::{Sweep, cases, gate},
    this_test,
};
use temper_faults::restarts;
use temper_system_world::composed::{CAMPAIGN_KINDS, GATE_KINDS, World};

#[test]
fn a_restart_changes_no_composed_outcome() -> Result<(), String> {
    const CAMPAIGN_CASES: u64 = 8;
    // A settled host-call stop and a model-query crash at the gate.
    let seeds = if gate() { vec![0, 29] } else { (0..CAMPAIGN_CASES).collect() };
    let mut sweep = Sweep::new("a_restart_changes_no_composed_outcome");
    for kind in restarts::MID_TURN {
        sweep.configure(kind);
    }
    cases(this_test!("a_restart_changes_no_composed_outcome"), seeds, |seed| {
        eprintln!("seed {seed}");
        let mut world = World::composed_workload(seed);
        eprintln!("workload {:?}", world.workload());
        world.run_workload()?;
        world.assert_workload()?;
        let before = world.workload_outcome();
        let points = world.workload_restart_points();
        eprintln!("seed {seed}: restart points (queries, host calls) {points:?}");
        drop(world);
        let mut restarted = World::restarted_workload(seed, points);
        restarted.run_workload()?;
        restarted.assert_workload()?;
        let after = restarted.workload_outcome();
        let left = restarted.workload_restart_left();
        eprintln!("seed {seed}: left {left:?}; fell {:?}", restarted.workload_restart_ledger().fell());
        before.compare(&after, &left)?;
        sweep.ran(restarted.workload_restart_ledger());
        Ok::<(), String>(())
    })
    .map_err(|case| case.to_string())?;
    sweep.check_fell()
}

#[test]
fn every_family_falls_in_the_composed_workload() -> Result<(), String> {
    const CAMPAIGN_CASES: u64 = 16;
    // Seed 6 draws a hold with its release and a red gate to read.
    let campaign_kinds;
    let (seeds, kinds) = if gate() {
        (6..7, GATE_KINDS)
    } else {
        campaign_kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
        (0..CAMPAIGN_CASES, campaign_kinds.as_slice())
    };
    let mut sweep = Sweep::new("every_family_falls_in_the_composed_workload");
    for &kind in kinds {
        sweep.configure(kind);
    }
    temper_engine_forge_world::open::exclude(&mut sweep, kinds)?;
    cases(this_test!("every_family_falls_in_the_composed_workload"), seeds, |seed| {
        eprintln!("mixed seed {seed}");
        let mut world = World::faulted_workload(seed, kinds);
        world.exclude_workload_close(&sweep);
        if !gate() {
            world.exclude_workload_other_merge(&sweep);
        }
        world.run_workload()?;
        world.assert_workload()?;
        world.judge_workload_faults()?;
        sweep.ran(world.workload_ledger());
        Ok::<(), String>(())
    })
    .map_err(|case| case.to_string())?;
    sweep.check_fell()
}
