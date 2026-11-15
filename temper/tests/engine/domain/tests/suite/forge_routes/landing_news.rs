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

fn candidate_repair() -> (World, u64, engine::Assignment) {
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
        })
        .clone();
    (world, change, repair)
}

#[test]
fn a_real_candidate_failure_delegates_an_own_head_repair_with_its_run_and_jobs() {
    let (world, change, repair) = candidate_repair();
    assert!(live_task(&world, repair.task).spec.words.starts_with(b"Repair landing gate 0 at "));
    let context = std::str::from_utf8(ci_words(&repair)).expect("failure context");
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
fn a_landing_repair_reads_gate_checks_at_its_head_and_candidate_and_follows_the_job_pin() {
    use super::super::change_tool::named_read;
    let (mut world, change, repair) = candidate_repair();
    let row = super::super::light_rules::change_row(&world, change);
    let head = row.change.last_head.expect("change head");
    let (_, temper_engine_domain_forge_change::Delegate::Repair(why)) = row.delegate.expect("repair") else {
        panic!("repair delegate")
    };
    let temper_engine_domain_forge_change::Repair::LandingAt { candidate, .. } = why else {
        panic!("candidate repair")
    };
    let failure = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Receipt { receipt, .. } if receipt.candidate == candidate && !receipt.passed => {
                    Some(receipt.clone())
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("receipt");
    assert_ne!(head, failure.candidate);
    let job = failure.jobs.first().expect("failed job");
    let expected_pin = format!(
        "read_job head {} run {} job {} attempt {}",
        hex(&failure.candidate),
        failure.run.id,
        job.job,
        job.attempt
    );
    let expected_status = format!("temper/gate/{}: failed", String::from_utf8_lossy(&job.name));
    let tasks::Party::Task(goal) = live_task(&world, change).requester else { panic!("goal") };
    let chat = world.assigned.iter().rev().find(|run| run.task == goal).expect("goal run").clone();
    for (index, commit) in [head, failure.candidate].into_iter().enumerate() {
        let raw_head = u64::from_be_bytes(commit[8..16].try_into().unwrap());
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: raw_head,
                context: b"own-check".as_slice().into(),
                state: raw::Check::Passed,
                description: Box::new([]),
                link: Box::new([]),
            }),
        );
        let input = format!(r#"{{"commit":"{}"}}"#, hex(&commit));
        let own = named_read(&mut world, &chat, 500 + u32::try_from(index).unwrap(), b"read_checks", input.as_bytes());
        let engine::CallAnswer::ForgeRead(ref own_result) = own else { panic!("forge checks") };
        let Ok(skein_forge::api::Answer::Checks(ref own_statuses)) = **own_result else { panic!("checks") };
        let checks =
            named_read(&mut world, &repair, 600 + u32::try_from(index).unwrap(), b"read_checks", input.as_bytes());
        let engine::CallAnswer::ForgeRead(ref result) = checks else { panic!("forge checks") };
        let Ok(skein_forge::api::Answer::Checks(ref statuses)) = **result else { panic!("checks") };
        assert!(!own_statuses.is_empty(), "the forge has the commit's own checks");
        assert_eq!(&statuses[..own_statuses.len()], own_statuses.as_ref());
        assert_eq!(statuses.len(), own_statuses.len() + failure.jobs.len());
        let rendered = temper_engine_smith::answer(&checks);
        let text = std::str::from_utf8(rendered.text()).expect("checks text");
        assert!(text.contains(&expected_status), "{text}");
        assert!(text.contains(std::str::from_utf8(&failure.run.url).unwrap()), "{text}");
        let pin = text.lines().find(|line| *line == expected_pin).expect("candidate job pin");
        let fields: Vec<_> = pin.split_ascii_whitespace().collect();
        let input =
            format!(r#"{{"head":"{}","run":{},"job":{},"attempt":{}}}"#, fields[2], fields[4], fields[6], fields[8]);
        let answer =
            named_read(&mut world, &repair, 510 + u32::try_from(index).unwrap(), b"read_job", input.as_bytes());
        let rendered = temper_engine_smith::answer(&answer);
        assert!(!rendered.error(), "{answer:?}");
        assert!(rendered.text().ends_with(b"failing check test 3\n"), "{}", String::from_utf8_lossy(rendered.text()));
    }
    let other = format!(r#"{{"commit":"{}"}}"#, hex(&failure.base.expect("round base")));
    let ordinary = named_read(&mut world, &chat, 520, b"read_checks", other.as_bytes());
    let repair_checks = named_read(&mut world, &repair, 620, b"read_checks", other.as_bytes());
    assert_eq!(repair_checks, ordinary, "another commit gets only forge checks");
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
    world.send(protocol::hello_with(
        Token::new(7),
        0,
        2,
        jig_domain_link::Bounds { message_bytes: 256, ..protocol::bounds() },
        Box::new([]),
    ));
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

#[test]
fn a_batch_request_grants_every_prefix_and_each_goal_hears_its_own_prefix() {
    let (mut world, goal, changes) = joined_batch();
    world.manual_prepare_jobs();
    world.send(protocol::hello_with(
        Token::new(7),
        0,
        2,
        jig_domain_link::Bounds { message_bytes: 256, ..protocol::bounds() },
        Box::new([]),
    ));
    for _ in 0..800 {
        world.tick();
        if !world.jobs.is_empty() {
            break;
        }
    }
    let job = world.jobs.first().expect("one stack prepare");
    let task = job.workspace.expect("workspace").workstream;
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == task).expect("landing");
    let branches: Vec<_> = row.landing.members.iter().map(|m| m.branch.clone()).collect();
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&job.spec),
    )
    .expect("job spec");
    let request: serde_json::Value = serde_json::from_slice(spec.request()).expect("worker request");
    assert_eq!(request["from"], hex(&row.landing.members[0].base.expect("base")));
    let members = request["members"].as_array().expect("members");
    assert_eq!(members.len(), 3);
    for (index, member) in members.iter().enumerate() {
        assert_eq!(member["number"], index + 1);
        assert_eq!(member["prefix"], String::from_utf8_lossy(&branches[index]).as_ref());
    }
    assert_eq!(
        world.job_workspaces.get(&job.run.raw()).expect("typed items").repositories[0].prefixes.as_ref(),
        branches
    );
    for branch in &branches {
        assert!(world.store.rows.values().any(|record| matches!(record, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Writer(slot))) if matches!(slot.writer, tasks::Writer::Run { .. }) && slot.resource.path.last().is_some_and(|last| *last == *branch))));
    }
    world.automatic_prepare_jobs();
    for _ in 0..2400 {
        world.tick();
        if ended(&world, task) && changes.iter().all(|change| ended(&world, *change)) {
            break;
        }
    }
    assert!(ended(&world, task), "batch procedure ends");
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == task).expect("landed batch");
    let top = row.landing.members[2].landed.expect("top landed");
    let lines = news(&world, goal.task, task);
    for (index, member) in row.landing.members.iter().enumerate() {
        let prefix = member.landed.expect("member landed");
        let proving = format!(
            "landing {task}: change {} proving at {} on {}{}",
            member.change,
            hex(&prefix),
            hex(&member.base.expect("base")),
            if index == 2 { String::new() } else { format!(", under {}", hex(&top)) }
        );
        assert_eq!(lines.iter().filter(|line| **line == proving).count(), 1, "{proving}: {lines:?}");
        assert_eq!(
            lines
                .iter()
                .filter(|line| **line == format!("landing {task}: change {} landed at {}", member.change, hex(&prefix)))
                .count(),
            1
        );
        assert!(world.fake.branch(b"org/repo", &member.branch).is_none(), "prefix released");
    }
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) =
        &task_row(&world, task).expect("ended").phase
    else {
        panic!("landing report")
    };
    for member in &row.landing.members {
        assert!(String::from_utf8_lossy(words).contains(&hex(&member.landed.expect("prefix"))));
    }
}

#[test]
fn batch_goals_hear_member_leaves_and_each_members_landing_hold() {
    for held in [false, true] {
        let (mut world, goal, changes) = joined_batch();
        if held {
            world.fake_env.limits.setup_faults = 1000;
        }
        world.send(protocol::hello_with(
            Token::new(7),
            0,
            2,
            jig_domain_link::Bounds { message_bytes: 256, ..protocol::bounds() },
            Box::new([]),
        ));
        let mut landing = None;
        for _ in 0..1200 {
            world.tick();
            if let Some(row) =
                landing_rows(&world).into_iter().find(|row| row.landing.phase == forge_top::LandingPhase::Proving)
            {
                landing = Some(row.landing.task);
                break;
            }
        }
        let landing = landing.expect("proving batch");
        let mut moved = None;
        if !held {
            let branch = super::light_rules::change_row(&world, changes[1]).branch.clone();
            let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"moved", b"new head", 1)
                .expect("outside delivery");
            moved = Some(translate::commit(head));
        }
        for _ in 0..2000 {
            world.tick();
            if held {
                if matches!(task_row(&world, landing).expect("task").phase, tasks::Phase::Held { .. })
                    && news(&world, goal.task, landing).iter().filter(|line| line.contains(": held:")).count() == 3
                {
                    break;
                }
            } else if ended(&world, landing) {
                break;
            }
        }
        let lines = news(&world, goal.task, landing);
        if held {
            let expected =
                format!("landing {landing}: held: gate gate of org/repo failed on its runner past its retries");
            assert_eq!(lines.iter().filter(|line| **line == expected).count(), 3);
        } else {
            assert!(ended(&world, landing));
            let why = format!("head moved to {}", hex(&moved.expect("moved head")));
            assert!(lines.contains(&format!("landing {landing}: change {} left: {why}", changes[1])), "{lines:?}");
            let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("ended batch");
            assert!(row.landing.members[0].landed.is_some() && row.landing.members[2].landed.is_some());
            let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) =
                &task_row(&world, landing).expect("ended").phase
            else {
                panic!("landing report")
            };
            assert!(String::from_utf8_lossy(words).contains(&format!("change {} left: {why}", changes[1])));
        }
    }
}
