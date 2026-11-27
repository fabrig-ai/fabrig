//! Goal creation from offered templates (temper/docs/design/client/ctl/areas.md).
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
    "0.80",
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
    draft.name = Some(b"draft-goal".as_slice().into());
    draft.executor = c::TemplateExecutor::Charter(5);
    let mut subagents = List::with_capacity(1);
    subagents.push(f::model_effort(b"gpt-6.1-sol", Some(b"medium"))).unwrap();
    draft.llm = Some(c::LlmChoice {
        later: Some(c::LlmLater { main: f::model_effort(b"later-model", Some(b"high")), subagents: subagents.clone() }),
        account: None,
        main: f::model_effort(b"gpt-6.1-sol", Some(b"high")),
        subagents,
    });
    draft.resources = List::with_capacity(1);
    draft.resources.push(c::Symbol { connector: 1, symbol: 1 }).unwrap();
    let mut templates = List::with_capacity(2);
    templates.push(goal).unwrap();
    templates.push(draft).unwrap();
    project.templates = templates;
    project.creating.as_mut().unwrap().within.charters = List::from_boxed(Box::new([1, 2, 5]));
    me.charters = List::from_boxed(Box::new([c::MeCharter { number: 2, name: b"implementer".as_slice().into() }]));
    me
}

#[test]
fn goal_start_fills_the_named_template() {
    for template in ["4", "draft-goal"] {
        let mut argv = f::extended(START, &[]);
        *argv.get_mut(5).expect("template selector") = template;
        let mut root = f::choice_root(goal_me());
        let call = root.submit(argv.as_slice());
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
            c::TaskFlags {
                ended_by: c::EndedBy::Result,
                conversation: false,
                listed: true,
                retention: c::Retention::Kept
            }
        );
        assert_eq!(task.budget, c::NewBudget::Named);
        assert_eq!(task.authority.spend, 80);
        assert_eq!(task.authority.deadline, Some(skein_lib::Wall::from_nanos(60_000_000_000)));
        assert_eq!(task.deadline, c::NewDeadline::Named);
        assert_eq!(task.symbols.as_slice(), &[c::Symbol { connector: 1, symbol: 1 }]);
        let llm = task.llm.as_ref().unwrap();
        assert_eq!(llm, goal_me().projects.get(0).unwrap().templates.get(1).unwrap().llm.as_ref().unwrap());
        assert_eq!(llm.account, None);
        assert_eq!(llm.main, f::model_effort(b"gpt-6.1-sol", Some(b"high")));
        assert_eq!(llm.subagents.as_slice(), &[f::model_effort(b"gpt-6.1-sol", Some(b"medium"))]);
    }
}

const STOCK: &[&str] = &[
    "goal",
    "start",
    "--project",
    "shop",
    "--charter",
    "1",
    "--budget",
    "0.80",
    "--deadline",
    "1970-01-01T00:01Z",
    "work",
    "--key",
    "stock-goal",
];

#[test]
fn stock_goal_requires_and_fills_the_creators_charter() {
    for (charter, number) in [("1", 1), ("2", 2), ("implementer", 2)] {
        let mut argv = f::extended(STOCK, &[]);
        *argv.get_mut(5).expect("charter selector") = charter;
        let mut root = f::choice_root(goal_me());
        let call = root.submit(argv.as_slice());
        let task = f::chat_task(&root, call);
        assert_eq!(task.executor, c::Executor::Agent { charter: number });
        assert_eq!(task.label, 2);
        assert_eq!(task.tracked, Some(0));
        assert!(task.symbols.is_empty());
    }
}

fn refused(me: c::Me, argv: &[&str], expected: &[u8]) {
    let mut root = f::choice_root(me);
    let call = root.submit(argv);
    assert!(root.sends.is_empty());
    let answer = root.answer(call).expect("usage refusal");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, expected));
}

#[test]
fn unknown_goal_template_names_send_nothing() {
    for extra in [&[][..], &["--milestone", "41"][..]] {
        let mut argv = f::extended(START, extra);
        *argv.get_mut(5).expect("template selector") = "lean-goal";
        let mut root = f::choice_root(goal_me());
        let call = root.submit(argv.as_slice());
        assert!(root.sends.is_empty() && root.reads.is_empty());
        let answer = root.answer(call).expect("unknown name is usage");
        assert_eq!(
            (answer.status, answer.stderr.as_ref()),
            (1, b"temper ctl: no template is named lean-goal; temper ctl settings show lists them\n".as_slice())
        );
    }
}

#[test]
fn chosen_executor_needs_a_charter() {
    refused(
        goal_me(),
        &["goal", "start", "--project", "shop", "work", "--key", "missing"],
        b"template 2 needs: --charter CHARTER; --budget AMOUNT or --as-allowed; --deadline TIME; help goal start\n",
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
        (&[][..], b"template 4 needs: --budget AMOUNT or --as-allowed; --deadline TIME; help goal start\n".as_slice()),
        (&["--budget", "0.80"][..], b"template 4 needs: --deadline TIME; help goal start\n".as_slice()),
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

pub(super) const TITLE_WARNING: &[u8] =
    b"warning: TEXT's first line, the title the pages show, is 61 characters, past 60; temper ctl help goal start\n";

const TITLE_WARNING_JSON: &[u8] = b"{\"warning\":\"TEXT's first line, the title the pages show, is 61 characters, past 60; temper ctl help goal start\"}\n";
const CREATED_JSON: &[u8] = b"{\"operation\":\"goal\",\"outcome\":{\"done\":{\"document\":{\"created\":{\"first\":41,\"last\":41,\"given\":null}}}},\"task\":null,\"to\":null}";

fn title_call(text: &str) -> crate::jig::call::Call {
    let mut argv = f::extended(START, &[]);
    *argv.get_mut(10).unwrap() = text;
    f::call(argv.as_slice())
}

fn created(root: &mut f::Harness, call: u32) {
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 41, last: 41, given: None }));
}

#[test]
fn goal_start_warns_on_long_untrimmed_unicode_title_lines() {
    for (text, warning) in [
        ("a".repeat(61), TITLE_WARNING),
        ("é".repeat(61), TITLE_WARNING),
        ("a".repeat(60) + " \nrest", TITLE_WARNING),
        ("a".repeat(60) + "\r\nrest", TITLE_WARNING),
        ("a".repeat(75), b"warning: TEXT's first line, the title the pages show, is 75 characters, past 60; temper ctl help goal start\n".as_slice()),
    ] {
        let mut root = f::choice_root(goal_me());
        let call = root.submit_prepared(title_call(&text), false, None);
        assert_eq!(f::chat_task(&root, call).spec.words.bytes(), text.as_bytes());
        assert!(root.answer(call).is_none(), "the warning waits for the answer");
        created(&mut root, call);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert_eq!(answer.stdout.as_ref(), b"done: task 41, goal\n");
        assert_eq!(answer.stderr.as_ref(), warning);
    }
}

#[test]
fn goal_start_keeps_short_title_lines_quiet() {
    for text in
        ["a".repeat(60), "é".repeat(60), "a".repeat(20) + "\n" + &"b".repeat(200), "\n".repeat(2) + &"b".repeat(200)]
    {
        let mut root = f::choice_root(goal_me());
        let call = root.submit_prepared(title_call(&text), false, None);
        created(&mut root, call);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert_eq!(answer.stdout.as_ref(), b"done: task 41, goal\n");
        assert_eq!(answer.stderr.as_ref(), b"");
    }
}

#[test]
fn goal_start_warns_on_proposed_answers() {
    let mut offered = goal_me();
    offered.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within.budget = 79;
    let mut root = f::choice_root(offered);
    let call = root.submit_prepared(title_call(&"a".repeat(61)), false, None);
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 17, to: 9 }));
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 0);
    assert_eq!(answer.stdout.as_ref(), b"proposed: p73/17, to role 9 in project 1\n");
    assert_eq!(answer.stderr.as_ref(), TITLE_WARNING);
}

#[test]
fn goal_start_refusals_do_not_warn() {
    let mut root = f::choice_root(goal_me());
    let call = root.submit_prepared(title_call(&"a".repeat(61)), false, None);
    root.answered(call, c::Answer::Refused { refusal: c::Refusal::KeyConflict, at: None });
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 2);
    assert_eq!(answer.stdout.as_ref(), b"");
    assert_eq!(answer.stderr.as_ref(), b"goal start: The request key names another act\n");
}

#[test]
fn goal_start_title_warning_is_json_and_replays_unchanged() {
    let mut root = f::choice_root(goal_me());
    let parsed = title_call(&"é".repeat(61));
    let mut first = None;
    for _ in 0_u32..2 {
        let call = root.submit_prepared(parsed.clone(), true, None);
        created(&mut root, call);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert_eq!(answer.stdout.as_ref(), CREATED_JSON);
        assert_eq!(answer.stderr.as_ref(), TITLE_WARNING_JSON);
        if let Some(first) = &first {
            assert_eq!(&(answer.status, answer.stdout, answer.stderr), first);
        } else {
            first = Some((answer.status, answer.stdout, answer.stderr));
        }
    }
}

#[test]
fn goal_start_checks_loaded_stdin_title_lines() {
    use crate::jig::call::{Value, Values};
    for json in [false, true] {
        let mut root = f::choice_root(goal_me());
        let mut parsed = title_call("-");
        assert_eq!(parsed.values.get(2), Some(&Some(Values::One(Value::Stdin))));
        let text = "é".repeat(61);
        *parsed.values.get_mut(2).unwrap() = Some(Values::One(Value::Text(Box::from(text.as_bytes()))));
        let call = root.submit_prepared(parsed, json, None);
        assert_eq!(f::chat_task(&root, call).spec.words.bytes(), text.as_bytes());
        created(&mut root, call);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        if json {
            assert_eq!(answer.stdout.as_ref(), CREATED_JSON);
            assert_eq!(answer.stderr.as_ref(), TITLE_WARNING_JSON);
        } else {
            assert_eq!(answer.stdout.as_ref(), b"done: task 41, goal\n");
            assert_eq!(answer.stderr.as_ref(), TITLE_WARNING);
        }
    }
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

#[test]
fn starts_model_flags_replace_the_whole_template_choice() {
    for (flags, main, children) in [
        (
            &["--model", "chosen-model"][..],
            f::model_effort(b"chosen-model", Some(b"high")),
            List::from_boxed(Box::new([f::model_effort(b"gpt-6.1-sol", Some(b"medium"))])),
        ),
        (
            &[
                "--model",
                "chosen-model",
                "--effort",
                "high",
                "--sub-agent",
                "plain-model:medium",
                "--sub-agent",
                "chosen-model:high",
            ][..],
            f::model_effort(b"chosen-model", Some(b"high")),
            List::from_boxed(Box::new([
                f::model_effort(b"plain-model", Some(b"medium")),
                f::model_effort(b"chosen-model", Some(b"high")),
            ])),
        ),
    ] {
        let mut me = goal_me();
        let child = me.projects.get_mut(0).unwrap().models.get_mut(1).unwrap();
        child.efforts = List::from_boxed(Box::new([f::act_text(b"low"), f::act_text(b"medium")]));
        let argv = f::extended(
            &[
                "goal",
                "start",
                "--project",
                "shop",
                "--template",
                "4",
                "--as-allowed",
                "--deadline",
                "1970-01-01T00:01Z",
                "work",
                "--key",
                "flags",
            ],
            flags,
        );
        let mut root = f::choice_root(me);
        let call = root.submit(argv.as_slice());
        let choice = f::chat_task(&root, call).llm.as_ref().unwrap();
        assert_eq!(choice, &c::LlmChoice { main, subagents: children, account: None, later: None });
    }
}

#[test]
fn goal_budget_argv_and_task_show_share_the_amount() {
    for (amount, spend) in [("130", 13000), ("1.25", 125), ("0.5", 50), ("0", 0), ("130.00", 13000)] {
        let mut me = goal_me();
        let bounds = &mut me.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within;
        bounds.budget = 13000;
        bounds.maximum_budget = 13000;
        bounds.pool_left = 13000;
        bounds.authority.authority.spend = 13000;
        let mut root = f::choice_root(me);
        let call = root.submit(&[
            "goal",
            "start",
            "--project",
            "1",
            "--charter",
            "1",
            "--budget",
            amount,
            "--no-deadline",
            "Work",
            "--key",
            "amount",
        ]);
        assert_eq!(f::chat_task(&root, call).authority.spend, spend, "{amount}");
        if spend == 13000 {
            let mut task = f::goal(41, c::WaitsOn::Nothing);
            task.budget.numbers.budget = spend;
            task.budget.numbers.spent = 0;
            let read = root.submit(&["task", "show", "41"]);
            root.serve(read, f::task_result(task));
            let answer = root.answer(read).unwrap();
            assert!(skein_lib::bytes::find(&answer.stdout, b"spent 0.00 of 130.00").is_some());
        }
    }
}

// temper/docs/design/testing/settings-tests.md, 7: P81's residual command must round-trip through ctl.
#[test]
#[ignore = "its own fix: the JSON answer's proposed command carries an amount in another unit (temper/docs/design/draft/failure-worlds.md, 7, P81 residual)"]
fn a_json_proposal_carries_hundredths() {
    let mut root = f::choice_root(goal_me());
    let number = root.submit(&[
        "goal",
        "start",
        "--project",
        "1",
        "--charter",
        "1",
        "--budget",
        "5.00",
        "--no-deadline",
        "Work",
        "--key",
        "amount",
        "--json",
    ]);
    // 500 is beyond this party's 100-unit budget, but within its 500-unit proposal ceiling.
    let c::Ask::Propose { action: c::ProposeAction::Batch { tasks }, .. } = f::act_ask(&root, number) else {
        panic!("the budget needs a proposal");
    };
    let spend = tasks.get(0).expect("one proposed goal").authority.spend;
    assert_eq!(spend, 500);
    root.answered(
        number,
        c::Answer::Refused {
            refusal: c::Refusal::Authority { lacking: c::Lacking::Approver, proposable: Some(true) },
            at: None,
        },
    );
    let answer = root.answer(number).expect("authority refusal answered");
    assert_eq!(answer.status, 2);
    let json: serde_json::Value = serde_json::from_slice(&answer.stderr).expect("JSON refusal");
    let command = json.get("propose_instead").expect("proposed command").as_str().expect("command string");
    // This fixture's arguments contain no quoting or whitespace within a word, and fit in 16 words.
    let mut words = List::with_capacity(16);
    for word in command.split_whitespace() {
        words.push(word).expect("fixture command fits");
    }
    let retried = f::call(words.as_slice());
    assert_eq!(
        crate::jig::call::argument(&crate::jig::rows::GOAL_START, &retried, b"budget"),
        Some(&crate::jig::call::Value::Number(spend)),
        "the proposed command's budget must parse to the requested spend units: {command}"
    );
    let mut budget = None;
    for pair in words.as_slice().windows(2) {
        if pair.first() == Some(&"--budget") {
            budget = pair.get(1);
        }
    }
    assert_eq!(budget, Some(&"5.00"));
}

#[test]
fn goal_call_budget_keeps_integer_spend_units() {
    let mut me = goal_me();
    let bounds = &mut me.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within;
    bounds.budget = 13000;
    bounds.maximum_budget = 13000;
    bounds.pool_left = 13000;
    bounds.authority.authority.spend = 13000;
    let call = f::from_json(br#"{"tool":"goal start","arguments":{"project":1,"charter":1,"budget":13000,"no_deadline":true,"text":"Work","key":"amount"}}"#);
    for json in [false, true] {
        let mut root = f::choice_root(me.clone());
        let number = root.submit_prepared(call.clone(), json, None);
        assert_eq!(f::chat_task(&root, number).authority.spend, 13000);
        root.answered(
            number,
            c::Answer::Refused {
                refusal: c::Refusal::Authority { lacking: c::Lacking::Approver, proposable: Some(true) },
                at: None,
            },
        );
        let answer = root.answer(number).unwrap();
        let budget = if json { b"--budget 13000".as_slice() } else { b"--budget 130.00" };
        assert!(skein_lib::bytes::find(&answer.stderr, budget).is_some());
    }
}
