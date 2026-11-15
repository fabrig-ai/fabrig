//! Goal trains through the root and worker requests (temper/docs/design/domain/landing.md, 5.2, 9, 15).
use super::super::{goal_branch, goal_landing};
use super::*;

fn goal_world() -> (World, engine::Assignment, Box<[u8]>) {
    goal_world_with(StoryLimits { turn: true, spec_bytes: Some(1024), ..StoryLimits::default() })
}

fn goal_world_with(limits: StoryLimits) -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(true, true, false, 1000, Gate::default(), None, limits);
    world.adopt();
    let mut hello = hello_prepare(limits.slots.unwrap_or(3));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(hello);
    let ask = goal_branch::goal_request(&world, true);
    let goal = assigned_chat(&mut world, 92, ask);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    (world, goal, branch)
}

fn main_producer(world: &mut World, goal: &engine::Assignment, call: u64, file: &[u8]) -> (u64, engine::Assignment) {
    let assigned = world.assigned.len();
    let answer = goal_landing::mine(world, goal, call, br#"{"words":"Independent main change","landing":"main"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else {
        panic!("main change: {answer:?}; errors {:?}", world.call_errors)
    };
    let producer = reviews::next_assignment(world, assigned);
    let branch = producer.workspace.repositories[0].push.clone().expect("producer branch");
    world.external(1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, file, b"added", 1).expect("producer pushes");
    (made[0].task, producer)
}

#[test]
fn drill_14_a_dependency_across_goal_and_main_trains_survives_restart() {
    let (mut world, goal, branch) = goal_world_with(StoryLimits {
        turn: true,
        batch: Some(3),
        slots: Some(4),
        tasks: Some(12),
        spec_bytes: Some(2048),
        inbox_messages: Some(16),
        ..StoryLimits::default()
    });
    world.manual_prepare_jobs();
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(200) };
    let chat = assigned_chat(&mut world, 93, people::Ask::Create { project, batch });
    let (upstream, producer_upstream) = goal_landing::producing_into_goal(&mut world, &goal, &branch, 101, b"upstream");
    let (dependent, producer_dependent) = main_producer(&mut world, &chat, 102, b"dependent");
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner"),
        key: [95; 16],
        ask: people::Ask::Amend {
            project: 1,
            task: dependent,
            revision: live_task(&world, dependent).revisions.spec,
            release: None,
            amendment: people::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                spec: None,
                wake: None,
                dependencies: Some(Box::new([upstream])),
                authority: None,
                llm: None,
                reason: b"Land the goal branch change first".as_slice().into(),
            },
        },
    }));
    world.until(Until::Ready);
    assert_eq!(live_task(&world, dependent).waiting_on.as_ref(), [upstream]);
    finish_child(&mut world, &producer_dependent, b"Dependent main change");
    let (other, producer_other) = main_producer(&mut world, &chat, 103, b"other");
    finish_child(&mut world, &producer_other, b"Independent main change");
    finish_child(&mut world, &producer_upstream, b"Goal change");
    for _ in 0..PAIR_TICKS {
        world.tick();
        if landing_rows(&world).len() == 2 && world.jobs.len() == 2 {
            break;
        }
    }
    assert_eq!(landing_rows(&world).len(), 2);
    assert_eq!(live_task(&world, dependent).waiting_on.as_ref(), [upstream]);
    assert_eq!(goal_landing::row(&world, dependent).change.landing, None);
    let before = landing_rows(&world).iter().map(|row| row.landing.task).collect::<Vec<_>>();
    world.drain_cut();
    world.restart_same();
    let mut hello = hello_prepare(4);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(hello);
    world.automatic_prepare_jobs();
    let mut saw_upstream = false;
    for _ in 0..PAIR_TICKS {
        world.tick();
        saw_upstream |= goal_landing::ended(&world, upstream).is_some();
        if goal_landing::row(&world, dependent).change.landing.is_some() {
            assert!(saw_upstream, "dependent only forms after upstream ends");
        }
        for change in [upstream, dependent, other] {
            assert!(!matches!(task_row(&world, change).expect("change task").phase, tasks::Phase::Held { .. }));
        }
        if [upstream, dependent, other].iter().all(|change| goal_landing::ended(&world, *change).is_some()) {
            break;
        }
    }
    assert!(
        saw_upstream,
        "landings {:?}; jobs {:?}; tasks {:?}",
        landing_rows(&world),
        world.jobs,
        world
            .store
            .rows
            .values()
            .filter_map(|record| {
                if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = record {
                    Some((row.number, &row.phase))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    );
    assert!(
        [upstream, dependent, other].iter().all(|change| goal_landing::ended(&world, *change).is_some()),
        "{:?}",
        landing_rows(&world)
    );
    let rows = landing_rows(&world);
    assert_eq!(rows.len(), 3, "no duplicate landing after restart");
    assert!(before.iter().all(|task| rows.iter().any(|row| row.landing.task == *task)));
    for change in [upstream, dependent, other] {
        assert_eq!(
            rows.iter().filter(|row| row.landing.members.iter().any(|member| member.change == change)).count(),
            1
        );
    }
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. }))
            .count(),
        3
    );
    world.close();
}

fn await_change(world: &mut World, change: u64) {
    for _ in 0..PAIR_TICKS {
        world.tick();
        assert!(
            !matches!(task_row(world, change).expect("change task").phase, tasks::Phase::Held { .. }),
            "{:?}",
            goal_landing::row(world, change)
        );
        if goal_landing::ended(world, change).is_some() {
            return;
        }
    }
    panic!("change did not land: {:?}; landings {:?}", goal_landing::row(world, change), landing_rows(world));
}

fn finish_child(world: &mut World, producer: &engine::Assignment, title: &[u8]) {
    goal_landing::finish_run(
        world,
        producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(title.into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"A change for the goal.\n\nFurther detail.".as_slice().into(),
        },
    );
}

fn assert_pull(world: &mut World, change: u64, commit: u64) {
    let number = goal_landing::row(world, change).pull.expect("member pull");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else { panic!("pull") };
    assert_eq!(pull.merged, Some(commit));
    assert!(matches!(goal_landing::row(world, change).change.state,
        temper_engine_domain_forge_change::State::Landed { merge, .. } if merge == translate::commit(commit)));
}

#[test]
fn drill_12_three_squashes_into_a_goal_then_its_gated_merge_into_main() {
    let (mut world, goal, branch) = goal_world();
    let main = goal_branch::branch_tip(&mut world, b"main");
    let mut below = main;
    let mut landed = Vec::new();
    for number in 1..=3 {
        let title = format!("Goal change {number}");
        let file = format!("file{number}");
        let (change, producer) =
            goal_landing::producing_into_goal(&mut world, &goal, &branch, 100 + number, file.as_bytes());
        finish_child(&mut world, &producer, title.as_bytes());
        await_change(&mut world, change);
        let commit = goal_branch::branch_tip(&mut world, &branch);
        let object = world.fake.object(commit).expect("squash commit");
        assert_eq!(object.parent, Some(below));
        assert_eq!(object.merge_parent, None);
        let row = goal_landing::row(&world, change);
        let expected = format!(
            "{title}\n\nA change for the goal.\n\nChange: {change}\nPull-request: {}\nHead: {}\nGoal: {}",
            row.pull.expect("pull"),
            String::from_utf8_lossy(&engine::commit_text(&row.change.last_head.expect("delivered head"))),
            goal.task
        );
        assert_eq!(object.message.as_ref(), expected.as_bytes());
        assert_pull(&mut world, change, commit);
        assert_eq!(goal_branch::branch_tip(&mut world, b"main"), main);
        let rows = landing_rows(&world);
        let landing = rows.iter().find(|row| row.landing.members[0].change == change).expect("goal landing");
        assert_eq!(landing.landing.members.len(), 1);
        assert!(landing.landing.members[0].gates.is_empty());
        assert_eq!(landing.landing.members[0].combined, Some(engine::Combine::Squash));
        landed.push(landing.landing.task);
        below = commit;
    }
    assert_eq!(landed.len(), 3);
    assert!(!world.writes.iter().any(|write| matches!(write, api::Write::Dispatch { .. })));
    let answer = goal_landing::mine(&mut world, &goal, 110, br#"{"words":"Land the goal whole","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing: {answer:?}") };
    let whole = made[0].task;
    await_change(&mut world, whole);
    let main = goal_branch::branch_tip(&mut world, b"main");
    assert_eq!(world.fake.object(main).expect("main merge").merge_parent, Some(below));
    assert_pull(&mut world, whole, main);
    assert_eq!(landing_rows(&world).len(), 4);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(), 1);
    goal_landing::assert_goal_holds(&world, goal.task, &branch);
    world.close();
}
