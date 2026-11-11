//! A goal lands its held branch without handing it to the change
//! (temper/docs/design/domain/agent/tools.md, 1; temper/docs/design/domain/forge/change/README.md, 1 and temper/docs/design/domain/forge/branches.md).
//! Its delegates' moves become its landing's new head, and its close releases the branch
//! (temper/docs/design/domain/forge/change/README.md, 3; temper/docs/design/domain/after-landing.md, 6).
use super::goal_branch::{branch_tip, change, forge_hold, resource, running_goal, writer};
use super::*;

const STEPS: u32 = 1000; // The forge world's bound for a full landing and hold release.
const LANDING_CALL: u64 = 95;

pub(super) fn mine(world: &mut World, run: &engine::Assignment, call: u64, input: &[u8]) -> Option<engine::CallAnswer> {
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

pub(super) fn row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change {task}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

pub(super) fn assert_goal_holds(world: &World, goal: u64, branch: &[u8]) {
    assert_eq!(forge_hold(world, branch).expect("goal branch hold").task, goal);
    assert!(live_task(world, goal).holdings.contains(&tasks::Holding::Write { resource: resource(branch), kind: 1 }));
}

fn enable_scope_questions(world: &mut World) {
    use temper_engine_domain_world::kinds;
    let revision = kinds::revision(&world.store.rows);
    world.send(kinds::change(
        world.signed_in.expect("owner"),
        revision,
        191,
        Box::new([kinds::item(
            6,
            1,
            1,
            &forge_top::settings::Value::Questions(temper_engine_domain_forge_change::Route::Up),
        )]),
    ));
    world.until(Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), revision + 1, "{:?}", world.web_replies);
}

fn settle_scope_question(
    world: &mut World,
    goal: &engine::Assignment,
    child: u64,
    producer: &engine::Assignment,
    call: u64,
    words: &[u8],
    release: bool,
) {
    world.producer_question(producer, call, words);
    world.until(Until::Ready);
    if release {
        for _ in 0..STEPS {
            world.tick();
            if matches!(live_task(world, child).phase, tasks::Phase::Held { .. }) && world.store.pending.is_empty() {
                break;
            }
        }
        assert!(matches!(live_task(world, child).phase, tasks::Phase::Held { .. }));
        let revision = live_task(world, child).revisions.hold;
        assert!(matches!(
            world.scripted_call(goal, call + 1, engine::Tool::Release { target: child, revision, again: None }),
            engine::CallAnswer::Revised { .. }
        ));
    }
    for _ in 0..STEPS {
        world.tick();
        if row(world, child)
            .questions
            .open
            .first()
            .is_some_and(|q| q.stands == temper_engine_domain_forge_change::Stands::Asked)
        {
            break;
        }
    }
    let question = questions::replies(world, goal.task, child)
        .into_iter()
        .find(|word| word.kind == tasks::MessageKind::Question && word.words.as_ref() == words)
        .expect("scope question reaches goal")
        .number;
    world.goal_answer(goal, call + 2, child, question, b"Yes, keep this scope.");
    questions::wait_answer(world, producer.task, child);
}

fn await_landed(world: &mut World, change: u64) {
    let mut reviewed = None;
    for _ in 0..STEPS {
        world.tick();
        if let Some((task, temper_engine_domain_forge_change::Delegate::Gate { .. })) = row(world, change).delegate
            && let Some(run) = world.assigned.iter().find(|run| run.task == task).cloned()
            && reviewed != Some(task)
        {
            reviewed = Some(task);
            reviews::finish_review(
                world,
                &run,
                tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
            );
        }
        if ended(world, change).is_some() {
            break;
        }
    }
    assert!(ended(world, change).is_some(), "change ended: {:?}", row(world, change));
    assert!(matches!(row(world, change).change.state, temper_engine_domain_forge_change::State::Landed { .. }));
}

#[test]
fn a_whole_goals_words_only_child_lands_into_its_goal_branch() {
    let (mut world, goal, branch) = goal_branch::running_stock_goal();
    let main = branch_tip(&mut world, b"main");
    enable_scope_questions(&mut world);
    // Reconstructed brief: the retained walk has no verbatim tool transcript.
    let answer = mine(&mut world, &goal, LANDING_CALL,
        br#"{"words":"Commit the source draft and the product document. Land this change into the goal branch; the whole goal lands into main after all tasks and checks complete."}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("child change admitted: {answer:?}") };
    let child = made[0].task;
    world.until(Until::Ready);
    assert_eq!(row(&world, child).base, branch, "words-only child stays on the whole goal's branch");
    let producer = questions::next_producer(&mut world, child);
    settle_scope_question(&mut world, &goal, child, &producer, 110, b"May I commit the source draft?", false);
    settle_scope_question(&mut world, &goal, child, &producer, 120, b"May I commit the product document?", true);
    let change_branch = row(&world, child).branch.clone();
    push_goal_child(&mut world, child, &branch, b"draft.md");
    let head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &change_branch, b"product.md", b"design", 1)
            .expect("producer pushes the product document");
    status(&mut world, head, raw::Check::Passed);
    finish_goal_change(&mut world, child, &producer);
    let number = row(&world, child).pull.expect("child pull");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("child pull")
    };
    assert_eq!(pull.base, branch);
    assert_ne!(branch_tip(&mut world, &branch), main);
    assert_eq!(branch_tip(&mut world, b"main"), main);
    assert!(ended(&world, goal.task).is_none(), "the goal remains live after its child lands");
    let head = branch_tip(&mut world, &branch);
    let answer = mine(&mut world, &goal, LANDING_CALL + 1, br#"{"words":"Land the goal whole","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("final landing admitted: {answer:?}") };
    let landing = made[0].task;
    status(&mut world, head, raw::Check::Passed);
    await_landed(&mut world, landing);
    let raw::Answer::Pull(pull) = world
        .external(1, raw::Op::Read(raw::Read::Pull { number: row(&world, landing).pull.expect("whole goal pull") }))
    else {
        panic!("whole goal pull")
    };
    assert_eq!(pull.base.as_ref(), b"main");
    assert_eq!(pull.head, branch);
    assert!(pull.merged.is_some());
    assert_ne!(branch_tip(&mut world, b"main"), main);
    finish_goal(&mut world, &goal);
    world.close();
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
            tasks::Parameter::Bytes { name: 10, value: b"Land the goal".as_slice().into() },
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
    let before = news_count(&world, goal.task);
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
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    await_new_forge_news(&mut world, goal.task, before);
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

const OUTSIDE_GOAL: &[u8] =
    b"landing: this task's goal holds a branch; land child changes into it, then land it whole with branch mine.";

#[test]
fn a_whole_goals_explicit_base_child_is_refused_before_delegation() {
    let (mut world, goal, _) = running_goal();
    refused(&mut world, &goal, LANDING_CALL, br#"{"words":"Bypass goal isolation","landing":"main"}"#, OUTSIDE_GOAL);
    refused(
        &mut world,
        &goal,
        LANDING_CALL + 1,
        br#"{"changes":[{"words":"Valid child"},{"words":"Bypass goal isolation","landing":"main"}]}"#,
        OUTSIDE_GOAL,
    );
    world.close();
}

fn parallel_goal() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { slots: Some(3), tasks_batch: Some(2), authority_batch: Some(2), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 3, Box::new([])));
    let request = goal_branch::goal_request(&world, true);
    let goal = assigned_chat(&mut world, 92, request);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    (world, goal, branch)
}

#[test]
fn a_whole_goals_batched_omissions_stay_on_its_goal_branch() {
    let (mut world, goal, branch) = parallel_goal();
    let answer =
        mine(&mut world, &goal, LANDING_CALL, br#"{"changes":[{"words":"First child"},{"words":"Second child"}]}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("batch admitted: {answer:?}") };
    assert_eq!(made.len(), 2);
    world.until(Until::Ready);
    let mut producers = Vec::new();
    for child in &made {
        producers.push(questions::next_producer(&mut world, child.task));
    }
    for (child, producer) in made.iter().zip(&producers) {
        assert!(
            live_task(&world, child.task)
                .spec
                .parameters
                .contains(&tasks::Parameter::Bytes { name: 2, value: branch.clone() })
        );
        push_goal_child(&mut world, child.task, &branch, &child.task.to_be_bytes());
        finish_goal_change(&mut world, child.task, producer);
    }
    finish_goal(&mut world, &goal);
    world.close();
}

#[test]
fn a_whole_goals_subtree_inherits_its_landing_and_refuses_the_base() {
    let (mut world, goal, branch) = parallel_goal();
    let mut authority = live_task(&world, goal.task).authority.clone();
    authority.budget.spend = 100;
    authority.delegation.tasks = 6;
    authority.delegation.depth = authority.delegation.depth.saturating_sub(1);
    let assigned = world.assigned.len();
    let answer = world.scripted_call(
        &goal,
        LANDING_CALL,
        engine::Tool::Delegate {
            batch: Box::new([engine::Delegate {
                authority,
                ..temper_engine_domain_world::fixtures::report_delegate(b"Coordinate a child", Box::new([]))
            }]),
        },
    );
    assert!(matches!(answer, engine::CallAnswer::Delegated(_)), "subtree coordinator admitted: {answer:?}");
    let coordinator = reviews::next_assignment(&mut world, assigned);
    refused(
        &mut world,
        &coordinator,
        LANDING_CALL + 1,
        br#"{"words":"Bypass from subtree","landing":"main"}"#,
        OUTSIDE_GOAL,
    );
    let answer = mine(&mut world, &coordinator, LANDING_CALL + 2, br#"{"words":"A child from the subtree"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("subtree child admitted: {answer:?}") };
    assert!(
        live_task(&world, made[0].task)
            .spec
            .parameters
            .contains(&tasks::Parameter::Bytes { name: 2, value: branch.clone() })
    );
    let child = made[0].task;
    let producer = questions::next_producer(&mut world, child);
    push_goal_child(&mut world, child, &branch, b"subtree");
    finish_goal_change(&mut world, child, &producer);
    finish_run(&mut world, &coordinator, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, coordinator.task).is_some() {
            break;
        }
    }
    assert!(ended(&world, coordinator.task).is_some());
    finish_goal(&mut world, &goal);
    world.close();
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
    // An unchanged goal branch ends with no change instead of keeping a live landing.
    let (mut world, goal, _, landing, _) = checking_landing();
    let words = format!("branch mine: change task {landing} is on it already.");
    refused(&mut world, &goal, LANDING_CALL + 2, br#"{"words":"Land again","branch":"mine"}"#, words.as_bytes());
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

pub(super) fn status(world: &mut World, head: u64, state: raw::Check) {
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
    // Scripted CI ends deliver their commit's news; a status written as the
    // worker alone sends no fake Actions delivery and would await fallback.
    if matches!(state, raw::Check::Passed | raw::Check::Failed) {
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
            hint: skein_forge::api::Hint {
                repository: forge_world::REPO,
                change: skein_forge::api::Change::Commit(translate::commit(head)),
                key: None,
            },
        })));
    }
}

pub(super) fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

pub(super) fn producing_into_goal(
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
    push_goal_child(world, change, branch, file);
    (change, producer)
}

fn push_goal_child(world: &mut World, change: u64, branch: &[u8], file: &[u8]) {
    let change_branch = row(world, change).branch.clone();
    let base = branch_tip(world, branch);
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: change_branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &change_branch, file, b"added", 1)
        .expect("producer pushes its change");
    status(world, head, raw::Check::Passed);
}

fn finish_goal_change(world: &mut World, change: u64, producer: &engine::Assignment) {
    let tasks::Contract::Change { connector, .. } = live_task(world, producer.task).contract else {
        panic!("producer's change contract")
    };
    finish_run(
        world,
        producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Add to the goal".as_slice().into()),
            connector,
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

pub(super) fn land_into_goal(
    world: &mut World,
    goal: &engine::Assignment,
    branch: &[u8],
    call: u64,
    file: &[u8],
) -> u64 {
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

pub(super) fn finish_run(world: &mut World, run: &engine::Assignment, result: tasks::TaskResult) {
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

#[test]
fn a_closers_landing_checks_its_updates_head_and_lands_with_or_without_a_close_out() {
    use temper_engine_domain_forge_change::State;
    for close_out in [false, true] {
        let (mut world, goal, branch, closer, landing) = super::goal_closer::updating_landing(close_out);
        let State::Updating { from, .. } = row(&world, landing).change.state else { panic!("updating") };
        world.forge_reads = Some(Vec::new());
        for _ in 0..STEPS {
            world.tick();
            assert_not_held(&world, landing);
            if matches!(row(&world, landing).change.state, State::Checking { .. }) {
                break;
            }
        }
        let head = branch_tip(&mut world, &branch);
        assert_ne!(translate::commit(head), from);
        assert_eq!(row(&world, landing).change.state, State::Checking { head: translate::commit(head) });
        assert_eq!(row(&world, landing).change.clean.as_ref(), &[from]);
        assert_eq!(row(&world, landing).change.updates, 1);
        assert_goal_holds(&world, goal.task, &branch);
        let mut read_ci = false;
        for _ in 0..STEPS {
            world.tick();
            assert_not_held(&world, landing);
            read_ci = world.forge_reads.as_ref().expect("reads recorded").iter().any(|(_, read)| {
                matches!(read, skein_forge::api::Read::Checks { commit } if *commit == translate::commit(head))
                || matches!(read, skein_forge::api::Read::Pull { number, ci: true } if row(&world, landing).pull == Some(*number))
            });
            if read_ci {
                break;
            }
        }
        assert!(read_ci, "the updated head is checked afresh");
        assert_landing_at(&mut world, landing, head);
        super::goal_closer::wait_ended(&mut world, closer);
        world.close();
    }
}

#[test]
fn a_push_from_outside_during_a_closers_landing_update_holds_with_both_heads() {
    use temper_engine_domain_forge_change::{Hold, State};
    let (mut world, goal, branch, _, landing) = super::goal_closer::updating_landing(false);
    let expected = row(&world, landing).change.last_head.expect("own head");
    world.hold_write_answers = Some(HoldWrite::All);
    let head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"outside", b"another identity", 2)
            .expect("outside push");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::BranchMoved {
                branch: branch.clone(),
                head: translate::commit(head),
                actor: 2,
            },
            key: None,
        },
    })));
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, landing).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    assert!(matches!(row(&world, landing).change.state, State::Held { why: Hold::BranchMoved, .. }));
    let tasks::Phase::Held { source: Some(source), why: tasks::Hold::Drift, .. } = &live_task(&world, landing).phase
    else {
        panic!("drift's task account: {:?}", live_task(&world, landing).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("drift account") };
    let before = forge_top::commit_text(&expected);
    let after = forge_top::commit_text(&translate::commit(head));
    assert_eq!(
        words.as_ref(),
        format!(
            "its branch moved: expected {}, found {}",
            core::str::from_utf8(&before[..12]).unwrap(),
            core::str::from_utf8(&after[..12]).unwrap()
        )
        .as_bytes()
    );
    world.hold_write_answers = None;
    world.events.extend(world.held_write_answers.drain(..));
    world.signed_in = None;
    world.sign_in();
    cancel_task(&mut world, goal.task);
    world.close();
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
                || matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Pull { number, ci: true })
                    if row(&world, landing).pull == Some(*number))
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
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::BranchMoved {
                branch: branch.clone(),
                head: translate::commit(head),
                actor: 2,
            },
            key: None,
        },
    })));
    for _ in 0..STEPS {
        world.tick();
        if matches!(row(&world, landing).change.state, State::Held { why: Hold::BranchMoved, .. })
            && matches!(live_task(&world, landing).phase, tasks::Phase::Held { .. })
            && forge_hold(&world, &branch).is_some_and(|hold| hold.drift.is_some())
        {
            break;
        }
    }
    assert!(matches!(row(&world, landing).change.state, State::Held { why: Hold::BranchMoved, .. }));
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
    // The slow branch observation can outlive the small profile's sign-in.
    world.signed_in = None;
    world.sign_in();
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
    let read = super::goal_branch::start_slow_keep_read(&mut world, &branch);
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
