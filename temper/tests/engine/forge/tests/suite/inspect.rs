//! Restored outbox and ordinary-round checks (temper/docs/design/testing/engine-checks.md, 2 and 6).
use skein_forge_client::{Condition, Effect, Entry};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_world::inspect::{Inspection, Moment, Release, Table, Wake, check_tables, check_waits};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_issues as issues;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{LIMITS, REPO, World, changes, landings};

const OWNER: u64 = landings::LANDING;
const WAITER: u64 = 70;

const CENSUS_MOMENT: Moment = Moment { seed: 307, at: Time::ZERO, iteration: 0 };
const CALL: Token = Token::new(101);
const CALL_TASK: u64 = 71;
const PROJECTION: u64 = 102;
const GOAL: u64 = 72;
const ASK_KEY: &str = "asked change (task 70)";
const CALL_KEY: &str = "agent call 101 (task 71)";
const PROJECTION_KEY: &str = "goal projection 102 (task 72)";

struct CensusFixture {
    forge: top::Domain,
    env: Env<top::Limits>,
}

impl CensusFixture {
    fn new() -> Self {
        let mut world = World::new(CENSUS_MOMENT.seed);
        world.adopt();
        let repository = world.stored().get(&top::Key::Repository(REPO)).expect("adopted repository").clone();
        let mut limits = LIMITS;
        limits.entries = 4;
        limits.client.entries = 4;
        let env = Env { limits, now: CENSUS_MOMENT.at, wall: Wall::EPOCH };
        let mut forge = top::Domain::new(
            &limits,
            CENSUS_MOMENT.seed,
            top::Config { push: None, namespace: b"census".as_slice().into(), writers: Box::default() },
            &[],
            &[],
            false,
        )
        .expect("fixture bounds");
        forge.restore_begin(env.wall);
        let mut fixture = Self { forge, env };
        fixture.step(top::Event::Restore { record: repository });
        let mut waiter = changes::change_row();
        waiter.task = WAITER;
        waiter.change.task = WAITER;
        waiter.change.state = change::State::Opening { requested: false };
        waiter.room = Some(1);
        fixture.step(top::Event::Restore { record: top::Stored::Change(Box::new(waiter)) });
        for number in 1..=4 {
            fixture.step(top::Event::Restore { record: super::goal_branch_restart::held_dispatch(number) });
        }
        fixture.forge.fit_restored(&limits).expect("restored fixture fits");
        fixture.step(top::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        fixture.step(top::Event::CatchUp);
        for entry in 1..=3 {
            fixture.step(top::Event::Resolve { entry, again: false });
        }
        assert!(fixture.forge.room_asked(WAITER), "the oldest waiter got the first place freed");
        assert!(fixture.forge.reserve_call(&fixture.env, CALL, CALL_TASK));
        let live = projection(PROJECTION, false);
        let out = fixture.step(live);
        assert!(out.iter().any(|request| matches!(request, top::Request::ProjectionEffect { .. })));
        fixture
    }

    fn step(&mut self, event: top::Event) -> Queue<top::Request> {
        let mut out = Queue::uncapped();
        top::step(&mut self.forge, &self.env, event, &mut out);
        out
    }

    fn inspect(&self) -> Inspection {
        let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
        temper_engine_inspect::forge(&self.forge, &mut inspection);
        inspection
    }
}

fn projection(entry: u64, ended: bool) -> top::Event {
    top::Event::Project {
        entry,
        repository: REPO,
        view: issues::GoalView {
            phase: if ended { issues::Phase::Done } else { issues::Phase::Active },
            goal: GOAL,
            repository: u64::from(REPO.repository),
            title: Box::from("Goal"),
            goal_text: Box::from("Plan"),
            plan: Box::new([]),
            milestones: Box::new([]),
            finished: ended.then(|| Box::from("Done")),
        },
    }
}

fn writes(inspection: &Inspection) -> &Table {
    inspection.tables().iter().find(|table| table.name == "forge write entries").expect("write table")
}

#[test]
fn the_forge_room_counts_its_entry_and_each_live_reservation() {
    let mut fixture = CensusFixture::new();
    let inspection = fixture.inspect();
    let table = writes(&inspection);
    let outbox = format!("entry 4 (task {})", fixture.forge.inspection_entry(4).expect("outbox entry").task);
    let keys: Vec<_> = table.entries.iter().map(|entry| entry.key.as_str()).collect();
    assert_eq!(keys, [outbox.as_str(), ASK_KEY, CALL_KEY, PROJECTION_KEY]);
    assert!(table.entries.iter().all(|entry| entry.release == Release::Pending));
    assert_eq!(fixture.forge.inspection_room_taken(), 4);
    assert_eq!(inspection.check_violation(CENSUS_MOMENT), Ok(()));
    assert_eq!(check_tables(&[&inspection], CENSUS_MOMENT), Ok(()));

    fixture.step(top::Event::ChangeEnding { task: WAITER, ending: top::Ending::Cancelled });
    let after = fixture.inspect();
    assert!(!writes(&after).entries.iter().any(|entry| entry.key == ASK_KEY), "ending gives back the ask in its step");
    assert_eq!(fixture.forge.inspection_room_taken(), 3);
    assert_eq!(after.check_violation(CENSUS_MOMENT), Ok(()));
    assert_eq!(check_tables(&[&after], CENSUS_MOMENT), Ok(()));
}

#[test]
fn a_call_reservation_still_held_after_its_owner_ends_is_due() {
    let mut fixture = CensusFixture::new();
    fixture.step(top::Event::ChangeEnding { task: CALL_TASK, ending: top::Ending::Cancelled });
    let inspection = fixture.inspect();
    let finding = check_tables(&[&inspection], CENSUS_MOMENT).expect_err("ended call's reservation is due");
    assert!(finding.contains(&format!("forge write entries {CALL_KEY}")), "{finding}");
    assert!(finding.contains("the call's entry or the end of its run"), "{finding}");
    fixture.forge.unreserve_call(&fixture.env, CALL, CALL_TASK, &mut Queue::uncapped());
    assert_eq!(check_tables(&[&fixture.inspect()], CENSUS_MOMENT), Ok(()));
}

#[test]
fn a_projection_reservation_still_held_after_its_goal_ends_is_due() {
    let mut fixture = CensusFixture::new();
    fixture.step(projection(103, true));
    let inspection = fixture.inspect();
    let finding = check_tables(&[&inspection], CENSUS_MOMENT).expect_err("ended goal's reservation is due");
    assert!(finding.contains(&format!("forge write entries {PROJECTION_KEY}")), "{finding}");
    assert!(finding.contains("the projection's entry or the end of its goal"), "{finding}");
    fixture.step(top::Event::DropProjection { entry: PROJECTION, goal: GOAL });
    assert_eq!(check_tables(&[&fixture.inspect()], CENSUS_MOMENT), Ok(()));
}

#[test]
fn a_forge_room_census_mismatch_names_both_counts_and_the_holders_by_kind() {
    let fixture = CensusFixture::new();
    let snapshot = fixture.inspect();
    let mut report = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::check_room_census(writes(&snapshot), 5, &mut report);
    let finding = report.check_violation(CENSUS_MOMENT).expect_err("one charge has no holder");
    assert!(finding.contains("engine forge room: census 4, taken 5"), "{finding}");
    for (kind, key) in [("asked waiters", ASK_KEY), ("agent calls", CALL_KEY), ("goal projections", PROJECTION_KEY)] {
        assert!(finding.contains(&format!("{kind}: [{key}]")), "{finding}");
    }
    assert!(finding.contains("outbox entries: [entry 4 (task "), "{finding}");
}

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
    let waiter = inspection.waits().iter().find(|wait| wait.waiter == "task 70").expect("room waiter");
    let due = world.now().saturating_add(LIMITS.waits.room);
    assert_eq!(waiter.bound, Some(due));
    for number in [1, 2] {
        assert!(waiter.wakes.contains(&Wake::Ended(format!("entry {number} of task {OWNER}"))));
    }
    assert!(waiter.wakes.contains(&Wake::Timer(due)));
    assert_eq!(check_waits(&[&inspection], moment(&world)), Ok(()));
    // Advance the referee, not the connector, to pin what it finds if the
    // waiter is still unheld after the timer should have held it.
    let late = Moment { at: due.saturating_add(skein_lib::Duration::from_secs(1)), ..moment(&world) };
    let waits = check_waits(&[&inspection], late).expect_err("unheld past its room bound");
    assert!(
        waits.contains(&format!("engine task {WAITER}: waits on change: room past bound {} ns", due.as_nanos())),
        "{waits}"
    );
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
            closer_waiting: false,
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
    world.event(top::Event::Names { task: 51, resources: Box::new([changes::branch_name(b"main")]) });
    world.run_for(1);
    // The first find still sees the open pull. The admitted copy lands
    // before the change's fresh reads, while the client still owes a retry.
    world.land_writes_late(skein_lib::Duration::from_millis(500));
    world.event(top::Event::Enqueue {
        entry: Entry {
            number: ENTRY,
            task: 51,
            repository: REPO,
            effect: Effect {
                write: skein_forge::api::Write::Merge {
                    number: pull,
                    head,
                    how: skein_forge::api::MergeHow::FastForwardOnly,
                },
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
    let merge = temper_engine_forge_world::translate::commit(world.branch(b"main"));
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
