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

#[test]
fn the_payers_task_watch_refreshes_and_ends_on_completion_or_cancellation() {
    use core::documents::{ChipWork, Watched};
    for cancel in [false, true] {
        let mut world = world();
        let task = make(&mut world);
        let attempt = world.assigned.as_ref().expect("worker run").attempt;
        // The client peer accepts host observations between durable task changes.
        world.client_peer = Some(jig_core_world::client_peer::ClientPeer::routed());
        watches::open(&mut world, 20, Watched::Task { task });
        assert_eq!(peer_task(&world, 20).kept_account, Some(1));
        world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(20) }));
        let before = world.client_peer.as_ref().expect("watch").delivered.len();
        world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Observe {
            channel: Token::new(7),
            run: Token::new(task),
            attempt: Token::new(attempt),
            report: fleet::Report::Activity(fleet::Activity::Active),
        })));
        let document = peer_task(&world, 20);
        assert_eq!(document.chip.details.observation.as_ref().expect("current run").work, ChipWork::Active);
        assert!(world.domain.core_mut().run_past_starting(task));
        assert!(world.client_peer.as_ref().expect("watch").delivered.len() > before, "run refreshes payer's watch");
        world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(20) }));
        if cancel {
            world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Cancel {
                reply_to: ReplyTo::new(Token::new(900)),
                by: tasks::Party::Deployment { project: PROJECT },
                operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
            })));
            world.end(tasks::End::Parked, 0);
        } else {
            world.end(
                tasks::End::Finished {
                    result: tasks::TaskResult::Report { words: b"answered".as_slice().into() },
                    cancel_delegates: false,
                },
                0,
            );
        }
        world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(20) }));
        assert!(world.client_peer.as_ref().expect("watch").delivered.iter().any(|(_, delivery)| matches!(
            delivery,
            root::Delivery::Core(core::Held::WatchEnded { watcher, why: jig_domain_views::End::Gone })
                if *watcher == Token::new(20)
        )));
        assert!(!world.domain.core_mut().run_past_starting(task));
    }
}

fn peer_task(world: &World, watcher: u64) -> &core::documents::Task {
    world
        .client_peer
        .as_ref()
        .expect("watch caller")
        .delivered
        .iter()
        .rev()
        .find_map(|(_, delivery)| {
            let root::Delivery::Core(core::Held::Delivery { watcher: received, document }) = delivery else {
                return None;
            };
            let core::documents::Document::Task(task) = document.as_ref() else { return None };
            (*received == Token::new(watcher)).then_some(task.as_ref())
        })
        .expect("Task document")
}

fn refused_watch(world: &mut World, watcher: u64, subject: core::documents::Watched) {
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::routed());
    watches::open(world, watcher, subject);
    let peer = world.client_peer.take().expect("watch caller");
    assert!(peer.delivered.is_empty(), "a refused watch serves no document");
    let [core::Now::WatchRefused { watcher: refused, why }] = peer.now.as_slice() else {
        panic!("one watch refusal: {:?}", peer.now)
    };
    assert_eq!(*refused, Token::new(watcher));
    assert_eq!(*why, core::WatchRefusal::NotAllowed);
}

#[test]
fn the_payers_exception_admits_no_other_viewer_or_subject() {
    use core::documents::Watched;
    let mut world = world();
    let task = make(&mut world);
    refused_watch(&mut world, 21, Watched::Tree { task });
    refused_watch(&mut world, 22, Watched::Run { task });
    world.send(jig_core_world::fixtures::sign_in(Token::new(902), 0, b"another", b"other", b"Other"));
    refused_watch(&mut world, 23, Watched::Task { task });
    let sign_in = world.sign_in.expect("another person");
    let person = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::People(people::Stored::SignIn { number, person, .. }))
                if *number == sign_in =>
            {
                Some(*person)
            }
            _ => None,
        })
        .expect("another person's stored identity");
    assert_ne!(person, PERSON);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
    })));
    refused_watch(&mut world, 24, Watched::Task { task });
}

#[test]
fn a_real_projects_task_watch_still_requires_a_project_role() {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    let task = world.assigned.as_ref().expect("real project's task").task;
    world.send(root::Event::Core(core::Event::People(people::Event::Roles { project: 1, holdings: Box::new([]) })));
    refused_watch(&mut world, 25, core::documents::Watched::Task { task });
}
