//! Root inspection and the bounded-wait referee (engine-checks.md, 6).
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Time};
use skein_world::inspect::{Inspection, Wake};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{checks::Checks, forge_routes::World};

fn preparing() -> (World, u64, u64) {
    let (mut world, change) = super::forge_routes::landing::produced_turn_change(None);
    world.manual_prepare_jobs();
    for _ in 0..600 {
        world.tick();
        if let Some(job) = world.jobs.first() {
            let landing = job.workspace.expect("prepare workspace").workstream;
            if world.events.is_empty() && world.store.idle() && world.root.quiescent() {
                return (world, change, landing);
            }
        }
    }
    panic!("prepare has a live job at a quiet point");
}

#[test]
fn a_live_prepare_job_passes_at_the_root() {
    let (world, _, landing) = preparing();
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::inspect(&world.root, &mut inspection);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {landing}")).expect("landing wait");
    assert_eq!(wait.kind, "landing: prepare");
    let child = world
        .root
        .tasks()
        .task(landing)
        .expect("landing task")
        .delegates
        .iter()
        .find(|child| {
            world
                .root
                .tasks()
                .task(**child)
                .is_some_and(|row| row.executor == tasks::Executor::Job { kind: forge::PREPARE_KIND })
        })
        .expect("prepare child");
    assert!(wait.wakes.contains(&Wake::Waiter(format!("task {child}"))));
    assert_eq!(Checks::new(17).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_restored_prepare_wait_without_its_job_is_a_root_finding() {
    let (mut world, change, landing) = preparing();
    let mut row = world.root.forge().landing(landing).expect("preparing row").clone();
    // A synthetic stored row: the task has no prepare child and no landing poll.
    // This is not a sequence a healthy engine produces. The forge alone cannot
    // know the task's children, and so accepts its external prepare-job wake.
    row.landing.task = change;
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Landing(row) })),
    );
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::forge(world.root.forge(), &mut inspection);
    assert_eq!(
        skein_world::inspect::check_waits(
            &[&inspection],
            skein_world::inspect::Moment { seed: 19, at: world.env.now, iteration: 2 }
        ),
        Ok(()),
        "the forge alone cannot see the missing job"
    );
    let mut checks = Checks::new(19);
    let finding = checks.quiet(&world.root, world.env.now, 2).expect_err("root knows the job is absent");
    assert_eq!(
        finding,
        format!(
            "seed 19: at {} ns, iteration 2: engine task {change}: waits on landing: prepare with no live wake: [ended prepare job]",
            world.env.now.as_nanos(),
        )
    );
    assert_eq!(checks.step(world.env.now, 3), Err(finding.clone()), "first finding is retained");
    assert_eq!(checks.quiet(&world.root, world.env.now, 4), Err(finding));
}

#[test]
fn a_goals_change_lands_with_checks_at_every_quiet_point() {
    use temper_engine_domain_world::kinds;
    let mut world = kinds::forge_world();
    let goal = kinds::goal(&mut world);
    let (change, producer) = kinds::goal_change(&mut world, &goal);
    kinds::produce(&mut world, change, &producer);
    let branch = kinds::change_row(&world, change).branch;
    let head = world.fake.branch(b"org/repo", &branch).expect("pushed head");
    super::forge_routes::goal_landing::status(&mut world, head, skein_fake_forge_domain::api::Check::Passed);
    let mut checks = Checks::new(31);
    for iteration in 0..1000 {
        world.tick();
        if world.events.is_empty() && world.store.idle() && world.root.quiescent() {
            assert_eq!(checks.quiet(&world.root, world.env.now, iteration), Ok(()));
            if world.root.tasks().task(change).is_none() {
                assert!(matches!(
                    kinds::change_row(&world, change).change.state,
                    temper_engine_domain_forge_change::State::Landed { .. }
                ));
                return;
            }
        }
    }
    panic!("goal's change lands");
}

#[test]
fn ticks_and_idle_skips_visit_a_stall_bound_between_grid_points() {
    for skip in [false, true] {
        let (mut world, change, bound) = checking();
        // The story isolates shell scheduling from network time: the reads
        // needed to decide the hold can all finish at the stall instant.
        world.fake_env.limits.latency_min = Duration::ZERO;
        world.fake_env.limits.latency_max = Duration::ZERO;
        let tick = Duration::from_millis(700);
        world.tick_interval = tick;
        assert_ne!((bound.as_nanos() - world.env.now.as_nanos()) % tick.as_nanos(), 0);
        let count = u32::try_from((bound.as_nanos() - world.env.now.as_nanos()) / tick.as_nanos() + 1)
            .expect("small test horizon");
        if skip {
            world.advance_ticks(count);
        } else {
            for _ in 0..count {
                world.tick();
            }
        }
        let mut checks = Checks::new(71);
        assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
        let task = world.root.tasks().task(change).expect("held change");
        assert!(
            matches!(task.phase, tasks::Phase::Held { why: tasks::Hold::Stalled, .. }),
            "skip={skip}, at={:?}, bound={bound:?}, task phase {:?}, change {:?}",
            world.env.now,
            task.phase,
            temper_engine_domain_world::kinds::change_row(&world, change).change
        );
        assert_eq!(
            task.phase_since.as_nanos(),
            bound.as_nanos(),
            "the hold commits at the bound, not the next grid tick"
        );
        assert_eq!(checks.step(bound.saturating_add(Duration::from_nanos(1)), 2), Ok(()));
    }
}

#[test]
fn a_wait_still_seen_at_its_bound_is_found_on_the_next_move() {
    let (world, change, bound) = checking();
    let mut checks = Checks::new(37);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    // A frozen root whose timer was not delivered: a quiet reading at the
    // bound still sees that wait. It cannot be forgotten merely for being due.
    assert_eq!(checks.quiet(&world.root, bound, 2), Ok(()));
    let finding = checks.step(bound.saturating_add(Duration::from_nanos(1)), 3).expect_err("wait still stands");
    assert!(
        finding
            .contains(&format!("engine task {change}: waits on change: checking past bound {} ns", bound.as_nanos())),
        "{finding}"
    );
}

fn checking() -> (World, u64, Time) {
    let (mut world, _, producer, _) = temper_engine_domain_world::forge_routes::change_world(true, 1000, None);
    let task = world.root.tasks().task(producer.task).expect("producer");
    let tasks::Party::Task(change) = task.requester else { panic!("change requester") };
    temper_engine_domain_world::kinds::produce(&mut world, change, &producer);
    for _ in 0..1000 {
        world.tick();
        if world.events.is_empty() && world.store.idle() && world.root.quiescent() {
            let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
            temper_engine_inspect::inspect(&world.root, &mut inspection);
            if let Some(bound) = inspection
                .waits()
                .iter()
                .find(|wait| wait.waiter == format!("task {change}") && wait.kind == "change: checking")
                .and_then(|wait| wait.bound)
            {
                return (world, change, bound);
            }
        }
    }
    panic!("change waits on CI with its stall bound");
}

#[test]
fn a_bounded_wait_cannot_be_starved_between_quiet_points() {
    let (world, change, bound) = checking();
    let mut checks = Checks::new(23);
    assert_eq!(checks.next_bound(), None);
    assert_eq!(checks.quiet(&world.root, world.env.now, 10), Ok(()));
    assert_eq!(checks.next_bound(), Some(bound));
    assert_eq!(checks.step(bound, 11), Ok(()), "the bound itself is not past");
    let at = bound.saturating_add(Duration::from_nanos(1));
    let finding = checks.step(at, 12).expect_err("no quiet point since arming");
    assert_eq!(
        finding,
        format!(
            "seed 23: at {} ns, iteration 12: engine task {change}: waits on change: checking past bound {} ns with no quiet point since",
            at.as_nanos(),
            bound.as_nanos(),
        )
    );
}

#[test]
fn a_quiet_point_clears_a_wait_that_has_moved_on() {
    let (mut world, change, bound) = checking();
    let mut checks = Checks::new(29);
    assert_eq!(checks.quiet(&world.root, world.env.now, 10), Ok(()));
    let branch = temper_engine_domain_world::kinds::change_row(&world, change).branch;
    let head = world.fake.branch(b"org/repo", &branch).expect("pushed head");
    super::forge_routes::goal_landing::status(&mut world, head, skein_fake_forge_domain::api::Check::Passed);
    for _ in 0..1000 {
        world.tick();
        if world.root.tasks().task(change).is_none()
            && world.events.is_empty()
            && world.store.idle()
            && world.root.quiescent()
        {
            break;
        }
    }
    assert!(world.root.tasks().task(change).is_none(), "change has landed");
    assert_eq!(checks.quiet(&world.root, world.env.now, 11), Ok(()));
    assert_eq!(checks.next_bound(), None);
    assert_eq!(checks.step(bound.saturating_add(Duration::from_nanos(1)), 12), Ok(()));
}
