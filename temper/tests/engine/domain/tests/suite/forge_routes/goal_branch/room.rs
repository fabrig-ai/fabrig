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
        ask(
            &mut world,
            95,
            people::Ask::Stop { project: 1, task: goal, reason: Box::from(b"Parking the task".as_slice()) }
        ),
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

fn making_goal() -> (World, u64) {
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.hold_write_answers = Some(HoldWrite::All);
    let goal = created(&mut world, GOAL_REQUEST, GOAL_BUDGET);
    for _ in 0..STEPS {
        if temper_engine_domain_world::acts::goal_branch_making(&world, goal) {
            break;
        }
        world.tick();
    }
    assert!(temper_engine_domain_world::acts::goal_branch_making(&world, goal), "goal {goal}; entries {:?}", world.store.rows.values().filter(|record| matches!(record, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Entry { .. }))).collect::<Vec<_>>());
    (world, goal)
}

#[test]
fn cancelling_a_goal_is_excluded_only_during_its_branch_making() {
    use skein_world::sweep::Sweep;
    use temper_engine_domain_world::acts::goal_branch_making;
    use temper_faults::people::CANCELS_A_GOAL;
    let (mut world, goal) = making_goal();
    let mut sweep = Sweep::new("branch making cancellation");
    sweep.configure(CANCELS_A_GOAL);
    temper_engine_forge_world::open::exclude(&mut sweep, &[CANCELS_A_GOAL]).expect("exclusion");
    let conditions = if goal_branch_making(&world, goal) { vec!["a goal branch making in flight"] } else { vec![] };
    assert!(sweep.leaves_out(CANCELS_A_GOAL, &conditions).is_some());
    assert!(!goal_branch_making(&world, goal + 1), "another goal is not making this branch");
    world.release_write_answers();
    world.until(Until::Ready);
    assert!(!goal_branch_making(&world, goal), "a made branch is no longer in flight");
    assert!(sweep.leaves_out(CANCELS_A_GOAL, &[]).is_none());
}

#[test]
#[ignore = "failure worlds: cancelling a goal while its branch is being made holds it instead of ending it, repaired as a red"]
fn a_goal_cancelled_while_its_branch_is_being_made_ends_cancelled() {
    let (mut world, goal) = making_goal();
    ask(
        &mut world,
        CHAT_REQUEST,
        people::Ask::Cancel { project: 1, task: goal, grace: None, reason: b"Cancel while making".as_slice().into() },
    );
    world.release_write_answers();
    world.advance_ticks(STEPS);
    assert!(
        world.store.rows.values().any(|record| matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
        if row.number == goal && matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })))),
        "goal {goal}: {:?}",
        live_task(&world, goal).phase
    );
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
