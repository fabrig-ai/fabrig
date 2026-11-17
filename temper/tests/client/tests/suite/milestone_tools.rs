//! A person's complete milestone script through ctl and the joined engine (client/ctl/milestones.md, 5).
use jig_domain_tasks as tasks;
use temper_client_world::{
    app::{Event, Root},
    fixtures,
    front::{Joint, account_acts as accounts},
};

fn call(front: &mut Joint, root: &mut Root, number: u32, words: &[&[u8]]) -> temper_client_tools::jig::Printed {
    let Event::Call { call, json, .. } = fixtures::call(words) else { unreachable!() };
    accounts::event(front, root, Event::Call { number, call, json, value: None });
    let printed = root.agent().printed(number).expect("ctl call answered");
    assert_eq!(printed.status, 0, "{printed:?}");
    temper_client_tools::jig::Printed {
        file: printed.file,
        status: printed.status,
        stdout: printed.stdout.clone(),
        stderr: printed.stderr.clone(),
    }
}
#[test]
fn a_second_as_allowed_goal_names_its_milestones_empty_budget() {
    let (mut front, mut root) = accounts::goal_choice_fixture(fixtures::SEED);
    let made = call(
        &mut front,
        &mut root,
        1,
        &[b"milestone", b"new", b"--project", b"1", b"--as-allowed", b"Store shape", b"--key", b"store-shape"],
    );
    let milestone = fixtures::done_task(&made);
    let milestone_arg = milestone.to_string();
    let first = call(
        &mut front,
        &mut root,
        2,
        &[
            b"goal",
            b"start",
            b"--project",
            b"1",
            b"--milestone",
            milestone_arg.as_bytes(),
            b"--as-allowed",
            "T1 · first\n\nThe first.".as_bytes(),
            b"--key",
            b"first",
        ],
    );
    let goal = fixtures::done_task(&first);
    assert_eq!(first.stdout.as_ref(), format!("done: task {goal}, goal\n").as_bytes());
    let Event::Call { call, json, .. } = fixtures::call(&[
        b"goal",
        b"start",
        b"--project",
        b"1",
        b"--milestone",
        milestone_arg.as_bytes(),
        b"--as-allowed",
        "T2 · second\n\nThe second, no wait.".as_bytes(),
        b"--key",
        b"second",
    ]) else {
        unreachable!()
    };
    accounts::event(&mut front, &mut root, Event::Call { number: 3, call, json, value: None });
    let refused = root.agent().printed(3).expect("second goal answered");
    assert_eq!(refused.status, 2, "{refused:?}");
    assert!(refused.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        format!(
            "goal start: refused: milestone {milestone} had 0 left, 1 needed; give each goal in it --budget AMOUNT\n"
        )
    );
}

#[test]
fn a_person_fills_and_concludes_a_milestone_with_a_waiting_goal() {
    let (mut front, mut root) = accounts::goal_choice_fixture(fixtures::SEED);
    let made = call(
        &mut front,
        &mut root,
        1,
        &[b"milestone", b"new", b"--project", b"1", b"--budget", b"400", b"Store shape", b"--key", b"store-shape"],
    );
    let milestone = fixtures::done_task(&made);
    let milestone_arg = milestone.to_string();
    let first = call(
        &mut front,
        &mut root,
        2,
        &[
            b"goal",
            b"start",
            b"--project",
            b"1",
            b"--budget",
            b"10",
            b"--no-deadline",
            b"--milestone",
            milestone_arg.as_bytes(),
            b"S1 Draft the layout",
            b"--key",
            b"draft-layout",
        ],
    );
    let first = fixtures::done_task(&first);
    let first_arg = first.to_string();
    let second = call(
        &mut front,
        &mut root,
        3,
        &[
            b"goal",
            b"start",
            b"--project",
            b"1",
            b"--budget",
            b"10",
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
    let second = fixtures::done_task(&second);
    let whole = call(&mut front, &mut root, 4, &[b"milestone", b"whole", milestone_arg.as_bytes(), b"--key", b"whole"]);
    assert_eq!(whole.stdout.as_ref(), format!("done: task {milestone}, whole\n").as_bytes());
    let concluded = front.engine.task(milestone).expect("milestone live").concluded;
    let replay =
        call(&mut front, &mut root, 5, &[b"milestone", b"whole", milestone_arg.as_bytes(), b"--key", b"whole"]);
    assert_eq!(replay.stdout, whole.stdout);
    assert_eq!(
        front.engine.task(milestone).expect("milestone live").concluded,
        concluded,
        "the key keeps the first conclusion"
    );
    let shown = call(&mut front, &mut root, 6, &[b"milestone", b"show", milestone_arg.as_bytes()]);
    let shown = String::from_utf8(shown.stdout.into_vec()).expect("ctl UTF-8");
    assert!(shown.starts_with(&format!("milestone {milestone} · Store shape · whole since ")), "{shown}");
    assert!(shown.contains("goals: 0 / 2 done · 0 failed · 0 closed · 2 live\n"), "{shown}");
    let expected_goals =
        format!("goals:\n  {first}  S1 Draft the layout  live\n  {second}  S2 Check the layout  live\n");
    assert!(shown.contains(&expected_goals), "{shown}");
    for goal in [first, second] {
        let task = front.engine.task(goal).expect("goal live");
        assert_eq!((task.requester, task.funder), (tasks::Party::Task(milestone), tasks::Funder::Task(milestone)));
    }
    let waiting = front.engine.task(second).expect("waiting goal");
    assert!(matches!(waiting.phase, tasks::Phase::Waiting));
    assert_eq!(waiting.dependencies.as_ref(), &[first]);
}
