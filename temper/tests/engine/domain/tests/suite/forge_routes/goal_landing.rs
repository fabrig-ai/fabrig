//! A goal lands its held branch without handing it to the change
//! (temper/docs/design/domain/agent.md, 4.3.1; temper/docs/design/domain/forge.md, 8.1 and 11).
//! Its delegates' moves become its landing's new head, and its close releases the branch
//! (temper/docs/design/domain/forge.md, 8.5; temper/docs/design/domain/after-landing.md, 6).
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

const PLAN_MERGES: u32 = 1; // The plan change into g, before L merges.
const TWO_CHANGE_MERGES: u32 = 2; // The plan and implementation changes into g.
const GOAL_MERGES: u32 = 3; // Those two changes, then L's whole goal branch into main.

fn status(world: &mut World, head: u64, state: raw::Check) {
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state,
                description: Box::new([]),
                link: Box::new([]),
            }),
        ),
        raw::Answer::Done
    );
}

fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

fn producing_into_goal(
    world: &mut World,
    goal: &engine::Assignment,
    branch: &[u8],
    call: u64,
    file: &[u8],
) -> (u64, engine::Assignment) {
    let assigned = world.assigned.len();
    let answer = mine(world, goal, call, br#"{"words":"Add to the goal","landing":"goal"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal change admitted: {answer:?}") };
    let change = made[0].task;
    let producer = reviews::next_assignment(world, assigned);
    let change_branch = row(world, change).branch.clone();
    let base = branch_tip(world, branch);
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: change_branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &change_branch, file, b"added", 1)
        .expect("producer pushes its change");
    status(world, head, raw::Check::Passed);
    (change, producer)
}

fn finish_goal_change(world: &mut World, change: u64, producer: &engine::Assignment) {
    finish_run(
        world,
        producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Add to the goal".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    for _ in 0..STEPS {
        world.tick();
        if ended(world, change).is_some() {
            break;
        }
    }
    assert!(ended(world, change).is_some(), "goal change ended: {:?}", row(world, change));
    assert!(matches!(row(world, change).change.state, temper_engine_domain_forge_change::State::Landed { .. }));
}

fn land_into_goal(world: &mut World, goal: &engine::Assignment, branch: &[u8], call: u64, file: &[u8]) -> u64 {
    let (change, producer) = producing_into_goal(world, goal, branch, call, file);
    finish_goal_change(world, change, &producer);
    branch_tip(world, branch)
}

fn checking_landing() -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    let (mut world, goal, branch) = running_goal();
    let head = land_into_goal(&mut world, &goal, &branch, LANDING_CALL + 1, b"plan");
    status(&mut world, head, raw::Check::Pending);
    let answer = mine(&mut world, &goal, LANDING_CALL, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing admitted: {answer:?}") };
    let landing = made[0].task;
    for _ in 0..STEPS {
        world.tick();
        if matches!(row(&world, landing).change.state, temper_engine_domain_forge_change::State::Checking { .. }) {
            break;
        }
    }
    assert_eq!(
        row(&world, landing).change.state,
        temper_engine_domain_forge_change::State::Checking { head: translate::commit(head) }
    );
    (world, goal, branch, landing, head)
}

fn assert_not_held(world: &World, landing: u64) {
    assert!(
        !matches!(row(world, landing).change.state, temper_engine_domain_forge_change::State::Held { .. }),
        "{:?}",
        row(world, landing)
    );
    assert_eq!(row(world, landing).drift, None);
    assert_eq!(row(world, landing).change.repairs, 0);
}

fn assert_landing_at(world: &mut World, landing: u64, head: u64) {
    let number = row(world, landing).pull.expect("goal landing pull");
    status(world, head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        assert_not_held(world, landing);
        if ended(world, landing).is_some() {
            break;
        }
    }
    assert!(matches!(row(world, landing).change.state, temper_engine_domain_forge_change::State::Landed { .. }));
    assert!(ended(world, landing).is_some(), "the goal receives its landing's result");
    assert_eq!(row(world, landing).change.last_head, Some(translate::commit(head)));
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { number: pull, head: merged, .. }
            if *pull == number && *merged == translate::commit(head))));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("goal landing pull")
    };
    assert_eq!(pull.commit, head, "the forge merged exactly the checked head");
    assert!(pull.merged.is_some());
}

fn finish_run(world: &mut World, run: &engine::Assignment, result: tasks::TaskResult) {
    let event = world.protocol.answer(
        Token::new(7),
        run.task,
        run.attempt,
        5,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
}

fn finish_goal(world: &mut World, goal: &engine::Assignment) {
    finish_run(world, goal, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    for _ in 0..STEPS {
        world.tick();
        if ended(world, goal.task).is_some() {
            break;
        }
    }
    assert!(matches!(ended(world, goal.task).expect("goal ended").phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    world.until(Until::Ready);
}

fn cancel_task(world: &mut World, task: u64) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(951)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [96; 16],
        ask: people::Ask::Cancel { project: 1, task, reason: b"stop the landing".as_slice().into(), grace: None },
    }));
    if let Some(run) = world.assigned.iter().find(|run| run.task == task).cloned() {
        // The scripted worker drains its cancelled run and returns its racing result.
        finish_run(world, &run, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    }
    for _ in 0..STEPS {
        world.tick();
        if ended(world, task).is_some() {
            break;
        }
    }
    assert!(matches!(
        ended(world, task).expect("cancelled task ended").phase,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. })
    ));
    world.until(Until::Ready);
}

#[test]
fn a_change_into_the_goal_branch_while_its_landing_is_checked_is_its_new_head() {
    let (mut world, goal, branch, landing, old) = checking_landing();
    let head = land_into_goal(&mut world, &goal, &branch, LANDING_CALL + 2, b"implementation");
    assert_ne!(head, old);
    status(&mut world, head, raw::Check::Pending);
    let mut read_ci = false;
    for _ in 0..STEPS {
        world.tick();
        assert_not_held(&world, landing);
        read_ci |= world.pending.values().any(|op| {
            matches!(op,
            skein_forge::api::Op::Read(skein_forge::api::Read::Checks { commit })
                if *commit == translate::commit(head))
        });
        if read_ci && row(&world, landing).change.last_head == Some(translate::commit(head)) {
            break;
        }
    }
    assert!(read_ci, "CI is read at the goal branch's new head");
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(head)));
    assert_eq!(world.merges, TWO_CHANGE_MERGES, "the goal landing waits for CI at its new head");
    assert_landing_at(&mut world, landing, head);
    assert_eq!(world.merges, GOAL_MERGES);
    assert_goal_holds(&world, goal.task, &branch);
    finish_goal(&mut world, &goal);
    world.close();
}

#[test]
fn a_move_of_the_goal_branch_by_another_identity_holds_its_landing_for_drift() {
    use temper_engine_domain_forge_change::{Hold, State};
    let (mut world, goal, branch, landing, old) = checking_landing();
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"outside", b"pushed", 2)
        .expect("another identity moves the goal branch");
    status(&mut world, head, raw::Check::Pending);
    for _ in 0..STEPS {
        world.tick();
        if matches!(row(&world, landing).change.state, State::Held { why: Hold::BranchMoved, .. })
            && matches!(live_task(&world, landing).phase, tasks::Phase::Held { .. })
            && forge_hold(&world, &branch).is_some_and(|hold| hold.drift.is_some())
        {
            break;
        }
    }
    assert_eq!(
        row(&world, landing).change.state,
        State::Held { was: Box::new(State::Checking { head: translate::commit(old) }), why: Hold::BranchMoved }
    );
    assert!(matches!(live_task(&world, landing).phase, tasks::Phase::Held { .. }));
    let hold = forge_hold(&world, &branch).expect("the goal retains its branch");
    assert_eq!(hold.task, goal.task);
    assert_eq!(
        hold.drift.as_ref().expect("the hold names the outside move").change,
        forge_top::DriftChange::Branch { expected: translate::commit(old), observed: Some(translate::commit(head)) }
    );
    assert_eq!(row(&world, landing).drift, Some(Hold::BranchMoved));
    assert_eq!(world.merges, PLAN_MERGES, "only the plan landed; the goal landing merges nothing");
    let number = row(&world, landing).pull.expect("goal landing pull");
    assert!(!world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { number: pull, .. } if *pull == number)));
    assert_eq!(branch_tip(&mut world, &branch), head, "the outside move is kept");
    cancel_task(&mut world, goal.task);
    assert_eq!(world.merges, PLAN_MERGES);
    world.close();
}

fn branch_deleted(world: &World, branch: &[u8]) -> bool {
    world.writes.iter().any(|write| {
        matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: deleted } if deleted.as_ref() == branch)
    })
}

#[test]
fn a_goal_closing_after_its_landing_deletes_its_branch() {
    let (mut world, goal, branch, landing, head) = checking_landing();
    assert_landing_at(&mut world, landing, head);
    assert!(matches!(
        ended(&world, landing).expect("landing result given").phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. }))
    ));
    assert!(live_task(&world, goal.task).inbox.iter().any(|message| message.from == tasks::Party::Task(landing)
        && matches!(message.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))));
    assert_goal_holds(&world, goal.task, &branch);
    assert!(!branch_deleted(&world, &branch), "L leaves g to G's release");
    finish_goal(&mut world, &goal);
    assert!(branch_deleted(&world, &branch), "G's release deletes the branch it landed");
    assert!(forge_hold(&world, &branch).is_none());
    let raw::Answer::Cloned { branches, .. } = world.external(1, raw::Op::Git(raw::Git::Clone)) else {
        panic!("repository branches")
    };
    assert!(!branches.iter().any(|found| found.branch == branch), "g is absent from the forge");
    world.close();
}

#[test]
fn a_goal_closing_after_cancelling_its_landing_keeps_its_branch() {
    let (mut world, goal, branch, landing, head) = checking_landing();
    cancel_task(&mut world, landing);
    assert_eq!(row(&world, landing).change.state, temper_engine_domain_forge_change::State::Ended);
    assert_eq!(world.merges, PLAN_MERGES, "L was cancelled before its merge");
    assert_goal_holds(&world, goal.task, &branch);
    assert!(!branch_deleted(&world, &branch), "L's cancel deletes no goal branch");
    finish_goal(&mut world, &goal);
    assert!(forge_hold(&world, &branch).is_none(), "G releases its hold");
    assert!(!branch_deleted(&world, &branch), "G keeps its unlanded branch");
    assert_eq!(branch_tip(&mut world, &branch), head);
    assert_eq!(world.merges, PLAN_MERGES);
    world.close();
}

#[test]
fn a_goal_branch_read_sent_before_a_merge_into_it_does_not_hold_its_landing() {
    let (mut world, goal, branch, landing, old) = checking_landing();
    let (change, producer) = producing_into_goal(&mut world, &goal, &branch, LANDING_CALL + 2, b"implementation");
    world.slow_branch = Some(branch.clone());
    let mut read = None;
    for _ in 0..STEPS {
        world.tick();
        read = world.pending.iter().find_map(|(call, op)| match op {
            skein_forge::api::Op::Read(skein_forge::api::Read::Branch { branch: found }) if *found == branch => {
                Some(*call)
            }
            _ => None,
        });
        if read.is_some() {
            break;
        }
    }
    let read = read.expect("a read of g is sent while L checks the old head");
    world.slow_branch = None;
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(old)));
    finish_goal_change(&mut world, change, &producer);
    let head = branch_tip(&mut world, &branch);
    assert_ne!(head, old);
    assert!(world.pending.contains_key(&read), "the change lands while g's old read is still unanswered");
    status(&mut world, head, raw::Check::Pending);
    let mut stale_answered = false;
    for _ in 0..STEPS {
        world.tick();
        assert_not_held(&world, landing);
        stale_answered |= world.events.iter().any(|event| {
            matches!(event,
            engine::Event::Forge(event) if matches!(event.as_ref(),
                forge_top::Event::Client(skein_forge_client::Event::Answered {
                    call, result: Ok(skein_forge::api::Answer::Commit(tip)), ..
                }) if *call == read && *tip == translate::commit(old)))
        });
        if stale_answered && row(&world, landing).change.last_head == Some(translate::commit(head)) {
            break;
        }
    }
    assert!(stale_answered, "the delayed read answers A after B has landed into g");
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(head)));
    assert_landing_at(&mut world, landing, head);
    assert_eq!(world.merges, GOAL_MERGES);
    finish_goal(&mut world, &goal);
    world.close();
}
