//! A finite goal's durable outcomes, including a process cut and a lasting fault.
use super::*;
use temper_engine_domain_world::ends::{Bound, Ends, Expect, OPENING};

const BOUND: Bound = Bound { minutes: 10, steps: 4000 };

#[test]
fn a_finite_goal_ends_within_its_bound_across_a_restart() {
    let (mut world, goal, branch) = goal_world(4);
    world.ends.expect(goal.task, Expect::Ends, BOUND).expect("nonzero bound");
    world.ends.faults_stop(world.env.now);
    let (change, producer) = make_change(&mut world, &goal, 93);
    park_goal(&mut world, &goal);
    produce(&mut world, change, &branch, &producer, b"Finite change");
    let (closer, landing) = finish_closing(&mut world, goal.task, &branch);
    assert_report(&world, goal.task, closer, landing, &[(change, b"Finite change")]);
    world.drain_cut();
    world.restart_same();
    world.until(Until::Ready);
    world.close();
}

#[test]
fn the_same_finite_goal_cannot_end_in_one_step() {
    let (mut world, goal, _) = goal_world(4);
    world.ends.faults_stop(world.env.now);
    world.ends.expect(goal.task, Expect::Ends, Bound { minutes: 10, steps: 1 }).expect("nonzero bound");
    let finding = world.try_tick().expect_err("the assigned goal has not done its work");
    assert!(finding.contains(&format!("engine task {}:", goal.task)), "{finding}");
    assert!(finding.contains("10 minutes and 1 steps"), "{finding}");
}

#[test]
fn a_permanently_refused_forge_holds_with_words_naming_the_fault() {
    let (mut world, goal, branch) = goal_world(4);
    let (change, producer) = make_change(&mut world, &goal, 93);
    world.fail_open_writes = true;
    world.ends.faults_stop(world.env.now);
    world.ends.expect(change, Expect::Held("permission"), BOUND).expect("nonzero bound");
    push_production(&mut world, change, &branch, &producer, b"Refused change");
    wait_held(&mut world, change, goal.task);
    world.ends.close(world.env.now, &world.store).expect("held with permission words");
    let mut wrong = Ends::new(71, OPENING);
    wrong.expect(change, Expect::Held("network timeout"), BOUND).expect("nonzero bound");
    let finding = wrong.step(world.env.now, &world.store).expect_err("different fault words");
    assert!(finding.contains(&format!("engine task {change}: held without expected words")), "{finding}");
    assert!(world.fail_open_writes, "the fault never stopped");
    world.close();
}
