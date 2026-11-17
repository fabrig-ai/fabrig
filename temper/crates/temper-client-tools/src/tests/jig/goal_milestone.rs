//! A goal joins its milestone and carries waits to jig (client/ctl/milestones.md, 3).
use super::goal_template::goal_me;
use crate::tests::fixtures as f;
use jig_client::{contract as c, shapes};
use skein_lib::{Duration, List, Wall};

const START: &[&str] = &[
    "goal",
    "start",
    "--project",
    "1",
    "--charter",
    "5",
    "--budget",
    "50",
    "--no-deadline",
    "S1 Draft the layout",
    "--key",
    "layout",
];
const MILESTONE: u64 = 812;

fn me() -> c::Me {
    let mut me = goal_me();
    let project = me.projects.get_mut(0).unwrap();
    project.templates.get_mut(0).unwrap().shallower = 1;
    let delegation = &mut project.creating.as_mut().unwrap().within.authority.authority.delegation;
    delegation.depth = 8;
    delegation.tasks = 20;
    me
}

fn milestone() -> c::Task {
    let mut task = f::goal(MILESTONE, c::WaitsOn::Nothing);
    task.chip.executor = c::ChipExecutor::Procedure { connector: shapes::RECURRING, kind: shapes::GATHERING };
    task.chip.details.details.executor =
        c::Executor::Procedure { connector: shapes::RECURRING, code: shapes::GATHERING };
    task.concluded = None;
    task.authority.authority.delegation.depth = 2;
    task.authority.authority.delegation.tasks = 30;
    task.tasks_left = 3;
    task.budget.left = 70;
    task.budget.deadline = None;
    task
}
fn beneath() -> List<&'static str> {
    f::extended(START, &["--milestone", "812"])
}
fn submit(root: &mut f::Harness, argv: &[&str], task: c::Task) -> u32 {
    let call = root.submit(argv);
    assert!(root.sends.is_empty(), "the milestone is read before any act");
    assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Task { task: MILESTONE });
    root.serve(call, f::task_result(task));
    call
}
fn delegated(root: &f::Harness, call: u32) -> &c::NewTask {
    let c::Ask::Delegate { project, task, tasks } = f::act_ask(root, call) else { panic!("delegate") };
    assert_eq!((*project, *task, tasks.len()), (1, MILESTONE, 1));
    tasks.get(0).unwrap()
}
fn usage(task: c::Task, expected: &[u8]) {
    let mut root = f::choice_root(me());
    let call = submit(&mut root, beneath().as_slice(), task);
    assert!(root.sends.is_empty());
    let answer = root.answer(call).unwrap();
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, expected));
}

#[test]
fn goal_start_delegates_a_narrowed_template_to_a_milestone() {
    let mut baseline = f::choice_root(me());
    let call = baseline.submit(START);
    let mut expected = f::chat_task(&baseline, call).clone();
    expected.authority.delegation.depth = expected.authority.delegation.depth.min(1);
    expected.authority.delegation.tasks = expected.authority.delegation.tasks.min(2);
    for json in [false, true] {
        let mut root = f::choice_root(me());
        let mut argv = beneath();
        if json {
            argv.push("--json").unwrap();
        }
        let call = submit(&mut root, argv.as_slice(), milestone());
        assert_eq!(delegated(&root, call), &expected);
        assert_eq!(root.sent_name(call), Some(f::named("layout")));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 90, last: 90, given: None }));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        if json {
            let mut ordinary = f::choice_root(me());
            let plain = ordinary.submit(f::extended(START, &["--json"]).as_slice());
            ordinary
                .done(plain, c::Outcome::Document(c::DocumentOutcome::Created { first: 90, last: 90, given: None }));
            assert_eq!(answer.stdout, ordinary.answer(plain).unwrap().stdout);
            assert_eq!(crate::jig::rows::GOAL_START.result, b"goal_create");
        } else {
            assert_eq!(answer.stdout.as_ref(), b"done: task 90, goal\n");
        }
    }
}

#[test]
fn goal_start_beneath_a_milestone_warns_on_long_title_lines() {
    let mut root = f::choice_root(me());
    let mut argv = beneath();
    let text = "é".repeat(61);
    *argv.get_mut(9).unwrap() = &text;
    let call = submit(&mut root, argv.as_slice(), milestone());
    assert_eq!(delegated(&root, call).spec.words.bytes(), text.as_bytes());
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 41, last: 41, given: None }));
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 0);
    assert_eq!(answer.stdout.as_ref(), b"done: task 41, goal\n");
    assert_eq!(answer.stderr.as_ref(), super::goal_template::TITLE_WARNING);
}

#[test]
fn milestone_tasks_left_reserves_the_goal_and_vanishes_at_zero_depth() {
    for (depth, tasks_left, expected_depth, expected_tasks) in
        [(2, 5, 1, 4), (2, 1, 1, 0), (2, 0, 1, 0), (1, 5, 0, 0), (100, 100, 8, 20)]
    {
        let mut root = f::choice_root(me());
        let mut parent = milestone();
        parent.authority.authority.delegation.depth = depth;
        parent.tasks_left = tasks_left;
        let call = submit(&mut root, beneath().as_slice(), parent);
        let allowance = &delegated(&root, call).authority.delegation;
        assert_eq!((allowance.depth, allowance.tasks), (expected_depth, expected_tasks));
    }
}

#[test]
fn milestone_start_requires_a_gathering_of_the_project() {
    for executor in [
        c::Executor::Agent { charter: 5 },
        c::Executor::Procedure { connector: 1, code: shapes::GATHERING },
        c::Executor::Procedure { connector: shapes::RECURRING, code: 1 },
    ] {
        let mut task = milestone();
        task.chip.details.details.executor = executor;
        usage(task, b"task 812 is not a milestone of project 1\n");
    }
    let mut task = milestone();
    task.project = 2;
    usage(task, b"task 812 is not a milestone of project 1\n");
}

#[test]
fn whole_milestone_is_usage_with_its_time() {
    let mut task = milestone();
    task.concluded = Some(c::Concluded { by: 1, at: Wall::from_nanos(123) });
    usage(task, b"milestone 812 is whole since 123\n");
}

#[test]
fn after_keeps_order_once_and_leaves_dependency_judgement_to_jig() {
    for in_milestone in [false, true] {
        let mut root = f::choice_root(me());
        let mut argv = f::extended(START, &["--after", "40", "--after", "41", "--after", "40"]);
        let call = if in_milestone {
            argv.push("--milestone").unwrap();
            argv.push("812").unwrap();
            submit(&mut root, argv.as_slice(), milestone())
        } else {
            root.submit(argv.as_slice())
        };
        let task = if in_milestone { delegated(&root, call) } else { f::chat_task(&root, call) };
        assert_eq!(task.dependencies.as_slice(), &[c::TaskRef::Task(40), c::TaskRef::Task(41)]);
        assert!(root.reads.is_empty(), "the tool never reads dependencies");
        root.answered(
            call,
            c::Answer::Refused {
                refusal: c::Refusal::Dependency { dependency: 40, how: c::TaskStatus::Failed },
                at: None,
            },
        );
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2);
        assert!(skein_lib::bytes::find(&answer.stderr, b"The added dependency has ended").is_some(), "{answer:?}");
    }
}

#[test]
fn no_deadline_meets_given_controls_and_replaces_defaults() {
    for deadline in
        [c::TemplateDeadline::Given, c::TemplateDeadline::Span(Duration::from_secs(5)), c::TemplateDeadline::None]
    {
        let mut me = me();
        me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().deadline = deadline;
        let mut root = f::choice_root(me);
        let call = root.submit(START);
        let task = f::chat_task(&root, call);
        assert_eq!((task.deadline, task.authority.deadline), (c::NewDeadline::Named, None));
    }
    let mut root = f::choice_root(me());
    let call = root.submit(&["goal", "start", "--project", "1", "--no-deadline", "work", "--key", "missing"]);
    assert!(root.sends.is_empty());
    assert_eq!(
        root.answer(call).unwrap().stderr.as_ref(),
        b"template 2 needs: --charter CHARTER; --budget AMOUNT or --as-allowed; help goal start\n"
    );
    f::parse(f::extended(START, &["--deadline", "1970-01-01T00:01Z"]).as_slice()).expect_err("deadline alternatives");
}

#[test]
fn no_deadline_cannot_escape_creating_or_milestone_deadlines() {
    for in_milestone in [false, true] {
        let mut me = me();
        me.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within.maximum_deadline = Some(Wall::from_nanos(60));
        let mut root = f::choice_root(me);
        let call = if in_milestone { submit(&mut root, beneath().as_slice(), milestone()) } else { root.submit(START) };
        assert!(root.sends.is_empty());
        let answer = root.answer(call).unwrap();
        assert_eq!(
            (answer.status, answer.stderr.as_ref()),
            (
                1,
                b"goal start: --no-deadline: the project's role ends every task by its ceiling; give --deadline\n"
                    .as_slice()
            )
        );
    }
    let mut task = milestone();
    task.budget.deadline = Some(Wall::from_nanos(60));
    usage(task, b"goal start: --no-deadline: milestone 812 ends by 60; give --deadline\n");
}

#[test]
fn milestone_deadlines_narrow_walls_and_spans_without_widening_authority() {
    for (template_deadline, flag, expected) in [
        (c::TemplateDeadline::Given, Some("1970-01-01T00:02Z"), 60_000_000_000),
        (c::TemplateDeadline::Given, Some("1970-01-01T00:00Z"), 0),
        (c::TemplateDeadline::None, None, 60_000_000_000),
        (c::TemplateDeadline::Span(Duration::from_secs(120)), None, 60_000_000_000),
    ] {
        let mut me = me();
        me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().deadline = template_deadline;
        let mut root = f::choice_root(me);
        let mut argv = List::with_capacity(32);
        for arg in START {
            if *arg != "--no-deadline" {
                argv.push(*arg).unwrap();
            }
        }
        argv.push("--milestone").unwrap();
        argv.push("812").unwrap();
        if let Some(deadline) = flag {
            argv.push("--deadline").unwrap();
            argv.push(deadline).unwrap();
        }
        let mut parent = milestone();
        parent.budget.deadline = Some(Wall::from_nanos(60_000_000_000));
        parent.authority.authority.delegation.depth = 0;
        parent.authority.authority.delegation.tasks = u32::MAX;
        let call = submit(&mut root, argv.as_slice(), parent);
        let task = delegated(&root, call);
        assert_eq!((task.deadline, task.authority.deadline), (c::NewDeadline::Named, Some(Wall::from_nanos(expected))));
        assert_eq!(task.authority.delegation.depth, 0);
        assert_eq!(task.authority.delegation.tasks, 0);
    }
}

#[test]
fn milestone_delegate_keeps_funding_and_admission_refusals() {
    for as_allowed in [false, true] {
        let mut me = me();
        let creating = me.projects.get_mut(0).unwrap().creating.as_mut().unwrap();
        creating.within.budget = 0;
        creating.within.maximum_budget = 0;
        creating.within.pool_left = 0;
        creating.beyond = c::Beyond::Refused(c::Refusal::NoEscalationHolder);
        let mut root = f::choice_root(me);
        let mut argv = List::with_capacity(32);
        for arg in &beneath() {
            if !as_allowed || (*arg != "--budget" && *arg != "50") {
                argv.push(*arg).unwrap();
            }
        }
        if as_allowed {
            argv.push("--as-allowed").unwrap();
        }
        for arg in ["--priority", "7", "--model", "chosen-model"] {
            argv.push(arg).unwrap();
        }
        let call = submit(&mut root, argv.as_slice(), milestone());
        let task = delegated(&root, call);
        assert_eq!(task.tracked, Some(7));
        assert_eq!(task.llm.as_ref().unwrap().main.model.bytes(), b"chosen-model");
        if as_allowed {
            assert_eq!(task.budget, c::NewBudget::AsAllowed { cap: None });
            assert_eq!(task.authority.spend, 0, "jig resolves against the milestone, not the party pool");
        } else {
            assert_eq!((task.budget.clone(), task.authority.spend), (c::NewBudget::Named, 50));
        }
        let refusal = if as_allowed {
            c::Refusal::Authority { lacking: c::Lacking::Approver, proposable: None }
        } else {
            c::Refusal::Llm { model: f::act_text(b"chosen-model"), why: c::LlmWhy::Charter }
        };
        root.answered(call, c::Answer::Refused { refusal, at: None });
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2);
        if !as_allowed {
            assert_eq!(answer.stderr.as_ref(), b"goal start: --model chosen-model: not one of charter 5's models\n");
        }
    }
}

#[test]
fn milestone_budget_refusals_name_the_funder_and_keep_json_and_pool_words() {
    for (in_milestone, needed, left, expected) in [
        (true, 1, 0, b"goal start: refused: milestone 812 had 0 left, 1 needed; give each goal in it --budget AMOUNT\n".as_slice()),
        (true, 12, 9, b"goal start: refused: milestone 812 had 9 left, 12 needed; give each goal in it --budget AMOUNT\n".as_slice()),
        (false, 1, 0, concat!(
            "goal start: refused: your pool in project 1 had 0 left, 1 needed\n",
            "read it: project show 1\n",
            "raise it: project fund 1 AMOUNT --role 0, and first the project's if its period has no room: project fund 1 AMOUNT\n"
        ).as_bytes()),
    ] {
        for json in [false, true] {
            let mut root = f::choice_root(me());
            let mut argv = if in_milestone { beneath() } else { f::extended(START, &[]) };
            if json {
                argv.push("--json").unwrap();
            }
            let call = if in_milestone {
                submit(&mut root, argv.as_slice(), milestone())
            } else {
                root.submit(argv.as_slice())
            };
            root.answered(call, c::Answer::Refused {
                refusal: c::Refusal::Authority { lacking: c::Lacking::Budget { needed, left }, proposable: None },
                at: None,
            });
            let answer = root.answer(call).unwrap();
            assert_eq!(answer.status, 2);
            assert!(answer.stdout.is_empty());
            if json {
                let record: serde_json::Value = serde_json::from_slice(&answer.stderr).expect("refusal record");
                assert_eq!(record, serde_json::json!({
                    "operation": "goal",
                    "outcome": { "refused": {
                        "refusal": { "authority": { "lacking": { "budget": { "needed": needed, "left": left } }, "proposable": null } },
                        "at": null
                    } },
                    "task": null,
                    "to": null
                }));
            } else {
                assert_eq!(answer.stderr.as_ref(), expected);
            }
        }
    }
}

#[test]
fn milestone_goal_replays_the_same_keyed_delegate() {
    let mut root = f::choice_root(me());
    let call = submit(&mut root, beneath().as_slice(), milestone());
    let ask = f::act_ask(&root, call).clone();
    let name = root.sent_name(call);
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 90, last: 90, given: None }));
    let expected = root.answer(call).unwrap();
    let replay = root.submit(beneath().as_slice());
    if root.reads.contains_key(&replay) {
        root.serve(replay, f::task_result(milestone()));
    }
    assert_eq!(f::act_ask(&root, replay), &ask);
    assert_eq!(root.sent_name(replay), name);
    root.done(replay, c::Outcome::Document(c::DocumentOutcome::Created { first: 90, last: 90, given: None }));
    let answer = root.answer(replay).unwrap();
    assert_eq!((answer.status, answer.stdout), (expected.status, expected.stdout));
}
