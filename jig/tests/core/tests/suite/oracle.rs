use jig_core_world::effects::World;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

#[test]
fn a_parked_conversation_keeps_its_documents_and_transcript() {
    let mut world = World::new(1901, false);
    world.turn(1, 1, None, b"durable answer");
    world.end(tasks::End::Parked, 1);
    world.close();
}

#[test]
#[should_panic(expected = "restart oracle document Task 1 field chip")]
fn a_writer_forgetting_a_durable_task_field_names_its_document() {
    let mut world = World::new(1902, false);
    world.end(tasks::End::Parked, 0);
    let task = world.assigned.as_ref().expect("assigned task").task;
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(mut row))) = world.store.rows[&key].clone() else {
        panic!("live task");
    };
    row.spec.words = b"writer forgot its answer".as_slice().into();
    world.seed(Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))]));
    world.close();
}

#[test]
fn a_deliberately_broken_fixture_skips_the_oracle() {
    let mut world = World::new(1903, false);
    world.stopped = true;
    // The fixture deliberately leaves the engine stopped.
    world.broken();
    world.close();
}

#[test]
fn a_dead_hosts_runtime_observation_is_rebuilt() {
    use jig_domain_fleet as fleet;
    use skein_lib::Token;
    let mut world = World::new(1904, false);
    let assigned = world.assigned.as_ref().expect("assigned task");
    let (run, attempt) = (Token::new(assigned.task), Token::new(assigned.attempt));
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Observe {
        channel: Token::new(7),
        run,
        attempt,
        report: fleet::Report::Started,
    })));
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    jig_core_world::reads::ask(&mut world, 1, core::reads::Read::Task { task: run.raw() });
    let core::reads::Answer::Document(document) = jig_core_world::reads::latest(&world) else {
        panic!("task document")
    };
    let core::documents::Document::Task(task) = &**document else { panic!("task document") };
    assert!(task.chip.details.observation.is_some());
    world.close();
}

#[test]
fn the_restart_sweep_erases_only_the_expired_sign_in() {
    use jig_domain_people as people;
    let mut world = World::new(1905, false);
    let key = root::Key::Core(core::Key::Core(core::CoreKey::Deployment));
    let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(mut header))) =
        world.store.rows[&key].clone()
    else {
        panic!("deployment header")
    };
    header.sign_ins += 1;
    let expired = header.sign_ins;
    world.seed(Box::new([
        root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))),
        root::Record::Core(core::Record::People(people::Stored::SignIn {
            number: expired,
            person: 1,
            expires: skein_lib::Wall::EPOCH,
        })),
    ]));
    world.close();
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::SignIn(expired)))));
    assert!(
        world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::SignIn(
            world.sign_in.expect("live sign-in")
        ))))
    );
}

#[test]
fn close_settles_a_commit_held_before_the_component() {
    let mut world = World::new(1906, false);
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    world.turn(1, 1, None, b"held answer");
    assert!(!world.store.held.is_empty());
    world.close();
    assert!(world.store.held.is_empty());
    assert!(world.store.pending.is_empty());
}

#[test]
#[should_panic(expected = "restart oracle document Task 1 field chip")]
fn dropping_a_world_runs_the_restart_oracle() {
    let mut world = World::new(1907, false);
    world.end(tasks::End::Parked, 0);
    let task = world.assigned.as_ref().expect("assigned task").task;
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(mut row))) = world.store.rows[&key].clone() else {
        panic!("live task");
    };
    row.spec.words = b"writer forgot its answer".as_slice().into();
    world.seed(Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))]));
}

#[test]
#[should_panic(expected = "the story's own assertion")]
fn unwinding_preserves_the_storys_failure_without_a_second_panic() {
    let mut world = World::new(1908, false);
    world.stopped = true;
    panic!("the story's own assertion");
}

#[test]
fn an_explicit_close_runs_only_one_cold_restart() {
    let mut world = World::new(1909, false);
    world.close();
    let applied = world.store.applied;
    world.close();
    assert_eq!(world.store.applied, applied);
}
