//! Empty repairs recheck CI and still spend the repair bound (change/repairs.md).
use super::questions::row;
use super::*;
use temper_engine_domain_forge_change::{Delegate, Repair, State};

const STEPS: u32 = 300;
const REPAIRS: u32 = 2;

fn repair() -> (World, engine::Assignment, u64) {
    let world = World::configured_limits(
        true,
        true,
        false,
        0,
        Gate::default(),
        None,
        StoryLimits { repairs: Some(REPAIRS), stall: Some(Duration::from_secs(30 * 60)), ..StoryLimits::default() },
    );
    let (mut world, _, producer, branch) = change_world_in(world, Change::default());
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: 1,
            context: b"build".as_slice().into(),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let repair = failed_ci_repair(&mut world, &producer, &branch, &[b"build"]);
    let tasks::Party::Task(change) = live_task(&world, repair.task).requester else { panic!("repair owner") };
    (world, repair, change)
}

fn answer(world: &mut World, repair: &engine::Assignment) {
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::NoChange {
                words: b"the test passed locally; no files changed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

#[test]
fn an_empty_ci_repair_checks_its_head_again_without_a_hold() {
    let (mut world, repair, change) = repair();
    let head = row(&world, change).change.last_head.expect("repaired head");
    answer(&mut world, &repair);
    let mut checked = false;
    for _ in 0..STEPS {
        world.tick();
        let row = row(&world, change);
        assert!(!matches!(row.change.state, State::Held { .. }), "{:?}", row.change);
        assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
        if row.change.state == (State::Checking { head }) {
            checked = true;
            assert_eq!(row.change.repairs, 1);
            break;
        }
    }
    assert!(checked, "empty repair checks its unchanged head: {:?}", row(&world, change));
    // This world's translated failed checks include jobs at the head.
    let row = row(&world, change);
    assert!(row.effect_account.as_ref().is_some_and(|account| contains(&account.words, b"a repair changed nothing")));
    world.close();
}

#[test]
fn repeated_empty_ci_repairs_hold_with_the_spent_bound_account() {
    let (mut world, mut repair, change) = repair();
    for count in 1..=REPAIRS {
        assert_eq!(row(&world, change).delegate, Some((repair.task, Delegate::Repair(Repair::Ci))));
        answer(&mut world, &repair);
        if count < REPAIRS {
            let after = world.assigned.len();
            for _ in 0..STEPS {
                world.tick();
                assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
                if let Some(next) = world.assigned.get(after) {
                    repair = next.clone();
                    break;
                }
            }
            assert_eq!(row(&world, change).change.repairs, count + 1);
        }
    }
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    assert!(matches!(
        row(&world, change).change.state,
        State::Held { why: temper_engine_domain_forge_change::Hold::Repairs, .. }
    ));
    assert_eq!(row(&world, change).change.repairs, REPAIRS);
    let tasks::Phase::Held { source, why: tasks::Hold::Procedure, .. } = &live_task(&world, change).phase else {
        panic!("repair bound holds the change: {:?}", live_task(&world, change).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref().expect("bound account").as_ref() else {
        panic!("repair bound account")
    };
    assert_eq!(words.as_ref(), b"its repairs are spent: 2 of 2 taken");
    world.close();
}
