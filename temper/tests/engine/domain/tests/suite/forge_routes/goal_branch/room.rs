//! Released makings return to their place (temper/docs/design/domain/forge/room.md, 3).
use super::*;

fn created(world: &mut World, key: u8, budget: u64) -> u64 {
    let people::Ask::Create { project, mut batch } = goal_request(world, true) else { panic!("goal batch") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(budget) };
    let people::Outcome::Created { first, .. } = ask(world, key, people::Ask::Create { project, batch }) else {
        panic!("goal created")
    };
    first
}

fn creations(world: &World, branch: &[u8]) -> usize {
    world
        .writes
        .iter()
        .filter(|write| {
            matches!(write,
        skein_forge::api::Write::CreateBranch { branch: made, .. } if made.as_ref() == branch)
        })
        .count()
}

pub(crate) fn passed_making() -> (World, u64, Box<[u8]>) {
    let mut world = World::configured(true, true).with_forge_entries(1);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.hold_write_answers = Some(HoldWrite::All);
    let first = created(&mut world, 90, 100);
    world.advance_ticks(20);
    assert!(!world.held_write_answers.is_empty(), "another write occupies the only outbox place");
    let goal = created(&mut world, GOAL_REQUEST, GOAL_BUDGET);
    let branch = format!("temper/{goal}/g{goal}").into_bytes().into_boxed_slice();
    world.advance_ticks(20);
    assert!(forge_hold(&world, &branch).is_some(), "the waiting goal took its branch");
    assert_eq!(creations(&world, &branch), 0);
    assert_eq!(
        ask(&mut world, 95, people::Ask::Stop { project: 1, task: goal }),
        people::Outcome::Stopped { task: goal }
    );
    world.advance_ticks(20);
    assert!(matches!(live_task(&world, goal).phase, tasks::Phase::Held { why: tasks::Hold::Placed { .. }, .. }));
    world.release_write_answers();
    world.until(Until::Ready);
    assert_eq!(creations(&world, &format!("temper/{first}/g{first}").into_bytes()), 1);
    assert_eq!(creations(&world, &branch), 0, "the held making's room ask is passed over");
    (world, goal, branch)
}

fn made_once(mut world: World, goal: u64, branch: &[u8]) {
    for _ in 0..STEPS {
        world.tick();
        if creations(&world, branch) > 0 {
            break;
        }
    }
    assert_eq!(creations(&world, branch), 1, "release makes the branch once without a restart");
    world.until(Until::Ready);
    let main = branch_tip(&mut world, b"main");
    assert_eq!(branch_tip(&mut world, branch), main);
    assert_eq!(creations(&world, branch), 1, "settling makes no duplicate branch");
    assert!(!matches!(live_task(&world, goal).phase, tasks::Phase::Held { .. }));
    assert!(live_task(&world, goal).placed.is_none());
    world.close();
}

#[test]
fn releasing_a_persons_hold_asks_a_passed_over_goal_making_again() {
    let (mut world, goal, branch) = passed_making();
    let revision = live_task(&world, goal).revisions.hold;
    ask(&mut world, CHAT_REQUEST, people::Ask::Release { project: 1, task: goal, revision, again: None });
    made_once(world, goal, &branch);
}

#[test]
fn lifting_a_placed_hold_asks_a_passed_over_goal_making_again() {
    let (mut world, goal, branch) = passed_making();
    let hold = live_task(&world, goal).placed.expect("person's placed hold");
    let revision = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::PlacedHold(row))) if row.number == hold => {
                Some(row.revision)
            }
            _ => None,
        })
        .expect("durable placed hold");
    ask(&mut world, CHAT_REQUEST, people::Ask::Lift { hold, revision, with_tasks: true });
    made_once(world, goal, &branch);
}
