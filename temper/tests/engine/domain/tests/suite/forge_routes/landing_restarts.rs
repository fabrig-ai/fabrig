//! Root restarts retain worker preparation, proof and the turn, and reload batch size
//! (temper/docs/design/domain/landing.md, 4.1, 5, 8, 9, 15–16).
use super::super::goal_landing;
use super::*;

const SLOTS: u32 = 9;

fn worker_hello(world: &World) -> engine::Event {
    let mut event = hello_prepare(SLOTS);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    event
}

fn turn_world() -> World {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            turn: true,
            // The provider and store admit four; the initial project's policy below admits one.
            batch: Some(4),
            landing_members: Some(4),
            slots: Some(SLOTS),
            tasks: Some(16),
            authority_batch: Some(4),
            spec_bytes: Some(2048),
            result_bytes: Some(1024),
            inbox_messages: Some(8),
            ..StoryLimits::default()
        },
    )
    .with_landing_batch(1)
    .with_forge_holds(16);
    let mut client = world.env.limits.forge.client;
    client.pending = 8;
    world = world.with_forge_client(client);
    world.adopt();
    world.send(worker_hello(&world));
    world.manual_prepare_jobs();
    world
}

fn tick(world: &mut World) {
    // Decisions take no simulated time; idle waits advance to the next peer or root timer.
    let interval = world.tick_interval;
    world.tick_interval = Duration::ZERO;
    if world.events.is_empty() && world.store.pending.is_empty() {
        let next = if world.pending.is_empty() {
            world.root.quiescent().then(|| world.root.next_deadline(&world.env)).flatten()
        } else {
            world.fake.next_deadline()
        };
        if let Some(next) = next {
            world.env.now = world.env.now.max(next);
        }
    }
    world.prepare_tick();
    world.tick_interval = interval;
}

fn restart(world: &mut World) {
    world.drain_cut();
    world.restart_same();
    world.send(worker_hello(world));
}

fn pushed_change(world: &mut World, owner: &engine::Assignment, call: u64) -> (u64, engine::Assignment) {
    let (change, producer) = make_change(world, owner, call, b"org/repo");
    let branch = producer.workspace.repositories[0].push.as_ref().expect("producer branch").clone();
    let base = world.fake.branch(b"org/repo", b"main").expect("main");
    world.external(1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base }));
    // Distinct paths stack without conflicts and keep successive landings' keys different.
    fake::advance(
        &mut world.fake,
        &world.fake_env,
        b"org/repo",
        &branch,
        format!("fix-{call}").as_bytes(),
        b"fixed",
        1,
    )
    .expect("distinct change");
    (change, producer)
}

fn assert_unheld(world: &World) {
    for row in landing_rows(world) {
        assert!(!matches!(row.landing.phase, forge_top::LandingPhase::Held { .. }), "{row:?}");
        for task in std::iter::once(row.landing.task).chain(row.landing.members.iter().map(|member| member.change)) {
            let task = task_row(world, task).expect("task");
            assert!(!matches!(task.phase, tasks::Phase::Held { .. }), "task {}: {:?}", task.number, task.phase);
        }
    }
}

fn await_prepare(world: &mut World) -> u64 {
    for _ in 0..PAIR_TICKS {
        tick(world);
        assert_unheld(world);
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert_eq!(world.jobs.len(), 1, "one worker prepare");
    let job = &world.jobs[0];
    assert_eq!(job.kind, jig_domain_link::JobKind(1));
    let landing = job.workspace.expect("prepare workspace").workstream;
    assert!(world.job_workspaces.contains_key(&job.run.raw()), "worker has the workspace");
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Preparing);
    landing
}

fn answer_prepare(world: &mut World) {
    // The world's worker stacks from the assigned workspace and answers through answer_job.
    world.automatic_prepare_jobs();
    tick(world);
    world.manual_prepare_jobs();
    assert!(world.jobs.is_empty(), "worker answered the assigned prepare");
}

fn await_landed(world: &mut World, landing: u64) {
    for _ in 0..PAIR_TICKS {
        tick(world);
        assert_unheld(world);
        assert!(world.jobs.is_empty(), "no second prepare job");
        let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
        if ended(world, landing) && row.landing.members.iter().all(|member| ended(world, member.change)) {
            return;
        }
    }
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    panic!(
        "landing {landing} did not finish at {:?}: {:?}; members {:?}",
        world.env.wall,
        row.landing.phase,
        row.landing
            .members
            .iter()
            .map(|member| (member.change, member.landed.is_some(), member.told, ended(world, member.change)))
            .collect::<Vec<_>>()
    );
}

fn assert_landed(world: &World, landing: u64, changes: &[u64]) {
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(row.landing.preparation, 1);
    assert_eq!(row.landing.members.iter().map(|member| member.change).collect::<Vec<_>>(), changes);
    for member in &row.landing.members {
        assert!(member.landed.is_some() && member.told);
        assert_eq!(member.left, None);
        assert!(ended(world, member.change));
        let change = goal_landing::row(world, member.change);
        assert_eq!(change.change.repairs, 0);
        assert!(matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }));
    }
    assert_eq!(
        world
            .job_workspaces
            .values()
            .filter(|workspace| workspace.repositories.iter().any(|item| item.prefixes.iter().any(|branch| row
                .landing
                .members
                .iter()
                .any(|member| member.branch == *branch))))
            .count(),
        1,
        "one prepare assignment per landing"
    );
    let top = row.landing.members.last().expect("top");
    assert_eq!(world.fake.branch(b"org/repo", b"main"), fake::commit_count(&top.landed.expect("top landed")));
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Dispatch { branch, .. } if *branch == top.branch))
            .count(),
        1,
        "one dispatch at the top"
    );
    assert_eq!(
        world.writes.iter().filter(|write| matches!(write, api::Write::Merge { number, how: api::MergeHow::FastForwardOnly, .. } if Some(*number) == row.landing.round.pull)).count(),
        1,
        "one fast-forward"
    );
}

fn running_proof(world: &mut World, landing: u64) -> u64 {
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("proving landing");
    let candidate = row.landing.members[0].candidate.expect("candidate");
    let raw::Answer::Runs(runs) = world.external(
        1,
        raw::Op::Read(raw::Read::Runs {
            workflow: Some(b"gate.yml".as_slice().into()),
            commit: Some(fake::commit_count(&candidate).expect("fake candidate")),
            number: None,
            page: 1,
            limit: 2,
        }),
    ) else {
        panic!("dispatched runs")
    };
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state, raw::RunState::Running, "the proof is in flight at the cut");
    runs[0].id
}

#[derive(Clone, Copy)]
enum Cut {
    Preparing,
    Proving,
    Turn,
}

#[test]
fn an_engine_restarts_in_each_turn_landing_phase() {
    let mut world = turn_world();
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(250) };
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    for (call, cut) in [(95, Cut::Preparing), (96, Cut::Proving), (97, Cut::Turn)] {
        let (change, producer) = pushed_change(&mut world, &chat, call);
        finish_producer(&mut world, &producer, forge_world::REPO);
        let landing = await_prepare(&mut world);
        let mut proof = None;
        if matches!(cut, Cut::Preparing) {
            let job = (world.jobs[0].run, world.jobs[0].attempt);
            restart(&mut world);
            assert_eq!(world.jobs.len(), 1);
            assert_eq!((world.jobs[0].run, world.jobs[0].attempt), job);
            answer_prepare(&mut world);
        } else {
            answer_prepare(&mut world);
            if matches!(cut, Cut::Turn) {
                world.hold_write_answers = Some(HoldWrite::Merge);
            }
            for _ in 0..PAIR_TICKS {
                tick(&mut world);
                assert_unheld(&world);
                assert!(world.jobs.is_empty(), "no second prepare");
                let ready = match cut {
                    Cut::Preparing => unreachable!(),
                    Cut::Proving => world.writes.iter().any(|write| matches!(write, api::Write::Dispatch { branch, .. } if *branch == landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landing").landing.round.branch)),
                    Cut::Turn => !world.held_write_answers.is_empty(),
                };
                if ready {
                    break;
                }
            }
            world.drain_cut();
            let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("cut landing");
            match cut {
                Cut::Preparing => unreachable!(),
                Cut::Proving => {
                    assert_eq!(row.landing.phase, forge_top::LandingPhase::Proving);
                    proof = Some(running_proof(&mut world, landing));
                }
                Cut::Turn => {
                    assert!(matches!(row.landing.phase, forge_top::LandingPhase::Turn { .. }));
                    assert_eq!(world.held_write_answers.len(), 1, "merge made, answer held");
                }
            }
            world.restart_same();
            world.forge_reads = Some(Vec::new());
            world.send(worker_hello(&world));
            world.release_write_answers();
        }
        await_landed(&mut world, landing);
        assert_landed(&world, landing, &[change]);
        if let Some(run) = proof {
            assert!(
                world
                    .forge_reads
                    .as_ref()
                    .expect("recorded reads")
                    .iter()
                    .any(|(_, read)| matches!(read, api::Read::Run { run: found } if *found == run)),
                "restart finds the same dispatched run"
            );
        }
    }
    assert_eq!(landing_rows(&world).len(), 3);
    assert_eq!(world.job_workspaces.len(), 3);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(), 3);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Merge { .. })).count(), 3);
    assert_eq!(world.assigned.len(), 4, "one chat and three producers, no repairs");
    world.close();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one restart couples four goals' readiness, preparation and per-goal landing news"
)]
fn four_goals_land_together_after_loading_batch_four() {
    let mut world = turn_world();
    world.env.limits.tasks.project_goals = 4;
    world.env.limits.people.requests = 12;
    world.drain_cut();
    world.restart_with_batch(4);
    world.send(worker_hello(&world));
    world.until(Until::Ready);
    let mut goals = Vec::new();
    let mut changes = Vec::new();
    let mut producers = Vec::new();
    for key in 92..96 {
        let ask = walking::goal_from(
            world.chat(1, b"Make one fix into main"),
            people::TaskExecutor::Agent { charter: 1 },
            jig_test_profile::fixtures::Goal { budget: 100, priority: 3, deadline: None },
        );
        let goal = assigned_chat(&mut world, key, ask);
        let (change, producer) = pushed_change(&mut world, &goal, 100 + u64::from(key));
        goals.push(goal);
        changes.push(change);
        producers.push(producer);
    }
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("latch chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(20) };
    let latch_run = assigned_chat(&mut world, 96, people::Ask::Create { project, batch });
    let latch = latch_run.task;
    for (index, change) in changes.iter().enumerate() {
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(300 + *change)),
            sign_in: world.signed_in.expect("owner"),
            key: [u8::try_from(index + 100).expect("key"); 16],
            ask: people::Ask::Amend {
                project: 1,
                task: *change,
                revision: live_task(&world, *change).revisions.spec,
                release: None,
                amendment: people::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    spec: None,
                    wake: None,
                    dependencies: Some(Box::new([latch])),
                    authority: None,
                    llm: None,
                    reason: b"Ready together".as_slice().into(),
                },
            },
        }));
        world.until(Until::Ready);
        assert_eq!(live_task(&world, *change).waiting_on.as_ref(), [latch]);
    }
    for producer in &producers {
        finish_producer(&mut world, producer, forge_world::REPO);
    }
    for _ in 0..PAIR_TICKS {
        tick(&mut world);
        if changes.iter().all(|change| {
            matches!(
                goal_landing::row(&world, *change).change.state,
                temper_engine_domain_forge_change::State::Gating { .. }
            )
        }) {
            break;
        }
    }
    assert!(
        changes.iter().all(|change| matches!(
            goal_landing::row(&world, *change).change.state,
            temper_engine_domain_forge_change::State::Gating { .. }
        )),
        "all four await the same readiness decision behind the latch: {:?}",
        changes.iter().map(|change| goal_landing::row(&world, *change).change.state).collect::<Vec<_>>()
    );
    assert!(landing_rows(&world).is_empty(), "the latch keeps all four changes");
    goal_landing::finish_run(
        &mut world,
        &latch_run,
        tasks::TaskResult::Report { words: b"Ready together".as_slice().into() },
    );
    let landing = await_prepare(&mut world);
    let job = &world.jobs[0];
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&job.spec),
    )
    .expect("prepare spec");
    let request: serde_json::Value = serde_json::from_slice(spec.request()).expect("prepare request");
    assert_eq!(request["members"].as_array().expect("stack").len(), 4);
    // Equal goal priorities leave first-ready time, then task number, to order the queue.
    let mut queue = changes.clone();
    queue.sort_by_key(|change| (goal_landing::row(&world, *change).change.ready_since.expect("ready"), *change));
    answer_prepare(&mut world);
    await_landed(&mut world, landing);
    assert_eq!(landing_rows(&world).len(), 1);
    assert_landed(&world, landing, &queue);
    let row = landing_rows(&world)[0];
    for (goal, change) in goals.iter().zip(changes.iter()) {
        assert_eq!(goal_landing::row(&world, *change).goal, Some(goal.task));
        let member = row.landing.members.iter().find(|member| member.change == *change).expect("goal's member");
        let prefix = String::from_utf8(engine::commit_text(&member.landed.expect("prefix")).into_vec()).expect("hex");
        let expected = format!("landing {landing}: change {} landed at {prefix}", member.change);
        assert_eq!(
            live_task(&world, goal.task)
                .inbox
                .iter()
                .filter(|message| matches!(message.kind, tasks::MessageKind::News { .. })
                    && message.words.as_ref() == expected.as_bytes())
                .count(),
            1,
            "each goal hears its own prefix"
        );
    }
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(), 1);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Merge { .. })).count(), 1);
}
