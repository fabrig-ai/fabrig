//! A change keeps its named producer and role LLM choices through delegation and assignment
//! (temper/docs/design/domain/agent.md, 4.3.1; temper/docs/design/domain/forge.md, 8.1 and 8.9).
use super::goal_branch::goal_request;
use super::*;

const CHANNEL: Token = Token::new(7);
const GOAL_REQUEST: u8 = 92;
const CHANGE_CALL: u64 = 93;
const STEPS: u32 = 1000; // The forge world's bound for a full landing.
const FIRST: &[u8] = b"scripted";
const SECOND: &[u8] = b"scripted-second";
const PRODUCER: u32 = 2;
const REVIEWER: u32 = 4;
const SECOND_MODEL: u8 = 1; // The second configured model's index.
const PRODUCER_RUN: usize = 1; // The goal is assigned before its change's producer.
const REVIEW_RUN: usize = 2; // The gate follows the goal and producer.
const ONE_CHANGE: usize = 1;
const RESULT_BYTES: u32 = 384; // Leaves room for the goal's messages in its 1024-byte inbox.

fn models() -> Box<[engine::Model]> {
    let first = walking::charter().policy.model;
    let second = engine::Model { name: SECOND.into(), ..first.clone() };
    Box::new([first, second])
}

fn choice(model: u8) -> tasks::LlmChoice {
    tasks::LlmChoice { account: None, main: tasks::ModelEffort { model, effort: None }, subagents: Box::new([]) }
}

fn goal(allow_producer: bool) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        STEPS,
        Gate { agent: true, ..Gate::default() },
        None,
        StoryLimits { result_bytes: Some(RESULT_BYTES), ..StoryLimits::default() },
    )
    .with_producer_model(SECOND);
    world.adopt();
    let mut event = protocol::hello_slots(CHANNEL, 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    hello.bounds.models = models()
        .iter()
        .map(|model| host::ModelBound { name: model.name.clone(), window: model.window, output: model.output })
        .collect();
    world.send(event);
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    if !allow_producer {
        batch[0].authority.delegation.kinds = Box::new([
            people::Executor::Charter(1),
            people::Executor::Charter(REVIEWER),
            people::Executor::Procedure(2),
        ]);
    }
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    (world, goal)
}

fn change(world: &mut World, goal: &engine::Assignment, producer: u32) -> engine::CallAnswer {
    let input = format!(
        r#"{{"words":"Small fix","producer":{producer},"llm":{{"producer":{{"main":{{"model":"{}"}}}},"review":{{"main":{{"model":"{}"}}}}}}}}"#,
        core::str::from_utf8(SECOND).expect("model name"),
        core::str::from_utf8(FIRST).expect("model name"),
    );
    let input = smith_domain_run::HostInput::attested(input.into_bytes().into()).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call_configured(
        smith_domain_run::CallName { activation: goal.attempt, completion: 1, position: 1 },
        b"change",
        &input,
        &models(),
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let before = world.answers.len();
    let event = world.protocol.call(CHANNEL, goal.task, goal.attempt, CHANGE_CALL, call);
    world.send(event);
    world.until(Until::Ready);
    world.answers.get(before).expect("change answered").clone()
}

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change {task}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn task_rows(world: &World) -> Vec<tasks::TaskRecord> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) => {
                Some(row.as_ref().clone())
            }
            _ => None,
        })
        .collect()
}

fn error_text(answer: &engine::CallAnswer) -> String {
    let rendered = temper_engine_smith::answer(answer);
    assert!(rendered.error(), "a refused change is an error");
    String::from_utf8(rendered.text().to_vec()).expect("UTF-8 call error")
}

fn produce(world: &mut World, producer: &engine::Assignment, branch: &[u8]) {
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.into(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"file", b"fixed", 1)
        .expect("producer pushes its work");
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
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

#[test]
fn a_change_names_its_producer_and_llm_and_its_tasks_run_under_them() {
    let (mut world, goal) = goal(true);
    let engine::CallAnswer::Delegated(made) = change(&mut world, &goal, PRODUCER) else {
        panic!("change admitted: {:?}; {:?}", world.answers, world.call_errors)
    };
    assert_eq!(made.len(), ONE_CHANGE);
    let change = made[0].task;
    assert!(
        live_task(&world, change)
            .spec
            .parameters
            .contains(&tasks::Parameter::Bytes { name: 11, value: b"2;1;0".as_slice().into() })
    );
    let row = change_row(&world, change);
    assert_eq!(row.producing_charter, PRODUCER);
    assert_eq!(row.producing_llm, Some(choice(SECOND_MODEL)));
    assert_eq!(row.review_charter, REVIEWER);
    assert_eq!(row.review_llm, Some(choice(0)));
    let branch = row.branch.clone();
    let producer = reviews::next_assignment(&mut world, PRODUCER_RUN);
    let record = live_task(&world, producer.task);
    assert_eq!(record.requester, tasks::Party::Task(change));
    assert_eq!(record.executor, tasks::Executor::Agent { charter: PRODUCER });
    assert_eq!(record.llm, Some(choice(SECOND_MODEL)));
    assert_eq!(producer.charter, PRODUCER);
    assert_eq!(producer.run.policy.model.name.as_ref(), SECOND);
    produce(&mut world, &producer, &branch);
    let reviewer = reviews::next_assignment(&mut world, REVIEW_RUN);
    let record = live_task(&world, reviewer.task);
    assert_eq!(record.requester, tasks::Party::Task(change));
    assert_eq!(record.executor, tasks::Executor::Agent { charter: REVIEWER });
    assert_eq!(record.llm, Some(choice(0)));
    assert_eq!(reviewer.charter, REVIEWER);
    assert_eq!(reviewer.run.policy.model.name.as_ref(), FIRST);
    world.close();
}

#[test]
fn a_change_naming_a_charter_the_run_may_not_delegate_to_is_refused() {
    let (mut world, goal) = goal(false);
    let before = task_rows(&world);
    let answer = change(&mut world, &goal, PRODUCER);
    assert!(matches!(&answer, engine::CallAnswer::DelegationDenied {
        answer: authority::Answer::Propose, findings,
    } if findings.iter().any(|finding| matches!(finding,
        authority::Finding::Authority { source: authority::Source::Task, lacks } if lacks.executors))));
    assert_eq!(task_rows(&world), before, "refusal makes no task");
    assert_eq!(world.assigned.len(), PRODUCER_RUN, "only the goal runs");
    let text = error_text(&answer);
    assert!(text.contains("Beyond authority") && text.contains("missing executors"), "{text}");
    world.close();
}

#[test]
fn a_change_naming_a_model_its_producer_s_charter_does_not_allow_is_refused() {
    let (mut world, goal) = goal(true);
    let before = task_rows(&world);
    let answer = change(&mut world, &goal, 1);
    assert!(matches!(&answer, engine::CallAnswer::DelegationRefused(problem)
        if problem.why == tasks::Refusal::Llm { model: SECOND_MODEL, why: tasks::LlmFault::Charter }));
    assert_eq!(task_rows(&world), before, "refusal makes no task");
    assert_eq!(world.assigned.len(), PRODUCER_RUN, "only the goal runs");
    let text = error_text(&answer);
    assert!(text.contains("scripted-second") && text.contains("model outside the charter"), "{text}");
    world.close();
}
