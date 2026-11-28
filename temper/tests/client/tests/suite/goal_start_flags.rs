//! A start's flags hold for every run (temper/docs/design/client/ctl/areas.md).
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use temper_client_world::{
    app::{Event, Root},
    fixtures,
    front::{Joint, account_acts as accounts},
};

fn call(front: &mut Joint, root: &mut Root, number: u32, words: &[&[u8]]) -> temper_client_tools::jig::runner::Printed {
    let Event::Call { call, json, .. } = fixtures::call(words) else { unreachable!() };
    accounts::event(front, root, Event::Call { number, call, json, value: None });
    let printed = root.agent().printed(number).expect("call answered");
    temper_client_tools::jig::runner::Printed {
        file: printed.file.clone(),
        status: printed.status,
        stdout: printed.stdout.clone(),
        stderr: printed.stderr.clone(),
    }
}
fn shown(front: &mut Joint, root: &mut Root, number: u32, goal: u64) -> String {
    let answer = call(front, root, number, &[b"task", b"show", goal.to_string().as_bytes()]);
    assert_eq!(answer.status, 0, "{answer:?}");
    String::from_utf8(answer.stdout.to_vec()).expect("task text")
}

fn gathering(front: &mut Joint) -> u64 {
    let people::Ask::Create { batch, .. } = front.engine.chat_ask(1, b"offered authority") else { panic!("create") };
    let mut authority = batch[0].authority.clone();
    authority.spend = 100;
    authority.deadline = None;
    authority.delegation.depth = 1;
    authority.delegation.tasks = 2;
    let gathering = people::Member {
        executor: people::TaskExecutor::Procedure {
            connector: jig_client::shapes::RECURRING,
            code: jig_client::shapes::GATHERING,
        },
        words: b"Store shape".as_slice().into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        contract: people::Contract::Report { words: 128 },
        authority,
        budget: people::Budget::Named,
        deadline: people::Deadline::Named,
        wake: None,
        dependencies: Box::new([]),
        named: Box::new([]),
        symbols: Box::new([]),
        tracked: Some(0),
        label: 0,
        flags: people::Flags {
            ended_by: people::EndedBy::Requester,
            conversation: false,
            listed: true,
            retention: people::Retention::Kept,
        },
        llm: None,
    };
    let outcome = accounts::party_ask(front, 91, people::Ask::Create { project: 1, batch: Box::new([gathering]) });
    let people::Outcome::Created { first, .. } = outcome else { panic!("gathering refused: {outcome:?}") };
    first
}

#[test]
fn a_person_starts_waiting_goals_beneath_a_gathering() {
    let (mut front, mut root) = accounts::goal_choice_fixture(fixtures::SEED);
    let milestone = gathering(&mut front);
    let milestone_arg = milestone.to_string();
    let first = call(
        &mut front,
        &mut root,
        1,
        &[
            b"goal",
            b"start",
            b"--project",
            b"1",
            b"--budget",
            b"0.10",
            b"--no-deadline",
            b"--milestone",
            milestone_arg.as_bytes(),
            b"S1 Draft the layout",
            b"--key",
            b"draft-layout",
        ],
    );
    assert_eq!(first.status, 0, "{first:?}");
    let first = fixtures::done_task(&first);
    let first_arg = first.to_string();
    let second = call(
        &mut front,
        &mut root,
        2,
        &[
            b"goal",
            b"start",
            b"--project",
            b"1",
            b"--budget",
            b"0.10",
            b"--no-deadline",
            b"--milestone",
            milestone_arg.as_bytes(),
            b"--after",
            first_arg.as_bytes(),
            b"S2 Check the layout",
            b"--key",
            b"check-layout",
        ],
    );
    assert_eq!(second.status, 0, "{second:?}");
    let second = fixtures::done_task(&second);
    for goal in [first, second] {
        let task = front.engine.task(goal).expect("goal row");
        assert_eq!(task.requester, tasks::Party::Task(milestone));
        assert_eq!(task.funder, tasks::Funder::Task(milestone));
        assert_eq!(task.authority.budget.deadline, None);
    }
    let waiting = front.engine.task(second).expect("second goal");
    assert!(matches!(waiting.phase, tasks::Phase::Waiting));
    assert_eq!(waiting.dependencies.as_ref(), &[first]);
    let parent = front.engine.task(milestone).expect("gathering");
    assert!(parent.delegates.contains(&first) && parent.delegates.contains(&second));
}

#[test]
fn a_persons_start_flags_hold_after_the_goals_first_run() {
    let (mut front, mut root) = accounts::goal_choice_fixture(fixtures::SEED);
    let started = call(
        &mut front,
        &mut root,
        1,
        &[
            b"goal",
            b"start",
            b"--project",
            b"1",
            b"--template",
            b"2",
            b"--as-allowed",
            b"--model",
            b"effort-model",
            b"--effort",
            b"high",
            b"--sub-agent",
            b"scripted:medium",
            b"Follow my model choice",
            b"--key",
            b"flags",
        ],
    );
    assert_eq!(started.status, 0, "{started:?}");
    let goal = fixtures::done_task(&started);
    let before = shown(&mut front, &mut root, 2, goal);
    assert!(before.contains("\nmain model: effort-model/high\nsub-agent model: scripted/medium\n"), "{before}");
    let run = front.assignments.last().expect("goal assigned to the worker").clone();
    assert_eq!(run.task, goal);
    front.engine.answer(
        fixtures::QUESTION_WORKER,
        goal,
        run.attempt,
        0,
        tasks::End::Parked,
        temper_engine_domain::Parts::none(),
    );
    front.engine.settle();
    let after = shown(&mut front, &mut root, 3, goal);
    assert!(after.contains("\nmain model: effort-model/high\nsub-agent model: scripted/medium\n"), "{after}");
}

#[test]
fn a_persons_unoffered_model_flag_makes_no_goal() {
    let (mut front, mut root) = accounts::goal_choice_fixture(fixtures::SEED);
    let before = call(&mut front, &mut root, 1, &[b"task", b"list", b"--project", b"1", b"--json"]);
    assert_eq!(before.status, 0, "{before:?}");
    let refused = call(
        &mut front,
        &mut root,
        2,
        &[
            b"goal",
            b"start",
            b"--project",
            b"1",
            b"--template",
            b"2",
            b"--as-allowed",
            b"--model",
            b"unoffered-model",
            b"Do not make this goal",
            b"--key",
            b"refused",
        ],
    );
    assert_eq!(refused.status, 1, "{refused:?}");
    assert_eq!(
        refused.stderr.as_ref(),
        b"--model unoffered-model: not among the models you may choose in this project\n"
    );
    assert!(refused.stdout.is_empty());
    let after = call(&mut front, &mut root, 3, &[b"task", b"list", b"--project", b"1", b"--json"]);
    assert_eq!(after.status, 0, "{after:?}");
    assert_eq!(after.stdout, before.stdout, "refusal makes no task");
}
