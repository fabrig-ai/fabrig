//! Complete composed draws and restart comparisons (workloads.md, 5 and 7).
use skein_world::{
    faults::Kind,
    sweep::{Sweep, cases},
    this_test,
};
use temper_faults::{bounds, forge, parks, people, restarts, stale};
use temper_system_world::composed::World;

#[test]
fn a_restart_changes_no_composed_outcome() -> Result<(), String> {
    let mut sweep = Sweep::new("a_restart_changes_no_composed_outcome");
    for kind in restarts::MID_TURN {
        sweep.configure(kind);
    }
    // Two seeds, as in the clean draw: a settled host-call stop and a model-query crash.
    cases(this_test!("a_restart_changes_no_composed_outcome"), [0, 29], |seed| {
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
    // The workload's repairs carry gate evidence, not a PullFiles read: the
    // bounds family's next reachable kind is the failed job's bounded log.
    // Its dependent goal starts after the predecessor landed, so it has no
    // early checkout with a squashed parent: move the base during landing.
    // COMPOSED has too few concurrent writes to fill the stock outbox.
    // Its read-admission table is bounded to two in this faulted world.
    // The worker's fake checkout supplies a byte capacity for DISK_FULL.
    const KINDS: [Kind; 13] = [
        forge::TIMED_OUT,
        forge::HEAD_BEHIND,
        forge::JOB_LOG,
        forge::CLOSED_BY_ANOTHER,
        temper_blame::LOST,
        stale::MOVED_BASE,
        skein_llm_kit::faults::RATE_WINDOW,
        restarts::ENGINE_CRASH_MID_TURN,
        temper_worker_inspect::faults::DISK_FULL,
        people::HOLDS_A_MEMBER,
        people::RELEASES_A_MEMBER,
        parks::DELIVERED_THEN_PARKED,
        bounds::FORGE_READ_ADMISSIONS,
    ];
    let mut sweep = Sweep::new("every_family_falls_in_the_composed_workload");
    for kind in KINDS {
        sweep.configure(kind);
    }
    temper_engine_forge_world::open::exclude(&mut sweep, &KINDS)?;
    // One seed is the minimum: 6 draws a hold with its release and a red
    // gate to read; the previous seed 1 draws a hold without a release.
    cases(this_test!("every_family_falls_in_the_composed_workload"), 6..7, |seed| {
        eprintln!("mixed seed {seed}");
        let mut world = World::faulted_workload(seed, &KINDS);
        world.run_workload()?;
        world.assert_workload()?;
        world.judge_workload_faults()?;
        sweep.ran(world.workload_ledger());
        Ok::<(), String>(())
    })
    .map_err(|case| case.to_string())?;
    sweep.check_fell()
}
