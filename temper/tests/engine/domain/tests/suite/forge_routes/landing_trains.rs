//! Goal trains through the root and worker requests (temper/docs/design/domain/landing.md, 5.2, 9, 15).
use super::super::{goal_branch, goal_landing};
use super::*;

fn goal_world() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, spec_bytes: Some(1024), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(hello_prepare(3));
    let ask = goal_branch::goal_request(&world, true);
    let goal = assigned_chat(&mut world, 92, ask);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    (world, goal, branch)
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
