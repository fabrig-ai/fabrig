//! One complete composed draw; restart comparison grows here (workloads.md, 5 and 7).
use skein_world::{
    faults::Kind,
    sweep::{Sweep, each_seed},
};
use temper_faults::{forge, restarts, stale};
use temper_system_world::composed::World;

#[test]
fn the_composed_workload_ends_within_its_bounds() {
    for seed in [17, 29] {
        eprintln!("seed {seed}");
        let mut world = World::composed_workload(seed);
        eprintln!("workload {:?}", world.workload());
        world.run_workload();
        world.assert_workload();
    }
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
