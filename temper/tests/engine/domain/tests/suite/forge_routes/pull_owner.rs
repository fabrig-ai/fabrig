//! A remade landing owns its own pull, including after restart
//! (temper/docs/design/domain/forge/change/README.md, 2, and forge/keeping-up.md, 3).
use super::goal_branch::{branch_tip, running_goal};
use super::goal_landing::{land_into_goal, mine, row, status};
use super::*;
use skein_forge::api::{Read, Write};

const STEPS: u32 = 1000;

fn landing(world: &mut World, goal: &engine::Assignment, call: u64) -> u64 {
    let answer = mine(world, goal, call, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("landing admitted: {answer:?}") };
    made[0].task
}

fn opened(world: &mut World, task: u64) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        if row(world, task).pull.is_some() && world.store.pending.is_empty() && world.pending.is_empty() {
            break;
        }
    }
    row(world, task).pull.unwrap_or_else(|| panic!("landing opens its own pull: {:?}", row(world, task)))
}

fn opens(world: &World) -> usize {
    world.writes.iter().filter(|write| matches!(write, Write::OpenPull { .. })).count()
}

#[test]
fn a_remade_goal_landing_opens_its_own_pull_and_reads_it_by_number_after_restart() {
    let (mut world, goal, branch) = running_goal();
    let head = land_into_goal(&mut world, &goal, &branch, 94, b"plan");
    status(&mut world, head, raw::Check::Pending);
    let before = opens(&world);
    let first = landing(&mut world, &goal, 95);
    let p = opened(&mut world, first);
    assert_eq!(opens(&world), before + 1);
    cancel_task(&mut world, first);
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: p })) else {
        panic!("cancelled landing's pull")
    };
    assert_eq!(pull.state, raw::State::Closed);
    assert_eq!(pull.merged, None);
    assert_eq!(branch_tip(&mut world, &branch), head);
    let second = landing(&mut world, &goal, 96);
    let q = opened(&mut world, second);
    assert_ne!(q, p, "a closed predecessor is not the new landing's pull");
    assert_eq!(opens(&world), before + 2, "one opening per landing");
    assert_eq!(row(&world, second).pull, Some(q));
    assert_eq!(row(&world, second).drift, None);
    assert!(!matches!(live_task(&world, second).phase, tasks::Phase::Held { .. }));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: q })) else {
        panic!("new landing's pull")
    };
    assert_eq!(pull.state, raw::State::Open);
    assert_eq!(pull.head, branch);
    assert_eq!(pull.commit, head);
    world.forge_reads = Some(Vec::new());
    world.restart_same();
    world.until(Until::Ready);
    let reads = world.forge_reads.as_ref().expect("record reads after restart");
    assert!(reads.iter().any(|(_, read)| matches!(read, Read::Pull { number, .. } if *number == q)));
    assert!(
        !reads.iter().any(|(_, read)| matches!(read, Read::PullFor { .. })),
        "numbered changes never look up branches: {reads:?}"
    );
    assert_eq!(row(&world, second).pull, Some(q));
    assert_eq!(row(&world, second).drift, None);
    assert_eq!(opens(&world), before + 2, "restart never opens another");
    world.close();
}
