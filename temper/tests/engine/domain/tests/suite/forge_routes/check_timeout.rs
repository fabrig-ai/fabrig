//! A change project's delivery checks' bound is read for each assignment
//! (temper/docs/design/domain/agent/charter.md, 4).
use super::goal_landing::{land_into_goal, mine, row, status};
use super::*;
use temper_engine_domain_forge_change::{Delegate, Repair, State};

const BOUND: Option<Duration> = Some(Duration::from_secs(900));
const STEPS: u32 = 1000;
const CHANNEL: Token = Token::new(7);

fn configured(bound: Option<Duration>) -> World {
    let world = World::configured_limits(true, true, true, STEPS, Gate::default(), None, StoryLimits::default());
    match bound {
        Some(_) => world.with_check_timeout(bound),
        None => world,
    }
}

#[test]
fn producers_and_resolutions_take_the_project_bound_but_chats_do_not() {
    for bound in [BOUND, None] {
        let (mut world, change, _, _) = light_rules::pushed_change_in(configured(bound), true, b"main");
        assert_eq!(world.assigned[0].check_timeout, None, "the requesting chat is not a change delegate");
        assert_eq!(world.assigned[1].check_timeout, bound, "the producer inherits its change project's bound");
        let resolution = reviews::next_assignment(&mut world, 2);
        assert!(
            matches!(row(&world, change).delegate, Some((child, Delegate::Resolve { .. })) if child == resolution.task)
        );
        assert_eq!(resolution.check_timeout, bound);
        world.close();
    }
}

#[test]
fn a_ci_repair_takes_its_change_project_bound() {
    let (mut world, _, producer, branch) = change_world_in(configured(BOUND), Change::default());
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("producer's change") };
    reviews::produce(&mut world, &producer, &branch);
    let head = world.fake.branch(b"org/repo", &branch).expect("pushed head");
    let assigned = world.assigned.len();
    status(&mut world, head, raw::Check::Failed);
    let repair = reviews::next_assignment(&mut world, assigned);
    assert_eq!(row(&world, change).delegate, Some((repair.task, Delegate::Repair(Repair::Ci))));
    assert_eq!(repair.check_timeout, BOUND);
    world.close();
}

#[test]
fn a_goal_landing_repair_takes_the_bound_but_the_goal_run_does_not() {
    let mut world = goal_landing_held::world_with_bound(BOUND);
    let people::Ask::Create { project, mut batch } = goal_branch::goal_request(&world, true) else {
        panic!("goal template")
    };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    assert_eq!(goal.check_timeout, None);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes();
    world.until(Until::Ready);
    let head = land_into_goal(&mut world, &goal, &branch, 96, b"plan");
    world.until(Until::Ready);
    status(&mut world, head, raw::Check::Pending);
    let Some(engine::CallAnswer::Delegated(made)) =
        mine(&mut world, &goal, 95, br#"{"words":"Land the goal","branch":"mine"}"#)
    else {
        panic!("goal landing admitted")
    };
    let landing = made[0].task;
    for _ in 0..STEPS {
        world.tick();
        if row(&world, landing).change.state == (State::Checking { head: translate::commit(head) }) {
            break;
        }
    }
    assert_eq!(row(&world, landing).change.state, State::Checking { head: translate::commit(head) });
    let assigned = world.assigned.len();
    status(&mut world, head, raw::Check::Failed);
    let repair = reviews::next_assignment(&mut world, assigned);
    assert_eq!(row(&world, landing).delegate, Some((repair.task, Delegate::Repair(Repair::Ci))));
    assert_eq!(repair.check_timeout, BOUND);
    world.close();
}

#[test]
fn the_next_attempt_reads_the_restarted_project_bound() {
    let (mut world, _, producer, _) = change_world_in(configured(BOUND), Change::default());
    world.until(Until::Ready);
    let next_bound = Some(Duration::from_secs(1200));
    world.restart_with_check_timeout(next_bound);
    world.until(Until::Ready);
    let assigned = world.assigned.len();
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    world.send(event);
    let retry = reviews::next_assignment(&mut world, assigned);
    assert_eq!(retry.task, producer.task);
    assert!(retry.attempt > producer.attempt);
    assert_eq!(retry.check_timeout, next_bound);
    world.close();
}
