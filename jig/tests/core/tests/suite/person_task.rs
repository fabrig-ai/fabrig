//! A person's projectless task reaches a worker and restores (authority.md, 6.1).
use jig_core_world::watches;
use jig_core_world::world::{Setup, World};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

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
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(configuration()),
        rows: Some(Box::new([])),
        require_assignment: false,
        ..Setup::default()
    });
    world.send(root::Event::Core(core::Event::Key {
        reply_to: ReplyTo::new(Token::new(901)),
        sign_in: world.sign_in.expect("signed in person"),
        key: [91; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::AddKey { provider: 1, join: None, digest: [7; 32] },
        }),
        carried: core::Carried { number: 1_u64 << 63, digest: [7; 32] },
    }));
    world
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
    let attempt = assignment.attempt;
    let row = world.domain.core_mut().tasks.task(task).expect("live task");
    assert_eq!(row.project, PROJECT);
    assert_eq!(row.kept_account, Some(1));
    assert_eq!(world.domain.core_mut().tasks.payer(task), Some(tasks::Payer::Party(PERSON)));
    world.watches = Some(watches::Parties::new(SEED, world.store.applied, true, world.sign_in.expect("person")));
    watches::open(&mut world, 20, core::documents::Watched::Task { task });
    let core::documents::Document::Task(document) = watches::latest(&world, 20) else { panic!("Task document") };
    assert_eq!(document.kept_account, Some(1));
    assert_eq!(document.run_wait, None);
    world.restart_with(configuration());
    assert!(world.domain.ready(), "restart checks stored links and opens");
    assert!(world.domain.core_mut().tasks.task(task).is_some(), "restored live");
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: fleet::Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([fleet::Hosting {
                run: Token::new(task),
                attempt: Token::new(attempt),
                phase: fleet::Phase::Active,
            }]),
        },
    })));
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Cancel {
        reply_to: ReplyTo::new(Token::new(900)),
        by: tasks::Party::Deployment { project: PROJECT },
        operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
    })));
    let attempt = world.domain.core_mut().tasks.task(task).expect("cancel drains the worker").attempt;
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: Token::new(7),
        task,
        attempt,
        cumulative: 0,
        end: tasks::End::Parked,
    });
    assert!(
        world.domain.core_mut().tasks.task(task).is_none_or(|row| matches!(row.phase, tasks::Phase::Ended(_))),
        "deployment cancels restored task"
    );
}

#[test]
fn a_projectless_failure_stays_held_and_reaches_no_person() {
    let mut world = world();
    let task = make(&mut world);
    for count in 1..=2 {
        world.end(tasks::End::Failed(tasks::Class::Permanent), 0);
        assert_eq!(world.domain.core_mut().tasks.task(task).expect("task held live").tries.permanent, count);
        if count == 1 {
            world.advance(Duration::from_secs(1));
            world.retry_due();
        }
    }
    assert!(matches!(
        world.domain.core_mut().tasks.task(task).unwrap().phase,
        tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Permanent), .. }
    ));
    world.watches = Some(watches::Parties::new(SEED, world.store.applied, true, world.sign_in.expect("person")));
    watches::open(&mut world, 20, core::documents::Watched::Inbox);
    let core::documents::Document::Inbox(inbox) = watches::latest(&world, 20) else { panic!("Inbox document") };
    assert!(inbox.needs.is_empty());
    assert!(inbox.updates.is_empty());
    world.restart_with(configuration());
    assert!(world.domain.ready(), "held task restores with no escalation role");
    assert!(matches!(world.domain.core_mut().tasks.task(task).unwrap().phase, tasks::Phase::Held { .. }));
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Cancel {
        reply_to: ReplyTo::new(Token::new(900)),
        by: tasks::Party::Deployment { project: PROJECT },
        operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
    })));
    assert!(world.domain.core_mut().tasks.task(task).is_none_or(|row| matches!(row.phase, tasks::Phase::Ended(_))));
}
