//! Start model refusals name the flag, choice part and bound (client/ctl/areas.md area 5).
#![expect(clippy::disallowed_macros, reason = "ordinary Rust expected refusal strings")]
use super::goal_template::{START, goal_me};
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;

const MODEL: &[&str] = &["--model", "chosen-model"];

fn refusal(model: &[u8], why: c::LlmWhy) -> c::Answer {
    c::Answer::Refused { refusal: c::Refusal::Llm { model: f::act_text(model), why }, at: None }
}
fn assert_text(mut root: f::Harness, call: u32, model: &[u8], why: c::LlmWhy, expected: &str) {
    root.answered(call, refusal(model, why));
    let answer = root.answer(call).expect("start refusal");
    assert_eq!((answer.status, answer.stderr.as_ref()), (2, expected.as_bytes()));
    assert!(answer.stdout.is_empty());
}

#[test]
fn goal_start_names_the_refused_flag_and_sent_charter() {
    for proposal in [false, true] {
        let mut me = goal_me();
        if proposal {
            let creating = me.projects.get_mut(0).unwrap().creating.as_mut().unwrap();
            creating.within.budget = 79;
            creating.beyond = c::Beyond::Proposed { to: c::Recipient::Party(f::PARTY) };
        }
        let mut root = f::choice_root(me);
        let argv = f::extended(START, MODEL);
        let call = root.submit(argv.as_slice());
        let ask = f::act_ask(&root, call);
        let tasks = if proposal {
            let c::Ask::Propose { action: c::ProposeAction::Batch { tasks }, .. } = ask else { panic!("proposal") };
            tasks
        } else {
            let c::Ask::Create { tasks, .. } = ask else { panic!("create") };
            tasks
        };
        let c::Executor::Agent { charter } = tasks.get(0).unwrap().executor else { panic!("agent") };
        assert_text(
            root,
            call,
            f::MODEL_NAME,
            c::LlmWhy::Charter,
            &format!("goal start: --model chosen-model: not one of charter {charter}'s models\n"),
        );
    }
}

fn subagent_me() -> c::Me {
    let mut me = goal_me();
    let plain = me.projects.get_mut(0).unwrap().models.get_mut(1).unwrap();
    plain.efforts = List::from_boxed(Box::new([f::act_text(b"low")]));
    plain.effort = Some(f::act_text(b"low"));
    me
}
#[test]
fn start_refusal_names_the_subagent_value_and_prefers_the_main_flag() {
    for (value, model, named) in [
        ("plain-model:low", b"plain-model".as_slice(), "--sub-agent plain-model:low"),
        ("chosen-model:high", b"chosen-model", "--model chosen-model"),
    ] {
        let mut root = f::choice_root(subagent_me());
        let argv = f::extended(START, &["--model", "chosen-model", "--sub-agent", value]);
        let call = root.submit(argv.as_slice());
        assert_text(
            root,
            call,
            model,
            c::LlmWhy::Authority,
            &format!("goal start: {named}: outside the authority the task is given\n"),
        );
    }
}

fn template_me() -> c::Me {
    let mut me = goal_me();
    let choice = me.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap().llm.as_mut().unwrap();
    choice.main = f::model_effort(b"template-main", None);
    choice.subagents = List::from_boxed(Box::new([f::model_effort(b"template-sub", None)]));
    choice.later = Some(c::LlmLater {
        main: f::model_effort(b"later-main", None),
        subagents: List::from_boxed(Box::new([f::model_effort(b"later-sub", None)])),
    });
    me
}
#[test]
fn start_refusal_names_unflagged_models_by_their_choice_parts() {
    for (extra, model, part) in [
        (&[][..], "template-main", "main"),
        (&[][..], "template-sub", "sub-agent"),
        (MODEL, "template-sub", "sub-agent"),
        (&[][..], "later-main", "main"),
        (&[][..], "later-sub", "sub-agent"),
    ] {
        let mut root = f::choice_root(template_me());
        let argv = f::extended(START, extra);
        let call = root.submit(argv.as_slice());
        assert_text(
            root,
            call,
            model.as_bytes(),
            c::LlmWhy::Authority,
            &format!("goal start: {part} model {model}: outside the authority the task is given\n"),
        );
    }
}

#[test]
fn start_refusal_explains_each_llm_bound() {
    for (why, expected) in [
        (c::LlmWhy::Charter, "not one of charter 5's models"),
        (c::LlmWhy::Authority, "outside the authority the task is given"),
        (c::LlmWhy::Effort, "its effort is not one the model declares"),
        (c::LlmWhy::Unconfigured, "not a configured model"),
        (c::LlmWhy::Procedure, "the task's executor runs no model"),
        (c::LlmWhy::Repeated, "named twice among the sub-agents"),
    ] {
        let mut root = f::choice_root(goal_me());
        let argv = f::extended(START, MODEL);
        let call = root.submit(argv.as_slice());
        assert_text(root, call, f::MODEL_NAME, why, &format!("goal start: --model chosen-model: {expected}\n"));
    }
}

#[test]
fn start_refusal_json_keeps_the_record_and_adds_only_the_named_flag() {
    for (extra, model, flag) in [
        (MODEL, "chosen-model", Some(("--model", "chosen-model"))),
        (
            &["--model", "chosen-model", "--sub-agent", "plain-model:low"][..],
            "plain-model",
            Some(("--sub-agent", "plain-model:low")),
        ),
        (&[][..], "gpt-6.1-sol", None),
    ] {
        let argv = f::extended(START, extra);
        let argv = f::extended(argv.as_slice(), f::JSON_FLAG);
        let mut root = f::choice_root(subagent_me());
        let call = root.submit(argv.as_slice());
        root.answered(call, refusal(model.as_bytes(), c::LlmWhy::Authority));
        let answer = root.answer(call).expect("JSON refusal");
        assert_eq!(answer.status, 2);
        f::unique_keys(&answer.stderr).expect("unique JSON keys");
        let mut expected = serde_json::json!({
            "operation": "goal",
            "outcome": {"refused": {"refusal": {"llm": {"model": model, "why": "authority"}}, "at": null}},
            "task": null, "to": null
        });
        if let Some((flag, value)) = flag {
            expected["flag"] = flag.into();
            expected["value"] = value.into();
        }
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&answer.stderr).unwrap(), expected);
    }
}

#[test]
fn chat_start_names_the_refused_model_flag() {
    let mut root = f::choice_root(f::choice_me(f::CHAT_BUDGET));
    let call = root.submit(f::MODEL_CHAT);
    assert_text(
        root,
        call,
        f::MODEL_NAME,
        c::LlmWhy::Authority,
        "chat start: --model chosen-model: outside the authority the task is given\n",
    );
}

#[test]
fn restored_start_refusal_names_the_choice_part_without_flag_metadata() {
    for json in [false, true] {
        let mut original = f::choice_root(goal_me());
        let argv = f::extended(START, MODEL);
        let call = original.submit(argv.as_slice());
        let ask = f::act_ask(&original, call).clone();
        let mut records = f::restored_message(f::PARTY);
        records.get_mut(0).unwrap().request.request = jig_client::Ask::Jig(Box::new(ask));
        let mut root = f::started_with(records, f::SEED, json);
        root.deliver(c::Delivery::Me(Box::new(goal_me())));
        root.answered(0, refusal(f::MODEL_NAME, c::LlmWhy::Authority));
        let line = root.kept_lines.get(0).expect("restored refusal");
        if json {
            f::unique_keys(line).expect("unique restored JSON keys");
            let record: serde_json::Value = serde_json::from_slice(line).unwrap();
            assert!(record.get("flag").is_none());
            assert!(record.get("value").is_none());
            assert_eq!(record["outcome"]["refused"]["refusal"]["llm"]["model"], "chosen-model");
        } else {
            assert_eq!(
                line.as_ref(),
                b"kept: goal, goal start: main model chosen-model: outside the authority the task is given\n"
            );
        }
    }
}
