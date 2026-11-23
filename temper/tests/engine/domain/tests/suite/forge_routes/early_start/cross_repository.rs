//! Cross-repository dependencies keep landing order, not producer order (testing/reference-case.md).
use super::*;

const APP: &[u8] = b"org/lib";

struct Pair {
    world: World,
    owner: engine::Assignment,
    kit: u64,
    app: u64,
    kit_producer: engine::Assignment,
    app_producer: engine::Assignment,
}

fn pair() -> Pair {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        STEPS,
        Gate { agent: true, ..Gate::default() },
        None,
        StoryLimits {
            across: true,
            slots: Some(WORKER_SLOTS),
            result_bytes: Some(RESULT_BYTES),
            ..StoryLimits::default()
        },
    );
    let rules = [b"repo".as_slice(), b"lib"].map(|repository| engine::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", repository, b"branch"].map(Box::from).into(),
            last: authority::Last::Exact(b"main".as_slice().into()),
        },
        ci: true,
        up_to_date: true,
        gates: Box::new([engine::Gate { number: 1, blocking: true, freshness: engine::Freshness::Exact }]),
        approvals: Box::new([]),
    });
    world.env.limits.tasks.batch = 2;
    world = world.with_landing_deployment(Box::new(rules));
    world.adopt();
    adopt_library(&mut world);
    world.send(protocol::hello_slots(CHANNEL, WORKER_SLOTS, Box::new([])));
    let request = world.chat(1, b"Land kit and repin app");
    let owner = assigned_chat(&mut world, 92, request);
    record_turn(&mut world, &owner);
    let input = smith_domain_run::HostInput::attested(br#"{"changes":[{"name":"e1","words":"Fix kit","repository":"org/repo"},{"words":"Repin app","repository":"org/lib","after":["e1"]}]}"#.as_slice().into()).expect("batch input");
    let temper_engine_smith::Decoded::Call(decoded) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: owner.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("batch decodes") else {
        panic!("change call")
    };
    let engine::CallAnswer::Delegated(made) = call(&mut world, &owner, decoded.tool) else {
        panic!("atomic pair admitted")
    };
    assert_eq!(made.len(), 2);
    let (kit, app) = (made[0].task, made[1].task);
    let kit_producer = questions::next_producer(&mut world, kit);
    let app_producer = questions::next_producer(&mut world, app);
    Pair { world, owner, kit, app, kit_producer, app_producer }
}

fn produce(world: &mut World, run: &engine::Assignment, repository: &[u8], file: &[u8]) -> u64 {
    record_turn(world, run);
    let branch = run.workspace.repositories[0].push.clone().expect("push branch");
    let base = world.fake.branch(repository, b"main").expect("base");
    assert_eq!(
        world.external_at(
            repository,
            2,
            raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: base, expected: None })
        ),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, repository, &branch, file, b"changed", 2).expect("push");
    let event = world.protocol.answer(
        CHANNEL,
        run.task,
        run.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: run.workspace.repositories[0].tag,
                words: b"Done: pushed.".as_slice().into(),
            },
            cancel_delegates: false,
        },
        pushed_parts(run, translate::commit(head)),
    );
    world.send(event);
    head
}

fn reviewer(world: &mut World, change: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if let Some((task, change::Delegate::Gate { .. })) = world.store.change_hot(change).expect("change").delegate
            && let Some(run) = world.assigned.iter().find(|run| run.task == task)
        {
            return run.clone();
        }
    }
    panic!("review assigned for {change}");
}

fn record_turn(world: &mut World, run: &engine::Assignment) {
    world.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 0, None, &run.task.to_be_bytes()));
}

fn approve(world: &mut World, reviewer: &engine::Assignment) {
    record_turn(world, reviewer);
    reviews::finish_review(
        world,
        reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"Reviewed".as_slice().into() },
    );
}

#[test]
fn one_atomic_batch_names_two_repositories_and_the_repins_dependency() {
    let Pair { mut world, owner, kit, app, kit_producer, app_producer } = pair();
    assert_eq!(live_task(&world, kit).requester, tasks::Party::Task(owner.task));
    assert_eq!(live_task(&world, app).requester, tasks::Party::Task(owner.task));
    assert_eq!(live_task(&world, app).dependencies.as_ref(), &[kit]);
    for (task, words) in [(kit, b"Fix kit".as_slice()), (app, b"Repin app".as_slice())] {
        assert_eq!(live_task(&world, task).project, 1);
        assert_eq!(live_task(&world, task).revisions.spec, 0);
        assert!(live_task(&world, task).spec.words.starts_with(words));
    }
    assert_eq!(kit_producer.workspace.repositories[0].provider, forge_world::REPO);
    assert_eq!(
        app_producer.workspace.repositories[0].provider,
        skein_forge::api::Repository { forge: 1, repository: 4 }
    );
    stop(&mut world, &owner);
}

#[test]
fn the_repin_produces_during_exact_head_review_and_finishes_after_its_dependency_without_relay() {
    let Pair { mut world, owner, kit, app, kit_producer, app_producer } = pair();
    let head = produce(&mut world, &kit_producer, b"org/repo", b"kit");
    let review = reviewer(&mut world, kit);
    reviews::assert_review_head(&world, &review, translate::commit(head));
    assert!(matches!(live_task(&world, app_producer.task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    produce(&mut world, &app_producer, APP, b"pin");
    let app_review = reviewer(&mut world, app);
    approve(&mut world, &app_review);
    world.until(Until::Ready);
    assert!(!ended(&world, kit));
    assert!(!ended(&world, app));
    assert_eq!(live_task(&world, app).waiting_on.as_ref(), &[kit]);
    assert_eq!(world.merges, 0);
    approve(&mut world, &review);
    landed(&mut world, app);
    assert!(ended(&world, kit));
    assert_eq!(goal_landing::ended(&world, app).expect("app ended").revisions.spec, 0);
    assert_eq!(world.assigned.iter().filter(|run| run.task == app_producer.task).count(), 1);
    assert!(
        !world.delivered_messages.keys().any(|(task, _)| *task == app_producer.task),
        "the producer needs no targeted relay"
    );
    stop(&mut world, &owner);
}

fn landed_pair() -> (Pair, [u64; 2]) {
    let mut pair = pair();
    let kit_head = produce(&mut pair.world, &pair.kit_producer, b"org/repo", b"kit");
    let kit_review = reviewer(&mut pair.world, pair.kit);
    let app_head = produce(&mut pair.world, &pair.app_producer, APP, b"pin");
    let app_review = reviewer(&mut pair.world, pair.app);
    approve(&mut pair.world, &app_review);
    approve(&mut pair.world, &kit_review);
    landed(&mut pair.world, pair.app);
    (pair, [kit_head, app_head])
}

#[test]
fn both_repositories_land_their_pushed_green_exactly_reviewed_heads_in_dependency_order() {
    let (Pair { mut world, owner, kit, app, .. }, heads) = landed_pair();
    let mut observations = Vec::new();
    while let Some(observation) = world.fake.pop_observation() {
        observations.push(observation);
    }
    assert_eq!(world.fake.observations_lost(), 0);
    let merges: Vec<_> = observations
        .iter()
        .enumerate()
        .filter_map(|(at, observation)| {
            if let fake::Observation::Merged { repository, head, .. } = observation {
                Some((at, repository.as_ref(), *head))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        merges.iter().map(|(_, repo, head)| (*repo, *head)).collect::<Vec<_>>(),
        [(b"org/repo".as_slice(), heads[0]), (APP, heads[1])]
    );
    for (at, repository, head) in merges {
        let latest = observations[..at].iter().rev().find_map(|observation| {
            if let fake::Observation::Reported { repository: repo, commit, state, .. } = observation
                && repo.as_ref() == repository
                && *commit == head
            {
                Some(*state)
            } else {
                None
            }
        });
        assert_eq!(latest, Some(raw::Check::Passed), "the merged head is already green");
    }
    for ((task, repository), head) in [(kit, b"org/repo".as_slice()), (app, APP)].into_iter().zip(heads) {
        let row = change_row(&world, task);
        assert_eq!(row.change.last_head, Some(translate::commit(head)));
        assert_eq!(row.verdicts.len(), 1);
        assert_eq!(row.verdicts[0].head, translate::commit(head));
        assert_eq!(row.verdicts[0].status, change::Status::Passed);
        let raw::Answer::Pull(pull) =
            world.external_at(repository, 1, raw::Op::Read(raw::Read::Pull { number: row.pull.expect("pull") }))
        else {
            panic!("pull")
        };
        assert!(pull.merged.is_some());
        assert_eq!(pull.commit, head);
    }
    stop(&mut world, &owner);
}

#[test]
fn both_change_results_name_their_repositorys_merge() {
    let (Pair { mut world, owner, kit, app, .. }, _) = landed_pair();
    for (task, repository) in [(kit, b"org/repo".as_slice()), (app, APP)] {
        let row = change_row(&world, task);
        let change::State::Landed { merge, .. } = row.change.state else { panic!("landed") };
        assert_eq!(Some(merge), world.fake.branch(repository, b"main").map(translate::commit));
        let ended = goal_landing::ended(&world, task).expect("ended task");
        let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) = &ended.phase else {
            panic!("change result")
        };
        let hex = engine::commit_text(&merge);
        assert!(words.windows(repository.len()).any(|part| part == repository), "result names its repository");
        assert!(words.windows(hex.len()).any(|part| part == hex.as_ref()), "result names the merge: {words:?}");
    }
    stop(&mut world, &owner);
}

#[test]
fn a_completed_pair_keeps_complete_successful_journals_and_no_live_tasks() {
    let (Pair { mut world, owner, .. }, _) = landed_pair();
    goal_landing::finish_run(&mut world, &owner, tasks::TaskResult::Report { words: b"Both landed".as_slice().into() });
    world.until(Until::Ready);
    assert!(
        !world
            .store
            .rows
            .values()
            .any(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
    );
    for run in &world.assigned {
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Attempt { task: run.task, attempt: run.attempt }));
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(attempt)))) =
            world.store.rows.get(&key)
        else {
            panic!("durable attempt")
        };
        assert_eq!(attempt.end, Some(tasks::EndKind::Finished));
        let turns: Vec<_> = world
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) = row
                    && turn.task == run.task
                    && turn.attempt == run.attempt
                {
                    Some(turn)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(turns.len(), 1, "each run has its whole committed journal");
        assert_eq!(turns[0].transcript.as_ref(), run.task.to_be_bytes());
    }
    world.close();
}
