//! Root inspection and the bounded-wait referee (engine-checks.md, 6).
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Time};
use skein_world::inspect::{Inspection, Wake};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{checks::Checks, forge_routes::World};

#[test]
fn a_parked_producing_change_without_its_delegate_is_found() {
    let mut driver = super::panel_before_change::before_change();
    driver.send(engine::Event::Forge(Box::new(forge::Event::Change {
        row: super::panel_before_change::producing_row(),
    })));
    driver.settle();
    let original = driver.root.forge().change(51).expect("producing change").clone();
    let mut row = original.clone();
    row.change.state = temper_engine_domain_forge_change::State::Producing { requested: false };
    row.delegate = None;
    engine::step(
        &mut driver.root,
        &driver.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    engine::fire(&mut driver.root, &driver.env);
    let finding = Checks::new(71).quiet(&driver.root, driver.env.now, 1).expect_err("parked change has no wake");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task 51: waits on change: producing with no live wake: []",
            driver.env.now.as_nanos(),
        )
    );
    // The referee has read the bad snapshot; restore the live wake before the
    // direct driver's automatic restart oracle drains the fixture on drop.
    engine::step(
        &mut driver.root,
        &driver.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(original)) })),
    );
    driver.settle();
}

#[test]
fn a_parked_producing_change_with_its_live_delegate_passes() {
    let mut driver = super::panel_before_change::before_change();
    driver.send(engine::Event::Forge(Box::new(forge::Event::Change {
        row: super::panel_before_change::producing_row(),
    })));
    driver.settle();
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::inspect(&driver.root, &mut inspection);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == "task 51").expect("change wait");
    assert_eq!(wait.kind, "change: producing");
    assert_eq!(wait.wakes, [Wake::Waiter("task 52".into())]);
    assert_eq!(Checks::new(71).quiet(&driver.root, driver.env.now, 1), Ok(()));
}

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
fn a_ready_change_waits_on_the_landing_filling_its_room() {
    let (world, change, landing) = super::forge_routes::landing::full_landing_room();
    let row = world.root.forge().change(change).expect("ready change");
    assert!(matches!(row.change.state, temper_engine_domain_forge_change::State::Ready { .. }));
    assert_eq!(row.change.landing, None);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, vec![Wake::Waiter(format!("task {landing}"))]);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

fn ready_without_landing() -> (World, u64) {
    let (mut world, change, landing) = super::forge_routes::landing::full_landing_room();
    // Freeze forming, then remove the only live landing without waking its follower.
    world.root.hold_landings(true);
    let mut row = world.root.forge().landing(landing).expect("landing").clone();
    row.landing.phase = temper_engine_domain_forge_landing::Phase::Cancelled;
    row.waiting = None;
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Landing(row) })),
    );
    engine::fire(&mut world.root, &world.env);
    // Remove the fixture's forming guard without driving the synthetic lost wake.
    world.root.hold_landings(false);
    (world, change)
}

#[test]
fn a_ready_change_without_a_live_landing_or_other_wake_is_found() {
    let (world, change) = ready_without_landing();
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, Vec::<Wake>::new());
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("ready change has lost its wake");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {change}: waits on change: ready with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
}

#[test]
fn a_ready_change_waits_for_its_trains_due_member() {
    let (world, ready, due) = super::forge_routes::landing::wait_drill::waiting_on_due_member();
    assert!(world.root.tasks().procedure_due(due).is_some());
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {ready}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, vec![Wake::Waiter(format!("task {due}"))]);
    let member = inspection.waits().iter().find(|wait| wait.waiter == format!("task {due}")).expect("member wait");
    assert_eq!(member.kind, "change: gating");
    assert!(member.wakes.iter().any(|wake| matches!(wake, Wake::Party(party) if party.starts_with("forge: call "))));
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_ready_change_has_no_wake_after_the_due_member_leaves_its_train() {
    let (mut world, ready, due) = super::forge_routes::landing::wait_drill::waiting_on_due_member();
    // Freeze forming after the member leaves, to expose the unwakeable snapshot.
    world.root.hold_landings(true);
    let mut member = world.root.forge().change(due).expect("due member").clone();
    member.base = b"another-train".as_slice().into();
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(member)) })),
    );
    engine::fire(&mut world.root, &world.env);
    world.root.hold_landings(false);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {ready}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, Vec::<Wake>::new());
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("no member holds this train");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {ready}: waits on change: ready with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
}

#[test]
fn a_ready_change_waits_for_the_operators_restart() {
    let (mut world, change) = ready_without_landing();
    world.root.hold_landings(true);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, vec![Wake::Party("operator restart".into())]);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_ready_change_waits_for_its_train_with_landing_room_free() {
    let (world, change, landing) = super::forge_routes::landing::ready_behind_landing(2);
    assert!(world.root.forge().inspection_landings().len() < world.root.forge().inspection_landings().capacity());
    let row = world.root.forge().change(change).expect("ready change");
    assert!(matches!(row.change.state, temper_engine_domain_forge_change::State::Ready { .. }));
    assert_eq!(row.change.landing, None);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, vec![Wake::Waiter(format!("task {landing}"))]);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_ready_change_has_no_wake_from_another_train_with_landing_room_free() {
    let (mut world, change, landing) = super::forge_routes::landing::ready_behind_landing(2);
    world.root.hold_landings(true);
    let mut row = world.root.forge().landing(landing).expect("landing").clone();
    row.landing.branch = b"another-train".as_slice().into();
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Landing(row) })),
    );
    engine::fire(&mut world.root, &world.env);
    world.root.hold_landings(false);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, Vec::<Wake>::new());
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("other train cannot wake this one");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {change}: waits on change: ready with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
}

fn refused_train() -> (World, u64) {
    use super::forge_routes::landing::{LIVE_ROOM, fill_live_room, produced_with_hold};
    use temper_engine_domain_world::forge_routes::StoryLimits;
    let (mut world, change) = produced_with_hold(
        &StoryLimits {
            turn: true,
            tasks: Some(LIVE_ROOM),
            person_executor: true,
            party_requests: Some(LIVE_ROOM + 2),
            ..StoryLimits::default()
        },
        true,
    );
    world.oracle_settle();
    fill_live_room(&mut world);
    world.root.hold_landings(false);
    world.send(engine::Event::Forge(Box::new(forge::Event::Priority { task: change, priority: 0 })));
    world.oracle_settle();
    (world, change)
}

#[test]
fn a_ready_change_waits_for_its_refused_trains_retry() {
    let (world, change) = refused_train();
    let row = world.root.forge().change(change).expect("ready change");
    assert!(matches!(row.change.state, temper_engine_domain_forge_change::State::Ready { .. }));
    assert_eq!(row.change.landing, None);
    assert!(world.root.forge().inspection_landings().is_empty());
    let due = world.root.inspection_forming_retry(change).expect("refused train has a retry");
    assert!(due > world.env.wall);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, vec![Wake::Timer(Time::from_nanos(due.as_nanos()))]);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_ready_change_has_no_retry_from_another_refused_train() {
    let (mut world, change) = refused_train();
    world.root.hold_landings(true);
    let mut row = world.root.forge().change(change).expect("ready change").clone();
    row.base = b"another-train".as_slice().into();
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    world.oracle_settle();
    world.root.hold_landings(false);
    assert!(matches!(
        world.root.forge().change(change).expect("ready change").change.state,
        temper_engine_domain_forge_change::State::Ready { .. }
    ));
    assert_eq!(world.root.inspection_forming_retry(change), None);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, Vec::<Wake>::new());
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("no retry for this train");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {change}: waits on change: ready with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
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

fn passed_making() -> (World, u64) {
    let (world, task, _) = super::forge_routes::goal_branch::passed_making();
    assert!(world.root.forge().inspection_entries().is_empty());
    assert!(!world.root.forge().room_asked(task));
    (world, task)
}

fn inspect(world: &World) -> Inspection {
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::inspect(&world.root, &mut inspection);
    inspection
}

#[test]
fn a_placed_making_waits_for_its_person_with_room_free() {
    let (world, task) = passed_making();
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("making wait");
    assert_eq!(wait.kind, "making: room");
    assert_eq!(wait.wakes, [Wake::Party("person".into())]);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
    let mut forge = Inspection::new("forge", temper_engine_inspect::KINDS);
    temper_engine_inspect::forge(world.root.forge(), &mut forge);
    let wait = forge.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("making wait");
    assert!(wait.wakes.is_empty(), "the forge alone cannot name the person's hold");
}

fn resource_held_making() -> (World, u64) {
    let (mut world, task) = passed_making();
    // Restore the same goal with a hold no person releases. Stop before another
    // driver tick once its making is passed over, to inspect the broken wait.
    let mut row = world.root.tasks().task(task).expect("goal").clone();
    row.phase = tasks::Phase::Held {
        was: tasks::Was::Active(tasks::Active::Preparing),
        why: tasks::Hold::ResourceUnavailable,
        source: None,
    };
    row.placed = None;
    world.store.seed(
        Box::new([engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(row))))]),
        Box::new([]),
    );
    world.restart_same();
    for _ in 0..1000 {
        world.tick();
        if world.root.forge().making_waits(task)
            && !world.root.forge().room_asked(task)
            && world.root.forge().inspection_entries().is_empty()
        {
            break;
        }
    }
    assert!(world.root.forge().making_waits(task));
    assert!(!world.root.forge().room_asked(task));
    assert!(world.root.forge().inspection_entries().is_empty());
    assert!(matches!(
        world.root.tasks().task(task).expect("held goal").phase,
        tasks::Phase::Held { why: tasks::Hold::ResourceUnavailable, .. }
    ));
    (world, task)
}

#[test]
fn a_resource_held_making_with_room_free_has_no_live_wake() {
    let (world, task) = resource_held_making();
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("resource hold has no release wake");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {task}: waits on making: room with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
}

#[test]
fn a_held_room_waiter_is_passed_over_not_refused() {
    let (world, task) = resource_held_making();
    let inspection = inspect(&world);
    let table = inspection.tables().iter().find(|table| table.name == "forge write entries").expect("write table");
    assert_eq!(table.capacity, 1);
    assert_eq!(table.entries, []);
    assert!(table.refused.is_empty(), "held task {task} is passed over");
    assert_eq!(
        skein_world::inspect::check_tables(
            &[&inspection],
            skein_world::inspect::Moment { seed: 71, at: world.env.now, iteration: 1 }
        ),
        Ok(())
    );
}

#[test]
fn an_unheld_room_waiter_is_refused_when_unasked_with_room_free() {
    let (mut world, _) = passed_making();
    let task = 1;
    assert!(!matches!(world.root.tasks().task(task).expect("live first goal").phase, tasks::Phase::Held { .. }));
    // A frozen change in the line, with no ask despite the room available.
    let mut row = temper_engine_forge_world::changes::change_row();
    row.task = task;
    row.change.task = task;
    row.room = Some(1);
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    engine::step(&mut world.root, &world.env, engine::Event::Forge(Box::new(forge::Event::RoomUnavailable { task })));
    assert!(!world.root.forge().room_asked(task));
    assert!(world.root.forge().inspection_reservations().is_empty());
    let inspection = inspect(&world);
    let finding = skein_world::inspect::check_tables(
        &[&inspection],
        skein_world::inspect::Moment { seed: 71, at: world.env.now, iteration: 1 },
    )
    .expect_err("unasked waiter was refused while room is free");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine forge write entries task {task}: refused admission while the table has room",
            world.env.now.as_nanos(),
        )
    );
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

fn completed_follow_gate() -> (World, u64, Time, Time) {
    let mut world = super::forge_routes::follow_changes::world(forge::settings::FollowStart::Code);
    let after = super::forge_routes::follow_changes::push(&mut world, b"Port for the gate bound");
    let change = super::forge_routes::follow_changes::change(&world, after);
    let producer = super::forge_routes::follow_changes::next_delegate(&mut world, change, 0);
    super::forge_routes::follow_changes::produce(&mut world, change, &producer);
    for _ in 0..1000 {
        world.tick();
        let row = world.root.forge().change(change).expect("follow change");
        if let Some(ended) = world.root.forge().inspection_follow_check_ended(change)
            && world.events.is_empty()
            && world.store.idle()
            && world.root.quiescent()
            && world.root.next_deadline(&world.env).is_none_or(|due| due > world.env.now)
        {
            let stall = world.root.forge().inspection_limits().change_policy.stall;
            let old_bound = Time::from_nanos(row.change.since.as_nanos()).saturating_add(stall);
            let bound = Time::from_nanos(ended.as_nanos()).saturating_add(stall);
            assert!(bound > old_bound, "the workflow's answer comes after its last policy step");
            assert!(row.delegate.is_none(), "the workflow has no agent delegate");
            let due = world.root.forge().inspection_change_due(change).expect("next procedure poll");
            assert_eq!(
                due.as_nanos(),
                ended.as_nanos().saturating_add(world.root.forge().inspection_limits().queue_window.as_nanos())
            );
            assert!(due.as_nanos() <= bound.as_nanos(), "the next poll is no later than the stall bound");
            return (world, change, old_bound, bound);
        }
    }
    panic!("follow gate has a completed workflow at a quiet point");
}

#[test]
fn a_completed_follow_gate_counts_its_bound_from_the_workflows_end() {
    let (world, change, old_bound, bound) = completed_follow_gate();
    for root in [true, false] {
        let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
        if root {
            temper_engine_inspect::inspect(&world.root, &mut inspection);
        } else {
            temper_engine_inspect::forge(world.root.forge(), &mut inspection);
        }
        let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("gate wait");
        assert_eq!(wait.kind, "change: gating");
        assert_eq!(wait.bound, Some(bound));
    }
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    assert_eq!(checks.quiet(&world.root, old_bound, 2), Ok(()));
    assert_eq!(checks.step(old_bound.saturating_add(Duration::from_nanos(1)), 3), Ok(()));
    assert_eq!(checks.quiet(&world.root, old_bound.saturating_add(Duration::from_nanos(1)), 4), Ok(()));
    assert_eq!(checks.quiet(&world.root, bound, 5), Ok(()));
}

#[test]
fn a_completed_follow_gate_still_waiting_past_its_bound_is_found() {
    let (world, change, _, bound) = completed_follow_gate();
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    // The same hand-off, but frozen: its next procedure step never arrives.
    assert_eq!(checks.quiet(&world.root, bound, 2), Ok(()));
    let at = bound.saturating_add(Duration::from_nanos(1));
    let finding = checks.step(at, 3).expect_err("completed gate still waits for its next step");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 3: engine task {change}: waits on change: gating past bound {} ns with no quiet point since",
            at.as_nanos(),
            bound.as_nanos(),
        )
    );
}

fn gated_dependency() -> (World, u64, Time) {
    let (world, upstream, change, _) = super::forge_routes::landing::dependency_across_trains();
    let row = world.root.forge().change(change).expect("dependent change");
    assert!(matches!(row.change.state, temper_engine_domain_forge_change::State::Gating { .. }));
    assert_eq!(world.root.tasks().task(change).expect("dependent task").waiting_on.as_ref(), [upstream]);
    assert!(world.root.forge().inspection_gates_settled(change));
    assert!(world.root.forge().inspection_change_due(change).is_some(), "the readiness poll remains armed");
    let old_bound = Time::from_nanos(row.change.since.as_nanos())
        .saturating_add(world.root.forge().inspection_limits().change_policy.stall);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("gate wait");
    assert_eq!(wait.bound, None, "a settled gate waits only for its dependency");
    assert!(wait.wakes.contains(&Wake::Waiter(format!("task {upstream}"))));
    (world, change, old_bound)
}

#[test]
fn a_settled_gate_waiting_for_a_dependency_has_no_stall_bound() {
    let (world, _, old_bound) = gated_dependency();
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    assert_eq!(checks.step(old_bound.saturating_add(Duration::from_nanos(1)), 2), Ok(()));
    assert_eq!(checks.quiet(&world.root, old_bound.saturating_add(Duration::from_nanos(1)), 3), Ok(()));
}

#[test]
fn an_unfinished_gate_past_its_bound_is_found_despite_a_dependency() {
    let (mut world, change, bound) = gated_dependency();
    let mut row = world.root.forge().change(change).expect("dependent change").clone();
    row.change.gates = Box::new([temper_engine_domain_forge_change::Gate {
        number: 9,
        kind: temper_engine_domain_forge_change::GateKind::Person,
        blocking: true,
        freshness: temper_engine_domain_forge_change::Freshness::Exact,
        eager: false,
    }]);
    // Freeze a still-unanswered gate, with the dependency unchanged.
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    assert!(!world.root.forge().inspection_gates_settled(change));
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("gate wait");
    assert_eq!(wait.bound, Some(bound));
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    assert_eq!(checks.quiet(&world.root, bound, 2), Ok(()));
    let at = bound.saturating_add(Duration::from_nanos(1));
    let finding = checks.step(at, 3).expect_err("dependencies do not excuse an unfinished gate");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 3: engine task {change}: waits on change: gating past bound {} ns with no quiet point since",
            at.as_nanos(),
            bound.as_nanos(),
        )
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
