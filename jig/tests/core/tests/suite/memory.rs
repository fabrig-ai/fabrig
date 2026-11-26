use jig_core_world::world::Setup;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_core::engine::{self, Admit};
use jig_domain_fleet as fleet;
use jig_domain_link::{Form, Scheme};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, Time, Wall};
use skein_world::domain::heap::Meter;

fn core_children_routes_and_an_in_flight_decision_stay_under_the_scenario_ceiling() {
    let limits = jig_test_profile::limits();
    let bound = SCENARIO_FAMILY_HEAP_CEILING;
    let configuration = jig_test_profile::config(91);
    let mut observer = jig_core_world::conformance::Observer::new(&configuration);
    let config = configuration.core;
    let meter = Meter::new();
    meter.start();
    let mut domain = core::Core::new(config, &limits.core).expect("fixture core configuration");
    let measured = meter.end();
    meter.check(measured, bound, &limits.core);

    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.core };
    let mut out = skein_lib::Queue::uncapped();
    let mut action = domain.start_begin(false, env.wall);
    for _ in 0_u16..64 {
        use core::start::{Action, Phase, SweepInput};
        action = match action {
            Action::Step(Phase::Sweep) => domain.start_sweep(&env, SweepInput::Finished, &mut out),
            Action::Step(phase) => domain.start_done(&env, phase, &mut out),
            Action::Commit(token) => domain.start_resume(&env, token, &mut out),
            Action::Idle => break,
            Action::Refused(why) => panic!("memory fixture start refused {why:?}"),
        };
        out.clear();
    }
    assert!(domain.start_opened());
    for event in [
        core::Event::Fleet(fleet::Event::Loaded),
        core::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    ] {
        meter.start();
        let requests = core::step(&mut domain, &env, event);
        let measured = meter.end();
        drop(requests);
        let held = meter.check(measured, bound, &limits.core);
        observer
            .observe(0, jig_core_world::referee::Observed::Heap { held, maximum: bound })
            .expect("heap within bound");
    }
    observer
        .observe(0, jig_core_world::referee::Observed::Heap { held: meter.held(), maximum: bound })
        .expect("heap within bound");
}

fn invalid_limits_are_refused() {
    let limits = jig_test_profile::limits().core;
    assert!(core::valid(&core::Limits { connectors: 0, ..limits }));
    assert!(core::valid(&core::Limits { routes: 1, ..limits }));
    assert!(!core::valid(&core::Limits { routes: 0, ..limits }));
    assert!(!core::valid(&core::Limits { resume_bytes: 0, ..limits }));
    assert!(!core::valid(&core::Limits { call_records: 0, ..limits }));
    assert!(!core::valid(&core::Limits { tasks: tasks::Limits { tasks: u32::MAX, ..limits.tasks }, ..limits }));
}

fn walking_and_effect_scripts_check_the_counted_heap_at_every_iteration() {
    let mut walking = jig_core_world::world::World::new(Setup {
        seed: 190,
        configuration: Some(jig_test_profile::config(190)),
        limits: jig_test_profile::limits(),
        walking: Some(jig_core_world::world::Scenario::Goal),
        ..Setup::default()
    });
    walking.drain();
    drop(walking);
    let mut effects = jig_core_world::world::World::new(Setup { seed: 191, ..Setup::default() });
    effects.delegate();
    effects.call(2, 9000);
    effects.restart(191, false);
}

#[test]
fn walking_settles_after_retention_erases_forgotten_remains() {
    use jig_test_domain as root;

    let mut walking = jig_core_world::world::World::new(Setup {
        seed: 190,
        configuration: Some(jig_test_profile::config(190)),
        limits: jig_test_profile::limits(),
        walking: Some(jig_core_world::world::Scenario::Goal),
        ..Setup::default()
    });
    let procedure = walking.walking.as_ref().expect("walking peer").procedure.expect("ended procedure");
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(record))) =
        &walking.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(procedure)))]
    else {
        panic!("ended procedure record")
    };
    let mut remains = tasks::TaskSummary::of(record);
    // The forgotten fixture does not name a task the walking script owns.
    remains.number = 900;
    remains.root = 900;
    walking.seed(Box::new([
        root::Record::Core(core::Record::Tasks(tasks::Stored::Summary(Box::new(remains)))),
        root::Record::Core(core::Record::Tasks(tasks::Stored::Retention(tasks::RetentionRow {
            stage: tasks::RetentionStage::Forgotten,
            since: Wall::EPOCH,
            number: 900,
        }))),
    ]));
    let (now, wall) = walking.clocks();
    walking.set_clock(now.saturating_add(Duration::from_secs(3601)), wall);
    let inspections = walking.inspections();
    walking.drain();
    assert!(walking.inspections() > inspections, "the quiet-point wait and table checks ran");
    let (now, wall) = walking.clocks();
    assert_eq!(engine::due(walking.domain.engine(), now, wall, Admit::All), None);
    assert!(!walking.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Summary(900)))));
    assert!(!walking.store.rows.keys().any(|key| matches!(key,
        root::Key::Core(core::Key::Tasks(tasks::Key::Retention {
            stage: tasks::RetentionStage::Forgotten, number, ..
        })) if *number == 900
    )));
}

// Peak 20,334 bytes with provider defaults and their startup snapshot; measured plus a quarter, rounded up.
const SCENARIO_FAMILY_HEAP_CEILING: u64 = 26_000;

// Each named scenario preserves its own fixture and original assertions.
#[test]
fn memory_scenario_family() {
    core_children_routes_and_an_in_flight_decision_stay_under_the_scenario_ceiling();
    invalid_limits_are_refused();
    walking_and_effect_scripts_check_the_counted_heap_at_every_iteration();
}

#[test]
fn populated_projection_and_call_owner_routes_restore_under_the_counted_heap_bound() {
    use jig_test_connector as connector;
    use jig_test_domain as root;
    let (mut configuration, limits) = jig_core_world::world::fixture(193, false);
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    policy.projections.clone_from(&policy.ceiling.grants);
    configuration.core.seed_projects[0].policy.policy = policy.clone();
    let mut restart = jig_core_world::world::fixture(193, false).0;
    restart.core.seed_projects[0].policy.policy = policy;
    let mut world = jig_core_world::world::World::new(Setup {
        seed: 193,
        configuration: Some(configuration),
        limits,
        ..Setup::default()
    });
    world.hold_writes = true;
    world.call(1, 230);
    let request = jig_core_world::fixtures::goal(
        &world,
        1,
        b"routing owners".as_slice().into(),
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 20, priority: 1, ..Default::default() },
    );
    crate::read_support::request(&mut world, 94, request);
    let goal = world
        .people_answers
        .iter()
        .rev()
        .find_map(|reply| match reply {
            people::Reply::Outcome(people::Outcome::Created { first, .. }) => Some(*first),
            people::Reply::Signer(_)
            | people::Reply::Full { .. }
            | people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Refused(_)
            | people::Reply::Again(_) => None,
        })
        .expect("tracked goal admitted alongside the caller");
    world.hold_projection_settlements = true;
    world.send(root::Event::ProjectionEffect { goal, number: 1, effect: jig_core_world::world::World::effect() });
    let pending = std::mem::take(&mut world.pending_writes);
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(row, root::Record::Connector { record: connector::Record::Outbox(_), .. }))
            .count(),
        2
    );
    world.restart_with(restart);
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(row, root::Record::Connector { record: connector::Record::Outbox(_), .. }))
            .count(),
        2
    );
    for (number, call) in pending {
        let event = world.systems[usize::from(number - 1)].answer(call, jig_test_system::Fault::None);
        world.send(root::Event::Connector { number, event: connector::Event::System(event) });
    }
    world.retry_due();
    world.release_writes();
    assert!(
        !world
            .store
            .rows
            .values()
            .any(|row| matches!(row, root::Record::Connector { record: connector::Record::Outbox(_), .. }))
    );
}
