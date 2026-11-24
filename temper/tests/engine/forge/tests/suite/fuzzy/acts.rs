//! People act on a seeded train (testing/workloads.md, 4.2, 6, 8.1).
use std::collections::{BTreeMap, BTreeSet};

use skein_fake_forge_domain::api as raw;
use skein_lib::{Duration, Wall};
use skein_world::{
    faults::{Kind, Struck},
    streams::{Stream, Streams},
    sweep::Sweep,
};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes, fake_config,
    landings::{self, Trains},
    open,
};
use temper_faults::people;
use temper_workload::parties::{Act, Action, When};

use super::super::landing_batches::heard;
use super::landing_batches::{self, BatchChecks, Workload};

const KINDS: [Kind; 4] =
    [people::CANCELS_A_LANDING, people::HOLDS_A_MEMBER, people::RELEASES_A_MEMBER, people::CANCELS_A_CHANGE];

struct Pending {
    action: Action,
    observed: Option<u64>,
    fired: bool,
}

const WAITER: u64 = 60;

struct Held {
    task: u64,
    landing: u64,
    ready: Option<Wall>,
    left: bool,
}

fn observe_holds(world: &World, held: &mut [Held]) {
    for hold in held {
        if let Some(top::Stored::Landing(row)) = world.stored().get(&top::Key::Landing(hold.landing)) {
            let member = row.landing.members.iter().find(|m| m.change == hold.task).expect("held member");
            if member.left == Some(landing::Leave::Held) {
                hold.left = true;
            }
        }
    }
}

fn add_waiter(world: &mut World) {
    let branch = b"temper/c60";
    world.external(
        raw::Op::Git(raw::Git::Create { branch: branch.as_slice().into(), commit: world.branch(b"main") }),
        2,
    );
    world.push_file(branch, b"waiter", b"later waiter");
    landings::ready_change(world, WAITER, branch);
    let mut row = world.change(WAITER).clone();
    row.change.ready_since = Some(world.wall());
    if let change::State::Ready { ready_since, .. } = &mut row.change.state {
        *ready_since = world.wall();
    }
    world.event(top::Event::Change { row });
}

fn task(action: &Action) -> u64 {
    51 + action.target.strip_prefix('m').expect("member name").parse::<u64>().expect("member index")
}

fn live(world: &World) -> Option<u64> {
    world.stored().iter().find_map(|(_, record)| match record {
        top::Stored::Landing(row)
            if !matches!(row.landing.phase, landing::Phase::Landed | landing::Phase::Cancelled) =>
        {
            Some(row.landing.task)
        }
        _ => None,
    })
}

fn moment(world: &World, action: &Action) -> bool {
    let change = world.change(task(action));
    match action.when {
        When::Opened => true, // change() above reads the durable row, not a driver flag.
        When::Landing => change.change.landing.is_some_and(|task| {
            let row = &world.landing(task).landing;
            !matches!(row.phase, landing::Phase::Landed | landing::Phase::Cancelled)
                && row
                    .members
                    .iter()
                    .any(|member| member.change == change.task && member.left.is_none() && member.landed.is_none())
        }),
        When::Held => matches!(change.change.state, change::State::Held { .. }),
        _ => panic!("not a train moment: {:?}", action.when),
    }
}

fn fire(world: &mut World, trains: &mut Trains, held: &mut Vec<Held>, action: &Action, step: u64) {
    let member = task(action);
    let ready = world.change(member).change.ready_since;
    let kind = match action.act {
        Act::CancelLanding => people::CANCELS_A_LANDING,
        Act::HoldMember => people::HOLDS_A_MEMBER,
        Act::ReleaseMember => people::RELEASES_A_MEMBER,
        Act::CancelChange => people::CANCELS_A_CHANGE,
        _ => panic!("not a train act: {:?}", action.act),
    };
    eprintln!("step {step}: {:?} {} after {:?}+{}", action.act, action.target, action.when, action.after);
    let instant = world.inject(kind, Struck { request: Some(action.target.clone()), ..Struck::default() });
    world.stop(instant);
    match action.act {
        Act::CancelLanding => {
            assert!(moment(world, action), "landing cancellation has a live member");
            {
                let task = world.change(member).change.landing.expect("live member");
                let members: Vec<_> = world
                    .landing(task)
                    .landing
                    .members
                    .iter()
                    .filter(|m| m.left.is_none() && m.landed.is_none())
                    .map(|m| (m.change, world.change(m.change).change.ready_since))
                    .collect();
                world.event(top::Event::SettleEffects { task, root: task, ending: top::ReleaseEnding::Cancelled });
                for (change, ready) in members {
                    assert_eq!(
                        world.change(change).change.landing,
                        None,
                        "cancelled landing returns its members to the line"
                    );
                    assert_eq!(world.change(change).change.ready_since, ready);
                }
            }
        }
        Act::HoldMember => {
            assert!(moment(world, action), "hold strikes a live member");
            held.push(Held {
                task: member,
                landing: world.change(member).change.landing.expect("live member"),
                ready,
                left: false,
            });
            world.event(top::Event::RelayRefused {
                task: member,
                questions: world.change(member).questions.clone(),
                words: b"person holds member".as_slice().into(),
            });
            trains.held.push((member, b"person holds member".as_slice().into()));
            assert!(matches!(world.change(member).change.state, change::State::Held { .. }));
            assert_eq!(world.change(member).change.ready_since, ready, "held member keeps its place");
            add_waiter(world);
        }
        Act::ReleaseMember => {
            let snapshot = held.iter().find(|snapshot| snapshot.task == member).expect("release follows hold");
            assert!(snapshot.left, "held member leaves its landing before release");
            assert_eq!(world.change(member).change.ready_since, snapshot.ready);
            assert!(moment(world, action), "release strikes a held member");
            world.event(top::Event::ReleaseChange { task: member, release: None });
            trains.held.retain(|(held, _)| *held != member);
        }
        Act::CancelChange => {
            cancel_change(world, member, 800_000 + step * 100);
        }
        _ => panic!("not a train act: {:?}", action.act),
    }
}

fn tick(world: &mut World, trains: &mut Trains, held: &mut Vec<Held>, actions: &mut [Pending], step: u64) {
    for pending in actions {
        if pending.fired {
            continue;
        }
        if pending.observed.is_none() && moment(world, &pending.action) {
            pending.observed = Some(step);
        }
        if pending.observed.is_some_and(|at| step == at + u64::from(pending.action.after)) {
            fire(world, trains, held, &pending.action, step);
            pending.fired = true;
        }
    }
}

fn cancel_change(world: &mut World, task: u64, entry: u64) {
    world.event(top::Event::StepChange {
        task,
        entry,
        heard: change::Heard {
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            delegate_running: false,
            flake: false,
            no_test: false,
            delegate_live: false,
            effect: change::EffectResult::None,
            released: false,
            cancelled: true,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
        merge_job: false,
        pending: Box::new([]),
    });
    world.run_for(1);
    world.event(top::Event::SettleEffects { task, root: task, ending: top::ReleaseEnding::Cancelled });
    world.event(top::Event::Release {
        task,
        root: task,
        ending: top::ReleaseEnding::Cancelled,
        reason: None,
        entry: entry + 1,
    });
    world.event(top::Event::Committed { entry: entry + 1 });
    for index in 2..20 {
        world.run_for(5);
        world.event(top::Event::ContinueRelease { task, entry: entry + index });
        world.event(top::Event::Committed { entry: entry + index });
    }
    assert_eq!(world.change(task).ending, Some(top::Ending::Cancelled));
}

fn advance(world: &mut World, task: u64, entry: u64) {
    let decision = changes::change_step_task(world, task, entry, change::Status::Passed, Box::new([]), false, false);
    if let change::Decision::Delegate(kind) = decision {
        let child = 100_000 + entry;
        world.event(top::Event::Delegated { task, child, kind });
        let branch = format!("temper/c{task}");
        match kind {
            change::Delegate::Repair(_) | change::Delegate::Resolve { .. } => {
                world.push_file(branch.as_bytes(), b"file", b"fixed");
                world.push_file(branch.as_bytes(), format!("member{}", task - 51).as_bytes(), b"resolved");
            }
            _ => panic!("train member's delegate: {kind:?}"),
        }
        world.event(top::Event::DelegateResult {
            ended_at: None,
            task,
            child,
            status: change::Status::Passed,
            no_change: false,
            flake: false,
            no_test: false,
            failed: false,
            cancelled: false,
            words: b"done".as_slice().into(),
        });
    }
}

fn close(world: &mut World, task: u64, cancelled: bool) {
    let row = world.landing(task).landing.clone();
    let ending = if cancelled { top::ReleaseEnding::Cancelled } else { top::ReleaseEnding::Done };
    world.event(top::Event::SettleEffects { task, root: task, ending });
    world.event(top::Event::Release { task, root: task, ending, reason: None, entry: 900_000 + task * 100 });
    world.event(top::Event::Committed { entry: 900_000 + task * 100 });
    world.run_for(5);
    let mut released = false;
    for index in 1..20 {
        let entry = 900_000 + task * 100 + index;
        world.event(top::Event::ContinueRelease { task, entry });
        world.event(top::Event::Committed { entry });
        world.run_for(5);
        if world
            .take_seen()
            .iter()
            .any(|request| matches!(request, top::Request::Released { task: ended } if *ended == task))
        {
            released = true;
            break;
        }
    }
    assert!(released, "landing {task} releases");
    assert!(world.landing(task).entries.is_empty(), "cancelled or finished landing returns its write entries");
    assert!(
        !world.stored().iter().any(|(key, record)| {
            matches!(key, top::Key::Flight { task: owner, .. } if *owner == task)
                || matches!(record, top::Stored::Entry { entry, .. } if entry.task == task)
        }),
        "no write of the closed landing keeps outbox room"
    );
    for member in &row.members {
        assert!(!world.has_branch(&member.branch), "prefix released");
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one bounded train driver interleaves acts, repairs and the shared batch checks"
)]
fn run(seed: u64) -> World {
    let streams = Streams::new(seed);
    let workload_seed = streams.seed_of(Stream::Workload, "landing_batches");
    let Workload { mut rng, count, squash, red, conflict, moving, red_member } = Workload::new(workload_seed);
    let mut actions: Vec<_> = super::super::acts::draw(streams, count)
        .into_iter()
        .map(|action| Pending { action, observed: None, fired: false })
        .collect();
    let mut workflow = landing_batches::workload_workflow(workload_seed);
    workflow.latency_min = Duration::from_secs(300);
    workflow.latency_max = workflow.latency_min;
    let world = World::batch_turn_in(&streams, Some(workflow), fake_config());
    let mut world = landing_batches::setup(world, count, red, conflict, red_member);
    let mut trains = Trains::default();
    trains.judge(&mut world);
    let mut checks = BTreeMap::<u64, BatchChecks>::new();
    let mut closed = BTreeSet::new();
    let mut held = Vec::new();
    let mut landed = Vec::new();
    let mut members: Vec<_> = (51..51 + count).collect();
    if actions.iter().any(|pending| pending.action.act == Act::HoldMember) {
        members.push(WAITER);
    }
    let mut pending = None;
    let mut reported = false;
    let mut moving = moving;
    let restart = rng.below(4);
    let mut settled = false;
    for step in 0..600 {
        if step == restart {
            let before: Vec<_> = (51..51 + count).map(|task| world.history(task)).collect();
            world.restart();
            assert_eq!(before, (51..51 + count).map(|task| world.history(task)).collect::<Vec<_>>());
        }
        observe_holds(&world, &mut held);
        tick(&mut world, &mut trains, &mut held, &mut actions, step);
        // Cancellation removes the members' landing links immediately, but the old row's writes must drain.
        let cancelled: Vec<_> = world
            .stored()
            .iter()
            .filter_map(|(_, record)| match record {
                top::Stored::Landing(row)
                    if row.landing.phase == landing::Phase::Cancelled && !closed.contains(&row.landing.task) =>
                {
                    Some(row.landing.task)
                }
                _ => None,
            })
            .collect();
        for task in cancelled {
            close(&mut world, task, true);
            closed.insert(task);
            pending = None;
        }
        if let Some(task) = live(&world) {
            let prior = world.landing(task).landing.clone();
            let before = world.branch(b"main");
            let mut news = pending.take().unwrap_or_else(|| heard(landing::Preparation::Idle, b""));
            news.held = trains.held.iter().map(|(task, _)| *task).collect();
            let decision = landings::step_landing_heard(&mut world, task, 10_000 + step * 10, news);
            checks.entry(task).or_default().step(&mut world, seed, task, &prior, before, decision);
            if matches!(decision, landing::Decision::Prepare { .. }) {
                let row = &world.landing(task).landing;
                let selected: Vec<_> = row
                    .members
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| reported || conflict != Some(m.change - 51))
                    .map(|(i, _)| i)
                    .collect();
                let conflict_member = row.members.iter().position(|m| conflict == Some(m.change - 51));
                let len = row.members.len();
                world.prepare_subset(task, squash, &selected);
                if !reported && let Some(member) = conflict_member {
                    let result = format!(
                        r#"{{"conflicts":[{{"member":{},"with":"member","files":["member0"],"cut":false}}]}}"#,
                        member + 1
                    );
                    let mut news = heard(landing::Preparation::Idle, result.as_bytes());
                    news.outcomes = top::preparation::outcomes(result.as_bytes(), u32::try_from(len).expect("members"))
                        .expect("conflict");
                    pending = Some(news);
                    reported = true;
                }
            }
            if world.landing(task).landing.phase == landing::Phase::Proving
                && let Some(index) = moving.take()
            {
                world.push_file(format!("temper/c{}", 51 + index).as_bytes(), b"moved", b"new delivery");
            }
            observe_holds(&world, &mut held);
            if decision == landing::Decision::Finish {
                let row = world.landing(task).landing.clone();
                landing_batches::check_members(&world, seed, &row);
                landed.extend(row.members.iter().filter(|m| m.landed.is_some()).map(|m| m.change));
                for member in &row.members {
                    if trains.held.iter().any(|(held, _)| *held == member.change) {
                        assert_eq!(member.left, Some(landing::Leave::Held), "held member leaves its landing");
                    }
                }
                close(&mut world, task, false);
                closed.insert(task);
            }
        }
        for &task in &members {
            if task == WAITER && held.is_empty() {
                continue;
            }
            if world.change(task).ending == Some(top::Ending::Cancelled)
                || trains.held.iter().any(|(held, _)| *held == task)
            {
                continue;
            }
            if world.change(task).change.landing.is_none() {
                advance(&mut world, task, 20_000 + step * 10 + task - 51);
            }
        }
        trains.judge(&mut world);
        // Once every act has fired, only the finite forge work remains; let its timers run.
        world.run_for(if actions.iter().all(|action| action.fired) { 10 } else { 1 });
        world.check().expect("world invariants");
        settled = actions.iter().all(|action| action.fired)
            && live(&world).is_none()
            && members.iter().all(|&task| {
                world.change(task).ending == Some(top::Ending::Cancelled)
                    || trains.held.iter().any(|(held, _)| *held == task)
                    || matches!(world.change(task).change.state, change::State::Landed { .. })
            });
        if settled {
            break;
        }
    }
    assert!(
        settled,
        "seed {seed}: train settles; actions {:?}; members {:?}; landing {:?}",
        actions.iter().map(|a| (&a.action, a.fired)).collect::<Vec<_>>(),
        (51..51 + count).map(|task| (task, &world.change(task).change.state)).collect::<Vec<_>>(),
        live(&world).map(|task| &world.landing(task).landing)
    );
    for hold in &held {
        assert!(hold.left, "held member left its landing");
    }
    for pending in &actions {
        if pending.action.act == Act::ReleaseMember {
            assert!(
                landed.iter().position(|task| *task == self::task(&pending.action)).expect("released member landed")
                    < landed.iter().position(|task| *task == WAITER).expect("later waiter landed"),
                "released member stays ahead of the later waiter"
            );
            assert!(
                matches!(world.change(task(&pending.action)).change.state, change::State::Landed { .. }),
                "released member lands"
            );
        }
        if pending.action.act == Act::CancelChange {
            let cancelled = task(&pending.action);
            assert_eq!(
                world.change(cancelled).ending,
                Some(top::Ending::Cancelled),
                "cancelled change leaves the train"
            );
            assert!(world.change(cancelled).change.landing.is_none());
            assert!(
                !world
                    .stored()
                    .iter()
                    .any(|(_, record)| matches!(record, top::Stored::Landed { task, .. } if *task == cancelled)),
                "cancelled member did not land"
            );
        }
    }
    for task in members {
        super::history::check(seed, task, &world.history(task));
    }
    world
}

#[test]
fn people_act_on_the_landing_train() {
    let mut sweep = Sweep::new("people_act_on_the_landing_train");
    for kind in KINDS {
        sweep.configure(kind);
    }
    open::exclude(&mut sweep, &KINDS).expect("engine exclusions");
    // Seed 1 has the hold/release and a change cancellation; seed 2 first cancels a landing.
    for seed in 1..=2 {
        eprintln!("seed {seed}");
        let world = run(seed);
        sweep.ran(world.ledger());
    }
    sweep.check_fell().expect("all four people's acts fell");
}
