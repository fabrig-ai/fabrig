//! A person's cancellation redispatches proof without repairing the member (domain/landing-proof.md, 2.4, and domain/landing.md, 15–16).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes,
    landings::{self, CHANGE, LANDING, Trains},
    translate,
};

const RETRIES: u32 = 1;

fn start() -> World {
    start_with_room(None).0
}

fn start_with_room(bound: Option<u32>) -> (World, Trains) {
    let mut workflow = super::landing_batches::workflow(false);
    workflow.latency_min = Duration::from_secs(300);
    workflow.latency_max = workflow.latency_min;
    let mut world = World::batch_turn(31, Some(workflow));
    if let Some(landings) = bound {
        world.limit_entries(1);
        world.limit_landings(landings);
    }
    let mut world = landings::turn_in(world);
    world.gate_policy(landing::Project { gate_retries: RETRIES, ..landing::Project::DEFAULT });
    world.produce(b"temper/51");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    world.push_as(b"main", b"base", b"another hand moved main", 2);
    let mut trains = Trains::default();
    trains.judge(&mut world);
    (world, trains)
}
fn step(world: &mut World, entry: u64) -> landing::Decision {
    let decision = landings::step_landing(world, LANDING, entry);
    changes::change_step_task(world, CHANGE, entry + 10000, change::Status::Passed, Box::new([]), false, false);
    decision
}
fn until_dispatch(world: &mut World, first: u64) {
    for entry in first..first + 60 {
        let decision = step(world, entry);
        assert!(!matches!(decision, landing::Decision::Hold(_)), "held before retry bound: {decision:?}");
        if matches!(decision, landing::Decision::Dispatch { .. } | landing::Decision::Retry { .. }) {
            return;
        }
    }
    panic!("no dispatch: {:?}", world.landing(LANDING));
}
const WAITER: u64 = 52;
const WAITING_ENTRY: u64 = 400;

fn dispatch_in_flight(world: &mut World) -> u64 {
    for entry in 100..160 {
        let heard = top::LandingHeard {
            job: None,
            outcomes: Box::new([]),
            dependencies: Box::new([]),
            held: Box::new([]),
            failure: Box::new([]),
            preparation: Some(landing::Preparation::Idle),
            released: false,
            cancelled: false,
            kept: Box::new([]),
        };
        let step = landings::read_landing_step(world, LANDING, entry, heard);
        if matches!(step.decision, landing::Decision::Dispatch { .. }) {
            assert_eq!(step.entry, Some(entry));
            world.slow_dispatches(Duration::from_secs(5));
            world.event(top::Event::Committed { entry });
            world.run_for(1);
            assert!(world.stored().contains_key(&top::Key::Flight { task: LANDING, entry }));
            assert!(world.take_calls().iter().any(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))));
            return entry;
        }
        if let Some(entry) = step.entry {
            world.event(top::Event::Committed { entry });
            world.run_for(1);
        }
        if matches!(step.decision, landing::Decision::Prepare { .. }) {
            world.prepare_landing(LANDING, false);
        }
        changes::change_step_task(world, CHANGE, entry + 10000, change::Status::Passed, Box::new([]), false, false);
    }
    panic!("no dispatch: {:?}", world.landing(LANDING));
}

fn open_waiter(world: &mut World, entry: u64) {
    world.event(top::Event::StepChange {
        task: WAITER,
        entry,
        heard: change::Heard {
            review_request: None,
            closer_waiting: false,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Passed,
            delegate_running: false,
            flake: false,
            no_test: false,
            delegate_live: false,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: false,
        merge_job: false,
    });
    world.run_for(1);
}

fn wait_for_room(world: &mut World) {
    let mut row = changes::change_row();
    row.task = WAITER;
    row.change.task = WAITER;
    row.branch = Box::from(b"temper/52".as_slice());
    row.change.state = change::State::Opening { requested: false };
    world.event(top::Event::Change { row });
    open_waiter(world, WAITING_ENTRY);
    let panel = world.change_panel(WAITER, landings::PROJECT);
    assert_eq!(panel.expect("opening change").wait, Some(top::PanelWait::OutboxRoom));
    assert!(!world.stored().contains_key(&top::Key::Entry(WAITING_ENTRY)));
    world.take_seen();
}

fn frees_room_for_waiter(world: &mut World, dispatch: u64) -> Box<[top::Request]> {
    world.run_for(5);
    let seen = world.take_seen();
    assert!(seen.iter().any(|request| matches!(request, top::Request::Outcome {
        task: LANDING, entry, outcome: skein_forge_client::Outcome::Made { .. }, ..
    } if *entry == dispatch)));
    assert!(!world.stored().contains_key(&top::Key::Entry(dispatch)), "made dispatch must free its room");
    assert!(!world.stored().contains_key(&top::Key::Flight { task: LANDING, entry: dispatch }));
    assert!(seen.iter().any(|request| matches!(request, top::Request::WakeProcedure { task: WAITER })));
    open_waiter(world, WAITING_ENTRY + 1);
    let panel = world.change_panel(WAITER, landings::PROJECT);
    assert_eq!(panel.expect("asked change").wait, None);
    assert!(matches!(world.stored().get(&top::Key::Entry(WAITING_ENTRY + 1)),
        Some(top::Stored::Entry { entry, held: false }) if entry.task == WAITER));
    world.event(top::Event::Committed { entry: WAITING_ENTRY + 1 });
    world.run_for(1);
    seen
}

#[test]
fn cancelled_landing_frees_its_write_entries() {
    let (mut world, mut trains) = start_with_room(Some(1));
    world.produce(b"temper/52");
    let dispatch = dispatch_in_flight(&mut world);
    wait_for_room(&mut world);
    world.event(top::Event::SettleEffects { task: LANDING, root: LANDING, ending: top::ReleaseEnding::Cancelled });
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Cancelled);
    assert!(world.stored().contains_key(&top::Key::Entry(dispatch)), "do not erase a sent write at cancel");
    assert!(!world.take_seen().iter().any(|request| matches!(request, top::Request::EffectsSettled { task: LANDING })));
    trains.judge(&mut world);
    assert_eq!(world.landing_of(CHANGE), None, "the unsettled write still holds the row's place");
    let seen = frees_room_for_waiter(&mut world, dispatch);
    assert!(seen.iter().any(|request| matches!(request, top::Request::ContinueRelease { task: LANDING })));
    world.event(top::Event::ContinueRelease { task: LANDING, entry: 0 });
    assert!(world.take_seen().iter().any(|request| matches!(request, top::Request::EffectsSettled { task: LANDING })));
    assert!(world.landing(LANDING).entries.is_empty());
    assert_eq!(world.landing(LANDING).follows(), []);
    trains.judge(&mut world);
    assert_eq!(world.landing_of(CHANGE), Some(LANDING + 1));
    assert!(!world.stored().contains_key(&top::Key::Landing(LANDING)));
}

#[test]
fn a_waiting_train_names_the_live_and_settling_landings_taking_its_room() {
    use jig_domain_core_vocabulary::waits::{ReportedBound, Unbounded, WaitKind, WaitObject};
    use temper_client_protocol_wire::waits as code;
    use temper_engine_forge_world::REPO;
    const OTHER: u64 = LANDING + 1;
    const GOAL: &[u8] = b"goal";
    const MEMBER: &[u8] = b"temper/53";
    let (mut world, mut trains) = start_with_room(Some(2));
    world.external(raw::Op::Git(raw::Git::Create { branch: GOAL.into(), commit: world.branch(b"main") }), 2);
    world.produce_on(MEMBER, GOAL);
    landings::ready_change_into(&mut world, 53, MEMBER, GOAL);
    trains.judge(&mut world);
    assert_eq!(world.landing_of(53), Some(OTHER));

    let dispatch = dispatch_in_flight(&mut world);
    world.event(top::Event::SettleEffects { task: LANDING, root: LANDING, ending: top::ReleaseEnding::Cancelled });
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Cancelled);
    assert!(world.stored().contains_key(&top::Key::Entry(dispatch)));
    world.take_seen();
    world.observe_waits();
    trains.judge(&mut world);
    assert!(world.landing_of(CHANGE).is_none());
    assert!(!world.stored().contains_key(&top::Key::Landing(OTHER + 1)), "no make was asked for");
    let Some(top::Stored::Repository(repository)) = world.stored().get(&top::Key::Repository(REPO)) else {
        panic!("stored repository")
    };
    assert_eq!(repository.refusals.len(), 1);
    assert_eq!(repository.refusals[0].branch.as_ref(), b"main");
    assert_eq!(repository.refusals[0].cause, top::FormingRefusal::Landings);
    let seen = world.take_seen();
    let waits = seen
        .iter()
        .find_map(|request| match request {
            top::Request::Waits { task: CHANGE, waits } => Some(waits),
            _ => None,
        })
        .expect("change's replacement wait");
    let [wait] = waits.as_slice() else { panic!("one ready wait") };
    assert_eq!(wait.kind, WaitKind::Room);
    assert_eq!(wait.bound, ReportedBound::None(Unbounded::Behind(OTHER)));
    let WaitObject::Procedure { step, things } = &wait.object else { panic!("procedure wait") };
    assert_eq!(*step, code::CHANGE_READY);
    assert_eq!(
        things.iter().map(|thing| (thing.what, thing.number)).collect::<Vec<_>>(),
        [(code::THING_LANDINGS, 2), (code::THING_TASK, LANDING), (code::THING_TASK, OTHER)]
    );

    let inspection = world.inspection();
    let wait = inspection
        .waits()
        .iter()
        .find(|wait| wait.waiter == format!("task {CHANGE}"))
        .expect("ready change's inspected wait");
    assert_eq!(
        wait.wakes,
        [
            skein_world::inspect::Wake::Party(format!("forge: entry {dispatch}")),
            skein_world::inspect::Wake::Waiter(format!("task {OTHER}")),
        ]
    );

    world.run_for(5);
    assert!(!world.stored().contains_key(&top::Key::Entry(dispatch)), "the write settled");
    trains.judge(&mut world);
    assert_eq!(world.landing_of(CHANGE), Some(OTHER + 1), "no make consumed a number before room yielded");
    assert!(!world.stored().contains_key(&top::Key::Landing(LANDING)));
    let Some(top::Stored::Repository(repository)) = world.stored().get(&top::Key::Repository(REPO)) else {
        panic!("stored repository")
    };
    assert!(repository.refusals.is_empty());
}

#[test]
fn a_landed_landing_frees_its_write_entries() {
    let (mut world, mut trains) = start_with_room(Some(1));
    world.produce(b"temper/52");
    let dispatch = dispatch_in_flight(&mut world);
    wait_for_room(&mut world);
    frees_room_for_waiter(&mut world, dispatch);
    assert_eq!(runs(&world)[0].state, raw::RunState::Running);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Proving);
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    assert_eq!(runs(&world)[0].conclusion, Some(raw::Conclusion::Success));
    world.event(top::Event::SettleEffects { task: LANDING, root: LANDING, ending: top::ReleaseEnding::Done });
    assert!(world.take_seen().iter().any(|request| matches!(request, top::Request::EffectsSettled { task: LANDING })));
    world.produce(b"temper/53");
    landings::ready_change(&mut world, 53, b"temper/53");
    trains.judge(&mut world);
    assert_eq!(world.landing_of(53), Some(LANDING + 1));
    assert!(!world.stored().contains_key(&top::Key::Landing(LANDING)));
}

fn runs(world: &World) -> Box<[raw::Run]> {
    let raw::Answer::Runs(runs) = world.inspect(&raw::Read::Runs {
        workflow: Some(b"gate.yml".as_slice().into()),
        commit: None,
        number: None,
        page: 1,
        limit: 16,
    }) else {
        panic!("workflow runs")
    };
    runs
}
fn cancel(world: &mut World) -> raw::Run {
    let run = runs(world)[0].clone();
    assert_eq!(run.state, raw::RunState::Running, "cancel while in flight");
    assert_eq!(world.external(raw::Op::Write(raw::Write::Cancel { run: run.id }), 2), raw::Answer::Done);
    let raw::Answer::Run(cancelled) = world.inspect(&raw::Read::Run { run: run.id }) else { panic!("run") };
    assert_eq!(cancelled.conclusion, Some(raw::Conclusion::Cancelled));
    assert_eq!(cancelled.cancelled_by, Some(2));
    run
}
fn no_receipt_or_repair(world: &World) {
    assert!(world.stored().iter().all(|(_, row)| !matches!(row, top::Stored::Receipt { .. })));
    assert_eq!(world.change(CHANGE).change.repairs, 0);
    assert!(!matches!(world.change(CHANGE).change.state, change::State::Repairing { .. }));
}
fn dispatches_without_reruns(calls: &[api::Op]) -> Vec<&api::Write> {
    assert!(!calls.iter().any(|op| matches!(op, api::Op::Write(api::Write::Rerun { .. }))));
    calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(write @ api::Write::Dispatch { .. }) => Some(write),
            _ => None,
        })
        .collect()
}

#[test]
fn a_persons_cancellation_dispatches_afresh_and_restart_finds_that_run() {
    let mut world = start();
    let moved_main = world.branch(b"main");
    let member_head = world.branch(b"temper/51");
    until_dispatch(&mut world, 100);
    let first = cancel(&mut world);
    until_dispatch(&mut world, 200);
    let second = runs(&world)[0].clone();
    assert_ne!(second.id, first.id);
    assert_eq!(second.state, raw::RunState::Running);
    assert_eq!(second.commit, first.commit);
    assert_eq!(second.inputs, first.inputs, "temper_commit and temper_key stay fixed");
    no_receipt_or_repair(&world);
    let candidate = world.landing(LANDING).landing.members[0].candidate.expect("candidate");
    assert_eq!(translate::commit(second.commit), candidate);
    let before_restart = world.take_calls();
    let dispatches = dispatches_without_reruns(&before_restart);
    assert_eq!(dispatches.len(), 2);
    assert_eq!(dispatches[0], dispatches[1]);

    world.restart();
    world.run_for(1);
    for entry in 300..305 {
        step(&mut world, entry);
    }
    assert_eq!(runs(&world)[0].state, raw::RunState::Running);
    let relearned = world.take_calls();
    assert!(relearned.iter().any(|op| matches!(op, api::Op::Read(api::Read::Run { run }) if *run == second.id)));
    assert_eq!(dispatches_without_reruns(&relearned), Vec::<&api::Write>::new());
    no_receipt_or_repair(&world);
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(dispatches_without_reruns(&world.take_calls()), Vec::<&api::Write>::new());
    assert_eq!(runs(&world).len(), 2);
    assert_eq!(world.branch(b"main"), second.commit);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    let row = world.change(CHANGE);
    assert!(matches!(row.change.state, change::State::Landed { merge, .. } if merge == candidate));
    assert_eq!(row.change.repairs, 0);
    let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: row.pull.expect("member pull") }) else {
        panic!("member pull")
    };
    assert_eq!(pull.merged, Some(second.commit));
    assert_eq!(pull.commit, member_head);
    assert_eq!(world.branch(b"temper/51"), member_head);
    let raw::Answer::Commits { commits, .. } =
        world.inspect(&raw::Read::Commits { before: moved_main, after: second.commit })
    else {
        panic!("candidate ancestry")
    };
    let merge = commits.iter().find(|commit| commit.id == second.commit).expect("candidate merge");
    assert_eq!(merge.parents.as_ref(), &[moved_main, member_head]);
    let raw::Answer::Statuses(statuses) = world.inspect(&raw::Read::Statuses { commit: second.commit }) else {
        panic!("gate status")
    };
    let gate = statuses.iter().find(|status| status.context.as_ref() == b"temper/gate/gate").expect("gate status");
    assert_eq!(gate.state, raw::Check::Passed);
    let receipts: Vec<_> = world
        .stored()
        .iter()
        .filter_map(|(_, row)| match row {
            top::Stored::Receipt { receipt, .. } => Some(receipt),
            _ => None,
        })
        .collect();
    assert_eq!(receipts.len(), 1);
    assert!(receipts[0].passed);
    assert_eq!(receipts[0].candidate, candidate);
    assert_eq!(receipts[0].run.id, second.id, "only the replacement run proves the key");
}

#[test]
fn cancellations_past_the_gate_retry_bound_hold_without_repair() {
    let mut world = start();
    until_dispatch(&mut world, 100);
    for retry in 0..=RETRIES {
        cancel(&mut world);
        if retry < RETRIES {
            until_dispatch(&mut world, 200 + u64::from(retry) * 100);
        }
    }
    let mut held = false;
    for entry in 500..560 {
        if matches!(step(&mut world, entry), landing::Decision::Hold(landing::Hold::Faults { member: 0, gate: 0 })) {
            held = true;
            break;
        }
    }
    assert!(held, "cancellations past gate_retries hold the landing");
    assert!(matches!(
        world.landing(LANDING).landing.phase,
        landing::Phase::Held { why: landing::Hold::Faults { member: 0, gate: 0 }, .. }
    ));
    let words = world.landing_hold_words(LANDING).expect("hold words");
    let words = String::from_utf8_lossy(&words);
    assert!(words.contains("gate gate") && words.contains("runner") && words.contains("retries"), "{words}");
    no_receipt_or_repair(&world);
    assert_eq!(runs(&world).len(), usize::try_from(RETRIES + 1).expect("tiny retry bound"));
    assert_eq!(
        dispatches_without_reruns(&world.take_calls()).len(),
        usize::try_from(RETRIES + 1).expect("tiny retry bound")
    );
}
