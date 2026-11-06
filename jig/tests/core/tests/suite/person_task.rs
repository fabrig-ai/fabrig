//! A person's projectless task reaches a worker and restores (authority.md, 6.1).
use jig_core_world::world::{Setup, World};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const SEED: u64 = 14_001;
const PERSON: u64 = 1;
const PROJECT: u32 = people::DEPLOYMENT;

fn configuration() -> root::Config {
    let mut config = jig_test_profile::effect_fixture(SEED, false).0;
    config.core.seed_projects = Box::new([]);
    config.core.owners = Box::new([]);
    config
}

fn world() -> World {
    World::new(Setup {
        seed: SEED,
        configuration: Some(configuration()),
        require_assignment: false,
        ..Setup::default()
    })
}

fn make(world: &mut World) -> u64 {
    let seed = world.domain.core_mut().person_seed(PERSON).expect("signed in person's pool");
    let task = seed.number();
    world.send(root::Event::Core(core::Event::PersonTask {
        seed: Box::new(seed),
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"answer once".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 64 },
    }));
    task
}

#[test]
fn a_person_without_a_project_runs_on_a_worker_and_restores_live() {
    let mut world = world();
    let task = make(&mut world);
    let assignment = world.assigned.as_ref().expect("person task reached worker");
    assert_eq!(assignment.task, task);
    let row = world.domain.core_mut().tasks.task(task).expect("live task");
    assert_eq!(row.project, PROJECT);
    assert_eq!(row.kept_account, Some(1));
    assert_eq!(world.domain.core_mut().tasks.payer(task), Some(tasks::Payer::Party(PERSON)));
    world.restart_with(configuration());
    assert!(world.domain.ready(), "restart checks stored links and opens");
    assert!(world.domain.core_mut().tasks.task(task).is_some(), "restored live");
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Cancel {
        reply_to: ReplyTo::new(Token::new(900)),
        by: tasks::Party::Deployment { project: PROJECT },
        operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
    })));
    assert!(world.domain.core_mut().tasks.task(task).is_none(), "deployment cancels restored task");
}
