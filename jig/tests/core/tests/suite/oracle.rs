use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

#[test]
fn a_parked_conversation_keeps_its_documents_and_transcript() {
    let mut world = World::new(Setup { seed: 1901, ..Setup::default() });
    world.turn(1, 1, None, b"durable answer");
    world.end(tasks::End::Parked, 1);
    world.close();
}

#[test]
fn a_writer_forgetting_a_durable_task_field_names_its_document() {
    let mut world = World::new(Setup { seed: 1902, ..Setup::default() });
    world.end(tasks::End::Parked, 0);
    let task = world.assigned.as_ref().expect("assigned task").task;
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(mut row))) = world.store.rows[&key].clone() else {
        panic!("live task");
    };
    row.spec.words = b"writer forgot its answer".as_slice().into();
    world.seed(Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))]));
    let finding = world.finish().expect_err("writer omitted a durable field");
    assert!(finding.contains("restart oracle document Task 1 field chip"), "{finding}");
    let applied = world.store.applied;
    assert_eq!(world.finish(), Ok(()), "the finding is returned only once");
    assert_eq!(world.store.applied, applied);
}

#[test]
fn a_readback_finding_reaches_finish_without_a_second_restart() {
    let mut world = World::new(Setup { seed: 1910, ..Setup::default() });
    world.turn(1, 1, None, b"durable answer");
    world.end(tasks::End::Parked, 1);
    world.store.unread_transcript(1);
    let finding = world.finish().expect_err("the committed transcript is unread");
    assert!(finding.contains("restart oracle readback: Transcript { task: 1, backwards: false"), "{finding}");
    assert!(finding.contains("terminal: Unread"), "{finding}");
    let applied = world.store.applied;
    assert_eq!(world.finish(), Ok(()));
    assert_eq!(world.store.applied, applied);
}

const PARKED_TASK: u64 = 1;
const CALLER_SIGN_IN: u64 = 1;
const READER_SIGN_IN: u64 = 2;

fn parked_conversation(seed: u64, keep_reads: bool) -> World {
    let mut world = World::new(Setup { seed, keep_reads, ..Setup::default() });
    world.turn(1, 1, None, b"durable answer");
    world.end(tasks::End::Parked, 1);
    world
}

#[test]
fn closing_oracle_agrees_with_and_without_read_reuse() {
    const SEED: u64 = 1913;
    let close = |keep_reads| {
        let mut world = parked_conversation(SEED, keep_reads);
        world.send(jig_core_world::fixtures::sign_in(skein_lib::Token::new(902), 0, b"another", b"reader", b"Reader"));
        world.finish().expect("closing oracle");
        (world.closed_documents.take().expect("oracle read set and answers"), world.store.applied)
    };
    let expected = close(true);
    let read_set = [
        (CALLER_SIGN_IN, None),
        (CALLER_SIGN_IN, Some(PARKED_TASK)),
        (READER_SIGN_IN, None),
        (READER_SIGN_IN, Some(PARKED_TASK)),
    ];
    assert!(expected.0.before.keys().copied().eq(read_set));
    assert!(expected.0.after.keys().copied().eq(read_set));
    assert_eq!(close(false), expected);
}

#[test]
fn a_read_transcript_still_fails_after_a_frame_changes() {
    const SEED: u64 = 1914;
    let finish = |keep_reads| {
        let mut world = parked_conversation(SEED, keep_reads);
        world.store.transcript(PARKED_TASK, false).expect("transcript before changing a frame");
        world.store.unread_transcript(PARKED_TASK);
        let finding = world.finish().expect_err("the committed transcript is unread");
        assert!(finding.contains("restart oracle readback: Transcript { task: 1, backwards: false"), "{finding}");
        assert!(finding.contains("terminal: Unread"), "{finding}");
        let applied = world.store.applied;
        assert_eq!(world.finish(), Ok(()));
        assert_eq!(world.store.applied, applied);
        finding
    };
    assert_eq!(finish(true), finish(false));
}

#[test]
fn a_forgotten_chats_remains_can_disappear_on_restart() {
    use crate::read_support::request;
    use jig_domain_people as people;

    let mut world = crate::read_support::forgettable_world(1911);
    let task = world.task();
    world.turn(1, 1, None, b"reply");
    request(&mut world, 71, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    jig_core_world::fixtures::finish_chat(&mut world, 1, b"done");
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(ended))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
    else {
        panic!("ended chat")
    };
    let through = ended.result_position;
    request(&mut world, 70, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    let summary = root::Key::Core(core::Key::Tasks(tasks::Key::Summary(task)));
    assert!(world.store.rows.contains_key(&summary));
    assert_eq!(world.finish(), Ok(()));
    assert!(!world.store.rows.contains_key(&summary));
}

#[test]
fn a_summary_without_a_due_forgotten_row_cannot_disappear_on_restart() {
    let mut world = crate::retention::world(1912, false);
    let record = crate::retention::finish(&mut world);
    crate::retention::read(&mut world, &record);
    crate::retention::at(&mut world, &record, 31);
    crate::retention::pass(&mut world);
    assert!(crate::retention::summary(&world, record.number));
    assert!(!world.store.rows.keys().any(|key| matches!(
        key,
        root::Key::Core(core::Key::Tasks(tasks::Key::Retention { stage: tasks::RetentionStage::Forgotten, .. }))
    )));
    // Only wall time moves: the next pass starts at the cold restart.
    crate::retention::at(&mut world, &record, 366);
    let finding = world.finish().expect_err("summary expiry is not a forgotten-task exception");
    assert!(finding.contains("restart oracle document Task 1 field answer"), "{finding}");
    assert!(finding.contains("expected None(Gone), found None(Unknown)"), "{finding}");
}

#[test]
fn a_deliberately_broken_fixture_skips_the_oracle() {
    let mut world = World::new(Setup { seed: 1903, ..Setup::default() });
    world.stopped = true;
    // The fixture deliberately leaves the engine stopped.
    world.broken();
    world.close();
}

#[test]
fn a_dead_hosts_runtime_observation_is_rebuilt() {
    use jig_domain_fleet as fleet;
    use skein_lib::Token;
    let mut world = World::new(Setup { seed: 1904, ..Setup::default() });
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
fn the_restart_oracle_compares_only_durable_wait_projections() {
    use jig_core_world::{reads, stories::instructions::task_document};
    use skein_lib::{Duration, Token};

    let mut world = World::new(Setup { seed: 1915, ..Setup::default() });
    let (task, attempt) = (world.task(), world.attempt());
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Limited {
        channel: Token::new(7),
        run: Token::new(task),
        attempt: Token::new(attempt),
        account: 1,
        reset: Some(Duration::from_secs(30)),
    })));
    world.end(tasks::End::Failed(tasks::Class::Run), 0);
    world.reads = Some(reads::Parties::new(world.store.applied));
    let before = task_document(&mut world, task, 1);
    let wait = before.wait.expect("the next run's wait is first");
    assert_eq!(wait.kind, core::WaitKind::Account);
    let core::Bound::At { at, then } = wait.bound else { panic!("account wait has a bound") };
    assert_eq!(before.nearest, Some(core::Nearest { at, then, kind: wait.kind }));
    world.finish().expect("the restart can rebuild the account wait");
    let after = task_document(&mut world, task, 2);
    assert_eq!(after.wait.expect("cold host wait").kind, core::WaitKind::Host);
    assert_eq!(after.nearest.expect("cold host bound").kind, core::WaitKind::Host);

    let mut world = World::new(Setup { seed: 1916, ..Setup::default() });
    let (task, attempt) = (world.task(), world.attempt());
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Observe {
        channel: Token::new(7),
        run: Token::new(task),
        attempt: Token::new(attempt),
        report: jig_domain_fleet::Report::Activity(jig_domain_fleet::Activity::Waiting),
    })));
    world.reads = Some(reads::Parties::new(world.store.applied));
    let before = task_document(&mut world, task, 1);
    assert_eq!(before.wait.expect("observed park's first wait").kind, core::WaitKind::News);
    world.finish().expect("a live observation does not survive restart");
    let after = task_document(&mut world, task, 2);
    assert!(after.wait.is_none());

    let mut world = World::new(Setup { seed: 1917, ..Setup::default() });
    world.park_for_message(0);
    let mut row = jig_core_world::fixtures::live_row(&world, PARKED_TASK).clone();
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Idle));
    row.phase_since = skein_lib::Wall::from_nanos(row.phase_since.as_nanos() - 1);
    world.seed(Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(row))))]));
    let finding = world.finish().expect_err("the idle task's durable news wait changed");
    assert!(finding.contains("restart oracle document Task 1 field wait ("), "{finding}");
}

#[test]
fn the_restart_sweep_erases_only_the_expired_sign_in() {
    use jig_domain_people as people;
    let mut world = World::new(Setup { seed: 1905, ..Setup::default() });
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
    let mut world = World::new(Setup { seed: 1906, ..Setup::default() });
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    world.turn(1, 1, None, b"held answer");
    assert!(!world.store.held.is_empty());
    world.close();
    assert!(world.store.held.is_empty());
    assert!(world.store.pending.is_empty());
}

#[test]
fn an_explicit_close_runs_only_one_cold_restart() {
    let mut world = World::new(Setup { seed: 1909, ..Setup::default() });
    world.close();
    let applied = world.store.applied;
    world.close();
    assert_eq!(world.store.applied, applied);
}
