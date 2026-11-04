//! Goal creation from offered templates (temper/docs/design/client/tools.md, 5.5).
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;

const WORDS: &str = "Implement `temper/docs/design/draft/archive/playground-reset.md`.";
/// Named-template creation shared with the keyed replay checklist.
pub(super) const START: &[&str] = &[
    "goal",
    "start",
    "--project",
    "shop",
    "--template",
    "4",
    "--budget",
    "80",
    "--deadline",
    "1970-01-01T00:01Z",
    WORDS,
    "--key",
    "draft-goal",
];

/// Offered goal templates and their creating bounds for goal and replay tests.
pub(super) fn goal_me() -> c::Me {
    let mut me = f::choice_me(f::CHAT_BUDGET);
    let project = me.projects.get_mut(0).expect("project");
    let mut goal = project.templates.get(0).expect("chat").clone();
    goal.number = 2;
    goal.executor = c::TemplateExecutor::Chosen;
    goal.budget = c::TemplateBudget::Given;
    goal.deadline = c::TemplateDeadline::Given;
    goal.tracked = Some(0);
    goal.label = 2;
    goal.flags.conversation = false;
    goal.flags.listed = true;
    let mut draft = goal.clone();
    draft.number = 4;
    draft.executor = c::TemplateExecutor::Charter(5);
    let mut subagents = List::with_capacity(1);
    subagents.push(f::model_effort(b"gpt-6.1-sol", Some(b"medium"))).unwrap();
    draft.llm = Some(c::LlmChoice { account: None, main: f::model_effort(b"gpt-6.1-sol", Some(b"high")), subagents });
    draft.resources = List::with_capacity(1);
    draft.resources.push(c::Symbol { connector: 1, symbol: 1 }).unwrap();
    let mut templates = List::with_capacity(2);
    templates.push(goal).unwrap();
    templates.push(draft).unwrap();
    project.templates = templates;
    let mut charters = List::with_capacity(2);
    charters.push(1).unwrap();
    charters.push(5).unwrap();
    project.creating.as_mut().unwrap().within.charters = charters;
    me
}

#[test]
fn goal_start_fills_the_named_template() {
    let mut root = f::choice_root(goal_me());
    let call = root.submit(START);
    let c::Ask::Create { project, tasks } = f::act_ask(&root, call) else { panic!("create") };
    assert_eq!(*project, f::PROJECT);
    assert_eq!(tasks.len(), 1);
    let task = tasks.get(0).unwrap();
    assert_eq!(task.executor, c::Executor::Agent { charter: 5 });
    assert_eq!(task.spec.words.bytes(), WORDS.as_bytes());
    assert!(!task.spec.asks);
    assert_eq!(task.contract, c::NewContract::Report { words: crate::limits::TEXT });
    assert_eq!(task.tracked, Some(0));
    assert_eq!(task.label, 2);
    assert_eq!(
        task.flags,
        c::TaskFlags { ended_by: c::EndedBy::Result, conversation: false, listed: true, retention: c::Retention::Kept }
    );
    assert_eq!(task.budget, c::NewBudget::Named);
    assert_eq!(task.authority.spend, 80);
    assert_eq!(task.authority.deadline, Some(skein_lib::Wall::from_nanos(60_000_000_000)));
    assert_eq!(task.deadline, c::NewDeadline::Named);
    assert_eq!(task.symbols.as_slice(), &[c::Symbol { connector: 1, symbol: 1 }]);
    let llm = task.llm.as_ref().unwrap();
    assert_eq!(llm.account, None);
    assert_eq!(llm.main, f::model_effort(b"gpt-6.1-sol", Some(b"high")));
    assert_eq!(llm.subagents.as_slice(), &[f::model_effort(b"gpt-6.1-sol", Some(b"medium"))]);
}

const STOCK: &[&str] = &[
    "goal",
    "start",
    "--project",
    "shop",
    "--charter",
    "1",
    "--budget",
    "80",
    "--deadline",
    "1970-01-01T00:01Z",
    "work",
    "--key",
    "stock-goal",
];

#[test]
fn stock_goal_requires_and_fills_the_creators_charter() {
    let mut root = f::choice_root(goal_me());
    let call = root.submit(STOCK);
    let task = f::chat_task(&root, call);
    assert_eq!(task.executor, c::Executor::Agent { charter: 1 });
    assert_eq!(task.label, 2);
    assert_eq!(task.tracked, Some(0));
    assert!(task.symbols.is_empty());
}

fn refused(me: c::Me, argv: &[&str], expected: &[u8]) {
    let mut root = f::choice_root(me);
    let call = root.submit(argv);
    assert!(root.sends.is_empty());
    let answer = root.answer(call).expect("usage refusal");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, expected));
}

#[test]
fn chosen_executor_needs_a_charter() {
    refused(
        goal_me(),
        &["goal", "start", "--project", "shop", "work", "--key", "missing"],
        b"template 2 lets its creator choose the executor; name it: --charter CHARTER\n",
    );
}

#[test]
fn fixed_executor_refuses_a_charter_override() {
    let argv = f::extended(START, &["--charter", "5"]);
    refused(goal_me(), argv.as_slice(), b"template 4's executor is charter 5; leave out --charter\n");
}

#[test]
fn a_charter_outside_creating_bounds_sends_nothing() {
    let mut me = goal_me();
    let mut charters = List::with_capacity(3);
    for number in [1, 2, 4] {
        charters.push(number).unwrap();
    }
    me.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within.charters = charters;
    refused(me, START, b"template 4 requires charter 5, which is not offered; offered charters: 1 2 4\n");
}

#[test]
fn absent_creating_offer_names_the_receiving_bound() {
    let mut me = goal_me();
    me.projects.get_mut(0).unwrap().creating = None;
    refused(me, START, b"project has no creating offer; check role and receiving bound documents.lines for charter and procedure lists\n");
}

#[test]
fn a_goal_past_the_ceiling_names_budget_or_deadline() {
    let mut me = goal_me();
    me.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within.maximum_budget = 79;
    refused(me, START, b"goal budget or deadline exceeds the project's offered ceiling\n");
}

#[test]
fn a_goal_without_a_proposal_recipient_keeps_the_refusal() {
    let mut me = goal_me();
    let creating = me.projects.get_mut(0).unwrap().creating.as_mut().unwrap();
    creating.within.budget = 79;
    creating.beyond = c::Beyond::Refused(c::Refusal::NoEscalationHolder);
    refused(me, START, b"No one to escalate to\n");
}

#[test]
fn given_values_are_required_before_sending() {
    for (extra, expected) in [
        (&[][..], b"this template needs --budget AMOUNT or --as-allowed\n".as_slice()),
        (&["--budget", "80"][..], b"this template needs --deadline TIME\n".as_slice()),
    ] {
        let argv =
            f::extended(&["goal", "start", "--project", "shop", "--template", "4", "work", "--key", "given"], extra);
        refused(goal_me(), argv.as_slice(), expected);
    }
}

#[test]
fn absent_controls_keep_template_defaults() {
    let mut me = goal_me();
    let template = me.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap();
    template.budget = c::TemplateBudget::Amount(71);
    template.deadline = c::TemplateDeadline::Span(f::CHAT_SPAN);
    let mut root = f::choice_root(me);
    let call = root.submit(&["goal", "start", "--project", "shop", "--template", "4", "work", "--key", "defaults"]);
    let task = f::chat_task(&root, call);
    assert_eq!(task.authority.spend, 71);
    assert_eq!(task.deadline, c::NewDeadline::Span(f::CHAT_SPAN));
    assert_eq!(task.tracked, Some(0));
}

#[test]
fn model_and_priority_overrides_keep_subagents() {
    let argv = f::extended(START, &["--model", "chosen-model", "--effort", "high", "--priority", "7"]);
    let mut root = f::choice_root(goal_me());
    let call = root.submit(argv.as_slice());
    let task = f::chat_task(&root, call);
    assert_eq!(task.tracked, Some(7));
    let llm = task.llm.as_ref().unwrap();
    assert_eq!(llm.main, f::model_effort(b"chosen-model", Some(b"high")));
    assert_eq!(llm.subagents.as_slice(), &[f::model_effort(b"gpt-6.1-sol", Some(b"medium"))]);
}

#[test]
fn explicit_subagents_replace_template_choices() {
    let argv = f::extended(START, &["--model", "chosen-model", "--effort", "high", "--sub-agent", "plain-model"]);
    let mut root = f::choice_root(goal_me());
    let call = root.submit(argv.as_slice());
    assert_eq!(
        f::chat_task(&root, call).llm.as_ref().unwrap().subagents.as_slice(),
        &[f::model_effort(b"plain-model", None)]
    );
}

#[test]
fn goals_use_the_keyed_create_outcome() {
    let mut root = f::choice_root(goal_me());
    let call = root.submit(START);
    assert_eq!(root.sent_name(call), Some(f::named("draft-goal")));
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 41, last: 41, given: None }));
    let answer = root.answer(call).unwrap();
    assert_eq!((answer.status, answer.stdout.as_ref()), (0, b"done: task 41, goal\n".as_slice()));
    let replay = root.submit(START);
    assert_eq!(f::act_ask(&root, replay), f::act_ask(&root, call));
    assert_eq!(root.sent_name(replay), Some(f::named("draft-goal")));
}

#[test]
fn goal_bounds_keep_proposal_and_refusal_judgements() {
    for (budget, expected) in [(100, b"create".as_slice()), (101, b"propose"), (500, b"propose"), (501, b"refuse")] {
        let mut me = goal_me();
        me.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap().budget = c::TemplateBudget::Amount(budget);
        let mut root = f::choice_root(me);
        let call = root.submit(&[
            "goal",
            "start",
            "--project",
            "shop",
            "--template",
            "4",
            "--deadline",
            "1970-01-01T00:01Z",
            "work",
            "--key",
            "bounds",
        ]);
        let seen = match root.sends.get(&call) {
            Some((_, _, jig_client::Ask::Jig(ask))) => {
                if let c::Ask::Create { .. } = ask.as_ref() {
                    b"create".as_slice()
                } else {
                    let c::Ask::Propose { .. } = ask.as_ref() else { panic!("creation") };
                    b"propose"
                }
            }
            None => {
                assert_eq!(root.answer(call).unwrap().status, 1);
                b"refuse"
            }
            _ => panic!("jig request"),
        };
        assert_eq!(seen, expected);
    }
}

#[test]
fn as_allowed_selects_engine_funding() {
    let mut root = f::choice_root(goal_me());
    let call = root.submit(&[
        "goal",
        "start",
        "--project",
        "shop",
        "--charter",
        "1",
        "--as-allowed",
        "--deadline",
        "1970-01-01T00:01Z",
        "work",
        "--key",
        "funded",
    ]);
    assert_eq!(f::chat_task(&root, call).budget, c::NewBudget::AsAllowed { cap: None });
}

#[test]
fn invalid_controls_are_usage_errors() {
    for extra in [&["--as-allowed"][..], &["--effort", "high"][..], &["--priority", "4294967296"][..]] {
        let argv = f::extended(START, extra);
        f::parse(argv.as_slice()).expect_err("invalid controls");
    }
    let mut argv = f::extended(STOCK, &[]);
    *argv.get_mut(9).unwrap() = "2026-02-30T00:00Z";
    f::parse(argv.as_slice()).expect_err("invalid date");
}
