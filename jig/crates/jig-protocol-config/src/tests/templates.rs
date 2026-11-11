use crate::{Segment, What, tests::fixtures};
use jig_domain_people::{Symbol, TemplateBudget, TemplateDeadline, TemplateExecutor};
use jig_domain_tasks::{Contract, Flags, LlmChoice, ModelEffort, Retention};
use skein_lib::Duration;
const CHAT: u32 = 1;
const GOAL: u32 = 2;

fn with_resources(resources: &[u8]) -> Box<[u8]> {
    let template = fixtures::with(br#""name": "goal", "resources": RESOURCES"#, b"RESOURCES", resources);
    fixtures::with(fixtures::ENGINE_SMALL, br#""name": "goal""#, &template)
}

fn resource_path() -> crate::Path {
    fixtures::path(&[b"templates"]).child(Segment::Index(1)).member(b"resources")
}

#[test]
fn a_template_naming_resources_parses_them_in_order() {
    let bytes = with_resources(
        br#"[{"connector": 65535, "symbol": 65535}, {"connector": 0, "symbol": 0},
            {"connector": 0, "symbol": 65535}, {"connector": 65535, "symbol": 0}]"#,
    );
    let engine = fixtures::engine(&bytes).expect("four resources fit the small profile's holdings");
    assert_eq!(engine.limits.core.tasks.holdings, 4);
    assert_eq!(
        engine.templates[1].resources.as_ref(),
        &[
            Symbol { connector: u16::MAX, symbol: u16::MAX },
            Symbol { connector: 0, symbol: 0 },
            Symbol { connector: 0, symbol: u16::MAX },
            Symbol { connector: u16::MAX, symbol: 0 },
        ],
    );
}

#[test]
fn a_template_resource_naming_an_unknown_member_is_refused() {
    let bytes = with_resources(br#"[{"connector": 1, "symbol": 1, "unknown": 0}]"#);
    let problems = fixtures::engine(&bytes).expect_err("unknown resource member");
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(
        problems.problems.get(0),
        Some(&crate::Problem { path: resource_path().child(Segment::Index(0)), what: What::Form }),
    );
}

#[test]
fn a_template_resource_named_twice_is_refused() {
    let bytes = with_resources(br#"[{"connector": 1, "symbol": 1}, {"connector": 1, "symbol": 1}]"#);
    let problems = fixtures::engine(&bytes).expect_err("duplicate resource pair");
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(
        problems.problems.get(0),
        Some(&crate::Problem { path: resource_path().child(Segment::Index(1)), what: What::Unique }),
    );
}

#[test]
fn a_template_with_more_resources_than_holdings_is_refused() {
    let bytes = with_resources(
        br#"[{"connector": 1, "symbol": 1}, {"connector": 1, "symbol": 2},
            {"connector": 1, "symbol": 3}, {"connector": 1, "symbol": 4}, {"connector": 1, "symbol": 5}]"#,
    );
    let problems = fixtures::engine(&bytes).expect_err("five resources exceed the small profile's holdings");
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(problems.problems.get(0), Some(&crate::Problem { path: resource_path(), what: What::Bound }),);
}

#[test]
fn empty_template_resources_name_no_symbols() {
    let engine = fixtures::engine(&with_resources(b"[]")).expect("empty resources");
    assert!(engine.templates[1].resources.is_empty());
}

#[test]
fn malformed_template_resources_are_refused_at_the_item() {
    for resource in [
        b"[{}]".as_slice(),
        br#"[{"connector": 1}]"#,
        br#"[{"symbol": 1}]"#,
        br#"[{"connector": 65536, "symbol": 1}]"#,
        br#"[{"connector": 1, "symbol": 65536}]"#,
        br#"[{"connector": -1, "symbol": 1}]"#,
        br#"[{"connector": 1, "symbol": 1.5}]"#,
        br#"[{"connector": "1", "symbol": 1}]"#,
        br#"[{"connector": 1, "symbol": null}]"#,
        b"[null]",
        b"[1]",
        b"[[]]",
    ] {
        let problems = fixtures::engine(&with_resources(resource)).expect_err("invalid resource form");
        assert_eq!(problems.problems.len(), 1, "{resource:?}");
        assert_eq!(
            problems.problems.get(0),
            Some(&crate::Problem { path: resource_path().child(Segment::Index(0)), what: What::Form }),
            "{resource:?}",
        );
    }
}

#[test]
fn the_chat_and_goal_templates_parse_with_their_defaults() {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).expect("templates");
    assert_eq!(
        engine.templates.as_ref(),
        &[
            jig_domain_core::Template {
                resources: Box::new([]),
                number: CHAT,
                name: Box::from(b"chat".as_slice()),
                executor: TemplateExecutor::Charter(CHAT),
                contract: Contract::Report { words: 512 },
                shallower: 1,
                budget: TemplateBudget::AsAllowed { cap: None },
                deadline: TemplateDeadline::None,
                tracked: None,
                flags: Flags {
                    ended_by: jig_domain_tasks::EndedBy::Result,
                    conversation: true,
                    listed: false,
                    retention: Retention::Kept
                },
                label: 1,
                grace: Some(Duration::from_secs(5)),
                llm: None,
            },
            jig_domain_core::Template {
                resources: Box::new([]),
                number: GOAL,
                name: Box::from(b"goal".as_slice()),
                executor: TemplateExecutor::Chosen,
                contract: Contract::Report { words: 512 },
                shallower: 1,
                budget: TemplateBudget::Given,
                deadline: TemplateDeadline::Given,
                tracked: Some(1),
                flags: Flags {
                    ended_by: jig_domain_tasks::EndedBy::Result,
                    conversation: false,
                    listed: true,
                    retention: Retention::Kept
                },
                label: 2,
                grace: None,
                llm: None,
            },
        ]
    );
}
#[test]
fn a_template_grace_past_the_tasks_limit_refuses_the_start() {
    fixtures::refuses(
        br#""grace_ms": 5000"#,
        br#""grace_ms": 600001"#,
        fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"grace_ms"),
        What::Grace { template: CHAT },
    );
}
#[test]
fn a_template_numbered_twice_is_refused() {
    fixtures::refuses(
        br#""number": 2, "name": "goal""#,
        br#""number": 1, "name": "goal""#,
        fixtures::path(&[b"templates"]).child(Segment::Index(1)).member(b"number"),
        What::Unique,
    );
}
#[test]
fn a_template_naming_an_unknown_charter_is_refused() {
    fixtures::refuses(
        br#""executor": {"charter": 1}"#,
        br#""executor": {"charter": 9}"#,
        fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"executor"),
        What::Refers,
    );
}
#[test]
fn an_approval_contract_in_a_template_is_refused() {
    fixtures::refuses(
        br#""contract": {"report": {"words": 512}}"#,
        br#""contract": {"approval": {}}"#,
        fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"contract"),
        What::Form,
    );
}
#[test]
fn a_seeded_override_naming_no_template_refuses_the_start() {
    let document = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""templates": []"#,
        br#""templates": [{"template": 99, "executor": null, "budget": null, "deadline": null, "priority": 5}]"#,
    );
    let problems = fixtures::engine(&document).expect_err("unknown template");
    assert_eq!(problems.problems.get(0).expect("one refusal").what, What::Refers);
    assert_eq!(
        problems.problems.get(0).expect("one refusal").path,
        fixtures::path(&[b"projects"])
            .child(Segment::Index(0))
            .member(b"policy")
            .member(b"templates")
            .child(Segment::Index(0))
            .member(b"template")
    );
}
#[test]
fn a_seeded_policy_overriding_a_template_twice_refuses_the_start() {
    fixtures::refuses(
        br#""templates": []"#,
        br#""templates": [{"template": 2, "executor": null, "budget": null, "deadline": null, "priority": 5},
            {"template": 2, "executor": null, "budget": null, "deadline": null, "priority": 7}]"#,
        fixtures::path(&[b"projects"])
            .child(Segment::Index(0))
            .member(b"policy")
            .member(b"templates")
            .child(Segment::Index(1))
            .member(b"template"),
        What::Unique,
    );
}

#[test]
fn a_seeded_override_reaches_the_cores_config() {
    const GOAL: u32 = 2;
    let document = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""templates": []"#,
        br#""templates": [{"template": 2, "executor": null, "budget": null, "deadline": null, "priority": 5}]"#,
    );
    let engine = fixtures::engine(&document).expect("seeded override");
    let config = crate::core(&engine, [0; 16], 7);
    assert_eq!(
        Some(&config.seed_projects[0].policy.templates),
        Some(&Box::from([jig_domain_people::TemplateOverride {
            template: GOAL,
            executor: None,
            budget: None,
            deadline: None,
            priority: Some(5),
        }]))
    );
}

/// The small engine whose model "small" takes two efforts, beside a model "large" that takes
/// none; charter 1 allows "large" too when `alternatives` names it.
fn with_models(alternatives: &[u8], llm: &[u8]) -> Box<[u8]> {
    let efforts = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""output": 1024,"#,
        br#""output": 1024, "efforts": ["low", "high"],"#,
    );
    let large = fixtures::with(
        &efforts,
        b"\"unit\": 1000}\n}]",
        br#""unit": 1000}
}, {"name": "large", "endpoint": "fake", "window": 8192, "output": 1024,
"prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000}}]"#,
    );
    let large = fixtures::with(&large, br#""models":["small"]"#, br#""models":["small","large"]"#);
    let charter = fixtures::with(&large, br#""alternatives": []"#, alternatives);
    fixtures::with(&charter, br#""grace_ms": 5000, "llm": null"#, llm)
}
fn chat_llm() -> crate::Path {
    fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"llm")
}
fn refuses_choice(alternatives: &[u8], llm: &[u8], path: crate::Path, what: What) {
    let problems = fixtures::engine(&with_models(alternatives, llm)).expect_err("refused choice");
    assert_eq!(problems.problems.get(0), Some(&crate::Problem { path, what }));
}
#[test]
fn a_templates_llm_choice_names_configured_models_and_their_efforts() {
    let document = with_models(
        br#""alternatives": ["large"]"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "high"},
            "sub_agents": [{"model": "large", "effort": null}],
            "later":{"main":{"model":"large","effort":null},"sub_agents":[{"model":"small","effort":"low"}]}}"#,
    );
    let engine = fixtures::engine(&document).expect("template choice");
    let chat = engine.templates.first().expect("chat");
    assert_eq!(
        chat.llm,
        Some(LlmChoice {
            later: Some(jig_domain_tasks::LlmLater {
                main: ModelEffort { model: 1, effort: None },
                subagents: Box::new([ModelEffort { model: 0, effort: Some(0) }])
            }),
            account: None,
            main: ModelEffort { model: 0, effort: Some(1) },
            subagents: Box::from([ModelEffort { model: 1, effort: None }]),
        })
    );
}
fn a_template_without_an_llm_member_is_refused() {
    refuses_choice(br#""alternatives": []"#, br#""grace_ms": 5000"#, chat_llm(), What::Missing);
}
fn a_choice_that_is_not_an_object_is_refused() {
    refuses_choice(br#""alternatives": []"#, br#""grace_ms": 5000, "llm": "default""#, chat_llm(), What::Kind);
}
fn a_choice_naming_an_unconfigured_model_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "huge", "effort": null}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"model"),
        What::Refers,
    );
}
fn an_effort_its_model_does_not_take_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "medium"}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"effort"),
        What::Refers,
    );
}
fn no_effort_for_a_model_that_declares_some_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": null}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"effort"),
        What::Refers,
    );
}
fn an_effort_for_a_model_that_declares_none_is_refused() {
    refuses_choice(
        br#""alternatives": ["large"]"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"},
            "sub_agents": [{"model": "large", "effort": "low"}]}"#,
        chat_llm().member(b"sub_agents").child(Segment::Index(0)).member(b"effort"),
        What::Refers,
    );
}
fn an_effort_longer_than_smiths_bound_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small",
            "effort": "an-effort-named-past-thirty-two-bytes"}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"effort"),
        What::Bound,
    );
}
fn a_model_outside_the_templates_charter_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"},
            "sub_agents": [{"model": "large", "effort": null}]}"#,
        chat_llm().member(b"sub_agents").child(Segment::Index(0)).member(b"model"),
        What::Refers,
    );
}
fn a_sub_agent_model_named_twice_is_refused() {
    refuses_choice(
        br#""alternatives": ["large"]"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"},
            "sub_agents": [{"model": "large", "effort": null}, {"model": "large", "effort": null}]}"#,
        chat_llm().member(b"sub_agents").child(Segment::Index(1)).member(b"model"),
        What::Unique,
    );
}
fn sub_agents_that_are_not_a_list_are_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"}, "sub_agents": {}}"#,
        chat_llm().member(b"sub_agents"),
        What::Kind,
    );
}
#[test]
fn a_templates_llm_choice_problems_name_the_invalid_fields() {
    table([
        (
            "a_later_choice_naming_an_unconfigured_model_is_refused",
            a_later_choice_naming_an_unconfigured_model_is_refused,
        ),
        ("a_template_without_an_llm_member_is_refused", a_template_without_an_llm_member_is_refused),
        ("a_choice_that_is_not_an_object_is_refused", a_choice_that_is_not_an_object_is_refused),
        ("a_choice_naming_an_unconfigured_model_is_refused", a_choice_naming_an_unconfigured_model_is_refused),
        ("an_effort_its_model_does_not_take_is_refused", an_effort_its_model_does_not_take_is_refused),
        ("no_effort_for_a_model_that_declares_some_is_refused", no_effort_for_a_model_that_declares_some_is_refused),
        ("an_effort_for_a_model_that_declares_none_is_refused", an_effort_for_a_model_that_declares_none_is_refused),
        ("an_effort_longer_than_smiths_bound_is_refused", an_effort_longer_than_smiths_bound_is_refused),
        ("a_model_outside_the_templates_charter_is_refused", a_model_outside_the_templates_charter_is_refused),
        ("a_sub_agent_model_named_twice_is_refused", a_sub_agent_model_named_twice_is_refused),
        ("sub_agents_that_are_not_a_list_are_refused", sub_agents_that_are_not_a_list_are_refused),
    ]);
}

type Case = (&'static str, fn());

fn with_executor_override(llm: &[u8], executor: &[u8]) -> Box<[u8]> {
    let document = with_models(br#""alternatives": []"#, llm);
    let document = fixtures::with(&document, br#""executor": {"charter": 1}"#, br#""executor": "chosen""#);
    let overrides = fixtures::with(
        br#""templates": [{"template": 1, "executor": EXECUTOR, "budget": null, "deadline": null, "priority": null}]"#,
        b"EXECUTOR",
        executor,
    );
    fixtures::with(&document, br#""templates": []"#, &overrides)
}

#[test]
fn a_seeded_executor_override_cannot_invalidate_the_retained_llm_choice() {
    for (name, llm, executor) in [
        (
            "main outside charter",
            br#""grace_ms": 5000, "llm": {"main": {"model": "large", "effort": null}, "sub_agents": []}"#.as_slice(),
            br#"{"charter": 1}"#.as_slice(),
        ),
        (
            "sub-agent outside charter",
            br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "high"}, "sub_agents": [{"model": "large", "effort": null}]}"#,
            br#"{"charter": 1}"#,
        ),
        ("unknown charter without choice", br#""grace_ms": 5000, "llm": null"#, br#"{"charter": 99}"#),
    ] {
        let problems = fixtures::engine(&with_executor_override(llm, executor)).expect_err(name);
        assert_eq!(
            problems.problems.get(0),
            Some(&crate::Problem {
                path: fixtures::path(&[b"projects"]).child(Segment::Index(0)).member(b"policy")
                    .member(b"templates").child(Segment::Index(0)).member(b"executor"),
                what: What::Refers,
            }),
            "{name}",
        );
    }
}

#[test]
fn a_seeded_executor_override_keeps_a_compatible_choice() {
    let document = with_executor_override(
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "high"}, "sub_agents": []}"#,
        br#"{"charter": 1}"#,
    );
    let engine = fixtures::engine(&document).expect("compatible override");
    let config = crate::core(&engine, [0; 16], 7);
    let merged = jig_domain_core::settings::templates::merged(
        &config.defaults.templates,
        &config.seed_projects[0].policy.templates,
    );
    assert_eq!(merged.first().expect("chat").executor, TemplateExecutor::Charter(CHAT));
    assert_eq!(
        merged.first().expect("chat").llm,
        Some(LlmChoice {
            later: None,
            account: None,
            main: ModelEffort { model: 0, effort: Some(1) },
            subagents: Box::new([])
        })
    );
}

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}

#[test]
fn configuration_templates_refuse_an_account_at_its_exact_path() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms":5000,"llm":{"main":{"model":"small","effort":"low"},"sub_agents":[],"account":1}"#,
        chat_llm().member(b"account"),
        What::Unknown,
    );
}

#[test]
fn requester_ending_is_explicit_and_requires_an_agent_conversation() {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""conversation": true"#,
        br#""conversation": true, "ended_by": "requester""#,
    );
    let engine = fixtures::engine(&bytes).unwrap();
    assert_eq!(engine.templates[0].flags.ended_by, jig_domain_tasks::EndedBy::Requester);
    for (from, to) in [
        (br#""conversation": true"#.as_slice(), br#""conversation": false"#.as_slice()),
        (br#""executor": {"charter": 1}"#.as_slice(), br#""executor": "chosen""#.as_slice()),
    ] {
        let bad = fixtures::with(&bytes, from, to);
        fixtures::engine(&bad).expect_err("invalid ending authority");
    }
}

#[test]
fn a_misspelled_requester_ending_refuses_the_template_at_ended_by() {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""conversation": true"#,
        br#""conversation": true, "ended_by": "requestor""#,
    );
    let problems = fixtures::engine(&bytes).expect_err("no engine or template is produced");
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(problems.past, 0);
    assert_eq!(
        problems.problems.get(0),
        Some(&crate::Problem {
            path: fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"ended_by"),
            what: What::Form,
        }),
    );
}

fn a_later_choice_naming_an_unconfigured_model_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms":5000,"llm":{"main":{"model":"small","effort":"low"},"sub_agents":[],
        "later":{"main":{"model":"absent","effort":null},"sub_agents":[]}}"#,
        chat_llm().member(b"later").member(b"main").member(b"model"),
        What::Refers,
    );
}

fn closer_template(ended_by: &[u8], conversation: bool) -> Box<[u8]> {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""conversation": true"#,
        if conversation {
            br#""conversation": true, "ended_by": CLOSER"#
        } else {
            br#""conversation": false, "ended_by": CLOSER"#
        },
    );
    fixtures::with(&bytes, b"CLOSER", ended_by)
}
#[test]
fn a_template_names_its_closer() {
    let bytes = closer_template(br#"{"closer":{"connector":1,"code":6}}"#, false);
    let engine = fixtures::engine(&bytes).expect("agent closer admitted");
    assert_eq!(
        engine.templates[0].flags.ended_by,
        jig_domain_tasks::EndedBy::Closer(jig_domain_tasks::Closer { connector: 1, code: 6 })
    );
}
#[test]
fn a_closer_on_a_conversation_template_is_refused() {
    let bytes = closer_template(br#"{"closer":{"connector":1,"code":6}}"#, true);
    assert_closer_form(&bytes);
}
fn assert_closer_form(bytes: &[u8]) {
    let problems = fixtures::engine(bytes).expect_err("invalid closer");
    assert_eq!(
        problems.problems.as_slice(),
        &[crate::Problem {
            path: fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"ended_by"),
            what: What::Form,
        }]
    );
}
#[test]
fn a_malformed_closer_is_refused() {
    for ended_by in [
        br#"{"closer":{"connector":1}}"#.as_slice(),
        br#"{"closer":{"code":6}}"#,
        br#"{"closer":{"connector":1,"code":6,"extra":0}}"#,
        br#"{"closer":{"connector":1,"code":4294967296}}"#,
        br#"{"closer":{"connector":65536,"code":6}}"#,
    ] {
        assert_closer_form(&closer_template(ended_by, false));
    }
}

#[test]
fn standing_templates_require_untracked_agents_without_conversations() {
    let bytes = closer_template(br#""standing""#, false);
    let engine = fixtures::engine(&bytes).expect("untracked standing agent template");
    assert_eq!(engine.templates[0].flags.ended_by, jig_domain_tasks::EndedBy::Standing);
    assert_eq!(engine.templates[0].tracked, None);
    for (from, to) in [
        (br#""conversation": false"#.as_slice(), br#""conversation": true"#.as_slice()),
        (br#""executor": {"charter": 1}"#, br#""executor": "chosen""#),
        (br#""tracked": null"#, br#""tracked": {"priority": 1}"#),
    ] {
        assert_closer_form(&fixtures::with(&bytes, from, to));
    }
}
