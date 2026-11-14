//! A start's flags hold for every run (temper/docs/design/client/ctl/areas.md).
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
        file: printed.file,
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
