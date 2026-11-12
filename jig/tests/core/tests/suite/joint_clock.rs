use jig_core_world::fixtures::sign_in as signin;
use jig_core_world::peers::Peers;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_link::Hello;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Time, Token, Wall};

fn sign_in(world: &mut World, number: u64) {
    world.send(signin(Token::new(number), 0, &number.to_be_bytes(), b"person", b"Person"));
}

#[test]
fn supplied_joint_clocks_survive_internal_commits_and_drive_the_actual_sign_in_expiry() {
    let limits = jig_test_profile::limits();
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(1_000_000_000_000);
    let mut world = World::new(Setup {
        configuration: Some(jig_test_profile::config(8451)),
        limits,
        peers: Some(Peers::new(Vec::new(), Vec::new())),
        clock: Some((now, wall)),
        ..Setup::default()
    });
    sign_in(&mut world, 7);
    let expiry = Wall::from_nanos(
        wall.as_nanos().checked_add(limits.core.people.sign_in_lifetime.as_nanos()).expect("bounded configured expiry"),
    );
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::SignedIn { person: 1, expires, .. } if *expires == expiry))
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, expires, .. })) if *expires == expiry)));
    assert!(world.store.applied > 0, "the sign-in was actually committed");
    let applied = world.store.applied;
    world.drain();
    assert_eq!(world.store.applied, applied);
    assert_eq!(world.clocks(), (now, wall), "draining creates no elapsed or wall time");
}

#[test]
fn corrected_joint_wall_is_independent_of_elapsed_time_and_is_persisted_for_a_new_sign_in() {
    let limits = jig_test_profile::limits();
    let mut configuration = jig_test_profile::config(8452);
    configuration.core.owners = Box::new([]);
    let mut world = World::new(Setup {
        configuration: Some(configuration),
        limits,
        peers: Some(Peers::new(Vec::new(), Vec::new())),
        clock: Some((Time::from_nanos(17), Wall::from_nanos(2_000_000_000_000))),
        ..Setup::default()
    });
    sign_in(&mut world, 7);
    let prior_expiry = Wall::from_nanos(2_000_000_000_000 + limits.core.people.sign_in_lifetime.as_nanos());
    let now = Time::from_nanos(31);
    let corrected = Wall::from_nanos(1_000_000_000_000);
    world.set_clock(now, corrected);
    sign_in(&mut world, 8);
    let expiry = Wall::from_nanos(
        corrected.as_nanos().checked_add(limits.core.people.sign_in_lifetime.as_nanos()).expect("corrected expiry"),
    );
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::SignedIn { person: 2, expires, .. } if *expires == expiry))
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 2, expires, .. })) if *expires == expiry)));
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, expires, .. })) if *expires == prior_expiry)), "the correction does not rewrite an existing durable expiry");
    assert_eq!(world.clocks(), (now, corrected));
}

#[test]
fn joint_scheduler_expires_the_actual_sign_in_at_its_armed_people_deadline() {
    let limits = jig_test_profile::limits();
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(1_000_000_000_000);
    let mut world = World::new(Setup {
        configuration: Some(jig_test_profile::config(8453)),
        limits,
        peers: Some(Peers::new(Vec::new(), Vec::new())),
        clock: Some((now, wall)),
        ..Setup::default()
    });
    sign_in(&mut world, 7);
    let due = Time::from_nanos(now.as_nanos() + limits.core.people.sign_in_lifetime.as_nanos());
    assert_eq!(world.domain.core().next_deadline(world.clocks().0, world.clocks().1), Some(due));
    let saved = world
        .store
        .rows
        .iter()
        .find_map(|(key, row)| {
            if matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, .. }))) {
                Some(key.clone())
            } else {
                None
            }
        })
        .expect("actual durable sign-in");
    let applied = world.store.applied;
    world.fire_due(Time::from_nanos(due.as_nanos() - 1), wall);
    assert!(world.store.rows.contains_key(&saved), "not yet due");
    assert_eq!(world.store.applied, applied);
    world.fire_due(due, wall);
    assert!(!world.store.rows.contains_key(&saved), "People fired through the normal durable root route");
    assert!(world.store.applied > applied);
    assert_eq!(world.clocks(), (due, wall), "an elapsed deadline needs no wall jump");
    assert_eq!(world.domain.core().people.next_deadline(), None);
}

#[test]
fn joint_scheduler_erases_a_sign_in_when_wall_jumps_before_its_elapsed_alarm() {
    let limits = jig_test_profile::limits();
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(1_000_000_000_000);
    let mut world = World::new(Setup {
        configuration: Some(jig_test_profile::config(8455)),
        limits,
        peers: Some(Peers::new(Vec::new(), Vec::new())),
        clock: Some((now, wall)),
        ..Setup::default()
    });
    sign_in(&mut world, 7);
    let saved = world
        .store
        .rows
        .iter()
        .find_map(|(key, row)| match row {
            root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, expires, .. })) => {
                Some((key.clone(), *expires))
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("actual durable sign-in and saved wall expiry");
    let applied = world.store.applied;
    let elapsed = Time::from_nanos(now.as_nanos() + 1);
    world.fire_due(elapsed, saved.1);
    assert!(
        !world.store.rows.contains_key(&saved.0),
        "the ordinary People timer erases the wall-expired sign-in before its original elapsed alarm"
    );
    assert_eq!(world.store.applied, applied + 1, "one durable expiry decision");
    assert_eq!(world.clocks(), (elapsed, saved.1));
    world.fire_due(elapsed, saved.1);
    assert_eq!(world.store.applied, applied + 1, "the expiry fires once");
}

#[test]
fn joint_scheduler_resamples_the_effect_wall_after_correction_and_consumes_the_right_without_a_new_commit() {
    let mut world = World::new(Setup { seed: 8454, ..Setup::default() });
    let now = world.clocks().0;
    let wall = Wall::from_nanos(1_000_000_000_000);
    // Settle People made due by the forward correction before isolating the effect deadline.
    world.fire_due(now, wall);
    world.hold_writes = true;
    let deadline = Wall::from_nanos(wall.as_nanos() + 500);
    let key = world.call_key(1);
    world.send(root::Event::EffectCall {
        to: ReplyTo::new(Token::new(9001)),
        key,
        number: 1,
        effect: World::effect(),
        deadline,
        proposal: None,
    });
    assert_eq!(world.answers.len(), 0);
    assert_eq!(world.domain.effect_deadline(), Some(deadline));
    assert_eq!(world.next_due(), Some(Time::from_nanos(now.as_nanos() + 500)));
    let corrected = Wall::from_nanos(wall.as_nanos() - 100);
    let elapsed = Time::from_nanos(now.as_nanos() + 20);
    world.set_clock(elapsed, corrected);
    assert_eq!(world.next_due(), Some(Time::from_nanos(elapsed.as_nanos() + 600)));
    let applied = world.store.applied;
    let makes = world.systems[0].observed().len();
    world.fire_due(Time::from_nanos(elapsed.as_nanos() + 599), Wall::from_nanos(deadline.as_nanos() - 1));
    assert_eq!(world.answers.len(), 0);
    world.fire_due(Time::from_nanos(elapsed.as_nanos() + 600), deadline);
    assert_eq!(world.answers.len(), 1);
    assert_eq!(world.answers[0].0, Token::new(9001));
    assert_eq!(world.answers[0].1, key);
    assert!(matches!(world.answers[0].2, core::CallPart::Effect { outcome: None, .. }));
    assert_eq!(world.store.applied, applied, "deadline returns the previously committed part");
    assert_eq!(world.systems[0].observed().len(), makes, "deadline makes no new external effect");
    assert_eq!(world.domain.effect_deadline(), None);
    world.fire_due(Time::from_nanos(elapsed.as_nanos() + 601), deadline);
    assert_eq!(world.answers.len(), 1, "the original reply right is consumed once");
}

fn read(world: &mut World, number: u64, value: core::reads::Read) {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(number)),
        sign_in: world.sign_in.expect("signed in"),
        read: value,
    }));
}
fn check_source_documents(world: &World, reset: Wall) {
    let documents: Vec<_> = world
        .client_peer
        .as_ref()
        .expect("client peer")
        .delivered
        .iter()
        .filter_map(|(_, delivery)| {
            let root::Delivery::Core(core::Held::Read { answer: core::reads::Answer::Document(document), .. }) =
                delivery
            else {
                return None;
            };
            Some(document.as_ref())
        })
        .collect();
    let deployment = documents
        .iter()
        .find_map(|document| {
            let core::documents::Document::Deployment(document) = document else { return None };
            Some(document.as_ref())
        })
        .expect("actual deployment read");
    let account = deployment.accounts.get(0).expect("deployment account");
    let state = core::documents::AccountState::Later { until: Some(reset) };
    assert_eq!(account.state, state);
    assert_eq!(account.spent_until, Some(reset));
    let source = account.sources.get(0).expect("deployment source");
    assert_eq!(source.state, state);
    assert_eq!(source.spent_until, Some(reset));
    let task = documents
        .iter()
        .find_map(|document| {
            let core::documents::Document::Task(document) = document else { return None };
            Some(document.as_ref())
        })
        .expect("actual task read");
    assert_eq!(
        task.run_wait,
        Some(core::documents::TaskRunWait::Account {
            account: Some(0),
            source: Some(1),
            why: core::documents::TaskWaitWhy::Source(core::documents::TaskSourceState::Limited { until: Some(reset) }),
        }),
        "task read at {:?}",
        world.clocks()
    );
}

fn ready_worker(world: &mut World) {
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: skein_lib::Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}

#[test]
fn source_wall_survives_held_reads_and_a_new_restore_origin() {
    use jig_domain_accounts as accounts;
    use jig_domain_tasks as tasks;
    let seed = 8456;
    let (configuration, mut limits) = jig_core_world::world::fixture(seed, false);
    limits.core.people.projects = 2;
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() });
    let assigned = world.assigned.as_ref().expect("assigned chat");
    let (task, attempt) = (assigned.task, assigned.attempt);
    let sign_in = world.sign_in.expect("signed in");
    let person = world
        .store
        .rows
        .values()
        .find_map(|record| {
            let root::Record::Core(core::Record::People(people::Stored::SignIn { number, person, .. })) = record else {
                return None;
            };
            (*number == sign_in).then_some(*person)
        })
        .expect("durable sign-in");
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 0,
        holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
    })));
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::default());
    world.set_clock(Time::from_nanos(1_000_000_000), Wall::from_nanos(10_000_000_000));
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Limited {
        channel: Token::new(7),
        run: Token::new(task),
        attempt: Token::new(attempt),
        account: 1,
        reset: Some(skein_lib::Duration::from_secs(30)),
    })));
    world.end(tasks::End::Failed(tasks::Class::Run), 1);
    let prerequisite = world.store.applied;
    world.set_clock(Time::from_nanos(2_000_000_000), Wall::from_nanos(20_000_000_000));
    read(&mut world, 9002, core::reads::Read::Deployment);
    world.set_clock(Time::from_nanos(3_000_000_000), Wall::from_nanos(5_000_000_000));
    assert!(
        world.client_peer.as_ref().expect("client peer").delivered.is_empty(),
        "reads cannot cross the held completion"
    );
    world.release_commits();
    read(&mut world, 9003, core::reads::Read::Task { task });
    let reset = Wall::from_nanos(40_000_000_000);
    check_source_documents(&world, reset);
    assert!(
        world.client_peer.as_ref().expect("client peer").delivered.iter().all(|(commit, _)| *commit >= prerequisite)
    );
    let key = root::Key::Core(core::Key::Accounts(accounts::Key::Source { account: 0, number: 1 }));
    let root::Record::Core(core::Record::Accounts(accounts::Stored::Source { held, .. })) = &world.store.rows[&key]
    else {
        panic!("source record")
    };
    assert_eq!(*held, Some(accounts::Held::Limited { reset: Some(reset) }));
    world.set_clock(Time::from_nanos(4_000_000_000), Wall::from_nanos(15_000_000_000));
    world.restart_with(jig_core_world::world::fixture(seed, false).0);
    let deadline = accounts::Deadline { at: Time::from_nanos(29_000_000_000), wall: reset };
    assert_eq!(world.domain.accounts().source(1).expect("restored source").spent_until, Some(deadline));
    ready_worker(&mut world);
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::default());
    read(&mut world, 9004, core::reads::Read::Deployment);
    world.set_clock(Time::from_nanos(5_000_000_000), Wall::from_nanos(30_000_000_000));
    read(&mut world, 9005, core::reads::Read::Task { task });
    check_source_documents(&world, reset);
    world.fire_due(Time::from_nanos(28_999_999_999), Wall::from_nanos(35_000_000_000));
    assert_eq!(world.domain.accounts().source(1).expect("not due").spent_until, Some(deadline));
    world.fire_due(deadline.at, Wall::from_nanos(35_000_000_000));
    assert_eq!(world.domain.accounts().source(1).expect("due").spent_until, None);
}
