//! Complete composed draws and restart comparisons (workloads.md, 5 and 7).
use skein_world::{
    faults::Kind,
    sweep::{Sweep, each_seed},
};
use temper_faults::{forge, restarts, stale};
use temper_system_world::composed::World;

#[test]
fn a_restart_changes_no_composed_outcome() {
    let mut sweep = Sweep::new("a_restart_changes_no_composed_outcome");
    for kind in restarts::MID_TURN {
        sweep.configure(kind);
    }
    // Two seeds, as in the clean draw: a settled host-call stop and a model-query crash.
    for seed in [0, 29] {
        eprintln!("seed {seed}");
        let mut world = World::composed_workload(seed);
        eprintln!("workload {:?}", world.workload());
        world.run_workload();
        world.assert_workload();
        let before = world.workload_outcome();
        let points = world.workload_restart_points();
        eprintln!("seed {seed}: restart points (queries, host calls) {points:?}");
        drop(world);
        let mut restarted = World::restarted_workload(seed, points);
        restarted.run_workload();
        restarted.assert_workload();
        let after = restarted.workload_outcome();
        let left = restarted.workload_restart_left();
        eprintln!("seed {seed}: left {left:?}; fell {:?}", restarted.workload_restart_ledger().fell());
        before.compare(&after, &left).unwrap_or_else(|finding| panic!("seed {seed}: {finding}"));
        sweep.ran(restarted.workload_restart_ledger());
    }
    sweep.check_fell().expect("both MID_TURN kinds fell");
}

#[test]
fn every_family_falls_in_the_composed_workload() {
    // The workload's repairs carry gate evidence, not a PullFiles read: the
    // bounds family's next reachable kind is the failed job's bounded log.
    // Its dependent goal starts after the predecessor landed, so it has no
    // early checkout with a squashed parent: move the base during landing.
    const KINDS: [Kind; 8] = [
        forge::TIMED_OUT,
        forge::HEAD_BEHIND,
        forge::JOB_LOG,
        forge::CLOSED_BY_ANOTHER,
        temper_blame::LOST,
        stale::MOVED_BASE,
        skein_llm_kit::faults::RATE_WINDOW,
        restarts::ENGINE_CRASH_MID_TURN,
    ];
    let mut sweep = Sweep::new("every_family_falls_in_the_composed_workload");
    for kind in KINDS {
        sweep.configure(kind);
    }
    temper_engine_forge_world::open::exclude(&mut sweep, &KINDS).expect("engine exclusions");
    each_seed(1..2, |seed| {
        eprintln!("mixed seed {seed}");
        let mut world = World::faulted_workload(seed, &KINDS);
        world.run_workload();
        world.assert_workload();
        world.judge_workload_faults();
        sweep.ran(world.workload_ledger());
    });
    sweep.check_fell().expect("all eight families fell");
}
