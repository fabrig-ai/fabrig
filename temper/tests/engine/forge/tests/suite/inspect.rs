//! Restored outbox and ordinary-round checks (temper/docs/design/testing/engine-checks.md, 2 and 6).
use skein_forge_client::{Condition, Effect, Entry};
use skein_world::inspect::{Moment, Wake, check_tables, check_waits};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{REPO, World, changes, landings};

const OWNER: u64 = landings::LANDING;
const WAITER: u64 = 70;

fn restored(ended: bool) -> World {
    let mut world = World::with_entries(307, 2);
    world.adopt();
    let mut member = changes::change_row();
    member.pull = Some(1);
    let mut owner = landings::landing_row(&member);
    owner.landing.phase = if ended {
        landing::Phase::Cancelled
    } else {
        landing::Phase::Held { was: Box::new(landing::Phase::Waiting), why: landing::Hold::Prepare }
    };
    owner.waiting = Some(landing::Awaited::Release);
    owner.entries = [1, 2]
        .map(|entry| top::LandingEntry {
            entry,
            preparation: None,
            base: None,
            member: 0,
            gate: 0,
            operation: top::Operation::Dispatch,
            run: None,
        })
        .into();
    world.store_before_restart(top::Stored::Landing(owner));
    for number in [1, 2] {
        world.store_before_restart(top::Stored::Entry {
            entry: Entry {
                number,
                task: OWNER,
                repository: REPO,
                effect: Effect {
                    write: skein_forge::api::Write::Dispatch {
                        workflow: b"gate.yml".as_slice().into(),
                        branch: b"main".as_slice().into(),
                        inputs: Box::new([]),
                    },
                    condition: Condition::None,
                },
                start: None,
                attempt: None,
                failures: 0,
                queued: false,
            },
            held: true,
        });
    }
    let mut waiter = changes::change_row();
    waiter.task = WAITER;
    waiter.change.task = WAITER;
    waiter.room = Some(1);
    waiter.change.state = change::State::Opening { requested: false };
    let (cold, hot) = waiter.split();
    world.store_before_restart(top::Stored::ChangeCold(Box::new(cold)));
    world.store_before_restart(top::Stored::ChangeHot(Box::new(hot)));
    world.restart();
    world
}

fn asked_making() -> World {
    let mut world = super::goal_branch_restart::before_restart();
    let holder = super::goal_branch_restart::held_landing();
    world.hold_task(holder.landing.task);
    world.store_before_restart(top::Stored::Landing(holder));
    for number in [1, 2] {
        world.store_before_restart(super::goal_branch_restart::held_dispatch(number));
    }
    world.restart();
    world.run_for(2);
    let inspection = world.inspection();
    let wait = inspection.waits().iter().find(|wait| wait.kind == "making: room").expect("making in line");
    assert_eq!(wait.wakes, [Wake::Waiter("task 63".into()), Wake::Waiter("task 63".into())]);
    for number in [1, 2] {
        world.event(top::Event::Resolve { entry: number, again: false });
    }
    world
}

#[test]
fn an_asked_making_has_the_forges_room_wake() {
    let world = asked_making();
    let inspection = world.inspection();
    let wait = inspection.waits().iter().find(|wait| wait.waiter == "task 77").expect("making wait");
    assert_eq!(wait.kind, "making: room");
    assert_eq!(wait.wakes, [Wake::Party("forge: room".into())]);
    assert_eq!(check_waits(&[&inspection], moment(&world)), Ok(()));
}

#[test]
fn a_making_without_its_ask_or_holders_has_no_live_wake() {
    let mut world = asked_making();
    world.event(top::Event::RoomUnavailable { task: 77 });
    let inspection = world.inspection();
    let finding = check_waits(&[&inspection], moment(&world)).expect_err("making has neither ask nor holder");
    assert_eq!(
        finding,
        format!(
            "seed 307: at {} ns, iteration 0: engine task 77: waits on making: room with no live wake: []",
            world.now().as_nanos(),
        )
    );
}

fn moment(world: &World) -> Moment {
    Moment { seed: 307, at: world.now(), iteration: 0 }
}

#[test]
fn ended_outbox_owners_are_found_by_each_check() {
    let world = restored(true);
    let inspection = world.inspection();
    let tables = check_tables(&[&inspection], moment(&world)).expect_err("ended owners release their entries");
    assert!(tables.contains("forge write entries entry 1 (task 61)"), "{tables}");
    let waits = check_waits(&[&inspection], moment(&world)).expect_err("only ended owners can free this room");
    assert!(waits.contains("task 70") && waits.contains("change: room"), "{waits}");
    for number in [1, 2] {
        assert!(waits.contains(&format!("ended entry {number} of task {OWNER}")), "{waits}");
    }
    assert!(world.check().is_err());
}

#[test]
fn live_outbox_owners_can_wake_the_room_waiter() {
    let world = restored(false);
    let inspection = world.inspection();
    assert_eq!(check_tables(&[&inspection], moment(&world)), Ok(()));
    assert_eq!(check_waits(&[&inspection], moment(&world)), Ok(()));
    assert_eq!(world.check(), Ok(()));
    let wait = inspection.waits().iter().find(|wait| wait.waiter == "task 70").expect("room waiter");
    assert!(wait.wakes.iter().all(|wake| *wake == Wake::Waiter("task 61".into())));
}

#[test]
fn an_unanswered_delegate_ask_waits_on_the_scripted_root() {
    let mut world = World::new(308);
    world.adopt();
    let mut row = changes::change_row();
    row.change.state = change::State::Gating { head: [1; 32], asked: Box::new([9]) };
    world.event(top::Event::Change { row });
    let inspection = world.inspection();
    let wait = inspection.waits().iter().find(|wait| wait.waiter == "task 51").expect("change wait");
    assert_eq!(wait.wakes, [Wake::Party("root".into())]);
    assert_eq!(world.check(), Ok(()));
}

#[test]
fn a_missing_delegate_without_an_ask_is_a_finding() {
    let mut world = World::new(309);
    world.adopt();
    let mut row = changes::change_row();
    row.change.state = change::State::Gating { head: [1; 32], asked: Box::new([]) };
    world.event(top::Event::Change { row });
    assert_eq!(world.check(), Ok(()), "registration owes its first scripted-root step");
    let finding = check_waits(&[&world.inspection()], moment(&world))
        .expect_err("no requested delegate, write, question or timer in the forge");
    assert!(finding.contains("change: gating") && finding.contains("no live wake"), "{finding}");
}

#[test]
fn an_emitted_wait_is_owed_to_the_scripted_root() {
    let mut world = World::new(310);
    world.adopt();
    world.landing_ci(b"main", true);
    world.produce(b"temper/51");
    let pull = world.open_pull(b"temper/51", b"main");
    let head = temper_engine_forge_world::translate::commit(world.branch(b"temper/51"));
    let mut row = changes::change_row();
    row.change.state = change::State::Checking { head };
    row.change.last_head = Some(head);
    row.change.since = world.wall();
    row.pull = Some(pull);
    world.event(top::Event::Change { row });
    world.status(b"temper/51", skein_fake_forge_domain::api::Check::Pending);
    world.event(top::Event::Hint {
        hint: skein_forge::api::Hint { repository: REPO, change: skein_forge::api::Change::Commit(head), key: None },
    });
    world.event(top::Event::StepChange {
        merge_job: false,
        task: 51,
        entry: 100,
        heard: change::Heard {
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            delegate_running: false,
            delegate_live: false,
            flake: false,
            no_test: false,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: false,
    });
    world.run_until_read(&skein_forge::api::Read::Branch { branch: b"temper/51".as_slice().into() });
    let in_flight = world.inspection();
    let wait = in_flight.waits().iter().find(|wait| wait.waiter == "task 51").expect("fresh-read change");
    assert!(wait.wakes.iter().any(|wake| matches!(wake, Wake::Party(party) if party.starts_with("forge: call "))));
    assert_eq!(check_waits(&[&in_flight], moment(&world)), Ok(()));
    world.run_for(30);
    assert!(
        world.seen().iter().any(|request| matches!(
            request,
            top::Request::ChangeDecision { task: 51, decision: change::Decision::Wait { .. }, .. }
        )),
        "{:?}",
        world.seen()
    );
    assert_eq!(world.check(), Ok(()));
    let answered = world.inspection();
    let finding = check_waits(&[&answered], moment(&world)).expect_err("the answered read no longer wakes its change");
    assert!(finding.contains("change: checking") && finding.contains("no live wake"), "{finding}");
}

fn settling_lost_merge() -> World {
    const ENTRY: u64 = 100;
    let mut world = World::new(311);
    world.adopt();
    world.produce(b"temper/51");
    let pull = world.open_pull(b"temper/51", b"main");
    let head = temper_engine_forge_world::translate::commit(world.branch(b"temper/51"));
    let mut row = changes::change_row();
    row.pull = Some(pull);
    row.pending = Some(ENTRY);
    row.change.state = change::State::Landing { head };
    row.change.last_head = Some(head);
    row.change.since = world.wall();
    world.event(top::Event::Change { row });
    world.lose_writes();
    world.event(top::Event::Enqueue {
        entry: Entry {
            number: ENTRY,
            task: 51,
            repository: REPO,
            effect: Effect {
                write: skein_forge::api::Write::Merge { number: pull, head, how: skein_forge::api::MergeHow::Style },
                condition: Condition::Merge { base: b"main".as_slice().into() },
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    });
    world.event(top::Event::Committed { entry: ENTRY });
    world.run_for(1);
    assert!(
        world.seen().iter().any(|request| matches!(
            request,
            top::Request::Outcome { entry: ENTRY, outcome: skein_forge_client::Outcome::Uncertain, .. }
        )),
        "{:?}",
        world.seen()
    );
    let merge = world.merge(pull, b"temper/51");
    assert_eq!(
        changes::change_step(&mut world, 101, change::Status::Passed, Box::new([])),
        change::Decision::Finish { merge }
    );
    assert_eq!(world.change(51).pending, None, "containment detached the uncertain write");
    assert!(world.stored().contains_key(&top::Key::Entry(ENTRY)), "withdrawal still awaits the client's find");
    world.event(top::Event::SettleEffects { task: 51, root: 51, ending: top::ReleaseEnding::Done });
    world
}

#[test]
fn a_landed_change_waits_for_its_timed_out_merge_to_settle() {
    let world = settling_lost_merge();
    let inspection = world.inspection();
    let wait = inspection.waits().iter().find(|wait| wait.waiter == "task 51").expect("change wait");
    assert_eq!(wait.kind, "change: landing");
    assert_eq!(wait.wakes, [Wake::Party("forge: entry 100".into())]);
    assert_eq!(check_waits(&[&inspection], moment(&world)), Ok(()));
}

#[test]
fn a_landed_change_without_its_settling_merge_has_no_live_wake() {
    let mut world = settling_lost_merge();
    world.run_for(20);
    assert!(!world.stored().contains_key(&top::Key::Entry(100)), "the client's withdrawal settled");
    let inspection = world.inspection();
    let finding = check_waits(&[&inspection], moment(&world)).expect_err("no unsettled write remains");
    assert!(finding.contains("change: landing") && finding.contains("no live wake"), "{finding}");
}

#[test]
fn an_emitted_landing_wait_is_owed_to_the_scripted_root() {
    let mut world = super::landing_proofs::start(60);
    super::landing_proofs::until_dispatch(&mut world);
    let mut waiting = false;
    for entry in 200..220 {
        let decision = landings::step_landing(&mut world, OWNER, entry);
        if matches!(decision, landing::Decision::Wait { on: landing::Awaited::Run { .. }, .. }) {
            waiting = true;
            break;
        }
    }
    assert!(waiting, "the gate's run is still outstanding");
    let inspection = world.inspection();
    let wait = inspection.waits().iter().find(|wait| wait.waiter == "task 61").expect("landing wait");
    assert_eq!(wait.kind, "landing: run");
    assert!(wait.wakes.is_empty(), "only the scripted root can see the unarmed poll: {wait:?}");
    assert!(check_waits(&[&inspection], moment(&world)).is_err());
    assert_eq!(world.check(), Ok(()));
}

fn answered_prepare_write(stale: bool) -> (World, u64) {
    let mut world = landings::turn_world();
    world.produce(b"temper/51");
    landings::ready_change(&mut world, 51, b"temper/51");
    let row = landings::landing_row(world.change(51));
    landings::register(&mut world, row);
    assert_eq!(landings::step_landing_raw(&mut world, OWNER, 100), landing::Decision::None);
    assert!(matches!(landings::step_landing_raw(&mut world, OWNER, 101), landing::Decision::Prepare { .. }));
    let mut job =
        top::history::JobStep { task: 701, attempt: 1, job: top::history::Job::Prepare, outcome: None, head: None };
    world.event(top::Event::JobHistory { task: OWNER, job });
    world.prepare_landing(OWNER, false);
    if stale {
        world.push_file(b"main", b"outside", b"new base");
    }
    let report = br#"{"combined":[{"member":1,"by":"merge"}]}"#;
    let mut heard = super::landing_batches::heard(landing::Preparation::Idle, report);
    heard.outcomes = top::preparation::outcomes(report, 1).expect("successful report");
    job.outcome = Some(top::history::JobOutcome::Done);
    heard.job = Some(job);
    let mut step = landings::read_landing_step(&mut world, OWNER, 102, heard);
    if stale {
        for entry in 103..110 {
            if matches!(step.decision, landing::Decision::Delete { .. }) {
                break;
            }
            step = landings::read_landing_step(
                &mut world,
                OWNER,
                entry,
                super::landing_batches::heard(landing::Preparation::Idle, b""),
            );
        }
        assert!(matches!(step.decision, landing::Decision::Delete { .. }), "{:?}", step.decision);
    } else {
        assert_eq!(step.decision, landing::Decision::Open);
    }
    assert_eq!(world.landing(OWNER).landing.phase, landing::Phase::Preparing);
    assert_eq!(world.landing(OWNER).waiting, None);
    assert_eq!(world.landing(OWNER).job.as_ref().expect("kept job").step.outcome, job.outcome);
    (world, step.entry.expect("round write"))
}

#[test]
fn an_answered_prepare_waits_on_its_rounds_write() {
    for stale in [false, true] {
        let (world, entry) = answered_prepare_write(stale);
        let inspection = world.inspection();
        let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {OWNER}")).expect("prepare wait");
        assert_eq!(wait.kind, "landing: prepare");
        assert_eq!(wait.wakes, [Wake::Party(format!("forge: entry {entry}"))]);
        assert_eq!(check_waits(&[&inspection], moment(&world)), Ok(()));
    }
}

#[test]
fn an_answered_prepare_without_its_rounds_write_is_found() {
    for stale in [false, true] {
        let (mut world, entry) = answered_prepare_write(stale);
        let inspection = world.inspection();
        let wait = inspection.waits().iter().find(|wait| wait.waiter == format!("task {OWNER}")).expect("prepare wait");
        assert_eq!(wait.wakes, [Wake::Party(format!("forge: entry {entry}"))]);
        // Resolve the write without giving the scripted root another step.
        // The ended job must not become an external prepare-job fallback.
        let top::Stored::Entry { entry: write, .. } = world.stored().get(&top::Key::Entry(entry)).expect("write")
        else {
            panic!("entry row")
        };
        world.event(top::Event::Restore { record: top::Stored::Entry { entry: write.clone(), held: true } });
        world.event(top::Event::Resolve { entry, again: false });
        assert!(!world.stored().contains_key(&top::Key::Entry(entry)));
        let inspection = world.inspection();
        let finding = check_waits(&[&inspection], moment(&world)).expect_err("answered prepare lost its write");
        assert!(finding.contains("landing: prepare with no live wake: [ended prepare job]"), "{finding}");
    }
}

#[test]
fn a_landing_round_releases_its_write_entries() {
    let world = landings::first_landing();
    assert_eq!(world.landing(OWNER).landing.phase, landing::Phase::Landed);
    assert_eq!(world.check(), Ok(()));
    let inspection = world.inspection();
    let table = inspection.tables().iter().find(|table| table.name == "forge write entries").expect("write table");
    assert!(table.entries.iter().all(|entry| !entry.key.contains("task 61")), "{table:?}");
}
