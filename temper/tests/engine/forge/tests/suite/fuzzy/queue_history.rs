//! Seeded queue steps and restarts (domain/landing-history.md, 8).
use std::convert::Infallible;

use skein_fake_forge_domain::api as raw;
use skein_lib::Rng;
use skein_world::sweep::{cases, gate};
use temper_engine_domain_forge::{
    self as top,
    history::{Ending, Event, What},
};
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{World, changes};

const DRAW_STEPS: u64 = 80;
const DRAIN_ROUNDS: u64 = 80;

#[derive(Debug, Default, PartialEq, Eq)]
struct Skipped {
    tasks: u64,
    commits: u64,
}

struct Draws {
    tasks_left: u32,
    skipped: Skipped,
}

impl Draws {
    fn new(changes: u64) -> Self {
        // One task per change to start. Every scripted child stays in the table
        // after its result and across restarts; nothing else adds tasks here.
        let tasks_left = jig_tasks_kit::LIMITS.tasks - u32::try_from(changes).expect("small queue");
        Self { tasks_left, skipped: Skipped::default() }
    }

    fn admits_commit(&mut self, world: &mut World) -> bool {
        if world.forge_mut().room().commits == 0 {
            self.skipped.commits += 1;
            return false;
        }
        true
    }

    fn admits_delegate(&mut self, world: &mut World) -> bool {
        if self.tasks_left == 0 {
            self.skipped.tasks += 1;
            return false;
        }
        if !self.admits_commit(world) {
            return false;
        }
        self.tasks_left -= 1;
        true
    }

    fn push(&mut self, world: &mut World, branch: &[u8], step: u64, checked: bool) {
        if self.admits_commit(world) {
            world.push_file(branch, b"file", &step.to_be_bytes());
            if checked {
                world.status(branch, raw::Check::Passed);
            }
        }
    }
}

fn advance(world: &mut World, draws: &mut Draws, task: u64, entry: u64) -> bool {
    let branch = format!("temper/{task}");
    let decision = changes::change_step_task(world, task, entry, change::Status::Passed, Box::new([]), false, false);
    if let change::Decision::Delegate(kind) = decision {
        let work: fn(&mut World, &[u8]) = match kind {
            change::Delegate::Produce => |world, branch| world.produce_file(branch, branch, b"change"),
            change::Delegate::Resolve { .. } => |world, branch| world.resolve(branch, b"resolved"),
            change::Delegate::Repair(_) => |world, branch| world.push(branch, b"repaired"),
            _ => panic!("queue has no other delegates: {kind:?}"),
        };
        if !draws.admits_delegate(world) {
            // No worker can finish this change in the fixture. Cancel it through
            // the procedure, rather than leaving a delegate that never answers.
            cancel(world, task, entry);
            return true;
        }
        let child = 100_000 + entry;
        world.event(top::Event::Delegated { task, child, kind });
        work(world, branch.as_bytes());
        world.status(branch.as_bytes(), raw::Check::Passed);
        world.event(top::Event::DelegateResult {
            ended_at: None,
            task,
            child,
            status: change::Status::Passed,
            no_change: false,
            flake: false,
            no_test: false,
            failed: false,
            lost: false,
            cancelled: false,
            words: b"done".as_slice().into(),
        });
    }
    matches!(decision, change::Decision::Finish { .. } | change::Decision::NoChange)
}
fn cancel(world: &mut World, task: u64, entry: u64) {
    world.event(top::Event::StepChange {
        task,
        entry,
        heard: change::Heard {
            review_request: None,
            closer_waiting: false,
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
}
fn histories(world: &World, count: u64) -> Vec<Vec<Event>> {
    (51..51 + count).map(|task| world.history(task)).collect()
}
fn register(world: &mut World, task: u64) {
    let mut row = changes::change_row();
    row.task = task;
    row.change.task = task;
    row.branch = format!("temper/{task}").into_bytes().into_boxed_slice();
    row.change.since = world.wall();
    world.event(top::Event::Change { row });
}
fn run(seed: u64) -> (Vec<Vec<Event>>, Skipped) {
    let mut rng = Rng::new(seed);
    let mut world = World::queue_steps(seed);
    world.adopt();
    let count = 2 + rng.below(3);
    let mut draws = Draws::new(count);
    let mut done = vec![false; usize::try_from(count).expect("small queue")];
    for task in 51..51 + count {
        register(&mut world, task);
    }
    let restart = rng.below(DRAW_STEPS);
    for step in 0..DRAW_STEPS {
        if step == restart {
            let before = histories(&world, count);
            world.restart();
            assert_eq!(histories(&world, count), before, "seed {seed}: restart writes nothing");
        }
        let index = usize::try_from(rng.below(count)).expect("small queue");
        let task = 51 + u64::try_from(index).expect("small queue");
        if done[index] {
            continue;
        }
        if matches!(world.change(task).change.state, change::State::Landed { .. }) {
            done[index] = advance(&mut world, &mut draws, task, 1000 + step);
            continue;
        }
        let branch = format!("temper/{task}");
        let action = rng.below(10);
        match action {
            0 if world.has_branch(branch.as_bytes()) => {
                draws.push(&mut world, branch.as_bytes(), step, true);
            }
            1 => draws.push(&mut world, b"main", step, false),
            2 if !matches!(world.change(task).change.state, change::State::Held { .. }) => {
                world.event(top::Event::RelayRefused {
                    task,
                    questions: world.change(task).questions.clone(),
                    words: b"hold".as_slice().into(),
                });
            }
            3 => {
                world.event(top::Event::ReleaseChange { task, release: None });
                done[index] = advance(&mut world, &mut draws, task, 1000 + step);
            }
            4 => {
                cancel(&mut world, task, 1000 + step);
                done[index] = true;
            }
            _ => done[index] = advance(&mut world, &mut draws, task, 1000 + step),
        }
    }
    // The two to four members now run with timely forge replies and no more
    // outside pushes. Each round advances each live member (and the clock by
    // at least a second); only the remaining queue's own merges move the base.
    for step in DRAW_STEPS..DRAW_STEPS + DRAIN_ROUNDS {
        for (index, ended) in done.iter_mut().enumerate() {
            if !*ended {
                let task = 51 + u64::try_from(index).expect("small queue");
                world.event(top::Event::ReleaseChange { task, release: None });
                *ended =
                    advance(&mut world, &mut draws, task, 1000 + step * count + u64::try_from(index).expect("index"));
            }
        }
        if done.iter().all(|ended| *ended) {
            break;
        }
    }
    assert!(done.iter().all(|ended| *ended), "seed {seed}: queue settles");
    let histories = histories(&world, count);
    for task in 51..51 + count {
        advance(&mut world, &mut draws, task, 10_000 + task);
    }
    assert_eq!(histories, self::histories(&world, count), "seed {seed}: terminal polls write nothing");
    for (index, events) in histories.iter().enumerate() {
        assert!(!events.is_empty(), "seed {seed}: history exercised");
        super::history::check(seed, 51 + u64::try_from(index).expect("index"), events);
    }
    (histories, draws.skipped)
}

fn cover(covered: &mut [bool; 10], histories: &[Vec<Event>]) {
    for event in histories.iter().flatten() {
        let index = match event.what {
            What::Pushed => Some(0),
            What::Update { .. } => Some(1),
            What::Updated { .. } => Some(2),
            What::Conflict { .. } => Some(3),
            What::Asked { .. } => Some(4),
            What::Answered { .. } => Some(5),
            What::Held { .. } => Some(6),
            What::Released { .. } => Some(7),
            What::Ended { ending: Ending::Cancelled } => Some(8),
            What::Landed => Some(9),
            _ => None,
        };
        if let Some(index) = index {
            covered[index] = true;
        }
    }
}

#[test]
fn seeded_queues_keep_histories_well_formed_across_restarts() {
    let at_gate = gate();
    let mut covered = [false; 10];
    let mut skipped = Skipped::default();
    cases(skein_world::this_test!("seeded_queues_keep_histories_well_formed_across_restarts"), 1..65, |seed| {
        let replay = run(seed);
        assert_eq!(replay, run(seed), "seed {seed} replays");
        let (histories, ineligible) = replay;
        skipped.tasks += ineligible.tasks;
        skipped.commits += ineligible.commits;
        if at_gate {
            cover(&mut covered, &histories);
        }
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        assert_eq!(skipped, Skipped::default(), "the gate needs no skips");
        assert_eq!(
            covered, [true; 10],
            "the seeds exercise pushes, updates, conflicts, asks, answers, holds, releases, cancels and landings"
        );
    } else {
        eprintln!("fuzzy reach: queue-history ineligible/task-table {}", skipped.tasks);
        eprintln!("fuzzy reach: queue-history ineligible/commit-room {}", skipped.commits);
    }
}

#[test]
fn a_full_task_table_skips_the_delegate_and_cancels_its_change() {
    const TASK: u64 = 51;
    let mut world = World::queue_steps(1);
    world.adopt();
    register(&mut world, TASK);
    for held in 1..u64::from(jig_tasks_kit::LIMITS.tasks) {
        world.hold_task(held);
    }
    let mut draws = Draws::new(u64::from(jig_tasks_kit::LIMITS.tasks));
    assert!(advance(&mut world, &mut draws, TASK, 1000));
    assert_eq!(draws.skipped, Skipped { tasks: 1, commits: 0 });
    assert!(!world.has_branch(b"temper/51"), "no child pushes a head");
    assert!(world.history(TASK).iter().any(|event| event.what == What::Ended { ending: Ending::Cancelled }));
}

#[test]
fn a_full_commit_store_skips_pushes_and_delegates() {
    const TASK: u64 = 51;
    let mut world = World::queue_steps(1);
    world.adopt();
    let room = world.forge_mut().room().commits;
    for step in 0..room {
        world.push_file(b"main", b"file", &step.to_be_bytes());
    }
    let head = world.branch(b"main");
    let mut draws = Draws::new(1);
    draws.push(&mut world, b"main", u64::from(room), false);
    assert_eq!(world.branch(b"main"), head);
    register(&mut world, TASK);
    assert!(advance(&mut world, &mut draws, TASK, 1000));
    assert_eq!(draws.skipped, Skipped { tasks: 0, commits: 2 });
    assert!(!world.has_branch(b"temper/51"), "no child pushes a head");
}
