//! A separate retention model follows each commit and cold restart.
use jig_core_world::{
    fixtures,
    world::{Setup, World},
};
use jig_domain_core::{self as core, settings};
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Rng, Token};
use skein_world::{sweep::cases, this_test};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
};

const PROJECT: u32 = 1;

fn set(revision: u64) -> core::ProjectCharters {
    core::ProjectCharters {
        project: PROJECT,
        revision,
        source: Box::from(revision.to_be_bytes()),
        charters: Box::new([core::ProjectCharter {
            name: Box::from(b"implementer".as_slice()),
            settings: Box::from(b"implementer".as_slice()),
            instructions: Box::from(b"Be careful.".as_slice()),
            parts: Box::new([]),
        }]),
    }
}

fn make(world: &mut World, template: &tasks::TaskRecord) -> u64 {
    let number = core::fresh(&mut world.domain.core_mut().counters, core::Family::Task).expect("task numbers");
    let mut authority = template.authority.clone();
    authority.budget.spend = 1;
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
        reads_project: false,
        maker: None,
        places: Box::new([]),
        reply_to: ReplyTo::new(Token::new(number)),
        creator: template.requester,
        batch: Box::new([tasks::New {
            deadline: tasks::Deadline::Named,
            label: 0,
            flags: tasks::Flags::PLAIN,
            number,
            project: PROJECT,
            executor: template.executor,
            llm: None,
            spec: template.spec.clone(),
            contract: template.contract.clone(),
            authority,
            numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
            funder: template.funder,
            dependencies: Box::new([]),
            holdings: Box::new([]),
            wake: tasks::WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        }]),
    })));
    assert!(
        world.domain.core().tasks.task(number).is_some(),
        "creation refused: {:?}",
        world.domain.core().tasks.capacity_refusals(),
    );
    number
}

fn end(world: &mut World, task: u64, by: tasks::Party) {
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Cancel {
        reply_to: ReplyTo::new(Token::new(task)),
        by,
        operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
    })));
    assert!(world.domain.core().tasks.task(task).is_none(), "an unclaimed task ends");
}

fn agree(world: &World, newest: u64, live: &BTreeMap<u64, u64>) {
    let core = world.domain.core();
    assert_eq!(core.project_charters(PROJECT), newest);
    let mut expected: BTreeSet<u64> = live.values().copied().filter(|rev| *rev != 0).collect();
    if newest != 0 {
        expected.insert(newest);
    }
    let kept: BTreeSet<u64> = world
        .store
        .rows
        .keys()
        .filter_map(|key| match key {
            root::Key::Core(core::Key::Core(core::CoreKey::Settings(settings::Key::ProjectCharters {
                project: PROJECT,
                revision,
            }))) => Some(*revision),
            _ => None,
        })
        .collect();
    assert_eq!(expected, kept, "exactly the held sets are durable");
    for revision in 1..=newest {
        let expected = expected.contains(&revision).then(|| set(revision));
        assert_eq!(core.project_charter_set(PROJECT, revision), expected.as_ref());
    }
    assert_eq!(core.tasks.view_tasks().len(), live.len());
    for (number, revision) in live {
        let row = core.tasks.task(*number).expect("model's live agent");
        assert_eq!(row.charters, *revision);
    }
}

#[test]
fn random_charter_sets_tasks_ends_and_restarts_keep_exactly_the_held_revisions() {
    cases(
        this_test!("random_charter_sets_tasks_ends_and_restarts_keep_exactly_the_held_revisions"),
        1720..1728,
        |seed| {
            let mut rng = Rng::new(seed);
            let mut setup = Setup { seed, ..Setup::default() };
            // Each step can create one live task; the model never runs out of task slots.
            setup.limits.core.tasks.tasks = 24;
            setup.limits.core.tasks.project_tasks = 24;
            let mut world = World::new(setup);
            let task = world.task();
            let template = world.domain.core().tasks.task(task).expect("fixture agent").clone();
            fixtures::finish_chat(&mut world, 0, b"fixture finished");
            // No worker reconnects: the model can end tasks without simulating a worker's stop.
            world.restart(seed, false);
            let mut live = BTreeMap::new();
            let mut newest = 0;
            agree(&world, newest, &live);
            for _ in 0..24 {
                match rng.below(4) {
                    0 => {
                        newest += 1;
                        world.send(root::Event::Core(core::Event::ProjectCharters(set(newest))));
                    }
                    1 => {
                        let number = make(&mut world, &template);
                        live.insert(number, newest);
                    }
                    2 => {
                        if let Some((&number, _)) = live.first_key_value() {
                            end(&mut world, number, template.requester);
                            live.remove(&number);
                        }
                    }
                    _ => world.restart(seed, false),
                }
                agree(&world, newest, &live);
            }
            Ok::<(), Infallible>(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
}
