//! Root inspection and the bounded-wait referee (engine-checks.md, 6).
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Time};
use skein_world::inspect::{Inspection, Wake};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{checks::Checks, direct::Driver, forge_routes::World, store::Store};

#[test]
fn a_composed_quiet_point_checks_its_other_roots() {
    let driver = Driver::new(Store::new());
    let mut other = Inspection::new("worker", &["queued"]);
    other.wait(skein_world::inspect::Wait {
        waiter: "task 2".into(),
        kind: "queued".into(),
        wakes: Vec::new(),
        bound: None,
    });
    let finding =
        Checks::new(91).quiet_with(&driver.root, driver.env.now, 1, &[&other]).expect_err("the worker has no wake");
    assert!(finding.contains("worker task 2: waits on queued with no live wake"), "{finding}");
}

#[test]
fn a_live_landing_without_a_jig_task_is_a_quiet_point_finding() {
    use temper_engine_forge_world::{changes, landings};

    let mut world = World::new();
    world.adopt();
    let mut change = changes::change_row();
    change.pull = Some(1);
    let row = landings::landing_row(&change);
    let task = row.landing.task;
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Landing(row) })),
    );
    assert!(world.root.tasks().task(task).is_none());
    assert_eq!(
        Checks::new(91).quiet_with(&world.root, world.env.now, 1, &[]),
        Err(format!(
            "seed 91: at {} ns, iteration 1: engine task {task}: a landing that can't move: its task is not live",
            world.env.now.as_nanos()
        ))
    );
}

#[test]
fn a_refused_procedure_wait_is_a_quiet_point_finding() {
    use jig_domain_core::ReportRefusal;
    use skein_lib::{Env, Wall};
    use temper_engine_domain_world::walking;

    let env = Env { limits: walking::limits(), now: Time::ZERO, wall: Wall::EPOCH };
    let mut root = engine::Domain::new(walking::config(91), &env.limits).expect("fixture engine");
    let task = 72;
    engine::step(
        &mut root,
        &env,
        engine::Event::Forge(Box::new(forge::Event::WaitsRefused { task, why: ReportRefusal::TooManyThings })),
    );
    let finding = Checks::new(91).quiet(&root, env.now, 1).expect_err("refused wait report is a fault");
    assert_eq!(finding, "seed 91: at 0 ns, iteration 1: engine task 72: wait report refused: TooManyThings");
}

#[test]
fn an_unopened_world_without_a_refusal_is_found_at_close() {
    use temper_engine_domain_world::ends::{Ends, OPENING};

    let mut driver = Driver::new(Store::new());
    driver.close();
    let closed = std::mem::replace(&mut driver.ends, Ends::new(91, OPENING));
    // The checked fixture skips a second readback, but a new root still owes an opening.
    driver.replace_root(temper_engine_domain_world::walking::config(91));
    let finding = driver.close_erasing(None).expect_err("an unexcused opening is still checked");
    // The finding is sticky; restore the closed referee after reading this deliberately frozen start.
    driver.ends = closed;
    assert!(finding.contains("engine opening: still unopened at close"), "{finding}");
}

#[test]
fn a_refused_store_rehearsal_reports_its_finding_without_an_opening() {
    let mut driver = Driver::new(Store::new());
    driver.close();
    let mut image = driver.store.fork();
    let (cold, _) = temper_engine_forge_world::changes::change_row().split();
    image.seed(
        Box::new([engine::Record::Forge { row: Box::new(forge::Stored::ChangeCold(Box::new(cold))) }]),
        Box::new([]),
    );
    let finding = temper_engine_domain_world::check::store(
        &image,
        &temper_engine_domain_world::walking::config(91),
        &driver.env.limits,
        driver.env.wall,
        image.applied,
    )
    .expect_err("the rehearsal refuses the half change");
    assert!(finding.contains("start refused at Link: ForgeRow(ChangeCold(51))"), "{finding}");
}

#[test]
fn a_broken_world_owes_no_opening_at_close() {
    let mut driver = Driver::new(Store::new());
    driver.broken("the story deliberately stops before startup");
    assert_eq!(driver.close_erasing(None), Ok(()));
}

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

#[test]
fn a_preparing_change_without_a_landing_waits_on_its_write() {
    // The new gather meets Busy at 69.7 s in this fixture and retries one second later.
    const READ_ADMISSION_RETRY: Time = Time::from_nanos(70_700_000_000);
    let (world, change) = super::forge_routes::reviews::preparing_review_repair(None);
    let row = world.root.forge().change(change).expect("preparing change");
    assert_eq!(world.root.forge().landing_of(row.task), None);
    assert_eq!(row.delegate, None);
    let entry = row.pending.expect("preparing write");
    assert_eq!(world.root.forge().inspection_entry(entry).expect("live write").task, change);
    for root in [true, false] {
        let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
        if root {
            temper_engine_inspect::inspect(&world.root, &mut inspection);
        } else {
            temper_engine_inspect::forge(world.root.forge(), &mut inspection);
        }
        let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("change wait");
        assert_eq!(wait.kind, "change: preparing");
        assert_eq!(wait.wakes, [Wake::Party(format!("forge: entry {entry}")), Wake::Timer(READ_ADMISSION_RETRY)]);
        assert_eq!(wait.bound, None);
    }
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_preparing_change_without_a_landing_or_live_write_is_found() {
    let change = 51;
    let (mut world, source) = super::forge_routes::reviews::preparing_review_repair(Some(change));
    let mut row = world.root.forge().change(source).expect("preparing change").clone();
    // Seed an unstepped preparing row on a live procedure, with a stale write
    // reference. Unlike restoration, registration arms no recovery timer.
    assert!(world.root.forge().change(change).is_none());
    row.task = change;
    row.change.task = change;
    row.pending = Some(u64::MAX);
    engine::step(&mut world.root, &world.env, engine::Event::Forge(Box::new(forge::Event::Change { row })));
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("change wait");
    assert_eq!(wait.kind, "change: preparing");
    assert_eq!(wait.wakes, Vec::<Wake>::new());
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("preparing write is absent");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {change}: waits on change: preparing with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
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
    use temper_engine_domain_world::forge_routes::Until;

    let (mut world, _, change) = preparing();
    let row = world.root.forge().landing(change).expect("preparing row").clone();
    // Let the landing leave its prepare wait normally and stop its merge
    // before dispatch, so the job has ended but the landing still owns the train.
    world.held_merge_calls = Some(Vec::new());
    world.automatic_prepare_jobs();
    world.tick();
    for _ in 0..200 {
        if world.held_merge_calls.as_ref().is_some_and(|calls| !calls.is_empty())
            && !world.root.forge().inspection_landing_stepping(change)
            && world.events.is_empty()
            && world.store.idle()
            && world.root.quiescent()
        {
            break;
        }
        world.tick();
    }
    assert_eq!(world.held_merge_calls.as_ref().expect("held merges").len(), 1);
    // Consume the poll with a live write still waking the landing. Freezing
    // the prepare wait itself would leave it unwakeable as soon as the job ends.
    world.root.hold_landings(true);
    let poll = world.root.forge().inspection_landing_due(change).expect("landing poll");
    world.advance_to(Time::from_nanos(poll.as_nanos()));
    world.until(Until::Ready);
    world.root.hold_landings(false);
    assert!(!world.root.forge().inspection_landing_stepping(change));
    assert_eq!(world.root.forge().inspection_landing_due(change), None);

    // Restore the stale wait in place, preserving membership and train ownership.
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
    assert_eq!(world.root.forge().landing_of(row.task), None);
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
fn a_ready_change_waits_for_its_trains_settling_member() {
    let (world, ready, settling) = super::forge_routes::landing::wait_drill::waiting_on_cohort_member();
    assert!(world.root.forge().change(settling).expect("member").settling);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {ready}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, vec![Wake::Waiter(format!("task {settling}"))]);
    let member = inspection.waits().iter().find(|wait| wait.waiter == format!("task {settling}")).expect("member wait");
    assert_eq!(member.kind, "change: gating");
    assert!(member.wakes.iter().any(|wake| matches!(wake, Wake::Party(party) if party.starts_with("forge: call "))));
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_ready_change_has_no_wake_after_the_cohort_member_leaves_its_train() {
    let (mut world, ready, settling) = super::forge_routes::landing::wait_drill::waiting_on_cohort_member();
    // Freeze forming after the member leaves, to expose the unwakeable snapshot.
    world.root.hold_landings(true);
    let mut member = world.root.forge().change(settling).expect("settling member").clone();
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
    assert_eq!(world.root.forge().landing_of(row.task), None);
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

fn held_branch(ended: bool) -> (World, forge::Name) {
    let mut world = World::new();
    world.adopt();
    let name = forge::Name {
        forge: 1,
        repository: 2,
        what: forge::What::Branch(vec![b"temper610030".as_slice().into(), b"l10030".as_slice().into()].into()),
    };
    let mut records = vec![forge::Stored::Hold(forge::Hold {
        name: name.clone(),
        task: super::reconcile::UNMADE,
        writer: None,
        design_writers: Box::new([]),
        beside: None,
        drift: None,
    })];
    if ended {
        // The taskless, recordless holder retained by reconcile's ended-forge story.
        records.push(forge::Stored::Forge(forge::ForgeRecord {
            number: 1,
            person: None,
            git_program: None,
            kind: forge::Kind::Forgejo,
            mode: forge::Mode::Owned,
            state: forge::ForgeState::Ended { secrets_erased: true, box_removed: true },
            place: forge::Place {
                api: forge::Address { host: b"forge.example".as_slice().into(), port: 3000, tls: None },
                git: b"http://forge.example:3000".as_slice().into(),
                web: None,
            },
            read_only: false,
            bot: forge::Bot {
                login: b"temper".as_slice().into(),
                name: b"Temper".as_slice().into(),
                email: Box::new([]),
            },
            asked: true,
            first: None,
        }));
    } else {
        let mut row = temper_engine_forge_world::changes::change_row();
        row.task = super::reconcile::UNMADE;
        row.change.task = row.task;
        row.branch = b"temper610030/l10030".as_slice().into();
        row.change.state = temper_engine_domain_forge_change::State::Ended;
        records.push(forge::Stored::Change(Box::new(row)));
    }
    // Freeze a leaked hold after closing, or a hold retained on an ended forge,
    // without asking the healthy engine to produce the leaked snapshot.
    for record in records {
        engine::step(&mut world.root, &world.env, engine::Event::Forge(Box::new(forge::Event::Restore { record })));
    }
    assert!(world.root.tasks().task(super::reconcile::UNMADE).is_none());
    assert_eq!(world.root.forge().forge_ended(1), ended);
    (world, name)
}

#[test]
fn a_closed_changes_hold_on_a_live_forge_is_found() {
    let (world, _) = held_branch(false);
    let finding =
        Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("closed change still holds its branch");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine branch holds forge/forge.example/org/repo/branch/temper610030/l10030 (task 10030): still held after release rule: the closing settlement of the landing or change that holds it",
            world.env.now.as_nanos(),
        )
    );
}

#[test]
fn a_taskless_hold_on_an_ended_forge_waits_for_its_own_release_rule() {
    let (world, name) = held_branch(true);
    assert!(world.root.forge().landing(super::reconcile::UNMADE).is_none());
    assert!(world.root.forge().change(super::reconcile::UNMADE).is_none());
    let inspection = inspect(&world);
    let table = inspection.tables().iter().find(|table| table.name == "branch holds").expect("hold table");
    assert_eq!(table.entries.len(), 1, "the hold is inspected, not excluded");
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
    let entry = table.entries.first().expect("held branch");
    assert_eq!(entry.rule, "the forge's return or a person's act");
    assert_eq!(entry.release, skein_world::inspect::Release::Pending);
    assert!(world.root.forge().inspection_holds().contains_key(&name));
}

fn ended_landing_write(retired: bool) -> (World, u64) {
    let (mut world, _, _) = preparing();
    world.hold_write_answers = Some(temper_engine_domain_world::forge_routes::HoldWrite::All);
    world.automatic_prepare_jobs();
    for _ in 0..1000 {
        world.tick();
        let landing = world
            .root
            .forge()
            .inspection_landings()
            .iter()
            .find_map(|(_, row)| (!row.entries.is_empty()).then(|| row.clone()));
        if let Some(mut row) = landing {
            let task = row.landing.task;
            let entry = row.entries.first_mut().expect("landing write");
            assert!(world.root.forge().inspection_entry(entry.entry).is_some());
            // Freeze closing while its write is outstanding. The broken row
            // instead retains a reference whose outbox entry has retired.
            if retired {
                entry.entry = u64::MAX;
            }
            row.landing.phase = temper_engine_domain_forge_landing::Phase::Cancelled;
            row.waiting = None;
            engine::step(
                &mut world.root,
                &world.env,
                engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Landing(row) })),
            );
            return (world, task);
        }
    }
    panic!("landing has an outstanding write");
}

#[test]
fn an_ended_landing_keeps_its_outstanding_write_until_settlement() {
    let (world, task) = ended_landing_write(false);
    let inspection = inspect(&world);
    let table = inspection.tables().iter().find(|table| table.name == "landing rows").expect("landing table");
    let row = table.entries.iter().find(|row| row.key == format!("landing {task}")).expect("landing row");
    assert_eq!(row.release, skein_world::inspect::Release::Pending);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn an_ended_landings_retired_write_reference_is_found() {
    let (world, task) = ended_landing_write(true);
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("write reference outlived its entry");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine landing rows landing {task}: still held after release rule: the closing settlement of the landing",
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
    assert_eq!(world.root.forge().landing_of(row.task), None);
    assert!(world.root.forge().inspection_landings().is_empty());
    let (refusal, _) = world.root.forge().forming_refusal(row.repository, &row.base).expect("stored refusal");
    let forge::FormingRefusal::Jig { retry: due, .. } = refusal.cause else { panic!("jig refusal") };
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
    let row = world.root.forge().change(change).expect("ready change");
    assert_eq!(world.root.forge().forming_refusal(row.repository, &row.base), None);
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
        hold: temper_engine_domain_forge_change::Hold::new(
            temper_engine_domain_forge_change::Cause::Stalled,
            skein_lib::Wall::EPOCH,
        ),
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

#[test]
fn a_current_limit_account_wakes_its_change_on_raise_or_restart() {
    let (world, task) = super::forge_routes::resource_admission::limit_held_change();
    let account = world.root.inspection_limit_hold(task).expect("current limit account");
    assert_eq!(account.revision, world.root.tasks().task(task).expect("held task").revisions.hold);
    assert_eq!(
        change_wakes(&world, task, true),
        [Wake::Party("operator limits: forge.client.resources".into()), Wake::Party("operator restart".into())]
    );
    assert!(change_wakes(&world, task, false).is_empty(), "the forge alone has no root limit account");
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_resource_hold_without_a_limit_account_has_no_limit_wake() {
    let (mut world, task, _) = held_producing(tasks::Hold::ResourceUnavailable);
    let mut row = world.root.forge().change(task).expect("change").clone();
    row.delegate = None;
    row.change.state = temper_engine_domain_forge_change::State::Producing { requested: false };
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    engine::fire(&mut world.root, &world.env);
    assert!(world.root.inspection_limit_hold(task).is_none());
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("no retained release path");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {task}: waits on change: producing with no live wake: []",
            world.env.now.as_nanos()
        )
    );
}

#[test]
fn a_live_task_room_wait_names_the_tasks_that_can_free_slots() {
    let (mut world, large, _, task) = super::forge_routes::reviews::pending_large_with_alpha_and_beta(8);
    world.advance_ticks(80);
    let row = world.root.tasks().task(task).expect("waiting change");
    assert_eq!(row.room_wait.map(|wait| wait.room), Some(tasks::LiveRoom::Deployment));
    let reviewer = world.root.forge().change(large).expect("reviewed change").delegate.expect("reviewer").0;
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("room wait");
    assert!(wait.wakes.contains(&Wake::Waiter(format!("task {reviewer}"))));
    assert!(!wait.wakes.contains(&Wake::Waiter(format!("task {task}"))), "ending oneself cannot free a slot");
    assert_eq!(wait.bound, None);
    assert!(change_wakes(&world, task, false).is_empty(), "the forge alone has no jig room wait");
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn live_tasks_do_not_wake_a_change_without_a_room_wait() {
    let mut driver = super::panel_before_change::before_change();
    driver.send(engine::Event::Forge(Box::new(forge::Event::Change {
        row: super::panel_before_change::producing_row(),
    })));
    driver.settle();
    let original = driver.root.forge().change(51).expect("change").clone();
    let producer = original.delegate.expect("live delegate").0;
    assert!(driver.root.tasks().task(producer).is_some());
    assert_eq!(driver.root.tasks().task(51).expect("change task").room_wait, None);
    // Freeze a lost delegate registration, not an admission refusal. Other
    // live tasks cannot supply the missing registration's wake.
    let mut row = original.clone();
    row.delegate = None;
    row.change.state = temper_engine_domain_forge_change::State::Producing { requested: false };
    engine::step(
        &mut driver.root,
        &driver.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    engine::fire(&mut driver.root, &driver.env);
    let finding = Checks::new(71).quiet(&driver.root, driver.env.now, 1).expect_err("no room release path");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task 51: waits on change: producing with no live wake: []",
            driver.env.now.as_nanos()
        )
    );
    engine::step(
        &mut driver.root,
        &driver.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(original)) })),
    );
    driver.settle();
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
fn a_sittings_clock_starts_after_the_setup_restart_drains() {
    let mut shared = super::forge_routes::status::shared::shared(false);
    let before = shared.world.env.now;
    super::forge_routes::status::restart_status_world(&mut shared.world);
    assert!(shared.world.env.now > before, "the restart drained work after the fixture's initial clock read");
    let start = shared.world.env.now;
    shared.first_run();
    assert!(shared.world.env.now >= start.saturating_add(Duration::from_secs(80 * 60)));
    shared.world.close();
}

#[test]
fn a_story_clock_refuses_to_move_backwards() {
    let mut world = temper_engine_domain_world::kinds::forge_world();
    let before = world.env.now;
    world.jump(Duration::from_secs(1));
    let now = world.env.now;
    let finding = world.try_advance_to(before).expect_err("a backwards clock move is refused");
    assert!(finding.contains("the story clock moves forward"), "{finding}");
    assert_eq!(world.env.now, now);
    world.close();
}

#[test]
fn a_story_clock_moves_forward_to_a_later_time() {
    let mut world = temper_engine_domain_world::kinds::forge_world();
    let later = world.env.now.saturating_add(Duration::from_secs(1));
    assert_eq!(world.try_advance_to(later), Ok(()));
    assert_eq!(world.env.now, later);
    world.close();
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

fn producer_wait(turn: bool) -> (World, u64, u64) {
    use super::forge_routes::merge_job::producer;
    let (world, change, child) = if turn {
        let (world, change, child, _, _) = producer::turn_producer();
        (world, change, child.task)
    } else {
        let (world, change, child, _, _, _) = producer::displaced_producer();
        (world, change, child.task)
    };
    wait_for_producer(world, change, child, turn)
}

fn wait_for_producer(mut world: World, change: u64, child: u64, turn: bool) -> (World, u64, u64) {
    for _ in 0..1000 {
        world.tick();
        let row = world.root.forge().change(change).expect("change");
        let waiting = if turn {
            matches!(row.change.state, temper_engine_domain_forge_change::State::Ready { .. })
        } else {
            matches!(row.change.state, temper_engine_domain_forge_change::State::WaitingProducer { .. })
                && row.delegate.is_none()
        };
        if waiting
            && world.events.is_empty()
            && world.store.idle()
            && world.root.quiescent()
            && world.root.next_deadline(&world.env).is_none_or(|due| due > world.env.now)
        {
            assert!(world.root.inspection_pushing_child(child));
            assert!(world.root.tasks().task(change).expect("owner").delegates.contains(&child));
            return (world, change, child);
        }
    }
    panic!("change waits for its pushing child");
}

#[test]
fn a_ready_change_waits_for_its_live_pushing_child() {
    let (world, change, child) = producer_wait(true);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("ready wait");
    assert_eq!(wait.kind, "change: ready");
    assert_eq!(wait.wakes, [Wake::Waiter(format!("task {child}"))]);
    assert_eq!(wait.bound, None);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

fn misplaced_producer_wait(turn: bool) -> (World, u64) {
    let (mut world, change, child) = producer_wait(turn);
    // A broken restored row names a task that has no pushing children. The
    // other change's live child must not become a fallback for this owner.
    let mut row = world.root.forge().change(change).expect("change").clone();
    row.task = child;
    row.change.task = child;
    row.delegate = None;
    world.root.hold_landings(true);
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    engine::fire(&mut world.root, &world.env);
    world.root.hold_landings(false);
    (world, child)
}

#[test]
fn a_ready_change_cannot_borrow_another_changes_pushing_child() {
    let (world, change) = misplaced_producer_wait(true);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("misplaced wait");
    assert_eq!(wait.wakes, Vec::<Wake>::new());
    let tasks::Party::Task(owner) = world.root.tasks().task(change).expect("misplaced owner").requester else {
        panic!("change requester");
    };
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("no pushing child of this owner");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {owner}: waits on change: ready with no live wake: [waiter task {change} (waits: change: ready)]",
            world.env.now.as_nanos(),
        )
    );
}

#[test]
fn a_displaced_pushing_child_exempts_its_conflict_wait() {
    let (world, change, child) = producer_wait(false);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("conflict wait");
    assert_eq!(wait.kind, "change: resolving");
    assert!(wait.wakes.contains(&Wake::Waiter(format!("task {child}"))));
    assert_eq!(wait.bound, None);
    let row = world.root.forge().change(change).expect("change");
    let old_bound = Time::from_nanos(row.change.since.as_nanos())
        .saturating_add(world.root.forge().inspection_limits().change_policy.stall);
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    assert_eq!(checks.step(old_bound.saturating_add(Duration::from_nanos(1)), 2), Ok(()));
    assert_eq!(checks.quiet(&world.root, old_bound.saturating_add(Duration::from_nanos(1)), 3), Ok(()));
}

#[test]
fn a_conflict_wait_cannot_borrow_another_changes_pushing_child() {
    let (world, change) = misplaced_producer_wait(false);
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("no pushing child of this owner");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {change}: waits on change: resolving with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
}

fn conflict_read(room_waiter: Option<u64>) -> (World, u64, Time, Time) {
    let (mut world, change, child) = match room_waiter {
        Some(task) => conflict_with_room_waiter(task),
        None => producer_wait(false),
    };
    let row = world.root.forge().change(change).expect("change");
    let old_bound = Time::from_nanos(row.change.since.as_nanos())
        .saturating_add(world.root.forge().inspection_limits().change_policy.stall);
    world.jump(world.root.forge().inspection_limits().change_policy.stall);
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(951)),
        sign_in: world.signed_in.expect("person"),
        key: [96; 16],
        ask: jig_domain_people::Ask::Cancel {
            project: 1,
            task: child,
            reason: b"finish pushing".as_slice().into(),
            grace: None,
        },
    }));
    for _ in 0..1000 {
        world.tick();
        if world.root.tasks().task(child).is_none() && world.root.forge().inspection_change_read_bound(change).is_some()
        {
            world.hold_reads_at_any_step = true;
            world.held_reads.get_or_insert_with(Vec::new);
        }
        if world.root.tasks().task(child).is_none()
            && let Some(bound) = world.root.forge().inspection_change_read_bound(change)
            && bound.as_nanos() > old_bound.as_nanos()
            && world.events.is_empty()
            && world.store.idle()
            && world.root.quiescent()
        {
            assert!(matches!(
                world.root.forge().change(change).expect("change").change.state,
                temper_engine_domain_forge_change::State::WaitingProducer { .. }
            ));
            return (world, change, old_bound, Time::from_nanos(bound.as_nanos()));
        }
    }
    panic!("a conflict's fresh gather after the producer's terminal");
}

fn conflict_with_room_waiter(task: u64) -> (World, u64, u64) {
    use super::forge_routes::merge_job::producer;

    let mut world = producer::displaced_producer_fixture();
    super::panel_before_change::seed_change_procedure(&mut world.store, task, 0, None, None);
    let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&key)
    else {
        panic!("live change procedure")
    };
    let mut row = row.clone();
    // Keep the second procedure live but not due when the broken room offer
    // is injected. The first change is the only one with a gather in flight.
    row.phase = tasks::Phase::Active(tasks::Active::BackingOff { until: skein_lib::Wall::from_nanos(u64::MAX) });
    world
        .store
        .seed(Box::new([engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))]), Box::new([]));
    let (world, change, child, _, _, _) = producer::displaced_producer_in(world);
    wait_for_producer(world, change, child.task, false)
}

fn room_read(own_read: bool) -> (World, u64) {
    const OTHER_CHANGE: u64 = 51;
    let waiter = (!own_read).then_some(OTHER_CHANGE);
    let (mut world, source, _, _) = conflict_read(waiter);
    let mut row = world.root.forge().change(source).expect("reading change").clone();
    let task = waiter.unwrap_or(source);
    // Freeze a room retry while its gather reads. The place stays on the row
    // until the fresh decision, even though its ask has reserved room. The
    // broken snapshot assigns that place to a task with no read of its own.
    row.task = task;
    row.change.task = task;
    row.change.state = temper_engine_domain_forge_change::State::Opening { requested: false };
    row.room = Some(1);
    row.delegate = None;
    row.pending = None;
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    engine::fire(&mut world.root, &world.env);
    assert!(world.root.forge().room_asked(task));
    if !own_read {
        assert!(world.root.forge().inspection_change_read_bound(source).is_some(), "the other change is reading");
        assert!(world.root.tasks().procedure_due(task).is_none(), "the room waiter is not due");
    }
    (world, task)
}

#[test]
fn a_room_wait_keeps_its_in_flight_read_wake() {
    let (world, task) = room_read(true);
    for root in [true, false] {
        let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
        if root {
            temper_engine_inspect::inspect(&world.root, &mut inspection);
        } else {
            temper_engine_inspect::forge(world.root.forge(), &mut inspection);
        }
        let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("room wait");
        assert_eq!(wait.kind, "change: room");
        assert!(wait.wakes.iter().any(|wake| matches!(wake, Wake::Party(name) if name.starts_with("forge: call "))));
        assert_eq!(wait.bound, None);
        assert_eq!(
            skein_world::inspect::check_waits(
                &[&inspection],
                skein_world::inspect::Moment { seed: 71, at: world.env.now, iteration: 1 },
            ),
            Ok(())
        );
    }
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_reserved_room_wait_keeps_its_due_procedure_wake() {
    let (world, task) = room_read(true);
    assert!(world.root.tasks().procedure_due(task).is_some(), "reserved procedure is due");
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("room wait");
    assert!(wait.wakes.contains(&Wake::Party("root".into())), "the reserved step wakes its owner");
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn a_room_wait_cannot_borrow_another_changes_read() {
    let (world, task) = room_read(false);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("room wait");
    assert_eq!(wait.wakes, Vec::<Wake>::new());
    let finding = Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("room waiter has no read of its own");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 1: engine task {task}: waits on change: room with no live wake: []",
            world.env.now.as_nanos(),
        )
    );
}

#[test]
fn a_conflict_read_after_the_producers_end_uses_its_gather_bound() {
    let (world, change, old_bound, bound) = conflict_read(None);
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("conflict wait");
    assert_eq!(wait.bound, Some(bound));
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    assert_eq!(checks.step(old_bound.saturating_add(Duration::from_nanos(1)), 2), Ok(()));
    assert_eq!(checks.quiet(&world.root, bound, 3), Ok(()));
}

#[test]
fn a_conflict_read_past_its_gather_bound_is_found() {
    let (world, change, _, bound) = conflict_read(None);
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, bound, 1), Ok(()));
    let at = bound.saturating_add(Duration::from_nanos(1));
    let finding = checks.step(at, 2).expect_err("the gather still has a bound after its producer ends");
    assert_eq!(
        finding,
        format!(
            "seed 71: at {} ns, iteration 2: engine task {change}: waits on change: resolving past bound {} ns with no quiet point since",
            at.as_nanos(),
            bound.as_nanos(),
        )
    );
}

fn first_withheld() -> (World, u64, Time) {
    let (mut world, change) = super::forge_routes::landing::produced_with_hold(
        &temper_engine_domain_world::forge_routes::StoryLimits::default(),
        true,
    );
    for _ in 0..1000 {
        world.tick();
        if world.root.forge().inspection_merge_withheld(change)
            && world.events.is_empty()
            && world.store.idle()
            && world.root.quiescent()
        {
            let row = world.root.forge().change(change).expect("queue first");
            assert!(matches!(row.change.state, temper_engine_domain_forge_change::State::First { .. }));
            let bound = Time::from_nanos(row.change.since.as_nanos())
                .saturating_add(world.root.forge().inspection_limits().change_policy.stall);
            return (world, change, bound);
        }
    }
    panic!("queue merge withheld by the operator");
}

#[test]
fn a_first_whose_merge_is_withheld_waits_for_restart_without_a_stall_bound() {
    let (world, change, old_bound) = first_withheld();
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("first wait");
    assert_eq!(wait.bound, None);
    assert!(wait.wakes.contains(&Wake::Party("operator restart".into())));
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, world.env.now, 1), Ok(()));
    let at = old_bound.saturating_add(Duration::from_nanos(1));
    assert_eq!(checks.step(at, 2), Ok(()));
    assert_eq!(checks.quiet(&world.root, at, 3), Ok(()));
}

#[test]
fn a_first_with_unknown_forge_facts_still_stalls_under_the_operators_hold() {
    let (mut world, change, bound) = first_withheld();
    // Freeze a restored first before its fresh forge facts arrive. No merge
    // has been decided in this incarnation: the operator does not hold its read.
    let row = world.root.forge().change(change).expect("queue first").clone();
    engine::step(
        &mut world.root,
        &world.env,
        engine::Event::Forge(Box::new(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) })),
    );
    assert!(world.root.forge().landing_held());
    assert!(!world.root.forge().inspection_merge_withheld(change));
    let inspection = inspect(&world);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {change}")).expect("first wait");
    assert_eq!(wait.kind, "change: first");
    assert_eq!(wait.bound, Some(bound));
    assert!(!wait.wakes.contains(&Wake::Party("operator restart".into())));
    let mut checks = Checks::new(71);
    assert_eq!(checks.quiet(&world.root, bound, 1), Ok(()));
    let at = bound.saturating_add(Duration::from_nanos(1));
    assert_eq!(
        checks.step(at, 2).expect_err("the forge still owes its facts"),
        format!(
            "seed 71: at {} ns, iteration 2: engine task {change}: waits on change: first past bound {} ns with no quiet point since",
            at.as_nanos(),
            bound.as_nanos(),
        )
    );
}

fn ended_owners_write(withdrawn: bool) -> (World, u64, u64) {
    use skein_forge_client::{Condition, Effect, Entry};
    let mut world = World::new();
    world.adopt();
    world.oracle_settle();
    let task = super::reconcile::UNMADE;
    let entry = Entry {
        number: 900_002,
        task,
        repository: temper_engine_forge_world::REPO,
        effect: Effect {
            write: skein_forge::api::Write::Post {
                number: 1,
                key: b"milestone".as_slice().into(),
                body: b"Task milestone".as_slice().into(),
            },
            condition: Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    let number = entry.number;
    if withdrawn {
        // Freeze the leaked entry after handoff and withdrawal from the client.
        engine::step(
            &mut world.root,
            &world.env,
            engine::Event::Forge(Box::new(forge::Event::Restore {
                record: forge::Stored::Entry { entry, held: true },
            })),
        );
    } else {
        // A prior uncommitted entry holds the handoff line. This committed
        // projection of an ended task is Ready, not yet in the client's outbox.
        let mut prior = entry.clone();
        prior.number -= 1;
        for entry in [prior, entry] {
            engine::step(&mut world.root, &world.env, engine::Event::Forge(Box::new(forge::Event::Enqueue { entry })));
        }
        engine::step(
            &mut world.root,
            &world.env,
            engine::Event::Forge(Box::new(forge::Event::Committed { entry: number })),
        );
    }
    assert!(world.root.tasks().task(task).is_none());
    assert_eq!(world.root.forge().inspection_entry_handed(number), withdrawn);
    assert!(world.root.forge().inspection_entry(number).is_some());
    assert!(world.root.forge().inspection_client().inspection_entry(number).is_none());
    (world, task, number)
}

#[test]
fn an_ended_owners_write_waiting_for_handoff_has_not_settled() {
    let (world, task, number) = ended_owners_write(false);
    let inspection = inspect(&world);
    let table = inspection.tables().iter().find(|table| table.name == "forge write entries").expect("write table");
    let entry = table.entries.iter().find(|entry| entry.key == format!("entry {number} (task {task})")).expect("entry");
    assert_eq!(entry.release, skein_world::inspect::Release::Pending);
    assert_eq!(Checks::new(71).quiet(&world.root, world.env.now, 1), Ok(()));
}

#[test]
fn an_ended_owners_write_kept_after_withdrawal_is_found() {
    let (world, task, number) = ended_owners_write(true);
    assert_eq!(
        Checks::new(71).quiet(&world.root, world.env.now, 1).expect_err("withdrawn entry outlives its owner"),
        format!(
            "seed 71: at {} ns, iteration 1: engine forge write entries entry {number} (task {task}): still held after release rule: the write's settlement, or its owner's end with the entry withdrawn",
            world.env.now.as_nanos(),
        )
    );
}

fn landed_waiting_for_restart_reads() -> (forge::Domain, skein_lib::Env<forge::Limits>, u64) {
    use skein_lib::{Env, Queue};
    let mut world = World::new();
    world.adopt();
    world.oracle_settle();
    let repository = world.root.forge().repository(temper_engine_forge_world::REPO).expect("repository").clone();
    let env = Env { now: world.env.now, wall: world.env.wall, limits: world.env.limits.forge };
    let mut domain = forge::Domain::new(
        &env.limits,
        71,
        forge::Config { push: None, namespace: b"checks".as_slice().into(), writers: Box::new([]) },
        &[forge::GitIdentity { forge: 1, credential: 1 }],
        &[forge::Configured {
            number: 1,
            kind: forge::Kind::Forgejo,
            mode: forge::Mode::Connected,
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            git: Box::new([]),
            web: None,
            read_only: false,
            asked: false,
        }],
        false,
    )
    .expect("fixture limits");
    let mut row = temper_engine_forge_world::changes::change_row();
    let task = row.task;
    row.change.state = temper_engine_domain_forge_change::State::Landed { merge: [7; 32], since: env.wall };
    row.change.last_head = Some([6; 32]);
    row.pull = Some(1);
    let mut out = Queue::uncapped();
    domain.restore_begin(env.wall);
    for event in [
        forge::Event::Restore { record: forge::Stored::Repository(repository) },
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        forge::Event::Restore {
            record: forge::Stored::Names {
                task,
                resources: Box::new([forge::Name { forge: 1, repository: 2, what: forge::What::Pull(1) }]),
            },
        },
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        forge::Event::CatchUp,
    ] {
        forge::step(&mut domain, &env, event, &mut out);
    }
    domain.offered_change(task);
    forge::step(
        &mut domain,
        &env,
        forge::Event::StepChange {
            task,
            entry: 900_003,
            heard: temper_engine_domain_forge_change::Heard {
                review_request: None,
                closer_waiting: false,
                start_conflict: None,
                lacking: None,
                spend: None,
                delegate: temper_engine_domain_forge_change::Status::Unknown,
                flake: false,
                no_test: false,
                delegate_running: false,
                delegate_live: false,
                effect: temper_engine_domain_forge_change::EffectResult::None,
                released: false,
                cancelled: false,
            },
            gates: Box::new([]),
            queue_repair_active: false,
            kept: false,
            merge_job: false,
            pending: Box::new([]),
        },
        &mut out,
    );
    assert_eq!(domain.inspection_restart_reads(task).len(), 2);
    (domain, env, task)
}

#[test]
fn a_landed_change_waits_for_its_repositories_restart_read() {
    let (domain, env, task) = landed_waiting_for_restart_reads();
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::forge(&domain, &mut inspection);
    let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {task}")).expect("landed wait");
    assert_eq!(
        wait.wakes,
        [
            Wake::Party("forge: restart read 1/2/Pull(1)".into()),
            Wake::Party("forge: restart read 1/2/Repository".into())
        ]
    );
    assert_eq!(
        skein_world::inspect::check_waits(
            &[&inspection],
            skein_world::inspect::Moment { seed: 71, at: env.now, iteration: 1 }
        ),
        Ok(())
    );
}

#[test]
fn a_landed_change_cannot_borrow_another_tasks_restart_read() {
    let (mut domain, env, task) = landed_waiting_for_restart_reads();
    let mut row = domain.change(task).expect("landed change").clone();
    row.task += 1;
    row.change.task = row.task;
    let unparked = row.task;
    // The unrelated row has not asked to wait for fresh repository facts.
    forge::step(&mut domain, &env, forge::Event::Change { row }, &mut skein_lib::Queue::uncapped());
    assert!(domain.inspection_restart_reads(unparked).is_empty());
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::forge(&domain, &mut inspection);
    assert_eq!(
        skein_world::inspect::check_waits(
            &[&inspection],
            skein_world::inspect::Moment { seed: 71, at: env.now, iteration: 1 }
        )
        .expect_err("no restart wait"),
        format!(
            "seed 71: at {} ns, iteration 1: engine task {unparked}: waits on change: landing with no live wake: []",
            env.now.as_nanos()
        ),
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
