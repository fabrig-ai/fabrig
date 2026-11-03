//! A goal lands its held branch without handing it to the change
//! (temper/docs/design/domain/agent.md, 4.3.1; temper/docs/design/domain/forge.md, 8.1 and 11).
use super::goal_branch::{branch_tip, change, forge_hold, resource, running_goal, writer};
use super::*;

const STEPS: u32 = 1000; // The forge world's bound for a full landing and hold release.
const LANDING_CALL: u64 = 95;

fn mine(world: &mut World, run: &engine::Assignment, call: u64, input: &[u8]) -> Option<engine::CallAnswer> {
    world.until(Until::Ready);
    let before = world.answers.len();
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(decoded) = temper_engine_smith::call(
        smith_domain_run::CallName {
            activation: run.attempt,
            completion: u32::try_from(call).expect("call"),
            position: 1,
        },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let event = world.protocol.call(Token::new(7), run.task, run.attempt, call, decoded);
    world.send(event);
    world.until(Until::Ready);
    world.answers.get(before).cloned()
}

fn row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change {task}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn assert_goal_holds(world: &World, goal: u64, branch: &[u8]) {
    assert_eq!(forge_hold(world, branch).expect("goal branch hold").task, goal);
    assert!(live_task(world, goal).holdings.contains(&tasks::Holding::Write { resource: resource(branch), kind: 1 }));
}

#[test]
fn a_goals_landing_lands_its_branch_into_main_whole_leaving_it_held_by_the_goal() {
    let (mut world, goal, branch) = running_goal();
    let engine::CallAnswer::Delegated(made) = change(&mut world, &goal, engine::ChangeLanding::Goal) else {
        panic!("goal's first change admitted")
    };
    let first = made[0].task;
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    let first_branch = row(&world, first).branch.clone();
    change_tool::land_change(&mut world, &producer, &first_branch);
    world.until(Until::Ready);
    assert!(matches!(row(&world, first).change.state, temper_engine_domain_forge_change::State::Landed { .. }));
    let head = branch_tip(&mut world, &branch);
    assert_ne!(head, branch_tip(&mut world, b"main"));
    assert_goal_holds(&world, goal.task, &branch);
    let assigned = world.assigned.len();
    let answer = mine(&mut world, &goal, LANDING_CALL, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing admitted: {answer:?}") };
    assert_eq!(made.len(), 1);
    let landing = made[0].task;
    assert_eq!(
        live_task(&world, landing).spec.parameters.as_ref(),
        &[
            tasks::Parameter::Resource {
                name: 1,
                connector: 0,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository)
            },
            tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
            tasks::Parameter::Bytes { name: 3, value: branch.clone() },
            tasks::Parameter::Number { name: 11, value: 1 },
        ]
    );
    assert!(live_task(&world, landing).holdings.is_empty());
    assert_goal_holds(&world, goal.task, &branch);
    assert_eq!(world.assigned.len(), assigned, "the goal landing has no producer");
    for _ in 0..STEPS {
        world.tick();
        assert_goal_holds(&world, goal.task, &branch);
        if row(&world, landing).pull.is_some() || row(&world, landing).delegate.is_some() {
            break;
        }
    }
    assert_eq!(world.assigned.len(), assigned, "the goal landing has no producer");
    let number = row(&world, landing)
        .pull
        .unwrap_or_else(|| panic!("the held branch opens immediately: {:?}", row(&world, landing)));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("landing pull")
    };
    assert_eq!(pull.head, branch);
    assert_eq!(pull.base.as_ref(), b"main");
    assert_eq!(pull.commit, head);
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: b"build".as_slice().into(),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    for _ in 0..STEPS {
        world.tick();
        assert_goal_holds(&world, goal.task, &branch);
        if world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(landing)))) {
            break;
        }
    }
    assert!(
        matches!(row(&world, landing).change.state, temper_engine_domain_forge_change::State::Landed { .. }),
        "{:?}",
        row(&world, landing)
    );
    assert!(world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Merge { number: pull, head: merged, .. } if *pull == number && *merged == translate::commit(head))));
    assert_eq!(world.merges, 2);
    assert_eq!(writer(&world, &branch), None);
    world.until(Until::Ready);
    assert_goal_holds(&world, goal.task, &branch);
    assert!(
        !world.writes.iter().any(
            |write| matches!(write, skein_forge::api::Write::DeleteBranch { branch: deleted } if *deleted == branch)
        )
    );
    assert_eq!(branch_tip(&mut world, &branch), head);
    world.close();
}

fn refused(world: &mut World, run: &engine::Assignment, call: u64, input: &[u8], words: &[u8]) {
    world.until(Until::Ready);
    let made = live_task(world, run.task).made;
    let tasks: Vec<_> = world
        .store
        .rows
        .keys()
        .filter(|key| {
            matches!(
                key,
                Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_) | tasks::Key::Ended(_)))
                    | Key::Forge(forge_top::Key::Change(_))
            )
        })
        .cloned()
        .collect();
    let errors = world.call_errors.len();
    assert_eq!(mine(world, run, call, input), None);
    assert_eq!(world.call_errors.len(), errors + 1);
    assert_eq!(world.call_errors[errors].text.as_ref(), words);
    assert_eq!(live_task(world, run.task).made, made, "a refused call builds no delegate");
    assert_eq!(
        world
            .store
            .rows
            .keys()
            .filter(|key| matches!(
                key,
                Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_) | tasks::Key::Ended(_)))
                    | Key::Forge(forge_top::Key::Change(_))
            ))
            .cloned()
            .collect::<Vec<_>>(),
        tasks
    );
}

#[test]
fn branch_mine_with_landing_goal_is_refused() {
    let (mut world, goal, _) = running_goal();
    refused(
        &mut world,
        &goal,
        LANDING_CALL,
        br#"{"words":"Land the goal","landing":"goal","branch":"mine"}"#,
        b"branch mine with landing goal: a branch cannot land into itself.",
    );
    world.close();
}

#[test]
fn branch_mine_without_a_goal_branch_is_refused() {
    let mut world = goal_branch::adopted_world();
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("create") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let chat = assigned_chat(&mut world, 93, people::Ask::Create { project, batch });
    refused(
        &mut world,
        &chat,
        LANDING_CALL,
        br#"{"words":"Land the goal","branch":"mine"}"#,
        b"branch mine: this task holds no goal branch in org/repo.",
    );
    world.close();
}

#[test]
fn a_second_landing_of_the_same_goal_branch_is_refused_naming_the_first() {
    let (mut world, goal, _) = running_goal();
    let answer = mine(&mut world, &goal, LANDING_CALL, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("landing admitted: {answer:?}") };
    let words = format!("branch mine: change task {} is on it already.", made[0].task);
    refused(&mut world, &goal, LANDING_CALL + 1, br#"{"words":"Land again","branch":"mine"}"#, words.as_bytes());
    world.close();
}

#[test]
fn two_landings_of_the_same_goal_branch_in_one_call_are_refused() {
    let (mut world, goal, _) = running_goal();
    refused(
        &mut world,
        &goal,
        LANDING_CALL,
        br#"{"changes":[{"words":"First landing","branch":"mine"},{"words":"Second landing","branch":"mine"}]}"#,
        b"branch mine: an earlier change of this call is on it.",
    );
    world.close();
}
