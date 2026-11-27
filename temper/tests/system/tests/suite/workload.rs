//! The composed comparison's control (temper/docs/design/testing/workloads.md, 9).
use temper_system_world::composed::{World, workload::GOAL_BOUND};
use temper_workload::outcome::End;

#[test]
fn a_changed_composed_outcome_names_its_goal() {
    // Run until every drawn goal is named and its outcome can be read. This
    // control checks the comparison; the fuzzy sweep checks the completed run
    // (temper/docs/design/testing/workloads.md, 9).
    let mut world = World::composed_workload(0);
    for _ in 0..GOAL_BOUND.steps {
        if world.workload().goals.iter().all(|goal| world.workload_names().task(&goal.name).is_some()) {
            break;
        }
        world.tick();
    }
    let before = world.workload_outcome();
    let mut restarted = before.clone();
    restarted.goals.get_mut("g0").expect("drawn goal").end = End::Failed;
    let finding = before.compare(&restarted, &[]).expect_err("a changed outcome is found");
    assert!(finding.contains("compare: g0: end Waiting without restarts, Failed with them"), "{finding}");
}

#[test]
fn a_refused_close_falls_and_every_goal_lands() -> Result<(), String> {
    use skein_world::sweep::Sweep;
    use temper_faults::{bounds::FORGE_READ_ADMISSIONS, forge::CLOSE_REFUSED};
    use temper_system_world::composed::{CAMPAIGN_KINDS, FaultDraw, GATE_KINDS};

    const SEED: u64 = 11_712;
    // Keep the mixed draw, except for the two-slot table that can refuse
    // the capability refresh in this combination (engine-checks.md, 5.2).
    let kinds: Vec<_> =
        GATE_KINDS.iter().chain(CAMPAIGN_KINDS).copied().filter(|&kind| kind != FORGE_READ_ADMISSIONS).collect();
    let draw = FaultDraw::new(SEED, &kinds);
    let (_, target) = draw.planned().into_iter().find(|&(kind, _)| kind == CLOSE_REFUSED).expect("planned close");
    let mut sweep = Sweep::new("a_refused_close_falls_and_every_goal_lands");
    sweep.configure(CLOSE_REFUSED);
    temper_engine_forge_world::open::exclude(&mut sweep, &[CLOSE_REFUSED])?;
    let mut world = World::faulted_workload(SEED, &kinds);
    world.exclude_workload_close(&sweep);
    world.run_workload()?;
    world.assert_workload()?;
    world.judge_workload_faults()?;

    let ledger = world.workload_ledger();
    sweep.ran(ledger);
    assert_eq!(sweep.ineligible(CLOSE_REFUSED), 0, "seed {SEED}: close refusal for {target} was ineligible");
    let strikes: Vec<_> = ledger.entries().iter().filter(|entry| entry.kind == CLOSE_REFUSED).collect();
    let [strike] = strikes.as_slice() else {
        return Err(format!("seed {SEED}: expected one close refusal, got {strikes:?}"));
    };
    assert_eq!(strike.struck.request.as_deref(), Some(target));
    assert!(strike.until.is_some_and(|until| until > strike.from));
    Ok(())
}
