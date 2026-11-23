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

fn held_producing(why: tasks::Hold) -> (World, u64, u64) {
    let (mut world, _, producer, _) = temper_engine_domain_world::forge_routes::change_world(true, 1000, None);
    let tasks::Party::Task(change) = world.root.tasks().task(producer.task).expect("producer").requester else {
        panic!("change requester")
    };
    world.hold_reads_at_any_step = true;
    world.held_reads = Some(Vec::new());
    let branch = world.root.forge().change(change).expect("change row").branch.clone();
    let base = world.root.forge().change(change).expect("change row").base.clone();
    let tip = world.fake.branch(b"org/repo", &base).expect("base");
    world.external(
        2,
        skein_fake_forge_domain::api::Op::Git(skein_fake_forge_domain::api::Git::Create {
            branch: branch.clone(),
            commit: tip,
        }),
    );
    let head =
        skein_fake_forge_domain::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
            .expect("push");
    let event = world.protocol.answer(
        skein_lib::Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge::tag(1, 2),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts {
            pushed: Box::new([forge::Pushed {
                tag: producer.workspace.repositories[0].tag,
                commit: temper_engine_forge_world::translate::commit(head),
                checked: true,
            }]),
            ..engine::Parts::none()
        },
    );
    world.send(event);
    for _ in 0..1000 {
        world.tick();
        if world.root.tasks().task(producer.task).is_none() && world.root.tasks().procedure_due(change).is_some() {
            break;
        }
    }
    let producer = producer.task;
    let (connector, code, step) = world.root.tasks().procedure_due(change).expect("change gathering reads");
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Party(engine::PartyInput::ProcedureStep {
            task: change,
            step,
            connector,
            code,
            decision: engine::ProcedureStep::new(engine::StepEnd::Hold(why)),
        }),
    );
    assert!(matches!(world.root.tasks().task(change).expect("held task").phase,
        tasks::Phase::Held { why: reason, .. } if reason == why));
    // Drain the old gather while the procedure itself is held, then freeze an
    // unheld procedure beside the independently held jig task.
    let mut row = world.root.forge().change(change).expect("change row").clone();
    row.change.state = temper_engine_domain_forge_change::State::Held {
        was: Box::new(row.change.state),
        why: temper_engine_domain_forge_change::Hold::Stalled,
    };
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    world.oracle_settle();
    assert!(matches!(world.root.tasks().task(change).expect("held task").phase,
        tasks::Phase::Held { why: reason, .. } if reason == why));
    let mut row = world.root.forge().change(change).expect("change row").clone();
    row.change.state = temper_engine_domain_forge_change::State::Producing { requested: true };
    row.delegate = Some((producer, temper_engine_domain_forge_change::Delegate::Produce));
    row.delegate_status = temper_engine_domain_forge_change::Status::Unknown;
    row.pending = None;
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    engine::fire(&mut world.root, &world.env);
    (world, change, producer)
}

#[test]
fn a_resource_held_change_with_an_ended_producer_has_no_live_wake() {
    let (world, change, producer) = held_producing(tasks::Hold::ResourceUnavailable);
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("resource hold has no release wake");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {change}: waits on change: producing with no live wake: [ended task {producer}]",
            world.env.now.as_nanos(),
        )
    );
}

#[test]
fn a_person_stopped_change_with_an_ended_producer_has_a_release_wake() {
    let (world, change, producer) = held_producing(tasks::Hold::Stopped);
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::inspect(&world.root, &mut inspection);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("change wait");
    assert_eq!(wait.kind, "change: producing");
    assert_eq!(wait.wakes, vec![Wake::Ended(format!("task {producer}")), Wake::Party("person".into())]);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

fn follow_change(world: &mut World, words: &[u8]) -> u64 {
    let after = super::forge_routes::follow_changes::push(world, words);
    world.oracle_settle();
    assert!(world.events.is_empty() && world.store.idle() && world.root.quiescent());
    super::forge_routes::follow_changes::change(world, after)
}

fn change_wakes(world: &World, task: u64, root: bool) -> Vec<Wake> {
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    if root {
        temper_engine_inspect::inspect(&world.root, &mut inspection);
    } else {
        temper_engine_inspect::forge(world.root.forge(), &mut inspection);
    }
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("change wait");
    assert_eq!(wait.kind, "change: producing");
    wait.wakes.clone()
}

#[test]
fn a_follow_waiting_for_its_person_and_a_live_change_has_wakes() {
    let mut world = super::forge_routes::follow_changes::world(forge::settings::FollowStart::Owner);
    let first = follow_change(&mut world, b"First port");
    let second = follow_change(&mut world, b"Second port");
    assert_eq!(world.root.tasks().task(second).expect("second change").waiting_on.as_ref(), &[first]);
    assert_eq!(
        world.root.forge().inspection_follow_wait(first).expect("first entry").1,
        &forge::FollowState::WaitsWord
    );
    assert_eq!(
        world.root.forge().inspection_follow_wait(second).expect("second entry").1,
        &forge::FollowState::WaitsFor(first)
    );
    for root in [true, false] {
        assert_eq!(change_wakes(&world, first, root), vec![Wake::Party("person".into())]);
        assert_eq!(change_wakes(&world, second, root), vec![Wake::Waiter(format!("task {first}"))]);
    }
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_follow_waiting_for_its_standing_owner_has_a_wake() {
    let (mut world, owner) =
        super::forge_routes::follow_changes::role::owned(forge::settings::FollowStart::Owner, 40, 1000);
    let change = follow_change(&mut world, b"Port for the role");
    for root in [true, false] {
        assert_eq!(change_wakes(&world, change, root), vec![Wake::Waiter(format!("task {owner}"))]);
    }
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_follow_waiting_for_a_missing_owner_has_no_live_wake() {
    let mut world = super::forge_routes::follow_changes::world(forge::settings::FollowStart::Owner);
    let change = follow_change(&mut world, b"Port without its owner");
    // An inconsistent stored setting: the follow's owner is gone but its
    // waiting change remains. Restore only that row, without driving a turn
    // that could cancel the orphan, as for the missing prepare job above.
    let mut setting = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            engine::Record::Forge { row } => match row.as_ref() {
                forge::Stored::Setting(row)
                    if row.key.kind == 12 && row.key.slot == forge::settings::SettingSlot::Change =>
                {
                    Some(row.clone())
                }
                _ => None,
            },
            engine::Record::Core(_) => None,
        })
        .expect("follow setting");
    let forge::settings::Edit::Set(value) = &mut setting.edit else { panic!("set follow") };
    let forge::settings::Value::Follow(follow) = &mut value.value else { panic!("follow setting") };
    let missing = u64::MAX;
    assert!(world.root.tasks().task(missing).is_none());
    follow.owner = forge::settings::FollowOwner::Task(missing);
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Setting(setting) })),
    );
    assert_eq!(change_wakes(&world, change, true), vec![Wake::Ended(format!("task {missing}"))]);
    let finding =
        Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("missing owner cannot wake the change");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {change}: waits on change: producing with no live wake: [ended task {missing}]",
            world.env.now.as_nanos(),
        )
    );
}

fn held_follow_tail() -> (World, u64, u64, u64) {
    let mut world = super::forge_routes::follow_changes::world(forge::settings::FollowStart::Owner);
    let ended = follow_change(&mut world, b"Ended port");
    let held = follow_change(&mut world, b"Held port");
    let tail = follow_change(&mut world, b"Held tail");
    super::forge_routes::follow_changes::cancel(&mut world, ended, 167);
    world.oracle_settle();
    assert!(world.events.is_empty() && world.store.idle() && world.root.quiescent());
    let task = world.root.tasks().task(held).expect("held change");
    assert!(matches!(task.phase, tasks::Phase::Held { why: tasks::Hold::Dependency(id), .. } if id == ended));
    assert!(task.waiting_on.is_empty());
    assert_eq!(world.root.tasks().task(tail).expect("tail").waiting_on.as_ref(), &[held]);
    (world, ended, held, tail)
}

#[test]
fn a_follow_held_behind_a_queued_ended_entry_has_its_owners_restart_wake() {
    let (world, _, held, tail) = held_follow_tail();
    assert_eq!(change_wakes(&world, held, true), vec![Wake::Party("person".into())]);
    assert_eq!(change_wakes(&world, tail, true), vec![Wake::Waiter(format!("task {held}"))]);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_follow_held_after_its_ended_entry_is_gone_has_no_live_wake() {
    let (mut world, ended, held, tail) = held_follow_tail();
    let mut row = world.root.forge().follow_row(2, 1).expect("follow row").clone();
    row.queue = row.queue.into_vec().into_iter().filter(|entry| entry.change != Some(ended)).collect();
    // An incomplete stored teardown: the ended entry is gone, but its tail's
    // dependency holds remain. No owner's act can restart the missing entry.
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Follow(row) })),
    );
    assert_eq!(change_wakes(&world, held, true), Vec::<Wake>::new());
    assert_eq!(change_wakes(&world, tail, true), vec![Wake::Waiter(format!("task {held}"))]);
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("no queued entry to restart");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {held}: waits on change: producing with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
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
