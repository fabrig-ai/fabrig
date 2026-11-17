//! The template's small document pages, independent of other client fixtures (client/ctl/testing.md, 2).
use super::{Joint, Temper, World, domain_limits, make_goal_with_delegation, report_task, world_with};
use jig_domain_authority::Executor;
use jig_domain_tasks as tasks;
use temper_engine_domain as engine;
use temper_engine_domain_world::fixtures::report_delegate;

pub const ROWS: u32 = 2;
const DELEGATES: u32 = 5;

#[must_use]
pub fn world(seed: u64) -> World<Temper, Joint> {
    let front =
        Joint::with_delegate_kinds(seed, &domain_limits(), DELEGATES, ROWS, &[Executor::Charter(1), Executor::Role(0)]);
    let mut world = world_with(seed, front);
    let front = world.front();
    let goal = make_goal_with_delegation(front, false, Some(DELEGATES));
    assert_eq!(goal, 1);
    let parent = front.engine.last_assignment();
    let mut person = report_delegate(b"Bounds person", Box::new([]));
    person.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    person.authority.tools = tasks::Tools(0);
    let batches = [
        vec![
            report_delegate(b"Bounds dependency", Box::new([])),
            report_delegate(b"Bounds ended", Box::new([engine::Dependency::Batch(0)])),
        ],
        vec![person, report_delegate(b"Bounds waiting one", Box::new([engine::Dependency::Batch(0)]))],
        vec![report_delegate(b"Bounds waiting last", Box::new([engine::Dependency::Existing(4)]))],
    ];
    for (index, batch) in batches.into_iter().enumerate() {
        let sequence = u32::try_from(index + 1).expect("three delegation calls");
        let call = super::DELEGATE_CALL + u64::from(sequence);
        front.engine.call(
            super::QUESTION_WORKER,
            goal,
            parent.attempt,
            call,
            engine::Call {
                completion: sequence,
                position: 1,
                tool: engine::Tool::Delegate { batch: batch.into_boxed_slice() },
            },
        );
        front.engine.settle();
        assert!(
            matches!(front.engine.answered(call), Some(engine::CallAnswer::Delegated(_))),
            "batch {sequence} admitted: {:?}",
            front.engine.answered(call)
        );
        super::delegate_turn(front, goal, None, b"Delegates declared");
    }
    let spent = front.engine.task(goal).expect("root run").run_spent;
    front.engine.answer(super::QUESTION_WORKER, goal, parent.attempt, spent, tasks::End::Parked, engine::Parts::none());
    front.engine.settle();
    report_task(front, 2, b"Dependency ended");
    super::delegate_turn(front, 3, None, b"Dependent complete");
    report_task(front, 3, b"Dependent ended");
    world
}
