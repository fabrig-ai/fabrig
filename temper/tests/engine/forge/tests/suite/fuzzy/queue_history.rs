//! Seeded queue steps and restarts (domain/landing-history.md, 8).
use skein_fake_forge_domain::api as raw;
use skein_lib::Rng;
use temper_engine_domain_forge::{self as top, history::Event};
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{World, changes};

fn advance(world: &mut World, task: u64, entry: u64) -> bool {
    let branch = format!("temper/{task}");
    let decision = changes::change_step_task(world, task, entry, change::Status::Passed, Box::new([]), false, false);
    if let change::Decision::Delegate(kind) = decision {
        let child = 100_000 + entry;
        world.event(top::Event::Delegated { task, child, kind });
        match kind {
            change::Delegate::Produce => world.produce_file(branch.as_bytes(), branch.as_bytes(), b"change"),
            change::Delegate::Resolve { .. } => world.resolve(branch.as_bytes(), b"resolved"),
            change::Delegate::Repair(_) => world.push(branch.as_bytes(), b"repaired"),
            _ => panic!("queue has no other delegates: {kind:?}"),
        }
        world.status(branch.as_bytes(), raw::Check::Passed);
        world.event(top::Event::DelegateResult {
            task,
            child,
            status: change::Status::Passed,
            no_change: false,
            failed: false,
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
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            delegate_running: false,
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
fn run(seed: u64) -> Vec<Vec<Event>> {
    let mut rng = Rng::new(seed);
    let mut world = World::queue_steps(seed);
    world.adopt();
    let count = 2 + rng.below(3);
    let mut done = vec![false; usize::try_from(count).expect("small queue")];
    for task in 51..51 + count {
        let mut row = changes::change_row();
        row.task = task;
        row.change.task = task;
        row.branch = format!("temper/{task}").into_bytes().into_boxed_slice();
        row.change.since = world.wall();
        world.event(top::Event::Change { row });
    }
    let restart = rng.below(80);
    for step in 0..80 {
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
            done[index] = advance(&mut world, task, 1000 + step);
            continue;
        }
        let branch = format!("temper/{task}");
        let action = rng.below(10);
        match action {
            0 if world.has_branch(branch.as_bytes()) => {
                world.push_file(branch.as_bytes(), b"file", &step.to_be_bytes());
                world.status(branch.as_bytes(), raw::Check::Passed);
            }
            1 => world.push_file(b"main", b"file", &step.to_be_bytes()),
            2 if !matches!(world.change(task).change.state, change::State::Held { .. }) => {
                world.event(top::Event::RelayRefused {
                    task,
                    questions: world.change(task).questions.clone(),
                    words: b"hold".as_slice().into(),
                });
            }
            3 => {
                world.event(top::Event::ReleaseChange { task });
                done[index] = advance(&mut world, task, 1000 + step);
            }
            4 => {
                cancel(&mut world, task, 1000 + step);
                done[index] = true;
            }
            _ => done[index] = advance(&mut world, task, 1000 + step),
        }
    }
    // Stop injecting faults and let the remaining queue land; held work is released.
    for step in 80..160 {
        for (index, ended) in done.iter_mut().enumerate() {
            if !*ended {
                let task = 51 + u64::try_from(index).expect("small queue");
                world.event(top::Event::ReleaseChange { task });
                *ended = advance(&mut world, task, 1000 + step * count + u64::try_from(index).expect("index"));
            }
        }
        if done.iter().all(|ended| *ended) {
            break;
        }
    }
    assert!(done.iter().all(|ended| *ended), "seed {seed}: queue settles");
    let histories = histories(&world, count);
    for task in 51..51 + count {
        advance(&mut world, task, 10_000 + task);
    }
    assert_eq!(histories, self::histories(&world, count), "seed {seed}: terminal polls write nothing");
    for (index, events) in histories.iter().enumerate() {
        assert!(!events.is_empty(), "seed {seed}: history exercised");
        super::history::check(seed, 51 + u64::try_from(index).expect("index"), events);
    }
    histories
}
#[test]
fn seeded_queues_keep_histories_well_formed_across_restarts() {
    let mut covered = [false; 10];
    for seed in 1..65 {
        let histories = run(seed);
        assert_eq!(histories, run(seed), "seed {seed} replays");
        for event in histories.iter().flatten() {
            use top::history::{Ending, What};
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
    assert_eq!(
        covered, [true; 10],
        "the seeds exercise pushes, updates, conflicts, asks, answers, holds, releases, cancels and landings"
    );
}
