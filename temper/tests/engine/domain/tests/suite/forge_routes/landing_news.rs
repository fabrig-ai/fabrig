//! The member's goal hears every committed transition (domain/landing.md, 4.2).
use super::*;

fn produced_goal() -> (World, u64, u64) {
    produced_goal_with_checks(1000)
}
fn produced_goal_with_checks(passes: u32) -> (World, u64, u64) {
    let world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, dispatch_passes: Some(passes), ..StoryLimits::default() },
    );
    let (mut world, goal, producer, _) = change_world_in(world, Change { tracked: true, ..Change::default() });
    world.send(hello_prepare(2));
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("change requester") };
    push_change(&mut world, &producer, b"org/repo");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"base moved", 2)
        .expect("base moves");
    finish_producer(&mut world, &producer, forge_world::REPO);
    (world, goal.task, change)
}
fn hex(commit: &[u8; 32]) -> String {
    String::from_utf8(engine::commit_text(commit).into_vec()).expect("hex commit")
}
fn news(world: &World, goal: u64, landing: u64) -> Vec<String> {
    let prefix = format!("landing {landing}: ");
    let mut lines: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| {
            let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(row))) = record else {
                return None;
            };
            if row.task != goal {
                return None;
            }
            let tasks::Line::Heard(message) = &row.line else { return None };
            if !matches!(message.kind, tasks::MessageKind::News { .. }) || !message.words.starts_with(prefix.as_bytes())
            {
                return None;
            }
            Some((row.position, String::from_utf8(message.words.to_vec()).expect("landing news")))
        })
        .collect();
    lines.sort_by_key(|(position, _)| *position);
    lines.into_iter().map(|(_, words)| words).collect()
}
fn joined(landing: u64, change: u64) -> String {
    format!("landing {landing}: change {change} joined, into org/repo main, round 1")
}
fn proving(row: &forge_top::LandingRow, change: u64) -> String {
    let member = &row.landing.members[0];
    format!(
        "landing {}: change {change} proving at {} on {}",
        row.landing.task,
        hex(&member.candidate.expect("candidate")),
        hex(&member.base.expect("base"))
    )
}
#[test]
fn a_members_goal_hears_joined_proving_and_landed_in_order() {
    let (mut world, goal, change) = produced_goal();
    for _ in 0..800 {
        world.prepare_tick();
        if task_row(&world, change).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_))) {
            break;
        }
    }
    let row = landing_rows(&world).into_iter().find(|row| row.landing.members[0].change == change).expect("landing");
    let landing = row.landing.task;
    let prefix = row.landing.members[0].landed.expect("landed prefix");
    assert_eq!(
        news(&world, goal, landing),
        [
            joined(landing, change),
            proving(row, change),
            format!("landing {landing}: change {change} landed at {}", hex(&prefix))
        ]
    );
}
#[test]
fn a_members_goal_hears_joined_proving_and_left_and_the_result_names_why() {
    let (mut world, goal, change) = produced_goal();
    for _ in 0..600 {
        world.prepare_tick();
        if landing_rows(&world).iter().any(|row| row.landing.phase == forge_top::LandingPhase::Proving) {
            break;
        }
    }
    let proving_row = landing_rows(&world).into_iter().next().expect("proving landing");
    let landing = proving_row.landing.task;
    let proving_words = proving(proving_row, change);
    cancel_task(&mut world, change);
    for _ in 0..600 {
        world.tick();
        if ended(&world, landing) {
            break;
        }
    }
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert!(row.landing.members[0].landed.is_none());
    assert_eq!(
        news(&world, goal, landing),
        [joined(landing, change), proving_words, format!("landing {landing}: change {change} left: cancelled")]
    );
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) =
        &task_row(&world, landing).expect("ended landing").phase
    else {
        panic!("landing report")
    };
    assert!(String::from_utf8_lossy(words).contains(&format!("change {change} left: cancelled")));
}
#[test]
fn a_members_goal_hears_the_runner_bound_and_the_landing_holds_with_that_account() {
    let (mut world, goal, change) = produced_goal();
    world.fake_env.limits.setup_faults = 1000;
    for _ in 0..1600 {
        world.prepare_tick();
        if landing_rows(&world).iter().any(|row| matches!(row.landing.phase, forge_top::LandingPhase::Held { .. })) {
            break;
        }
    }
    let row = landing_rows(&world).into_iter().next().expect("held landing");
    let landing = row.landing.task;
    let why = "gate gate of org/repo failed on its runner past its retries";
    assert!(matches!(
        row.landing.phase,
        forge_top::LandingPhase::Held { why: forge_top::LandingHold::Faults { .. }, .. }
    ));
    assert_eq!(
        news(&world, goal, landing),
        [joined(landing, change), proving(row, change), format!("landing {landing}: held: {why}")]
    );
    let tasks::Phase::Held { source: Some(source), .. } = &task_row(&world, landing).expect("held task").phase else {
        panic!("held task")
    };
    assert!(matches!(source.as_ref(), tasks::HoldSource::Account { words, .. } if words.as_ref() == why.as_bytes()));
}

#[test]
fn a_real_candidate_failure_delegates_an_own_head_repair_with_its_run_and_jobs() {
    let (mut world, _, change) = produced_goal_with_checks(0);
    for _ in 0..1200 {
        world.prepare_tick();
        if world.assigned.iter().any(|run| {
            task_row(&world, run.task).is_some_and(|row| row.spec.words.starts_with(b"Repair landing gate 0 at "))
        }) {
            break;
        }
    }
    let repair = world
        .assigned
        .iter()
        .find(|run| {
            task_row(&world, run.task).is_some_and(|row| row.spec.words.starts_with(b"Repair landing gate 0 at "))
        })
        .unwrap_or_else(|| {
            panic!(
                "candidate failure assigned a repair: {:?}",
                world
                    .assigned
                    .iter()
                    .map(|run| task_row(&world, run.task).map(|row| &row.spec.words))
                    .collect::<Vec<_>>()
            )
        });
    assert!(live_task(&world, repair.task).spec.words.starts_with(b"Repair landing gate 0 at "));
    let context = std::str::from_utf8(ci_words(repair)).expect("failure context");
    for words in
        ["Gate: gate", "Run: ", "Candidate: ", "Base: ", "Failed job: test", "Failed step: ", "End of its log:"]
    {
        assert!(context.contains(words), "{words}: {context}");
    }
    let row = super::super::light_rules::change_row(&world, change);
    let own_head = row.change.last_head.expect("own head");
    assert!(
        matches!(row.change.state, temper_engine_domain_forge_change::State::Repairing { why: temper_engine_domain_forge_change::Repair::LandingAt { candidate, .. }, .. } if candidate != own_head)
    );
    assert!(
        matches!(&repair.workspace.repositories[0].start, engine::ForgeStart::Branch(branch) if *branch == row.branch)
    );
}

#[test]
fn every_batched_members_goal_hears_joined_with_its_ordered_peers() {
    let (mut world, goal, changes) = joined_batch();
    let row = landing_rows(&world)[0];
    let landing = row.landing.task;
    assert_eq!(row.landing.members.iter().map(|member| member.change).collect::<Vec<_>>(), changes);
    for (index, member) in row.landing.members.iter().enumerate() {
        assert_eq!(member.unit, u16::try_from(index).unwrap());
        let expected = if index == 0 {
            format!("temper/{landing}/l{landing}")
        } else {
            format!("temper/{landing}/l{landing}-{}", index + 1)
        };
        assert_eq!(member.branch.as_ref(), expected.as_bytes());
        let task = task_row(&world, landing).expect("landing task");
        assert_eq!(task.holdings.len(), 3, "sealed branches held from making");
        assert!(task.holdings.iter().any(|holding| matches!(holding, tasks::Holding::Write { resource, .. } if resource.path.last().is_some_and(|branch| branch.as_ref() == expected.as_bytes()))));
    }
    let expected: Vec<_> = changes
        .iter()
        .map(|change| {
            let peers =
                changes.iter().filter(|peer| *peer != change).map(ToString::to_string).collect::<Vec<_>>().join(", ");
            format!("landing {landing}: change {change} joined, into org/repo main, round 1, with changes {peers}")
        })
        .collect();
    // Every member in this goal's three-change plan sends its own joined line.
    for _ in 0..100 {
        if news(&world, goal.task, landing).len() == changes.len() {
            break;
        }
        world.tick();
    }
    assert_eq!(news(&world, goal.task, landing), expected);
}

fn joined_batch() -> (World, engine::Assignment, Vec<u64>) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, landing_members: Some(0), batch: Some(3), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_with(Token::new(7), 0, 2, jig_domain_link::Bounds { message_bytes: 256, ..protocol::bounds() }, Box::new([])));
    let mut changes = Vec::new();
    let ask = walking::goal_from(
        world.chat(1, b"make three fixes"),
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 200, priority: 3, deadline: None },
    );
    let goal = assigned_chat(&mut world, 92, ask);
    for call in [94, 95, 96] {
        changes.push(ready_goal_change(&mut world, &goal, call));
    }
    for _ in 0..600 {
        world.tick();
    }
    assert!(landing_rows(&world).is_empty(), "full room leaves all three waiting");
    for change in &changes {
        assert!(matches!(
            super::light_rules::change_row(&world, *change).change.state,
            temper_engine_domain_forge_change::State::Ready { .. }
        ));
    }
    world.restart_with_batch(3);
    for _ in 0..600 {
        world.tick();
        if !landing_rows(&world).is_empty() {
            break;
        }
    }
    assert!(!landing_rows(&world).is_empty(), "restart forms the waiting batch");
    (world, goal, changes)
}

fn ready_goal_change(world: &mut World, goal: &engine::Assignment, call: u64) -> u64 {
    let mut authority = live_task(world, goal.task).authority.clone();
    authority.delegation.tasks = 1;
    authority.delegation.depth = 1;
    authority.budget.spend = 20;
    let delegate = engine::Delegate {
        executor: tasks::Executor::Procedure { connector: 0, code: 2 },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"Small fix".as_slice().into(),
            parameters: Box::new([
                tasks::Parameter::Resource {
                    name: 1,
                    connector: 0,
                    resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                },
                tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
            ]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Change { connector: 0, kind: 1, words: world.env.limits.tasks.result_bytes },
        authority,
        ..temper_engine_domain_world::fixtures::report_delegate(b"fix", Box::new([]))
    };
    let engine::CallAnswer::Delegated(made) =
        world.scripted_call(goal, call, engine::Tool::Delegate { batch: Box::new([delegate]) })
    else {
        panic!("change")
    };
    let change = made[0].task;
    for _ in 0..600 {
        if world
            .assigned
            .iter()
            .any(|run| task_row(world, run.task).is_some_and(|row| row.requester == tasks::Party::Task(change)))
        {
            break;
        }
        world.tick();
    }
    let producer = world
        .assigned
        .iter()
        .find(|run| task_row(world, run.task).is_some_and(|row| row.requester == tasks::Party::Task(change)))
        .expect("producer")
        .clone();
    push_change(world, &producer, b"org/repo");
    finish_producer(world, &producer, forge_world::REPO);
    for _ in 0..100 {
        world.tick();
    }
    change
}
