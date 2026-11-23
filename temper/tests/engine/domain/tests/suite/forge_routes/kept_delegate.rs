//! A settled landing meets an added dependency before its kept delegate runs
//! (temper/docs/design/testing/checks.md, 1).
use super::*;
use temper_engine_domain_world::fixtures::{report_delegate, task_amendment};

// One landing, its cleanup and the dependent's first assignment.
const STEPS: u32 = 1000;
const ONE_RUN: usize = 1;

// The chat's initial delegation occupies position one of its first completion.
fn call(world: &mut World, chat: &engine::Assignment, position: u32, tool: engine::Tool) -> engine::CallAnswer {
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        u64::from(position),
        engine::Call { completion: 1, position, tool },
    );
    let before = world.answers.len();
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if world.answers.len() > before && world.store.pending.is_empty() {
            return world.answers[before].clone();
        }
    }
    panic!("chat call answered: {:?}", world.call_errors);
}

#[test]
fn a_landing_meets_the_kept_delegates_added_dependency_before_its_only_run() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else {
        panic!("producer belongs to its change")
    };
    world.until(Until::Ready);
    let engine::CallAnswer::Delegated(made) = call(
        &mut world,
        &chat,
        2,
        engine::Tool::Delegate { batch: Box::new([report_delegate(b"Continue after landing", Box::new([]))]) },
    ) else {
        panic!("report delegate created")
    };
    let kept = made[0].task;
    let mut amendment = task_amendment(None, None, b"Wait for the change to land");
    amendment.dependencies = Some(Box::new([change]));
    let revision = live_task(&world, kept).revisions.spec;
    call(&mut world, &chat, 3, engine::Tool::Amend { keep_parameters: false, target: kept, revision, amendment });
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed");
    world.held_merge_calls = Some(Vec::new());
    delegates::fixtures::answer_producer(&mut world, &producer);
    for _ in 0..STEPS {
        world.tick();
        if world.held_merge_calls.as_ref().is_some_and(|calls| !calls.is_empty()) {
            break;
        }
    }
    assert!(world.held_merge_calls.as_ref().is_some_and(|calls| !calls.is_empty()), "merge reached the forge");
    assert_eq!(live_task(&world, kept).waiting_on.as_ref(), [change]);
    assert!(!world.assigned.iter().any(|run| run.task == kept), "no run before landing");

    world.release_merge_calls();
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.iter().any(|run| run.task == kept) {
            break;
        }
    }
    let landed = goal_landing::ended(&world, change).expect("change ended before dependent's first run");
    assert!(matches!(landed.phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    assert!(live_task(&world, kept).run_since.expect("dependent started") >= landed.ended_at.expect("landing ended"));
    assert_eq!(world.merges, 1);
    let run = world.assigned.iter().find(|run| run.task == kept).expect("kept delegate assigned").clone();
    assert!(live_task(&world, kept).waiting_on.is_empty());
    assert!(
        run.messages.iter().any(|word| word.kind == tasks::MessageKind::Met { dependency: change, removed: false })
    );
    goal_landing::finish_run(&mut world, &run, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    for _ in 0..STEPS {
        world.tick();
        if goal_landing::ended(&world, kept).is_some() {
            break;
        }
    }
    assert!(goal_landing::ended(&world, kept).is_some(), "dependent ended");
    world.until(Until::Ready);
    assert_eq!(world.assigned.iter().filter(|run| run.task == kept).count(), ONE_RUN);
}
