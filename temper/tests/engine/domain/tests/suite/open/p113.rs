//! P113: wait on another goal without borrowing a question's lifetime.
//! jig/docs/design/domain/tasks/lifecycle.md, 1, and jig/docs/design/domain/tasks/lifecycle-endings.md, 1.
use crate::forge_routes::{assigned_chat, live_task, park_after_turn, questions};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{
    forge_routes::{Gate, StoryLimits, Until, World},
    protocol, walking,
};

const STEPS: u32 = 200; // Settle dependency admission and release within the one-hour forge stall.

fn goal(world: &mut World, key: u8, words: &[u8]) -> engine::Assignment {
    let mut ask = walking::goal_from(
        world.chat(1, words),
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 200, priority: 3, deadline: None },
    );
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("goal creation") };
    batch[0].symbols = Box::new([people::Symbol { connector: 0, symbol: 1 }]);
    assigned_chat(world, key, ask)
}

#[test]
fn a_change_waits_on_another_goal() {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { slots: Some(3), stall: Some(Duration::from_secs(3600)), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 3, Box::new([])));
    let upstream = goal(&mut world, 92, b"Finish the prerequisite goal");
    let parent = goal(&mut world, 93, b"Make a change after the other goal");
    assert_eq!(live_task(&world, upstream.task).root, upstream.task);
    assert_eq!(live_task(&world, parent.task).root, parent.task);
    assert!(live_task(&world, parent.task).references.is_empty());

    let mut change = engine::ChangeCall::words(b"Wait for the prerequisite goal".as_slice().into(), None);
    change.after = Box::new([engine::ChangeAfter::Task(upstream.task)]);
    change.questions = Some(engine::ChangeQuestions::None);
    let answers = world.answers.len();
    questions::call(&mut world, &parent, 94, engine::Tool::Change { changes: Box::new([change]) });
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.get(answers) else {
        panic!(
            "a change can depend on another goal without opening a question: answers={:?}; errors={:?}",
            &world.answers[answers..],
            world.call_errors
        );
    };
    let change = made[0].task;
    world.advance_ticks(STEPS);
    assert_eq!(live_task(&world, change).waiting_on.as_ref(), [upstream.task]);
    assert!(live_task(&world, change).questions.is_empty());
    assert!(world.store.change(change).expect("waiting change").questions.open.is_empty());
    assert!(!matches!(live_task(&world, parent.task).phase, tasks::Phase::Held { .. }));
    assert!(live_task(&world, parent.task).waiting_on.is_empty());
    assert!(live_task(&world, parent.task).questions.is_empty());
    let turns = live_task(&world, parent.task).turn;
    park_after_turn(&mut world, &parent, None, turns);
    world.until(Until::Ready);

    let event = world.protocol.answer(
        Token::new(7),
        upstream.task,
        upstream.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Prerequisite complete".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if live_task(&world, change).waiting_on.is_empty() {
            break;
        }
    }
    assert!(live_task(&world, change).waiting_on.is_empty(), "the goal's end releases the change");
    let producer = questions::next_producer(&mut world, change);
    assert!(!matches!(live_task(&world, parent.task).phase, tasks::Phase::Held { .. }));

    // Finish the released change before the world's teardown checks restart equivalence.
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::NoChange { words: b"No edit needed after the prerequisite".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    world.until(Until::Ready);
    world.close();
}
