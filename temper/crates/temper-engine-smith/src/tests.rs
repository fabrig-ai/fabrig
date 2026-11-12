//! Typed boundary tests through the public conversions.
#![expect(clippy::disallowed_macros, reason = "tests assemble JSON cases from shared fields")]
use jig_domain_brief as brief;
use jig_domain_link::{Form, Scheme};
use jig_domain_tasks as tasks;
use skein_lib::{Duration, ReplyTo, Token};
use smith_domain as smith;
use smith_domain_run as run;
use temper_engine_domain as engine;

const CALL_TASK: u64 = 23;

use crate::{ChangeResource, Problem, answer, call, result, start, tools, turn};

fn decoded(tool: &[u8], json: &[u8]) -> Result<crate::Decoded, Problem> {
    let input = run::HostInput::attested(json.into()).expect("bounded object exterior");
    call(run::CallName { activation: 7, completion: 2, position: 1 }, tool, &input)
}

fn decided(tool: &[u8], json: &[u8]) -> Result<engine::Call, Problem> {
    match decoded(tool, json)? {
        crate::Decoded::Call(call) => Ok(call),
        crate::Decoded::Answered(_) => panic!("decided tool answered at decode"),
    }
}

fn assignment() -> engine::Assignment {
    engine::Assignment {
        check_timeout: None,
        task: 23,
        attempt: 7,
        conversation: 7,
        live: jig_domain_host::LiveAsk::NONE,
        charter: 1,
        run: Box::new(engine::RunCharter {
            revision: 1,
            ended_by: jig_domain_tasks::EndedBy::Result,
            sources: Box::new([Some(1)]),
            accounts: Box::new([jig_domain_authority::AccountBinding { provider: 1, account: 1, source: 1 }]),
            placement: jig_domain_fleet::PlacementConstraint {
                channel: Token::new(1),
                kind: jig_domain_fleet::HostKind::Worker,
            },
            policy: engine::RunPolicy {
                instructions: b"Do the task".as_slice().into(),
                waiting: Duration::from_secs(1),
                resume: true,
                turns: 8,
                time: Duration::from_secs(60),
                reserve_turns: 1,
                reserve_time: Duration::from_secs(10),
                model: engine::Model {
                    provider: 1,
                    program: None,
                    dialect: 1,
                    endpoint: 1,
                    name: b"scripted".as_slice().into(),
                    cache_life_ms: 0,
                    window: 8192,
                    output: 1024,
                    input_price: 1,
                    cached_price: 1,
                    cache_write_price: None,
                    output_price: 1,
                    price_unit: 1000,
                    efforts: Box::new([]),
                    effort: None,
                },
                alternatives: Box::new([]),
                inspect: true,
                modify: false,
                shell: false,
                agents: false,
                call_timeout: Duration::from_secs(10),
                tools: engine::RunPolicy::EVERY_FAMILY,
            },
            contract: tasks::Contract::Report { words: 128 },
            authority: tasks::Authority {
                tools: tasks::Tools(1),
                models: tasks::Models::NONE,
                grants: Box::new([]),
                delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
                budget: tasks::Budget { spend: 40, deadline: None },
                notes: tasks::Scopes(0),
                note_resources: Box::new([]),
            },
            budget: 40,
        }),
        sections: Box::new([engine::BriefSection {
            kind: engine::BriefKind::Core(brief::Core::Task),
            body: engine::BriefBody::Text(b"Say hello".as_slice().into()),
        }]),
        messages: Box::new([]),
        saved: Box::new([]),
        workspace: engine::ForgeWorkspace { repositories: Box::new([]) },
        workstream: 23,
        transcript: Box::new([]),
        answered: Box::new([]),
        settled: Box::new([]),
        grants: Box::new([jig_domain_link::Grant {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 2,
            valid: Duration::from_secs(30),
        }]),
    }
}

#[test]
fn root_charter_becomes_a_smith_start_with_ordered_brief_and_exact_budget() {
    let window = smith::Window { turns: 1, bytes: 100_000 };
    let smith::Event::Start(smith::Start { activation, charter, grants, transcript, window: started_window, .. }) =
        start(
            &assignment(),
            None,
            None,
            ReplyTo::new(Token::new(50)),
            Token::new(23),
            window,
            &smith_agent_world::LIMITS.run,
        )
    else {
        panic!("start conversion produces Smith Start")
    };
    assert_eq!(activation, 7);
    assert_eq!(started_window, window);
    assert_eq!(charter.instructions.as_ref(), b"Do the task");
    assert_eq!(charter.brief.sections[0].title.as_ref(), b"Task");
    assert_eq!(charter.brief.sections[0].text.as_ref(), b"Say hello");
    assert_eq!(charter.budget.spend, 40);
    assert_eq!(charter.grants.host_tools.len(), 37);
    assert_eq!(grants[0].name.generation, 2);
    assert!(transcript.is_none());
}

fn goal_tools(timeout: Duration) -> Box<[run::HostTool]> {
    let mut expected = crate::calls::tools_for(
        timeout,
        engine::RunPolicy::EVERY_FAMILY,
        tasks::Tools(!jig_domain_core::ToolFamilies::JIG.control.0),
    );
    for tool in &mut expected {
        tool.reserve = tool.name.as_ref() == b"message";
    }
    expected
}

#[test]
fn settings_tools_are_offered_only_with_the_runs_settings_family() {
    // Without the charter's delegate family, message and settings add one and two declarations.
    const WITHOUT_SETTINGS: usize = 35;
    const WITH_SETTINGS: usize = 37;
    let settings = jig_domain_core::ToolFamilies::JIG.settings;
    let authority = 1 | jig_domain_core::ToolFamilies::JIG.message.0;
    for (charter, authority, allowed, count) in [
        (0, authority, false, WITHOUT_SETTINGS),
        (0, authority | settings, false, WITHOUT_SETTINGS),
        (settings, authority, false, WITHOUT_SETTINGS),
        (settings, authority | settings, true, WITH_SETTINGS),
    ] {
        let mut assignment = assignment();
        assignment.run.policy.tools = tasks::Tools(charter);
        assignment.run.authority.tools = tasks::Tools(authority);
        let charter = crate::charter(&assignment);
        assert_eq!(charter.grants.host_tools.len(), count);
        for name in [b"settings".as_slice(), b"propose_settings"] {
            let mut found = false;
            for tool in &charter.grants.host_tools {
                if tool.name.as_ref() == name {
                    found = true;
                }
            }
            assert_eq!(found, allowed);
        }
        for tool in goal_tools(Duration::from_secs(10)) {
            if tool.name.as_ref() != b"settings"
                && tool.name.as_ref() != b"propose_settings"
                && tool.name.as_ref() != b"read_task"
                && tool.name.as_ref() != b"goal"
                && tool.name.as_ref() != b"change"
                && tool.name.as_ref() != b"delegate"
            {
                assert!(charter.grants.host_tools.contains(&tool), "unchanged declaration: {:?}", tool.name);
            }
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks asking schemas and both charter-family gates")]
fn settings_and_delegate_tools_need_the_charters_family() {
    type AskingNames = Option<&'static [&'static [u8]]>;
    let stock = 1_539;
    let mut assignment = assignment();
    assignment.run.policy.tools = tasks::Tools(512);
    assignment.run.authority.tools = tasks::Tools(stock | 1024);
    let charter = crate::charter(&assignment);
    let mut offered = charter.grants.host_tools.iter();
    for tool in goal_tools(Duration::from_secs(10)) {
        if tool.name.as_ref() != b"settings"
            && tool.name.as_ref() != b"propose_settings"
            && tool.name.as_ref() != b"goal"
            && tool.name.as_ref() != b"change"
            && tool.name.as_ref() != b"delegate"
        {
            assert_eq!(offered.next(), Some(&tool), "unchanged declaration and order: {:?}", tool.name);
        }
    }
    assert_eq!(offered.next(), None, "no settings, delegate or additional tools");
    let timeout = Duration::from_secs(13);
    let rows: [(u64, u64, AskingNames); 8] = [
        (u64::MAX, u64::MAX, None),
        (3_584, 3_584, None),
        (3_072, 3_072, Some(&[b"see", b"document", b"remember", b"propose_settings"])),
        (2_048, 3_072, Some(&[b"see", b"document", b"remember"])),
        (3_072, 1_024, Some(&[b"propose_settings"])),
        (1_023, u64::MAX, None),
        (1_920, u64::MAX, None),
        (1_022, u64::MAX, None),
    ];
    for (charter, authority, asking_names) in rows {
        let offered = crate::calls::tools_for(timeout, tasks::Tools(charter), tasks::Tools(authority));
        if charter == 1_022 {
            for (name, declared) in [
                (b"goal".as_slice(), false),
                (b"change", false),
                (b"delegate", false),
                (b"note", true),
                (b"recall", true),
                (b"read_branch", true),
                (b"read_file", true),
            ] {
                let mut found = false;
                for tool in &offered {
                    if tool.name.as_ref() == name {
                        found = true;
                    }
                }
                assert_eq!(found, declared, "drafter: {name:?}");
            }
        }
        for tool in &offered {
            assert_eq!(tool.timeout, timeout);
        }
        if let Some(names) = asking_names {
            assert_eq!(offered.len(), names.len(), "charter {charter}");
            for (tool, name) in offered.iter().zip(names) {
                assert_eq!(tool.name.as_ref(), *name, "charter {charter}");
            }
            for tool in &offered {
                if tool.name.as_ref() == b"propose_settings" {
                    let schema = crate::json::parse(&tool.schema).unwrap();
                    let properties = schema.required(b"properties").unwrap();
                    assert!(properties.field(b"note").unwrap().is_none());
                    assert!(properties.field(b"more").unwrap().is_some());
                    assert_eq!(
                        schema.required(b"required").unwrap(),
                        &crate::json::Value::Array(Box::new([
                            crate::json::Value::String(b"against".as_slice().into()),
                            crate::json::Value::String(b"reason".as_slice().into()),
                            crate::json::Value::String(b"items".as_slice().into()),
                        ]))
                    );
                }
            }
        } else {
            let expected = crate::calls::tools_for(
                timeout,
                engine::RunPolicy::EVERY_FAMILY,
                tasks::Tools(authority & if charter & 1_024 == 0 { !1_024 } else { u64::MAX }),
            );
            let mut declarations = offered.iter();
            for tool in &expected {
                if charter & 1 == 0
                    && (tool.name.as_ref() == b"goal"
                        || tool.name.as_ref() == b"change"
                        || tool.name.as_ref() == b"delegate")
                {
                    continue;
                }
                assert_eq!(declarations.next(), Some(tool), "unchanged declarations for charter {charter}");
            }
            assert_eq!(declarations.next(), None, "no additional tools for charter {charter}");
            let mut proposal = false;
            for tool in &offered {
                assert!(
                    tool.name.as_ref() != b"see"
                        && tool.name.as_ref() != b"document"
                        && tool.name.as_ref() != b"remember"
                );
                if tool.name.as_ref() == b"propose_settings" {
                    proposal = true;
                    assert!(
                        crate::json::parse(&tool.schema)
                            .unwrap()
                            .required(b"properties")
                            .unwrap()
                            .field(b"note")
                            .unwrap()
                            .is_some()
                    );
                }
            }
            assert_eq!(proposal, charter & authority & 1_024 != 0);
        }
    }
}

#[test]
fn message_is_offered_only_with_the_runs_message_family() {
    let families = jig_domain_core::ToolFamilies::JIG;
    for settings in [false, true] {
        for (role, authority, message) in [
            ("coder", families.read.0 | families.effect.0, false),
            ("repairer", families.read.0 | families.effect.0, false),
            ("reviewer", families.read.0, false),
            ("chat", families.read.0 | families.message.0, true),
            ("goal", families.delegate.0 | families.message.0, true),
        ] {
            let mut assignment = assignment();
            assignment.run.authority.tools = tasks::Tools(authority | if settings { families.settings } else { 0 });
            let charter = crate::charter(&assignment);
            let expected = goal_tools(Duration::from_secs(10));
            let mut offered = charter.grants.host_tools.iter();
            for tool in &expected {
                if (!settings && (tool.name.as_ref() == b"settings" || tool.name.as_ref() == b"propose_settings"))
                    || (!message && tool.name.as_ref() == b"message")
                    || (authority & families.read.0 == 0 && tool.name.as_ref() == b"read_task")
                {
                    continue;
                }
                assert_eq!(offered.next(), Some(tool), "{role}: unchanged declaration and order");
            }
            assert_eq!(offered.next(), None, "{role}: no additional tools");
            assert_eq!(
                charter.grants.host_tools.len(),
                37 + usize::from(message) + 2 * usize::from(settings) + usize::from(authority & families.read.0 != 0)
            );
        }
    }
}

#[test]
fn a_charters_mode_is_its_tasks_ending() {
    let change = run::outcome::ChangeSpec {
        checks_must_pass: true,
        fields: Box::new([
            run::outcome::FieldRule { name: b"title".as_slice().into(), max: 128 },
            run::outcome::FieldRule { name: b"body".as_slice().into(), max: 128 },
        ]),
    };
    for (ended_by, mode, deliver) in
        [(tasks::EndedBy::Result, run::Mode::Finite, Some(change)), (tasks::EndedBy::Requester, run::Mode::Open, None)]
    {
        let mut assignment = assignment();
        assignment.run.ended_by = ended_by;
        assignment.run.contract = tasks::Contract::Change { connector: 1, kind: 1, words: 128 };
        let charter = crate::charter(&assignment);
        assert_eq!(charter.mode, mode);
        assert!(charter.grants.wait);
        assert_eq!(charter.grants.deliver, deliver);
    }
}

#[test]
fn a_report_run_cannot_modify_without_delivery() {
    let mut assignment = assignment();
    assignment.run.policy.modify = true;
    let charter = crate::charter(&assignment);
    assert_eq!(charter.mode, run::Mode::Finite);
    assert!(charter.grants.deliver.is_none());
    assert!(!charter.grants.tools.modify);
    assert!(charter.grants.tools.inspect);
}

fn delivering_assignment() -> engine::Assignment {
    let mut assignment = assignment();
    assignment.run.policy.modify = true;
    assignment.workspace.repositories = Box::new([engine::ForgeRepository {
        delivers: true,
        save: Some(b"temper/23/s23".as_slice().into()),
        tag: 1,
        provider: skein_forge::api::Repository { forge: 1, repository: 1 },
        name: b"repo".as_slice().into(),
        host: b"forge.example".as_slice().into(),
        owner: b"org".as_slice().into(),
        start: engine::ForgeStart::Base(b"temper/23/g23".as_slice().into()),
        push: Some(b"temper/23/r23".as_slice().into()),
        landing: None,
        identity: 1,
    }]);
    assignment
}

#[test]
fn a_report_run_delivers_its_repository_with_checks_and_keeps_its_report_outcome() {
    let charter = crate::charter(&delivering_assignment());
    assert_eq!(charter.mode, run::Mode::Finite);
    let deliver = charter.grants.deliver.expect("the workspace delivers");
    assert!(deliver.checks_must_pass);
    assert!(charter.grants.tools.modify);
    assert_eq!(charter.conventions.expect("repository conventions").checks.as_ref(), b".temper/pre-pr");
    assert!(charter.outcome.change.is_none());
    assert!(charter.outcome.report.is_some());
}

#[test]
fn an_assignments_check_bound_reaches_the_smith_charter() {
    for bound in [Some(Duration::from_secs(900)), None] {
        let mut assignment = delivering_assignment();
        assignment.check_timeout = bound;
        assert_eq!(crate::charter(&assignment).check_timeout, bound);
        // A resumed attempt uses the same application assignment field.
        assignment.transcript = Box::new([b"history".as_slice().into()]);
        assert_eq!(crate::charter(&assignment).check_timeout, bound);
    }
}

#[test]
fn an_open_run_cannot_deliver_its_repository() {
    let mut assignment = delivering_assignment();
    assignment.run.ended_by = tasks::EndedBy::Requester;
    let charter = crate::charter(&assignment);
    assert_eq!(charter.mode, run::Mode::Open);
    assert!(charter.grants.deliver.is_none());
    assert!(!charter.grants.tools.modify);
}

#[test]
fn an_open_run_cannot_modify_even_when_policy_requests_it() {
    let mut assignment = assignment();
    assignment.run.ended_by = tasks::EndedBy::Requester;
    assignment.run.contract = tasks::Contract::Change { connector: 1, kind: 1, words: 128 };
    assignment.run.policy.modify = true;
    assignment.run.policy.shell = true;
    let charter = crate::charter(&assignment);
    assert_eq!(charter.mode, run::Mode::Open);
    assert!(charter.grants.deliver.is_none());
    assert!(!charter.grants.tools.modify);
    assert!(charter.grants.tools.shell);
}

#[test]
fn a_finite_change_run_keeps_its_modify_policy() {
    for modify in [false, true] {
        let mut assignment = assignment();
        assignment.run.contract = tasks::Contract::Change { connector: 1, kind: 1, words: 128 };
        assignment.run.policy.modify = modify;
        let charter = crate::charter(&assignment);
        assert_eq!(charter.mode, run::Mode::Finite);
        assert!(charter.grants.deliver.is_some());
        assert_eq!(charter.grants.tools.modify, modify);
    }
}

#[test]
fn a_starting_run_hears_its_met_dependency_by_name() {
    // A run resumed by its last dependency's end hears the Met notice that woke it (jig's
    // domain/tasks/messages.md, section 1), as a live run does.
    let mut assignment = assignment();
    assignment.messages = Box::new([tasks::Word {
        number: 2,
        from: tasks::Party::Task(2),
        kind: tasks::MessageKind::Met { dependency: 2, removed: false },
        words: Box::new([]),
        at: skein_lib::Wall::EPOCH,
        hits: 1,
        eligible: true,
    }]);
    let window = smith::Window { turns: 1, bytes: 100_000 };
    let smith::Event::Start(smith::Start { messages, .. }) = start(
        &assignment,
        None,
        None,
        ReplyTo::new(Token::new(50)),
        Token::new(23),
        window,
        &smith_agent_world::LIMITS.run,
    ) else {
        panic!("start conversion produces Smith Start")
    };
    assert_eq!(messages[0].label.as_ref(), b"2");
    assert_eq!(messages[0].text.as_ref(), b"T2 is done; the task can go on");
}

#[test]
fn a_starting_inbox_keeps_question_and_answer_references() {
    let mut assignment = assignment();
    assignment.messages = Box::new([
        tasks::Word {
            number: 4,
            from: tasks::Party::Task(2),
            kind: tasks::MessageKind::Question,
            words: b"The plan?".as_slice().into(),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        },
        tasks::Word {
            number: 9,
            from: tasks::Party::Person(1),
            kind: tasks::MessageKind::Answer { question: 4 },
            words: b"Approved.".as_slice().into(),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        },
    ]);
    let smith::Event::Start(smith::Start { messages, .. }) = start(
        &assignment,
        None,
        None,
        ReplyTo::new(Token::new(50)),
        Token::new(23),
        smith::Window { turns: 1, bytes: 100_000 },
        &smith_agent_world::LIMITS.run,
    ) else {
        panic!("start conversion")
    };
    assert_eq!(
        messages[0].label.as_ref(),
        b"2 asks (question 4; answer with message target=2, form=answer, question=4)"
    );
    assert_eq!(messages[0].text.as_ref(), b"The plan?");
    assert_eq!(messages[1].label.as_ref(), b"1 answers question 4");
    assert_eq!(messages[1].text.as_ref(), b"Approved.");
}

#[test]
fn selected_llm_bindings_reach_smith() {
    let mut assignment = assignment();
    assignment.run.policy.model.effort = Some(b"high".as_slice().into());
    let alternative = engine::Model {
        name: b"alternative".as_slice().into(),
        effort: Some(b"low".as_slice().into()),
        ..assignment.run.policy.model.clone()
    };
    assignment.run.policy.alternatives = Box::new([alternative.clone()]);
    assignment.run.sources = Box::new([Some(17), Some(23)]);
    assignment.run.accounts = Box::new([
        jig_domain_authority::AccountBinding { provider: 1, account: 11, source: 17 },
        jig_domain_authority::AccountBinding { provider: 2, account: 19, source: 23 },
    ]);
    let charter = crate::charter(&assignment);
    assert_eq!(charter.llm.account, Some(17));
    assert_eq!(charter.llm.effort.as_deref(), Some(b"high".as_slice()));
    assert_eq!(charter.models[0].account, Some(23));
    assert_eq!(charter.models[0].model, alternative.name);
    assert_eq!(charter.models[0].effort, alternative.effort);
    assignment.run.sources = Box::new([None, None]);
    let charter = crate::charter(&assignment);
    assert_eq!(charter.llm.account, None);
    assert_eq!(charter.models[0].account, None);
}

#[test]
fn message_schema_decodes_a_named_question_and_rejects_a_malformed_body() {
    assert_eq!(
        decided(b"message", br#"{"target":23,"form":"question","words":"Ready?"}"#),
        Ok(engine::Call {
            completion: 2,
            position: 1,
            tool: engine::Tool::Message {
                target: 23,
                form: engine::MessageForm::Question,
                words: b"Ready?".as_slice().into(),
            },
        })
    );
    assert_eq!(decided(b"message", br#"{"target":0,"form":"words","words":"x"}"#), Err(Problem::Range(b"target")));
    assert_eq!(decided(b"message", br#"{"target":23,,"form":"words","words":"x"}"#), Err(Problem::Malformed));
}

fn tool_cases() -> &'static [(&'static [u8], &'static [u8], &'static [u8])] {
    &[
        (b"see", br#"{"address":" settings charter 3 "}"#, br#"{"address":3}"#),
        (b"document", br##"{"action":"write","body":"# Title"}"##, br#"{"action":"unknown","body":""}"#),
        (b"remember", br#"{"action":"add","text":"Keep it brief."}"#, br#"{"action":"unknown"}"#),
        (b"settings", br"{}", br#"{"from":-1}"#),
        (b"read_task", br#"{"task":23}"#, br#"{"task":0}"#),
        (b"propose_settings", br#"{"against":1,"items":[],"reason":"why"}"#, br#"{"against":0,"items":[],"reason":"why"}"#),
        (b"read_items", br#"{"repository":"qa/pilot","since":0,"page":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_item", br#"{"repository":"qa/pilot","number":1,"after":0}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_pull", br#"{"repository":"qa/pilot","number":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_pull_for", br#"{"repository":"qa/pilot","head":"h","base":"b"}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_reviews", br#"{"repository":"qa/pilot","number":1,"page":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_statuses", br#"{"repository":"qa/pilot","commit":"0000000000000000000000000000000000000000000000000000000000000000","page":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_remarks", br#"{"repository":"qa/pilot","number":1,"review":1,"page":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_branch", br#"{"repository":"qa/pilot","branch":"main"}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_branches", br#"{"repository":"qa/pilot","prefix":"temper/"}"#, br#"{"repository":"qa/pilot","kind":"wrong"}"#),
        (b"read_pull_files", br#"{"repository":"qa/pilot","number":1,"head":"0000000000000000000000000000000000000000000000000000000000000000","page":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_compare", br#"{"repository":"qa/pilot","before":"0000000000000000000000000000000000000000000000000000000000000000","after":"0000000000000000000000000000000000000000000000000000000000000000"}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_checks", br#"{"repository":"qa/pilot","commit":"0000000000000000000000000000000000000000000000000000000000000000"}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_file", br#"{"repository":"qa/pilot","head":"0000000000000000000000000000000000000000000000000000000000000000","path":"file","max_bytes":32}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_job", br#"{"repository":"qa/pilot","head":"0000000000000000000000000000000000000000000000000000000000000000","run":1,"job":1,"attempt":1,"max_bytes":32}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_protection", br#"{"repository":"qa/pilot","branch":"main"}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_settings", br#"{"repository":"qa/pilot"}"#, br#"{"repository":"qa/pilot","kind":"wrong"}"#),
        (b"read_collaborators", br#"{"repository":"qa/pilot","page":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"read_permission", br#"{"repository":"qa/pilot","user":1}"#, br#"{"repository":"qa/pilot","page":0}"#),
        (b"effect_forge", br#"{"repository":{"forge":1,"repository":2},"resource":{"kind":"repository"},"write":{"kind":"close","number":3}}"#, br#"{"repository":{"forge":1,"repository":2},"resource":{},"write":{"kind":"close","number":3}}"#),
        (b"subscribe_forge", br#"{"topic":{"repository":{"forge":1,"repository":2},"kind":"pull","number":3}}"#, br#"{"topic":{}}"#),
        (b"change", br#"{"words":"edit README"}"#, br#"{"words":""}"#),
        (b"decided", br#"{"question":"Q","options":["A","B"],"took":0,"why":"W","kind":"retry"}"#, br"{}"),
        (b"goal", br#"{"words":"plan"}"#, br"{}"),
        (b"delegate", br#"{"batch":[]}"#, br#"{"batch":{}}"#),
        (b"amend", br#"{"target":3,"revision":1,"amendment":{"reason":"x"}}"#, br#"{"target":3,"amendment":{}}"#),
        (b"message", br#"{"target":3,"form":"words","words":"x"}"#, br#"{"target":3,"form":"unknown","words":"x"}"#),
        (b"cancel", br#"{"target":3,"reason":"x"}"#, br#"{"target":3}"#),
        (b"release", br#"{"target":3,"revision":1}"#, br#"{"target":0}"#),
        (b"introduce", br#"{"left":3,"right":4}"#, br#"{"left":3}"#),
        (b"unsubscribe", br#"{"subscription":3}"#, br#"{"subscription":"x"}"#),
        (b"withdraw", br#"{"proposal":3}"#, br#"{"proposal":0}"#),
        (b"decide_escalation", br#"{"task":3,"revision":1,"decision":"release"}"#, br#"{"task":3,"revision":1,"decision":"unknown"}"#),
        (b"decide", br#"{"proposer":3,"proposal":1,"decision":"accept"}"#, br#"{"proposer":3,"proposal":1,"decision":"unknown"}"#),
        (b"subscribe", br#"{"kind":"task","target":3}"#, br#"{"kind":"task"}"#),
        (b"propose", br#"{"action":"release","task":3,"hold":12,"revision":1,"reason":"x"}"#, br#"{"action":"release","reason":"x"}"#),
        (b"note", br#"{"scope":{"kind":"project","project":1},"description":"slow start","body":"warm it first"}"#, br#"{"scope":{"kind":"project"},"description":"slow start","body":"warm it first"}"#),
        (b"read_docs", br#"{"topic":"box"}"#, br#"{"topic":3}"#),
        (b"recall", br#"{"kind":"name","name":1}"#, br#"{"kind":"name","name":0}"#),
    ]
}

fn note_input(members: &str) -> Box<[u8]> {
    format!(r#"{{"scope":{{"kind":"project","project":7}},"description":"draft","body":"next"{members}}}"#)
        .into_bytes()
        .into_boxed_slice()
}

#[test]
fn a_line_range_decodes_to_note_lines() {
    for join in [false, true] {
        let input =
            note_input(&format!(r#", "name":20,"recalled":7,"first":3,"remove":2,"references":[9],"join":{join}"#));
        let decoded = decided(b"note", &input).expect("range decodes");
        let engine::Tool::NoteLines { entry, recalled, range } = decoded.tool else { panic!("range tool") };
        assert_eq!(recalled, 7);
        assert_eq!(range, jig_domain_notes::LineRange { first: 3, remove: 2, join });
        assert_eq!(entry.name, 20);
        assert_eq!(entry.scope, jig_domain_notes::Scope::Project { project: 7 });
        assert_eq!(&*entry.description, b"draft");
        assert_eq!(&*entry.body, b"next");
        assert_eq!(entry.references.as_slice(), &[9]);
    }
}

#[test]
fn a_line_range_needs_its_revision() {
    for (members, field) in [
        (r#", "name":20,"first":3,"remove":0"#, b"recalled".as_slice()),
        (r#", "recalled":1,"first":3,"remove":0"#, b"name".as_slice()),
    ] {
        assert_eq!(decided(b"note", &note_input(members)), Err(Problem::Range(field)));
    }
}

#[test]
fn first_and_remove_come_together() {
    for (members, field) in [
        (r#", "name":20,"recalled":1,"first":3"#, b"remove".as_slice()),
        (r#", "name":20,"recalled":1,"remove":0"#, b"first".as_slice()),
        (r#", "name":20,"recalled":1,"join":true"#, b"first".as_slice()),
    ] {
        assert_eq!(decided(b"note", &note_input(members)), Err(Problem::Range(field)));
    }
}

#[test]
fn first_zero_is_out_of_range() {
    assert_eq!(
        decided(b"note", &note_input(r#", "name":20,"recalled":1,"first":0,"remove":0"#)),
        Err(Problem::Range(b"first"))
    );
}

#[test]
fn a_note_without_first_decodes_as_today() {
    for (members, name, revision) in [("", 0, None), (r#", "name":20,"recalled":7"#, 20, Some(7))] {
        let call = decided(b"note", &note_input(members)).expect("whole note decodes");
        let engine::Tool::Note { entry, recalled } = call.tool else { panic!("whole-note tool") };
        assert_eq!(entry.name, name);
        assert_eq!(recalled, revision);
        assert_eq!(&*entry.body, b"next");
    }
}

#[test]
fn every_declared_tool_decodes_its_minimum_shape_and_rejects_a_wrong_shape() {
    assert_eq!(
        decided(b"see", br#"{"address":" settings charter 3 "}"#).unwrap().tool,
        engine::Tool::See { line: b"settings charter 3".as_slice().into(), from: 0 }
    );
    for (length, valid) in [(0, false), (512, true), (513, false)] {
        let input = format!(r#"{{"address":"{}"}}"#, "x".repeat(length));
        match decoded(b"see", input.as_bytes()).unwrap() {
            crate::Decoded::Call(_) => assert!(valid, "address {length}"),
            crate::Decoded::Answered(answer) => {
                assert!(!valid, "address {length}");
                assert!(answer.error());
                assert!(skein_lib::bytes::find(answer.text(), b"512").is_some());
            }
        }
    }
    for (action, expected) in [
        ("write", engine::DocumentAction::Write),
        ("append", engine::DocumentAction::Append),
        ("replace", engine::DocumentAction::Replace),
    ] {
        let input = format!(r###"{{"action":"{action}","body":"## Section"}}"###);
        assert_eq!(
            decided(b"document", input.as_bytes()).unwrap().tool,
            engine::Tool::Document { action: expected, body: b"## Section".as_slice().into() }
        );
    }
    for (input, expected) in [
        (
            br#"{"action":"add","text":"Brief"}"#.as_slice(),
            engine::RememberAction::Add { text: b"Brief".as_slice().into() },
        ),
        (
            br#"{"action":"change","line":1,"text":"Brief"}"#,
            engine::RememberAction::Change { line: 1, text: b"Brief".as_slice().into() },
        ),
        (br#"{"action":"drop","line":4294967295}"#, engine::RememberAction::Drop { line: u32::MAX }),
    ] {
        assert_eq!(decided(b"remember", input).unwrap().tool, engine::Tool::Remember { action: expected });
    }
    for (input, expected) in [
        (br"{}".as_slice(), Problem::Missing(b"action")),
        (br#"{"action":1}"#, Problem::Type(b"action")),
        (br#"{"action":"unknown"}"#, Problem::Range(b"action")),
        (br#"{"action":"add"}"#, Problem::Missing(b"text")),
        (br#"{"action":"add","text":1}"#, Problem::Type(b"text")),
        (br#"{"action":"add","text":"Brief","line":1}"#, Problem::Range(b"action")),
        (br#"{"action":"change","text":"Brief"}"#, Problem::Missing(b"line")),
        (br#"{"action":"change","line":1}"#, Problem::Missing(b"text")),
        (br#"{"action":"change","line":1,"text":1}"#, Problem::Type(b"text")),
        (br#"{"action":"drop"}"#, Problem::Missing(b"line")),
        (br#"{"action":"drop","line":0}"#, Problem::Range(b"line")),
        (br#"{"action":"drop","line":-1}"#, Problem::Range(b"line")),
        (br#"{"action":"drop","line":1.5}"#, Problem::Range(b"line")),
        (br#"{"action":"drop","line":4294967296}"#, Problem::Range(b"line")),
        (br#"{"action":"drop","line":"1"}"#, Problem::Type(b"line")),
        (br#"{"action":"drop","line":1,"text":"Brief"}"#, Problem::Range(b"action")),
        (br#"{"action":"add","text":"Brief","other":1}"#, Problem::Range(b"remember")),
    ] {
        assert_eq!(decided(b"remember", input), Err(expected), "{input:?}");
    }
    let cases = tool_cases();
    let declared = tools(Duration::from_secs(10));
    assert_eq!(declared.len(), cases.len());
    for tool in &declared {
        crate::json::parse(&tool.schema).expect("declared JSON schema");
        assert!(tool.description.len() <= 8_192, "description: {:?}", tool.name);
        assert!(tool.schema.len() <= 65_536, "schema: {:?}", tool.name);
    }
    for (name, valid, invalid) in cases {
        let mut found = false;
        for tool in &declared {
            if tool.name.as_ref() == *name {
                found = true;
            }
        }
        assert!(found, "tool declared: {name:?}");
        assert!(decoded(name, valid).is_ok(), "valid input for {name:?}");
        assert!(decoded(name, invalid).is_err(), "invalid input for {name:?}");
        if name.starts_with(b"read_") && *name != b"read_docs" && *name != b"read_task" {
            let start = skein_lib::bytes::find(valid, b"qa/pilot").unwrap();
            let end = start.checked_add(b"qa/pilot".len()).unwrap();
            let mut input = skein_lib::Writer::new(valid.len().checked_sub(b"qa/pilot".len()).unwrap());
            input.put(valid.get(..start).unwrap()).unwrap();
            input.put(valid.get(end..).unwrap()).unwrap();
            let engine::Tool::ReadNamed { repository, .. } = decided(name, &input.finish()).unwrap().tool else {
                panic!("forge read");
            };
            assert_eq!(repository, None, "empty name for {name:?}");
        }
    }
}

#[test]
fn delegate_tracking_is_optional_and_requires_a_priority_object() {
    let member = r#""executor":{"kind":"agent","charter":1},"spec":{"words":"x"},"contract":{"kind":"report","words":128},"authority":{"tools":1,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":40},"notes":0}"#;
    for (field, expected) in [("", None), (r#", "tracked":{"priority":4294967295}"#, Some(u32::MAX))] {
        let input = format!("{{\"batch\":[{{{member}{field}}}]}}");
        let engine::Tool::Delegate { batch } = decided(b"delegate", input.as_bytes()).expect("delegate").tool else {
            panic!("delegate tool")
        };
        assert_eq!(batch[0].tracked, expected);
        assert_eq!(batch[0].label, 0);
    }
    for (tracked, problem) in [
        ("8", Problem::Type(b"tracked")),
        (r#""untracked""#, Problem::Type(b"tracked")),
        (r#"{"priority":"8"}"#, Problem::Type(b"priority")),
        (r#"{"priority":4294967296}"#, Problem::Range(b"priority")),
        ("{}", Problem::Missing(b"priority")),
    ] {
        let input = format!("{{\"batch\":[{{{member},\"tracked\":{tracked}}}]}}");
        assert_eq!(decided(b"delegate", input.as_bytes()), Err(problem), "{tracked}");
    }
}

#[test]
fn amendment_parameter_omission_is_distinct_from_replacement() {
    for (parameters, keep, expected) in [
        ("", true, 0),
        (r#", "parameters":[]"#, false, 0),
        (r#", "parameters":[{"kind":"number","name":14,"value":1}]"#, false, 1),
    ] {
        let input = format!(
            "{{\"target\":3,\"revision\":1,\"amendment\":{{\"spec\":{{\"words\":\"new\"{parameters}}},\"reason\":\"scope\"}}}}"
        );
        let engine::Tool::Amend { amendment, keep_parameters, .. } =
            decided(b"amend", input.as_bytes()).expect("amend").tool
        else {
            panic!("amend tool")
        };
        assert_eq!(keep_parameters, keep, "{parameters}");
        let spec = amendment.spec.expect("amended spec");
        assert_eq!(spec.parameters.len(), expected);
        if expected == 1 {
            assert_eq!(spec.parameters.as_ref(), &[tasks::Parameter::Number { name: 14, value: 1 }]);
        }
    }
}

#[test]
fn only_a_priority_amendment_can_omit_its_revision() {
    let engine::Tool::Amend { keep_parameters: false, target, revision, amendment } =
        decided(b"amend", br#"{"target":3,"amendment":{"reason":"x","tracked":{"priority":8}}}"#)
            .expect("priority without revision")
            .tool
    else {
        panic!("amend tool")
    };
    assert_eq!((target, revision), (3, 0));
    assert_eq!(amendment.tracked, Some(tasks::Tracking::Tracked { priority: 8 }));
    for other in [
        r#", "spec":{"words":"x"}"#,
        r#", "wake":{"words":{"kind":"never"},"notices":{"kind":"never"},"news":{"kind":"never"},"results":"never","questions":false,"answers":false,"timers":false}"#,
        r#", "dependencies":[]"#,
        r#", "authority":{"tools":1,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":40},"notes":0}"#,
        r#", "instructions":"""#,
        r#", "llm":{"main":{"model":0}}"#,
    ] {
        let input =
            format!("{{\"target\":3,\"amendment\":{{\"reason\":\"x\",\"tracked\":{{\"priority\":8}}{other}}}}}");
        assert_eq!(decided(b"amend", input.as_bytes()), Err(Problem::Missing(b"revision")), "{other}");
    }
    assert_eq!(
        decided(b"amend", br#"{"target":3,"amendment":{"reason":"x","tracked":"untracked"}}"#),
        Err(Problem::Missing(b"revision"))
    );
    assert_eq!(
        decided(b"amend", br#"{"target":3,"revision":"0","amendment":{"reason":"x","tracked":{"priority":8}}}"#),
        Err(Problem::Type(b"revision"))
    );
}

#[test]
fn amendments_replace_tracking_and_instructions_including_empty_words() {
    for instructions in ["x", ""] {
        let input = format!(
            "{{\"target\":3,\"revision\":1,\"amendment\":{{\"reason\":\"x\",\"tracked\":\"untracked\",\"instructions\":\"{instructions}\"}}}}"
        );
        let engine::Tool::Amend { amendment, .. } = decided(b"amend", input.as_bytes()).expect("amend").tool else {
            panic!("amend tool")
        };
        assert_eq!(amendment.tracked, Some(tasks::Tracking::Untracked));
        assert_eq!(amendment.instructions.as_deref(), Some(instructions.as_bytes()));
    }
    for (tracked, problem) in [
        ("8", Problem::Type(b"tracked")),
        (r#""tracked""#, Problem::Range(b"tracked")),
        (r#"{"priority":"8"}"#, Problem::Type(b"priority")),
        (r#"{"priority":4294967296}"#, Problem::Range(b"priority")),
        ("{}", Problem::Missing(b"priority")),
    ] {
        let input = format!("{{\"target\":3,\"revision\":1,\"amendment\":{{\"reason\":\"x\",\"tracked\":{tracked}}}}}");
        assert_eq!(decided(b"amend", input.as_bytes()), Err(problem), "{tracked}");
    }
    assert_eq!(
        decided(b"amend", br#"{"target":3,"revision":1,"amendment":{"reason":"x","instructions":8}}"#),
        Err(Problem::Type(b"instructions"))
    );
}

#[test]
fn the_declared_deadline_example_decodes_the_whole_authority() {
    const EXAMPLE: &[u8] = br#"{"target":3,"revision":1,"amendment":{"reason":"Finish by Friday.","authority":{"tools":5,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":1000,"deadline":1793404800000000000},"notes":0}}}"#;
    for tool in tools(Duration::from_secs(10)) {
        if tool.name.as_ref() == b"amend" {
            let schema = crate::json::parse(&tool.schema).expect("amend schema");
            assert_eq!(
                schema.required(b"examples").expect("examples").array(b"examples").expect("examples array"),
                &[crate::json::parse(EXAMPLE).expect("example")]
            );
        }
    }
    let engine::Tool::Amend { keep_parameters: false, target, revision, amendment } =
        decided(b"amend", EXAMPLE).expect("deadline example").tool
    else {
        panic!("amend tool")
    };
    assert_eq!((target, revision), (3, 1));
    assert_eq!(
        amendment.authority,
        Some(tasks::Authority {
            tools: tasks::Tools(5),
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget {
                spend: 1000,
                deadline: Some(skein_lib::Wall::from_nanos(1_793_404_800_000_000_000))
            },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
            models: tasks::Models::NONE,
        })
    );
    assert_eq!(amendment.reason.as_ref(), b"Finish by Friday.");
    assert!(amendment.tracked.is_none() && amendment.instructions.is_none());
}

const DELEGATE_MEMBER: &str = r#""executor":{"kind":"agent","charter":1},"spec":{"words":"x"},"contract":{"kind":"report","words":128},"authority":{"tools":1,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":40},"notes":0}"#;

#[test]
fn a_delegate_deadline_in_seconds_is_a_span() {
    const PERSON_MEMBER: &str = r#""executor":{"kind":"person","person":1},"spec":{"words":"Which colour?"},"contract":{"kind":"report","words":4096},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":0},"notes":0}"#;
    for (member, field, deadline, executor, tools, spend, words) in [
        (DELEGATE_MEMBER, "", tasks::Deadline::Named, tasks::Executor::Agent { charter: 1 }, 1, 40, 128),
        (
            DELEGATE_MEMBER,
            r#", "deadline_seconds":14400"#,
            tasks::Deadline::Span(Duration::from_secs(14_400)),
            tasks::Executor::Agent { charter: 1 },
            1,
            40,
            128,
        ),
        (
            PERSON_MEMBER,
            "",
            tasks::Deadline::Named,
            tasks::Executor::Person(tasks::PersonAddress::Person(1)),
            0,
            0,
            4096,
        ),
    ] {
        let input = format!("{{\"batch\":[{{{member}{field}}}]}}");
        let engine::Tool::Delegate { batch } = decided(b"delegate", input.as_bytes()).expect("delegate deadline").tool
        else {
            panic!("delegate tool")
        };
        assert_eq!(batch[0].deadline, deadline);
        assert_eq!(batch[0].executor, executor);
        assert_eq!(batch[0].contract, tasks::Contract::Report { words });
        assert_eq!(
            batch[0].authority,
            tasks::Authority {
                tools: tasks::Tools(tools),
                models: tasks::Models::NONE,
                grants: Box::new([]),
                delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
                budget: tasks::Budget { spend, deadline: None },
                notes: tasks::Scopes(0),
                note_resources: Box::new([]),
            }
        );
    }
}

#[test]
fn a_delegate_names_one_deadline() {
    const MAX_SECONDS: u64 = u64::MAX / 1_000_000_000;
    for (seconds, wall, accepted) in
        [(1, true, false), (0, false, false), (MAX_SECONDS + 1, false, false), (MAX_SECONDS, false, true)]
    {
        let member = if wall {
            DELEGATE_MEMBER.replace(r#""spend":40"#, r#""spend":40,"deadline":20000000000"#)
        } else {
            DELEGATE_MEMBER.into()
        };
        let input = format!("{{\"batch\":[{{{member},\"deadline_seconds\":{seconds}}}]}}");
        let result = decided(b"delegate", input.as_bytes());
        if accepted {
            let engine::Tool::Delegate { batch } = result.expect("largest whole second").tool else {
                panic!("delegate tool")
            };
            assert_eq!(batch[0].deadline, tasks::Deadline::Span(Duration::from_secs(MAX_SECONDS)));
        } else {
            assert_eq!(result, Err(Problem::Range(b"deadline_seconds")), "seconds={seconds}, wall={wall}");
        }
    }
}

#[test]
fn proposed_amendments_share_controls_but_still_require_a_revision() {
    let engine::Tool::Propose { action: engine::ProposedAction::Amend { revision, amendment, .. }, .. } =
        decided(b"propose", br#"{"action":"amend","task":3,"revision":1,"amendment":{"reason":"x","tracked":{"priority":8},"instructions":""},"reason":"x"}"#)
            .expect("proposed amendment").tool
    else { panic!("proposed amendment") };
    assert_eq!(revision, 1);
    assert_eq!(amendment.tracked, Some(tasks::Tracking::Tracked { priority: 8 }));
    assert_eq!(amendment.instructions.as_deref(), Some(b"".as_slice()));
    assert_eq!(
        decided(
            b"propose",
            br#"{"action":"amend","task":3,"amendment":{"reason":"x","tracked":{"priority":8}},"reason":"x"}"#
        ),
        Err(Problem::Missing(b"revision"))
    );
}

#[test]
fn an_effect_proposal_decodes_its_connector_payload_and_rejects_an_unknown_resource() {
    assert_eq!(
        decided(b"propose", br#"{"action":"effect","repository":{"forge":1,"repository":2},"resource":{"kind":"repository"},"write":{"kind":"close","number":3},"reason":"x"}"#),
        Ok(engine::Call {
            completion: 2, position: 1,
            tool: engine::Tool::Propose {
                action: engine::ProposedAction::Effect {
                    repository: skein_forge::api::Repository { forge: 1, repository: 2 },
                    resource: temper_engine_domain_forge::What::Repository,
                    write: Box::new(skein_forge::api::Write::Close { number: 3 }),
                },
                reason: b"x".as_slice().into(), as_holder: false,
            },
        })
    );
    assert_eq!(decided(b"propose", br#"{"action":"effect","repository":{"forge":1,"repository":2},"resource":{},"write":{"kind":"close","number":3},"reason":"x"}"#), Err(Problem::Missing(b"kind")));
}

fn spend(units: u64) -> run::Spend {
    run::Spend {
        turns: 1,
        input: Some(2),
        output: Some(3),
        cache_read: Some(0),
        cache_write: Some(0),
        reasoning: None,
        units,
    }
}

fn change_answer() -> run::Answer {
    run::Answer::Accepted {
        outcome: run::outcome::Declared::Change(run::outcome::Change {
            fields: Box::new([
                run::outcome::Field { name: b"title".as_slice().into(), value: b"Fix".as_slice().into() },
                run::outcome::Field { name: b"body".as_slice().into(), value: b"Details".as_slice().into() },
            ]),
        }),
        spent: spend(9),
        turns: 1,
    }
}

#[test]
fn smith_terminals_preserve_charge_and_the_root_result_contract() {
    assert_eq!(
        result(run::Answer::Parked { spent: spend(5), turns: 1 }, None),
        Ok(crate::Terminal { cumulative: 5, turns: 1, end: tasks::End::Parked })
    );
    assert_eq!(
        result(
            run::Answer::Accepted {
                outcome: run::outcome::Declared::Report(run::outcome::Report {
                    text: b"done".as_slice().into(),
                    fields: Box::new([]),
                }),
                spent: spend(7),
                turns: 1,
            },
            None,
        ),
        Ok(crate::Terminal {
            cumulative: 7,
            turns: 1,
            end: tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
                cancel_delegates: false,
            },
        })
    );
    assert_eq!(result(change_answer(), None), Err(Problem::Missing(b"resource")));
    assert_eq!(
        result(change_answer(), Some(ChangeResource { connector: 1, kind: 2, resource: 42 })),
        Ok(crate::Terminal {
            cumulative: 9,
            turns: 1,
            end: tasks::End::Finished {
                result: tasks::TaskResult::Change {
                    title: jig_domain_tasks::ChangeTitle::new(b"Fix".as_slice().into()),
                    connector: 1,
                    kind: 2,
                    resource: 42,
                    words: b"Details".as_slice().into(),
                },
                cancel_delegates: false,
            },
        })
    );
}

#[test]
fn root_call_answers_become_bounded_smith_text_or_error() {
    use skein_forge::api;

    let done = answer(&temper_engine_domain::CallAnswer::Controlled);
    assert_eq!(done.text(), b"Done");
    assert!(!done.error());
    let denied = answer(&temper_engine_domain::CallAnswer::Unavailable);
    assert!(denied.error());
    assert!(!denied.text().is_empty());

    for (value, phrase) in [
        (api::Answer::Runs { runs: Box::new([]), more: false }, "answer of kind Runs"),
        (
            api::Answer::Run(api::Run {
                id: 1,
                number: 1,
                workflow: Box::new([]),
                commit: SHA1_HEAD,
                event: api::Event::Dispatch,
                actor: 1,
                state: api::RunState::Completed,
                conclusion: Some(api::Conclusion::Success),
                url: Box::new([]),
                created: skein_lib::Time::ZERO,
                jobs: Box::new([]),
            }),
            "answer of kind Run",
        ),
        (api::Answer::Commits { commits: Box::new([]), more: false }, "answer of kind Commits"),
        (api::Answer::Tree { entries: Box::new([]), more: false }, "answer of kind Tree"),
        (api::Answer::Queued, "answer of kind Queued"),
    ] {
        let read = engine::CallAnswer::ForgeRead(Box::new(Ok(value)));
        assert_eq!(crate::unasked_kind(&read), Some(phrase));
        for (rendered, tool) in [
            (answer(&read), "the read"),
            (crate::answer_for_tool(&read, b"read_file", 4096, crate::ForgeReadBounds::STOCK), "read_file"),
            (crate::answer_for_tool(&read, b"read_compare", 4096, crate::ForgeReadBounds::STOCK), "read_compare"),
            (crate::answer_bounded(&read, 4096), "the read"),
            (
                crate::answer_for_task(&read, b"read_commit", 4096, CALL_TASK, crate::ForgeReadBounds::STOCK),
                "read_commit",
            ),
        ] {
            let expected = format!(
                "Forge read refused: {tool} got an {phrase}, which it never asks for. Nothing was read; read again."
            );
            assert!(rendered.error(), "{tool}: {phrase}");
            assert_eq!(rendered.text(), expected.as_bytes());
        }
        for cap in [0, 32] {
            let rendered = crate::answer_for_tool(&read, b"read_compare", cap, crate::ForgeReadBounds::STOCK);
            let expected = format!(
                "Forge read refused: read_compare got an {phrase}, which it never asks for. Nothing was read; read again."
            );
            assert!(rendered.error());
            assert_eq!(rendered.text(), &expected.as_bytes()[..usize::try_from(cap).unwrap()]);
        }
    }
    let read = engine::CallAnswer::ForgeRead(Box::new(Ok(api::Answer::Commit(SHA1_HEAD))));
    assert_eq!(crate::unasked_kind(&read), None);
}

const SHA1_HEAD: [u8; 32] = [
    0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const COMMIT_SPELLINGS: &[([u8; 32], &[u8])] = &[
    (SHA1_HEAD, b"abcdef0123456789abcdef0123456789abcdef01"),
    ([0xab; 32], b"abababababababababababababababababababababababababababababababab"),
];

fn pull_answer(state: skein_forge::api::State, head: [u8; 32], merged: Option<[u8; 32]>) -> skein_forge::api::Answer {
    use skein_forge::api;
    api::Answer::Pull(api::Pull {
        reviewers: Box::new([]),
        number: 1,
        state,
        head: b"work".as_slice().into(),
        base: b"main".as_slice().into(),
        commit: head,
        base_commit: None,
        merged,
        mergeable: true,
        ci: api::Ci::Passed,
        queued: None,
        dequeued: None,
        size: None,
    })
}

#[test]
fn permission_answers_name_the_forge_user_and_available_login() {
    use skein_forge::api;
    for (permission, label) in [
        (api::Permission::None, "No permission"),
        (api::Permission::Read, "Read"),
        (api::Permission::Write, "Write"),
        (api::Permission::Admin, "Admin"),
    ] {
        for (login, suffix) in [(b"ada".as_slice(), ", login ada"), (b"".as_slice(), "")] {
            let read = api::Answer::Permission(api::UserPermission { user: 3, login: login.into(), permission });
            let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
            assert!(!rendered.error());
            assert_eq!(rendered.text(), format!("{label}; forge user 3{suffix}").as_bytes());
        }
    }
}

#[test]
fn forge_answers_print_commits_at_the_identifier_width() {
    use skein_forge::api;
    for (commit, digits) in COMMIT_SPELLINGS {
        for (read, prefix, suffix) in [
            (api::Answer::Commit(*commit), b"".as_slice(), b"".as_slice()),
            (api::Answer::Merged(*commit), b"".as_slice(), b"".as_slice()),
            (
                api::Answer::File { head: *commit, path: b"file".as_slice().into(), bytes: Box::new([]), truncated: false },
                b"file at ".as_slice(),
                b":\n".as_slice(),
            ),
            (pull_answer(api::State::Open, *commit, None), b"Pull 1 head work commit ".as_slice(), b" base main state open CI passed".as_slice()),
            (pull_answer(api::State::Closed, SHA1_HEAD, Some(*commit)), b"Pull 1 head work commit abcdef0123456789abcdef0123456789abcdef01 base main state closed merged commit ".as_slice(), b" CI passed".as_slice()),
        ] {
            let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
            assert!(!rendered.error());
            let mut expected = skein_lib::Writer::new(prefix.len() + digits.len() + suffix.len());
            expected.put(prefix).unwrap();
            expected.put(digits).unwrap();
            expected.put(suffix).unwrap();
            assert_eq!(rendered.text(), expected.finish().as_ref());
        }
        for made in [
            skein_forge_client::Made::Merged(*commit),
            skein_forge_client::Made::Updated(*commit),
            skein_forge_client::Made::Branch(*commit),
        ] {
            let rendered = answer(&engine::CallAnswer::ForgeEffect {
                entry: 1,
                outcome: Some(skein_forge_client::Outcome::Made { made, found: false }),
                deadline: skein_lib::Wall::EPOCH,
            });
            assert!(!rendered.error());
            assert_eq!(rendered.text().strip_prefix(b"Forge effect 1: made "), Some(*digits));
        }
    }
}

#[test]
fn pull_answers_name_open_closed_and_merged_state_and_the_merge_commit() {
    use skein_forge::api;
    for (state, merged, expected) in [
        (api::State::Open, None, b"Pull 1 head work commit abcdef0123456789abcdef0123456789abcdef01 base main state open CI passed".as_slice()),
        (api::State::Closed, None, b"Pull 1 head work commit abcdef0123456789abcdef0123456789abcdef01 base main state closed CI passed".as_slice()),
        (api::State::Closed, Some([0xab; 32]), b"Pull 1 head work commit abcdef0123456789abcdef0123456789abcdef01 base main state closed merged commit abababababababababababababababababababababababababababababababab CI passed".as_slice()),
    ] {
        let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(pull_answer(state, SHA1_HEAD, merged)))));
        assert!(!rendered.error());
        assert_eq!(rendered.text(), expected);
    }
}

#[test]
fn a_charter_resumes_when_its_policy_does_and_turns_are_kept() {
    let mut assignment = assignment();
    for (resume, turns, expected) in
        [(false, false, false), (false, true, false), (true, false, false), (true, true, true)]
    {
        assignment.run.policy.resume = resume;
        assignment.transcript = if turns { Box::new([b"turn".as_slice().into()]) } else { Box::new([]) };
        assert_eq!(crate::charter(&assignment).resume, expected, "resume={resume}, turns={turns}");
    }
}

#[test]
fn resolved_checks_and_statuses_name_the_api_job_attempt_in_every_state() {
    use skein_forge::api;
    for (check, state) in
        [(api::Check::Pending, "pending"), (api::Check::Passed, "passed"), (api::Check::Failed, "failed")]
    {
        for (head, digits) in COMMIT_SPELLINGS {
            let status = api::Status {
                at: skein_lib::Time::ZERO,
                context: b"build".as_slice().into(),
                check,
                description: b"CI result".as_slice().into(),
                url: b"https://forge.example/org/repo/actions/runs/3/jobs/0".as_slice().into(),
                job: Some(api::JobAttempt { head: *head, run: 7, job: 8, attempt: 2 }),
            };
            let status_line = format!(
                "build: {state} CI result https://forge.example/org/repo/actions/runs/3/jobs/0\nread_job head "
            );
            let attempt_line = b" run 7 job 8 attempt 2\n";
            for (read, prefix, suffix) in [
                (api::Answer::Checks(Box::new([status.clone()])), "", ""),
                (
                    api::Answer::Statuses { ci: api::Ci::Passed, statuses: Box::new([status]), more: true },
                    "CI passed\n",
                    "More statuses available.\n",
                ),
            ] {
                let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
                assert!(!rendered.error());
                let mut expected = skein_lib::Writer::new(
                    prefix.len() + status_line.len() + digits.len() + attempt_line.len() + suffix.len(),
                );
                expected.put(prefix.as_bytes()).unwrap();
                expected.put(status_line.as_bytes()).unwrap();
                expected.put(digits).unwrap();
                expected.put(attempt_line).unwrap();
                expected.put(suffix.as_bytes()).unwrap();
                assert_eq!(rendered.text(), expected.finish().as_ref());
            }
        }
    }
}

#[test]
fn unresolved_checks_and_statuses_say_no_job_attempt_was_resolved() {
    use skein_forge::api;
    let status = api::Status {
        at: skein_lib::Time::ZERO,
        context: b"external".as_slice().into(),
        check: api::Check::Pending,
        description: b"waiting".as_slice().into(),
        url: b"https://checks.example/build".as_slice().into(),
        job: None,
    };
    for read in [
        api::Answer::Checks(Box::new([status.clone()])),
        api::Answer::Statuses { ci: api::Ci::Pending, statuses: Box::new([status]), more: false },
    ] {
        let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
        assert!(!rendered.error());
        assert!(rendered.text().ends_with(b"external: pending waiting https://checks.example/build\nNo job attempt resolved from this check's link.\n"));
    }
}

#[test]
fn job_attempt_text_counts_against_the_host_answer_bound() {
    use skein_forge::api;
    for job in [None, Some(api::JobAttempt { head: SHA1_HEAD, run: u64::MAX, job: u64::MAX, attempt: u32::MAX })] {
        let mut status = api::Status {
            at: skein_lib::Time::ZERO,
            context: b"build".as_slice().into(),
            check: api::Check::Passed,
            description: Box::new([]),
            url: Box::new([]),
            job,
        };
        let read = api::Answer::Checks(Box::new([status.clone()]));
        let baseline = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
        assert!(!baseline.error());
        let room = run::HostAnswer::CAPACITY - baseline.text().len();
        for extra in [0, 1] {
            status.description = vec![b'x'; room + extra].into_boxed_slice();
            let read = api::Answer::Checks(Box::new([status.clone()]));
            let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
            if extra == 0 {
                assert!(!rendered.error());
                assert_eq!(rendered.text().len(), run::HostAnswer::CAPACITY);
                assert!(rendered.text().ends_with(b"\n"));
            } else {
                assert!(rendered.error());
                assert_eq!(rendered.text(), b"The recorded answer exceeded the host text limit; ask a narrower read.");
            }
        }
    }
}

#[test]
fn a_job_answer_names_its_failed_step_before_its_log() {
    use skein_forge::api;
    let attempt = api::JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 };
    let step = Some(api::Step { number: 1, name: Box::from(&b"build"[..]) });
    let read = api::Answer::Job { attempt, step, log: Box::from(&b"failing check build 3\n"[..]), truncated: true };
    let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
    assert_eq!(rendered.text(), b"Job 8 run 7, failed step 1 build:\n[log bytes cut]\nfailing check build 3\n");
    assert!(!rendered.error(), "A read answer is no error");
}

#[test]
fn a_job_answer_with_no_failed_step_says_none_failed() {
    use skein_forge::api;
    let attempt = api::JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 };
    let step = None;
    let read = api::Answer::Job { attempt, step, log: Box::from(&b"failing check build 3\n"[..]), truncated: false };
    let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
    assert_eq!(rendered.text(), b"Job 8 run 7, no step failed:\nfailing check build 3\n");
    assert!(!rendered.error(), "A read answer is no error");
}

#[test]
fn a_job_answer_with_an_empty_step_name_names_its_number_alone() {
    use skein_forge::api;
    let attempt = api::JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 };
    let step = Some(api::Step { number: 1, name: Box::new([]) });
    let read = api::Answer::Job { attempt, step, log: Box::from(&b"failing check build 3\n"[..]), truncated: false };
    let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
    assert_eq!(rendered.text(), b"Job 8 run 7, failed step 1:\nfailing check build 3\n");
    assert!(!rendered.error(), "A read answer is no error");
}

struct RevisionCall {
    name: &'static [u8],
    missing: &'static [u8],
    valid: &'static [u8],
    again: Option<bool>,
}
const REVISION_CALLS: &[RevisionCall] = &[
    RevisionCall {
        name: b"amend",
        missing: br#"{"target":23,"amendment":{"reason":"change"}}"#,
        valid: br#"{"target":23,"revision":0,"amendment":{"reason":"change"}}"#,
        again: None,
    },
    RevisionCall {
        name: b"release",
        missing: br#"{"target":23}"#,
        valid: br#"{"target":23,"revision":0}"#,
        again: None,
    },
    RevisionCall {
        name: b"release",
        missing: br#"{"target":23,"again":false}"#,
        valid: br#"{"target":23,"revision":0,"again":false}"#,
        again: Some(false),
    },
    RevisionCall {
        name: b"release",
        missing: br#"{"target":23,"again":true}"#,
        valid: br#"{"target":23,"revision":0,"again":true}"#,
        again: Some(true),
    },
];
#[test]
fn amend_and_release_calls_name_their_revision() {
    for row in REVISION_CALLS {
        assert_eq!(decided(row.name, row.missing), Err(Problem::Missing(b"revision")), "{:?}", row.name);
        let expected = if row.name == b"amend" {
            engine::Tool::Amend {
                keep_parameters: false,
                target: CALL_TASK,
                revision: 0,
                amendment: tasks::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    release: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: None,
                    llm: None,
                    reason: Box::new(*b"change"),
                },
            }
        } else {
            engine::Tool::Release { target: CALL_TASK, revision: 0, again: row.again }
        };
        let body = decided(row.name, row.valid).expect("revision-bearing tool");
        assert_eq!(body.tool, expected, "{:?}", row.name);
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "the answer table allocates boxed task-read cases at runtime")]
fn each_of_jigs_new_call_answers_renders_its_task_and_value() {
    let live = jig_domain_core::reads::run_task::TaskRead {
        task: 7,
        state: jig_domain_core::reads::run_task::TaskState::Active(tasks::Active::Due),
        spec_revision: 2,
        dependencies: Box::new([]),
        questions: Box::new([]),
        words: jig_domain_core::reads::TaskText {
            task: 7,
            field: jig_domain_core::reads::TaskTextField::Specification,
            from: 0,
            total: 5,
            next: None,
            bytes: b"hello".as_slice().into(),
        },
        verdict_code: None,
        result_pending: true,
        hold: None,
    };
    let answer_cases: &[(engine::CallAnswer, &[u8], bool)] = &[
        (
            engine::CallAnswer::TaskRead(Box::new(Ok(live))),
            b"Task 7, spec revision 2\nState: active\nDependencies:\nResult: none yet; it comes when the task ends. Its specification follows; continue it with field left out.\nWords from 0:\nhello",
            false,
        ),
        (
            engine::CallAnswer::TaskRead(Box::new(Err(jig_domain_tasks::Refusal::Unknown))),
            b"Task read refused: unknown: no such task, no result yet, or no text at that offset. A task has a result only once it has ended, and its requester hears it then as a result message; while it runs, read its specification (leave out field) for its state, holds and questions.",
            true,
        ),
        (
            engine::CallAnswer::TaskRead(Box::new(Err(jig_domain_tasks::Refusal::Reference))),
            b"Task read refused: no reference",
            true,
        ),
        (
            engine::CallAnswer::Moved {
                task: CALL_TASK,
                now: jig_domain_people::Moved::Hold { revision: 2, reason: jig_domain_people::HoldReason::Placed { hold: u64::MAX } },
            },
            b"Moved: task 23 is held at hold revision 2 now, because a placed hold holds it",
            true,
        ),
        (
            engine::CallAnswer::Moved {
                task: CALL_TASK,
                now: jig_domain_people::Moved::Hold { revision: 2, reason: jig_domain_people::HoldReason::Circle },
            },
            b"Moved: task 23 is held at hold revision 2 now, for a circle of waits",
            true,
        ),
        (
            engine::CallAnswer::Again { task: CALL_TASK, required: true },
            b"Refused: task 23 holds an effect whose outcome is uncertain; say again: true or false",
            true,
        ),
        (
            engine::CallAnswer::Again { task: CALL_TASK, required: false },
            b"Refused: task 23 holds no uncertain effect; leave again out",
            true,
        ),
        (
            engine::CallAnswer::PassedUp { task: CALL_TASK },
            b"Beyond your authority: task 23's escalation was passed up",
            false,
        ),
        (engine::CallAnswer::Revised { task: CALL_TASK, revision: 2 }, b"Done: task 23 is at spec revision 2", false),
        (
            engine::CallAnswer::Amended { task: 3, revision: 1, changed: engine::Changed { tracked: None, fields: 0 } },
            b"Done: task 3 is at spec revision 1",
            false,
        ),
        (
            engine::CallAnswer::Amended { task: 3, revision: 1, changed: engine::Changed {
                tracked: None, fields: engine::Changed::AUTHORITY | engine::Changed::INSTRUCTIONS,
            } },
            b"Done: task 3 is at spec revision 1; changed: authority, instructions",
            false,
        ),
        (
            engine::CallAnswer::Amended { task: 3, revision: 0, changed: engine::Changed {
                tracked: Some(tasks::Tracking::Tracked { priority: 8 }), fields: 0,
            } },
            b"Done: task 3 is at spec revision 0; changed: priority 8",
            false,
        ),
        (
            engine::CallAnswer::Amended { task: 3, revision: 2, changed: engine::Changed {
                tracked: Some(tasks::Tracking::Untracked), fields: engine::Changed::SPEC,
            } },
            b"Done: task 3 is at spec revision 2; changed: untracked, spec",
            false,
        ),
        (
            engine::CallAnswer::Amended { task: 3, revision: 2, changed: engine::Changed {
                tracked: Some(tasks::Tracking::Tracked { priority: u32::MAX }),
                fields: engine::Changed::SPEC | engine::Changed::WAKE | engine::Changed::DEPENDENCIES
                    | engine::Changed::AUTHORITY | engine::Changed::INSTRUCTIONS | engine::Changed::LLM,
            } },
            b"Done: task 3 is at spec revision 2; changed: priority 4294967295, spec, wake, dependencies, authority, instructions, llm",
            false,
        ),
        (
            engine::CallAnswer::Moved { task: CALL_TASK, now: jig_domain_people::Moved::Spec { revision: 2 } },
            b"Moved: task 23 is at spec revision 2 now",
            true,
        ),
        (
            engine::CallAnswer::Moved {
                task: CALL_TASK,
                now: jig_domain_people::Moved::Hold { revision: 2, reason: jig_domain_people::HoldReason::Stopped },
            },
            b"Moved: task 23 is held at hold revision 2 now, because it was stopped",
            true,
        ),
        (
            engine::CallAnswer::Moved { task: CALL_TASK, now: jig_domain_people::Moved::Person { revision: 2 } },
            b"Moved: task 23 is at person revision 2 now",
            true,
        ),
        (
            engine::CallAnswer::Moved { task: CALL_TASK, now: jig_domain_people::Moved::Note { revision: 2 } },
            b"Moved: task 23 is at note revision 2 now",
            true,
        ),
    ];
    for (value, words, error) in answer_cases {
        let rendered = answer(value);
        assert_eq!(rendered.text(), *words, "{value:?}");
        assert_eq!(rendered.error(), *error, "{value:?}");
    }
}

#[test]
fn a_job_request_is_not_a_turn() {
    assert_eq!(
        turn(
            smith::Request::Lower(smith::Lower::Job { owner: Token::new(1), act: smith::tools::JobAct::Hasten }),
            Box::default()
        )
        .err(),
        Some(Problem::Type(b"request"))
    );
}

#[test]
fn account_notices_are_not_turns_and_terminal_failure_classes_stay_distinct() {
    for notice in [
        smith::Request::Unauthorized { grant: smith::GrantName { account: 17, generation: 3 } },
        smith::Request::Limited { account: 17, reset: None },
        smith::Request::Limited { account: 17, reset: Some(Duration::ZERO) },
        smith::Request::Refused { account: 17, code: b"quota".as_slice().into(), reason: b"refused".as_slice().into() },
    ] {
        assert_eq!(turn(notice, Box::default()), Err(Problem::Type(b"request")));
    }
    for (failure, class) in [
        (run::RunFailure::Stale, tasks::Class::Transient),
        (run::RunFailure::Transcript(run::TranscriptRefusal::Malformed), tasks::Class::Transient),
        (
            run::RunFailure::Budget { exhausted: run::Exhausted::Turns, used: Some(2), bound: Some(2) },
            tasks::Class::Run,
        ),
        (run::RunFailure::Policy(run::Policy::Unfinished { nudges: 1, rejected: 1 }), tasks::Class::Run),
        (run::RunFailure::Model(run::Fault::Provider), tasks::Class::Run),
        (run::RunFailure::Cancelled, tasks::Class::Agent),
    ] {
        assert_eq!(
            result(run::Answer::Failed { failure, spent: spend(13), turns: 2 }, None),
            Ok(crate::Terminal { cumulative: 13, turns: 2, end: tasks::End::Failed(class) })
        );
    }
}

#[test]
fn queued_names_the_forge_outcome() {
    let rendered = answer(&engine::CallAnswer::ForgeEffect {
        entry: 1,
        deadline: skein_lib::Wall::EPOCH,
        outcome: Some(skein_forge_client::Outcome::Queued),
    });
    assert_eq!(rendered.text(), b"Forge effect 1: queued");
    assert!(!rendered.error());
}

#[test]
fn not_installed_names_the_forge_refusal() {
    let rendered = answer(&engine::CallAnswer::ForgeEffectRefused(skein_forge::api::Error::NotInstalled));
    assert_eq!(rendered.text(), b"Forge effect refused: not installed");
    assert!(rendered.error());
}

#[test]
fn a_delegate_keeps_its_llm_preference() {
    let call = decided(
        b"delegate",
        br#"{"batch":[{
        "executor":{"kind":"agent","charter":1},"spec":{"words":"work"},
        "contract":{"kind":"report","words":32},
        "authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},
            "budget":{"spend":0},"notes":0},
        "llm":{"main":{"model":2,"effort":1},"subagents":[{"model":3}],"account":43}
    }]}"#,
    )
    .expect("valid delegate");
    let engine::Tool::Delegate { batch } = call.tool else { panic!("delegate") };
    assert_eq!(
        batch[0].llm,
        Some(tasks::LlmChoice {
            later: None,
            account: Some(43),
            main: tasks::ModelEffort { model: 2, effort: Some(1) },
            subagents: Box::new([tasks::ModelEffort { model: 3, effort: None }]),
        })
    );
}

#[test]
fn an_amendment_keeps_its_llm_preference() {
    let call = decided(
        b"amend",
        br#"{"target":23,"revision":7,"amendment":{
        "reason":"change payer","llm":{"main":{"model":2,"effort":1},"subagents":[{"model":3}],"account":43}
    }}"#,
    )
    .expect("valid amendment");
    let engine::Tool::Amend { amendment, .. } = call.tool else { panic!("amendment") };
    assert_eq!(
        amendment.llm,
        Some(tasks::LlmChoice {
            later: None,
            account: Some(43),
            main: tasks::ModelEffort { model: 2, effort: Some(1) },
            subagents: Box::new([tasks::ModelEffort { model: 3, effort: None }]),
        })
    );
}

fn refusal_words(why: tasks::Refusal, blocked: Option<Box<[u64]>>, expected: &[u8]) {
    let value =
        engine::CallAnswer::ControlRefused(tasks::Problem { task: Some(99), capacity: None, why, blocked_by: blocked });
    let rendered = answer(&value);
    assert_eq!(rendered.text(), expected);
    assert!(rendered.error());
}

fn busy_problem(action: tasks::CapacityAction, cause: tasks::CapacityCause) -> tasks::Problem {
    tasks::Problem::capacity(
        Some(2),
        tasks::Refusal::Busy,
        tasks::Capacity { project: Some(1), by: tasks::Party::Task(2), action, cause },
    )
}

#[test]
fn a_proposal_refused_for_its_own_full_inbox_names_it_its_bounds_and_the_way_on() {
    let value = engine::CallAnswer::ProposalRefused(busy_problem(
        tasks::CapacityAction::Proposal { number: 23 },
        tasks::CapacityCause::Inbox { task: 2, messages: 8, bytes: 12288 },
    ));
    let rendered = answer(&value);
    assert_eq!(rendered.text(), b"Refused: task 2 busy: its inbox is full (at most 8 messages and 12288 bytes, counting the room it keeps for the answers to its open questions, its delegates' results and a proposal's decision); room returns as those arrive and are read: retry then, or ask fewer at once");
    assert!(rendered.error());
}

#[test]
fn a_message_refused_for_another_tasks_full_inbox_names_that_task() {
    let value = engine::CallAnswer::MessageRefused(busy_problem(
        tasks::CapacityAction::Message { task: 5, number: 23 },
        tasks::CapacityCause::Inbox { task: 5, messages: 8, bytes: 12288 },
    ));
    let rendered = answer(&value);
    assert_eq!(rendered.text(), b"Refused: task 2 busy: task 5's inbox is full (at most 8 messages and 12288 bytes, counting the room it keeps for the answers to its open questions, its delegates' results and a proposal's decision); room returns as those arrive and are read: retry then, or ask fewer at once");
    assert!(rendered.error());
}

#[test]
fn a_second_proposal_is_refused_naming_the_one_pending() {
    let value = engine::CallAnswer::ProposalRefused(busy_problem(
        tasks::CapacityAction::Proposal { number: 23 },
        tasks::CapacityCause::Proposals { limit: 1 },
    ));
    let rendered = answer(&value);
    assert_eq!(
        rendered.text(),
        b"Refused: task 2 busy: it has a proposal pending (at most 1 at once): wait for its decision"
    );
    assert!(rendered.error());
}

#[test]
fn a_subscription_refused_for_a_full_room_names_its_bound() {
    let value = engine::CallAnswer::SubscriptionRefused(busy_problem(
        tasks::CapacityAction::Subscription { number: 23 },
        tasks::CapacityCause::Subscriptions { limit: 4 },
    ));
    let rendered = answer(&value);
    assert_eq!(rendered.text(), b"Refused: task 2 busy: it holds its most subscriptions (at most 4, counting the topics the engine keeps for a goal); unsubscribe one it made to make room");
    assert!(rendered.error());
}

#[test]
fn a_busy_without_a_capacity_keeps_its_words() {
    let value = engine::CallAnswer::ControlRefused(tasks::Problem::new(Some(2), tasks::Refusal::Busy));
    let rendered = answer(&value);
    assert_eq!(rendered.text(), b"Refused: task 2 busy");
    assert!(rendered.error());
}

#[test]
fn a_circle_names_its_tasks() {
    refusal_words(
        tasks::Refusal::Circle { cut: false },
        Some(Box::new([3, 5, 7])),
        b"Refused: these tasks would wait for one another: T3, T5, T7",
    );
}

#[test]
fn a_cut_circle_says_its_lists_are_incomplete() {
    refusal_words(
        tasks::Refusal::Circle { cut: true },
        Some(Box::new([3, 5, 7])),
        b"Refused: these tasks would wait for one another: T3, T5, T7; some task lists are incomplete",
    );
}

#[test]
fn an_unreachable_dependency_refusal_names_the_dependency_to_the_agent() {
    // Replaces the real-loop test `unreachable_dependency_refusal_names_the_task_to_the_agent`.
    refusal_words(
        tasks::Refusal::Dependencies,
        Some(Box::new([4])),
        b"Refused: task 99 dependency T4 is outside your reach",
    );
}

#[test]
fn a_failed_dependency_is_named() {
    refusal_words(
        tasks::Refusal::DependencyEnded { how: tasks::Status::Failed },
        Some(Box::new([4])),
        b"Refused: dependency T4 failed",
    );
}

#[test]
fn a_cancelled_dependency_is_named() {
    refusal_words(
        tasks::Refusal::DependencyEnded { how: tasks::Status::Cancelled },
        Some(Box::new([4])),
        b"Refused: dependency T4 was cancelled",
    );
}

#[test]
fn a_done_dependency_is_named() {
    refusal_words(
        tasks::Refusal::DependencyEnded { how: tasks::Status::Done },
        Some(Box::new([4])),
        b"Refused: T4 is done",
    );
}

#[test]
fn kept_names_each_dependency() {
    refusal_words(
        tasks::Refusal::Kept,
        Some(Box::new([4, 6])),
        b"Refused: kept until T4 is done, kept until T6 is done",
    );
}

#[test]
fn a_refused_effect_names_its_pending_dependencies() {
    let value = engine::CallAnswer::EffectRefused(tasks::Problem {
        task: Some(99),
        capacity: None,
        why: tasks::Refusal::Kept,
        blocked_by: Some(Box::new([4, 6])),
    });
    let rendered = answer(&value);
    assert_eq!(rendered.text(), b"Refused: kept until T4 is done, kept until T6 is done");
    assert!(rendered.error());
}

#[test]
fn dependency_refusals_without_tasks_are_plain_lines() {
    let lists: [Option<Box<[u64]>>; 2] = [None, Some(Box::new([]))];
    for blocked in lists {
        for (why, expected) in [
            (tasks::Refusal::Dependencies, b"Refused: task 99 dependencies".as_slice()),
            (tasks::Refusal::Circle { cut: false }, b"Refused: these tasks would wait for one another".as_slice()),
            (tasks::Refusal::Circle { cut: true }, b"Refused: these tasks would wait for one another"),
            (tasks::Refusal::DependencyEnded { how: tasks::Status::Failed }, b"Refused: a dependency has ended"),
            (tasks::Refusal::Kept, b"Refused: kept by its dependencies"),
        ] {
            refusal_words(why, blocked.clone(), expected);
        }
    }
}

#[test]
fn change_decodes_words_and_an_optional_repository() {
    // Measured CPU: 0.006 s, including the reference input and the vocabulary overflow.
    let room = crate::json::parse(include_bytes!("../../../docs/examples/reference/room.json")).unwrap();
    let payload = usize::try_from(
        room.required(b"agent").unwrap().required(b"tool_payload").unwrap().number(b"tool_payload").unwrap(),
    )
    .unwrap();
    let words = "s".repeat(payload - br#"{"words":""}"#.len());
    let input = format!(r#"{{"words":"{words}"}}"#);
    assert_eq!(input.len(), payload);
    let engine::Tool::Change { changes } = decided(b"change", input.as_bytes()).expect("reference spec").tool else {
        panic!("change")
    };
    assert_eq!(changes.as_ref(), &[engine::ChangeCall::words(words.into_bytes().into(), None)]);
    let too_large = format!(r#"{{"words":"{}"}}"#, "s".repeat(run::HostInput::CAPACITY + 1 - br#"{"words":""}"#.len()));
    assert_eq!(crate::json::parse(too_large.as_bytes()).err(), Some(Problem::TooLarge));
    for input in [br#"{"words":"edit README"}"#.as_slice(), br#"{"words":"edit README","repository":"work/hello"}"#] {
        let call = decided(b"change", input).expect("change is declared and decoded");
        assert_eq!(call.completion, 2);
        assert_eq!(call.position, 1);
        let engine::Tool::Change { changes } = call.tool else { panic!("change") };
        let repository = if input.contains(&b'/') { Some(b"work/hello".as_slice().into()) } else { None };
        assert_eq!(changes.as_ref(), &[engine::ChangeCall::words(b"edit README".as_slice().into(), repository)]);
    }
}

#[test]
fn change_decodes_a_plan_member_each() {
    for (landing, expected) in
        [("goal", engine::ChangeLanding::Goal), ("main", engine::ChangeLanding::Branch(b"main".as_slice().into()))]
    {
        let input = format!(r#"{{"words":"a","name":"x","landing":"{landing}","after":[4,"w"]}}"#);
        let engine::Tool::Change { changes } = decided(b"change", input.as_bytes()).expect("plan change").tool else {
            panic!("change")
        };
        assert_eq!(
            changes.as_ref(),
            &[engine::ChangeCall {
                words: b"a".as_slice().into(),
                repository: None,
                name: Some(b"x".as_slice().into()),
                landing: Some(expected),
                branch: None,
                merge: None,
                after: Box::new([engine::ChangeAfter::Task(4), engine::ChangeAfter::Name(b"w".as_slice().into())]),
                producer: None,
                llm: engine::ChangeLlm::default(),
                questions: None,
                repairs: None,
            }]
        );
    }
    let name = "x".repeat(64);
    let input = format!(r#"{{"words":"a","name":"{name}","after":[]}}"#);
    let engine::Tool::Change { changes } = decided(b"change", input.as_bytes()).expect("64-byte name").tool else {
        panic!("change")
    };
    assert_eq!(changes[0].name.as_deref(), Some(name.as_bytes()));
    assert!(changes[0].after.is_empty());
}

#[test]
fn change_decodes_repairs_alone_and_in_changes_at_each_commit_width() {
    for digits in [40, 64] {
        let text = "ab".repeat(digits / 2);
        let mut commit = [0_u8; 32];
        commit[..digits / 2].fill(0xab);
        for (input, questions) in [
            (format!(r#"{{"words":"a","repairs":"{text}"}}"#), None),
            (format!(r#"{{"changes":[{{"words":"a","repairs":"{text}"}}]}}"#), None),
            (format!(r#"{{"words":"a","repairs":"{text}","questions":"up"}}"#), Some(engine::ChangeQuestions::Up)),
            (
                format!(r#"{{"changes":[{{"words":"a","repairs":"{text}","questions":"up"}}]}}"#),
                Some(engine::ChangeQuestions::Up),
            ),
        ] {
            let engine::Tool::Change { changes } = decided(b"change", input.as_bytes()).expect("repair change").tool
            else {
                panic!("change")
            };
            assert_eq!(
                changes.as_ref(),
                &[engine::ChangeCall {
                    repairs: Some(commit),
                    questions,
                    ..engine::ChangeCall::words(b"a".as_slice().into(), None)
                }]
            );
        }
    }
}

#[test]
fn change_refuses_repairs_that_are_not_lowercase_commit_digits() {
    for text in ["AB".repeat(20), "AB".repeat(32), "a".repeat(39), "a".repeat(41), "a".repeat(63), "g".repeat(64)] {
        for input in [
            format!(r#"{{"words":"a","repairs":"{text}"}}"#),
            format!(r#"{{"changes":[{{"words":"a","repairs":"{text}"}}]}}"#),
        ] {
            assert_eq!(decided(b"change", input.as_bytes()), Err(Problem::Range(b"repairs")));
        }
    }
    for input in [br#"{"words":"a","repairs":1}"#.as_slice(), br#"{"changes":[{"words":"a","repairs":null}]}"#] {
        assert_eq!(decided(b"change", input), Err(Problem::Type(b"repairs")));
    }
}

#[test]
fn change_decodes_branch_mine() {
    for input in [br#"{"words":"a","branch":"mine"}"#.as_slice(), br#"{"changes":[{"words":"a","branch":"mine"}]}"#] {
        let engine::Tool::Change { changes } = decided(b"change", input).expect("branch mine").tool else {
            panic!("change")
        };
        assert_eq!(
            changes.as_ref(),
            &[engine::ChangeCall {
                branch: Some(engine::ChangeBranch::Mine),
                ..engine::ChangeCall::words(b"a".as_slice().into(), None)
            }]
        );
    }
}

#[test]
fn change_refuses_branch_run_as_an_unknown_choice() {
    for input in [
        br#"{"words":"a","branch":"run","landing":"goal"}"#.as_slice(),
        br#"{"changes":[{"words":"a","branch":"run","landing":"goal"}]}"#,
    ] {
        assert_eq!(decided(b"change", input), Err(Problem::Range(b"branch")));
    }
}

#[test]
fn change_keeps_branch_repairs_producer_and_llm_together() {
    let input =
        br#"{"words":"a","branch":"mine","repairs":"abababababababababababababababababababababababababababababababab","producer":6,"llm":{"producer":{"main":{"model":"sol","effort":"high"}}}}"#;
    let engine::Tool::Change { changes } = configured_change(input).expect("branch and role choices").tool else {
        panic!("change")
    };
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].branch, Some(engine::ChangeBranch::Mine));
    assert_eq!(changes[0].producer, Some(6));
    assert_eq!(changes[0].repairs, Some([0xab; 32]));
    assert_eq!(
        changes[0].llm.producer.as_ref().expect("producer choice").main,
        tasks::ModelEffort { model: 0, effort: Some(1) }
    );
}

#[test]
fn change_refuses_a_branch_other_than_mine() {
    let cases: &[(&[u8], Problem)] = &[
        (br#"{"words":"a","branch":"theirs"}"#, Problem::Range(b"branch")),
        (br#"{"words":"a","branch":""}"#, Problem::Range(b"branch")),
        (br#"{"words":"a","branch":1}"#, Problem::Type(b"branch")),
        (br#"{"changes":[{"words":"a","branch":"theirs"}]}"#, Problem::Range(b"branch")),
        (br#"{"changes":[{"words":"a","branch":""}]}"#, Problem::Range(b"branch")),
        (br#"{"changes":[{"words":"a","branch":1}]}"#, Problem::Type(b"branch")),
    ];
    for (input, expected) in cases {
        assert_eq!(decided(b"change", input), Err(*expected), "{input:?}");
    }
}

#[test]
fn change_decodes_its_questions() {
    for (questions, expected) in [("up", engine::ChangeQuestions::Up), ("none", engine::ChangeQuestions::None)] {
        let member = format!(r#"{{"words":"a","questions":"{questions}"}}"#);
        for input in [member.clone(), format!(r#"{{"changes":[{{"words":"first"}},{member}]}}"#)] {
            let engine::Tool::Change { changes } = decided(b"change", input.as_bytes()).expect("questions").tool else {
                panic!("change")
            };
            assert_eq!(
                changes.last().expect("change member"),
                &engine::ChangeCall {
                    questions: Some(expected.clone()),
                    ..engine::ChangeCall::words(b"a".as_slice().into(), None)
                }
            );
            if changes.len() == 2 {
                assert_eq!(changes[0].questions, None);
            }
        }
    }
}

#[test]
fn change_refuses_unknown_questions() {
    for (questions, expected) in [
        (r#""foreman""#, Problem::Range(b"questions")),
        (r#""ask""#, Problem::Range(b"questions")),
        ("true", Problem::Type(b"questions")),
        ("0", Problem::Type(b"questions")),
        ("null", Problem::Type(b"questions")),
        ("[]", Problem::Type(b"questions")),
        ("{}", Problem::Type(b"questions")),
    ] {
        let member = format!(r#"{{"words":"a","questions":{questions}}}"#);
        for input in [member.clone(), format!(r#"{{"changes":[{member}]}}"#)] {
            assert_eq!(decided(b"change", input.as_bytes()), Err(expected), "{input}");
        }
    }
}

#[test]
fn change_decodes_several_changes_in_order() {
    let engine::Tool::Change { changes } =
        decided(b"change", br#"{"changes":[{"words":"a","name":"a"},{"words":"b","after":["a"]}]}"#)
            .expect("batch change")
            .tool
    else {
        panic!("change")
    };
    assert_eq!(
        changes.as_ref(),
        &[
            engine::ChangeCall {
                name: Some(b"a".as_slice().into()),
                ..engine::ChangeCall::words(b"a".as_slice().into(), None)
            },
            engine::ChangeCall {
                after: Box::new([engine::ChangeAfter::Name(b"a".as_slice().into())]),
                ..engine::ChangeCall::words(b"b".as_slice().into(), None)
            },
        ]
    );
    let engine::Tool::Change { changes } =
        decided(b"change", br#"{"changes":[{"words":"a","repository":"work/hello","landing":"main","after":[1]}]}"#)
            .expect("one batch member")
            .tool
    else {
        panic!("change")
    };
    assert_eq!(
        changes.as_ref(),
        &[engine::ChangeCall {
            words: b"a".as_slice().into(),
            repository: Some(b"work/hello".as_slice().into()),
            name: None,
            landing: Some(engine::ChangeLanding::Branch(b"main".as_slice().into())),
            branch: None,
            merge: None,
            after: Box::new([engine::ChangeAfter::Task(1)]),
            producer: None,
            llm: engine::ChangeLlm::default(),
            questions: None,
            repairs: None,
        }]
    );
}

#[test]
fn change_decodes_a_producer_and_llm_by_role() {
    let input = r#"{"words":"a","producer":6,"llm":{"producer":{"main":{"model":"sol","effort":"high"},"sub_agents":[{"model":"haiku"}]},"review":{"main":{"model":"sol","effort":"medium"}}}}"#;
    let expected = engine::ChangeCall {
        producer: Some(6),
        llm: engine::ChangeLlm {
            producer: Some(tasks::LlmChoice {
                later: None,
                account: None,
                main: tasks::ModelEffort { model: 0, effort: Some(1) },
                subagents: Box::new([tasks::ModelEffort { model: 1, effort: None }]),
            }),
            review: Some(tasks::LlmChoice {
                later: None,
                account: None,
                main: tasks::ModelEffort { model: 0, effort: Some(0) },
                subagents: Box::new([]),
            }),
        },
        ..engine::ChangeCall::words(b"a".as_slice().into(), None)
    };
    let engine::Tool::Change { changes } = configured_change(input.as_bytes()).expect("role choices").tool else {
        panic!("change")
    };
    assert_eq!(changes.as_ref(), core::slice::from_ref(&expected));
    let input = format!(
        r#"{{"changes":[{input},{{"words":"b","producer":1,"llm":{{"review":{{"main":{{"model":"haiku"}}}}}}}},{{"words":"c","producer":4294967295,"llm":{{}}}},{{"words":"d","llm":{{"producer":{{"main":{{"model":"sol"}}}}}}}}]}}"#
    );
    let engine::Tool::Change { changes } = configured_change(input.as_bytes()).expect("per-change choices").tool else {
        panic!("change")
    };
    assert_eq!(
        changes.as_ref(),
        &[
            expected,
            engine::ChangeCall {
                producer: Some(1),
                llm: engine::ChangeLlm {
                    producer: None,
                    review: Some(tasks::LlmChoice {
                        later: None,
                        account: None,
                        main: tasks::ModelEffort { model: 1, effort: None },
                        subagents: Box::new([]),
                    }),
                },
                ..engine::ChangeCall::words(b"b".as_slice().into(), None)
            },
            engine::ChangeCall { producer: Some(u32::MAX), ..engine::ChangeCall::words(b"c".as_slice().into(), None) },
            engine::ChangeCall {
                llm: engine::ChangeLlm {
                    producer: Some(tasks::LlmChoice {
                        later: None,
                        account: None,
                        main: tasks::ModelEffort { model: 0, effort: None },
                        subagents: Box::new([]),
                    }),
                    review: None,
                },
                ..engine::ChangeCall::words(b"d".as_slice().into(), None)
            },
        ]
    );
}

fn configured_change(json: &[u8]) -> Result<engine::Call, Problem> {
    let model = assignment().run.policy.model;
    let models = [
        engine::Model {
            name: b"sol".as_slice().into(),
            efforts: Box::new([b"medium".as_slice().into(), b"high".as_slice().into()]),
            ..model.clone()
        },
        engine::Model { name: b"haiku".as_slice().into(), efforts: Box::new([]), ..model },
    ];
    let input = run::HostInput::attested(json.into()).expect("bounded object exterior");
    match crate::call_configured(
        run::CallName { activation: 7, completion: 2, position: 1 },
        b"change",
        &input,
        &models,
        &[],
    )? {
        crate::Decoded::Call(call) => Ok(call),
        crate::Decoded::Answered(_) => panic!("change answered at decode"),
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test joins JSON entries at the sub-agent count boundary")]
fn change_refuses_a_producer_or_llm_out_of_range() {
    let cases: &[(&str, Problem)] = &[
        (r#""producer":0"#, Problem::Range(b"producer")),
        (r#""producer":-1"#, Problem::Range(b"producer")),
        (r#""producer":"6""#, Problem::Type(b"producer")),
        (r#""producer":4294967296"#, Problem::Range(b"producer")),
        (r#""producer":1.5"#, Problem::Range(b"producer")),
        (r#""llm":{"foreman":{}}"#, Problem::Range(b"llm")),
        (r#""llm":[]"#, Problem::Type(b"llm")),
        (r#""llm":{"producer":true}"#, Problem::Type(b"llm")),
        (r#""llm":{"producer":{"main":{"model":"opus"}}}"#, Problem::Range(b"model")),
        (r#""llm":{"review":{"main":{"model":"Sol"}}}"#, Problem::Range(b"model")),
        (r#""llm":{"producer":{"main":{"model":"sol","effort":"max"}}}"#, Problem::Range(b"effort")),
        (r#""llm":{"review":{"main":{"model":"haiku","effort":"high"}}}"#, Problem::Range(b"effort")),
        (r#""llm":{"producer":{}}"#, Problem::Missing(b"main")),
        (r#""llm":{"review":{"main":{}}}"#, Problem::Missing(b"model")),
        (r#""llm":{"producer":{"main":{"model":0}}}"#, Problem::Type(b"model")),
        (r#""llm":{"review":{"main":{"model":"sol","effort":0}}}"#, Problem::Type(b"effort")),
        (r#""llm":{"producer":{"main":{"model":"sol"},"sub_agents":{}}}"#, Problem::Type(b"sub_agents")),
        (r#""llm":{"review":{"main":{"model":"sol"},"sub_agents":[{"model":"opus"}]}}"#, Problem::Range(b"model")),
    ];
    for (member, expected) in cases {
        for input in [format!(r#"{{"words":"a",{member}}}"#), format!(r#"{{"changes":[{{"words":"a",{member}}}]}}"#)] {
            assert_eq!(configured_change(input.as_bytes()), Err(*expected), "{input}");
        }
    }
    // Repeated models reach the root's fit check; decode checks the list's bound.
    for count in [tasks::MODELS, tasks::MODELS + 1] {
        let items = vec![r#"{"model":"haiku"}"#; usize::try_from(count).unwrap()].join(",");
        let input =
            format!(r#"{{"words":"a","llm":{{"producer":{{"main":{{"model":"sol"}},"sub_agents":[{items}]}}}}}}"#);
        if count > tasks::MODELS {
            assert_eq!(configured_change(input.as_bytes()), Err(Problem::TooLarge));
        } else {
            let engine::Tool::Change { changes } = configured_change(input.as_bytes()).expect("model limit").tool
            else {
                panic!("change")
            };
            assert_eq!(changes[0].llm.producer.as_ref().unwrap().subagents.len(), usize::try_from(count).unwrap());
        }
    }
}

#[test]
fn change_batch_refuses_outer_options() {
    let members = r#"{"name":"lexer","words":"lex"},{"name":"parser","words":"parse"},{"name":"checks","words":"check"},{"name":"docs","words":"document"}"#;
    let options =
        r#""landing":"goal","producer":2,"questions":"up","llm":{"producer":{"main":{"model":"sol","effort":"high"}}}"#;
    let input = format!(r#"{{"changes":[{members}],{options}}}"#);
    assert_eq!(
        configured_change(input.as_bytes()),
        Err(Problem::Beside(b"landing")),
        "combined recorded option placement must fail before returning an engine call"
    );
    let cases: &[(&str, &str)] = &[
        ("repository", r#""owner/name""#),
        ("name", r#""batch""#),
        ("landing", r#""goal""#),
        ("branch", r#""mine""#),
        ("merge", r#""aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#),
        ("after", "[1]"),
        ("producer", "2"),
        ("llm", r#"{"producer":{"main":{"model":"sol","effort":"high"}}}"#),
        ("questions", r#""up""#),
        ("repairs", r#""aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#),
    ];
    for (field, value) in cases {
        for value in [*value, "null"] {
            let input = format!(r#"{{"changes":[{members}],"{field}":{value}}}"#);
            assert_eq!(configured_change(input.as_bytes()), Err(Problem::Beside(field.as_bytes())), "{input}");
        }
    }
    let input = format!(r#"{{"changes":[{members}],"words":"outer"}}"#);
    assert_eq!(configured_change(input.as_bytes()), Err(Problem::Range(b"changes")));
    change_batch_keeps_member_options(options);
}

#[expect(clippy::disallowed_methods, reason = "the test joins four change members into one JSON batch")]
fn change_batch_keeps_member_options(options: &str) {
    let mut members = skein_lib::List::with_capacity(4);
    let mut expected = skein_lib::List::with_capacity(4);
    for (name, words) in [("lexer", "lex"), ("parser", "parse"), ("checks", "check"), ("docs", "document")] {
        let member = format!(r#"{{"name":"{name}","words":"{words}",{options}}}"#);
        let engine::Tool::Change { changes } = configured_change(member.as_bytes()).expect("single change").tool else {
            panic!("change")
        };
        let change = &changes[0];
        assert_eq!(change.name.as_deref(), Some(name.as_bytes()));
        assert_eq!(change.words.as_ref(), words.as_bytes());
        assert_eq!(change.landing, Some(engine::ChangeLanding::Goal));
        assert_eq!(change.producer, Some(2));
        assert_eq!(change.questions, Some(engine::ChangeQuestions::Up));
        assert_eq!(
            change.llm.producer.as_ref().expect("producer choice").main,
            tasks::ModelEffort { model: 0, effort: Some(1) }
        );
        expected.push(change.clone()).expect("four expected changes");
        members.push(member).expect("four batch members");
    }
    let input = format!(r#"{{"changes":[{}]}}"#, members.into_boxed().join(","));
    let engine::Tool::Change { changes } = configured_change(input.as_bytes()).expect("member options").tool else {
        panic!("change")
    };
    assert_eq!(changes.as_ref(), expected.into_boxed().as_ref(), "all four members keep options and call order");
}

#[test]
fn change_batch_declares_each_member() {
    let mut found = false;
    for tool in tools(Duration::from_secs(10)) {
        if tool.name.as_ref() != b"change" {
            continue;
        }
        found = true;
        let schema = crate::json::parse(&tool.schema).expect("plan schema");
        let properties = schema.required(b"properties").unwrap();
        let batch = properties.required(b"changes").unwrap();
        assert_eq!(batch.required(b"minItems").unwrap().number(b"minItems"), Ok(1));
        let item = batch.required(b"items").unwrap();
        assert_eq!(
            item.required(b"required").unwrap().array(b"required").unwrap(),
            &[crate::json::Value::String(b"words".as_slice().into())]
        );
        let member = item.required(b"properties").unwrap();
        for field in [
            b"words".as_slice(),
            b"repository",
            b"name",
            b"landing",
            b"branch",
            b"merge",
            b"after",
            b"producer",
            b"llm",
            b"questions",
            b"repairs",
        ] {
            assert_eq!(member.required(field), properties.required(field), "{field:?}");
            assert!(member.field(field).unwrap().is_some(), "member declares {field:?}");
        }
        assert!(member.field(b"changes").unwrap().is_none(), "batches do not nest");
        let words = member.required(b"words").unwrap();
        assert_eq!(words.required(b"minLength").unwrap().number(b"minLength"), Ok(1));
    }
    assert!(found, "change offered");
}

#[test]
fn change_refuses_words_with_changes_and_empty_members() {
    let cases: &[(&[u8], Problem)] = &[
        (br#"{"questions":"up","producer":2,"changes":[{"words":"a"}]}"#, Problem::Beside(b"producer")),
        (br#"{"questions":"up","repairs":true,"changes":[{"words":"a"}]}"#, Problem::Beside(b"repairs")),
        (br#"{"landing":"goal","changes":[{}]}"#, Problem::Beside(b"landing")),
        (br#"{"words":"a","changes":[{"words":"b"}]}"#, Problem::Range(b"changes")),
        (br#"{"words":"a","landing":"goal","changes":[{"words":"b"}]}"#, Problem::Range(b"changes")),
        (br#"{"changes":[]}"#, Problem::Range(b"changes")),
        (br#"{"changes":{}}"#, Problem::Type(b"changes")),
        (br#"{"changes":[true]}"#, Problem::Type(b"changes")),
        (br#"{"changes":[{"words":"a","changes":[{"words":"b"}]}]}"#, Problem::Range(b"changes")),
        (br#"{"changes":[{}]}"#, Problem::Missing(b"words")),
        (br#"{"changes":[{"words":""}]}"#, Problem::Range(b"words")),
        (br#"{"changes":[{"words":"a","repository":"hello"}]}"#, Problem::Range(b"repository")),
        (br#"{"words":"a","name":""}"#, Problem::Range(b"name")),
        (br#"{"words":"a","name":1}"#, Problem::Type(b"name")),
        (br#"{"words":"a","landing":""}"#, Problem::Range(b"landing")),
        (br#"{"words":"a","landing":true}"#, Problem::Type(b"landing")),
        (br#"{"words":"a","after":0}"#, Problem::Type(b"after")),
        (br#"{"words":"a","after":[0]}"#, Problem::Range(b"after")),
        (br#"{"words":"a","after":[-1]}"#, Problem::Range(b"after")),
        (br#"{"words":"a","after":[1.5]}"#, Problem::Range(b"after")),
        (br#"{"words":"a","after":[18446744073709551616]}"#, Problem::Range(b"after")),
        (br#"{"words":"a","after":[""]}"#, Problem::Range(b"after")),
        (br#"{"words":"a","after":[true]}"#, Problem::Type(b"after")),
        (br#"{"words":"a","after":[{}]}"#, Problem::Type(b"after")),
        (br#"{"words":"a","after":[[]]}"#, Problem::Type(b"after")),
        (br#"{"words":"a","after":[null]}"#, Problem::Type(b"after")),
    ];
    for (input, expected) in cases {
        assert_eq!(decided(b"change", input), Err(*expected), "{input:?}");
    }
    let name = "x".repeat(65);
    let input = format!(r#"{{"words":"a","name":"{name}"}}"#);
    assert_eq!(decided(b"change", input.as_bytes()), Err(Problem::Range(b"name")));
}

#[test]
fn change_offers_its_plan_members_only_with_control() {
    let legacy = br#"{"type":"object","properties":{"words":{"type":"string","description":"What to change: the files, and the exact text where it matters. It is all the change's coder reads."},"repository":{"type":"string","description":"The repository, as owner/name; leave it out when the project has one you may write."}},"required":["words"]}"#;
    let plan = br#"{"type":"object","properties":{"words":{"type":"string","description":"Change's complete brief.","minLength":1},"repository":{"type":"string","description":"owner/name; omit if sole writable."},"name":{"type":"string","description":"A name for after."},"landing":{"type":"string","description":"goal/name; omit: held goal/base."},"branch":{"type":"string","enum":["mine"],"description":"mine: goal into base; omit: a new."},"merge":{"type":"string","description":"Catch-up: 40/64 hex commit."},"after":{"type":"array","description":"Live delegate IDs/earlier names.","items":{"oneOf":[{"type":"integer","minimum":1},{"type":"string"}]}},"producer":{"type":"integer","minimum":1,"description":"Producer charter; omit: forge default."},"llm":{"type":"object","description":"producer/review:{main:{model,effort},sub_agents:[{model,effort}]}."},"changes":{"type":"array","description":"Batch: put all options inside each item.","items":{"type":"object","properties":{"words":{"type":"string","description":"Change's complete brief.","minLength":1},"repository":{"type":"string","description":"owner/name; omit if sole writable."},"name":{"type":"string","description":"A name for after."},"landing":{"type":"string","description":"goal/name; omit: held goal/base."},"branch":{"type":"string","enum":["mine"],"description":"mine: goal into base; omit: a new."},"merge":{"type":"string","description":"Catch-up: 40/64 hex commit."},"after":{"type":"array","description":"Live delegate IDs/earlier names.","items":{"oneOf":[{"type":"integer","minimum":1},{"type":"string"}]}},"producer":{"type":"integer","minimum":1,"description":"Producer charter; omit: forge default."},"llm":{"type":"object","description":"producer/review:{main:{model,effort},sub_agents:[{model,effort}]}."},"repairs":{"type":"string","description":"Red landing's 40/64 hex merge commit.","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"},"questions":{"type":"string","enum":["none","up"],"description":"up, none; omit: project setting."}},"required":["words"]},"minItems":1},"repairs":{"type":"string","description":"Red landing's 40/64 hex merge commit.","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"},"questions":{"type":"string","enum":["none","up"],"description":"up, none; omit: project setting."}}}"#;
    let stock = 1_539;
    for (authority, schema) in [(stock, legacy.as_slice()), (stock | 0b100, plan.as_slice())] {
        let mut found = false;
        for tool in
            crate::calls::tools_for(Duration::from_secs(10), engine::RunPolicy::EVERY_FAMILY, tasks::Tools(authority))
        {
            if tool.name.as_ref() == b"change" {
                found = true;
                assert_eq!(tool.schema.as_ref(), schema);
                assert_eq!(tool.description.as_ref(), b"Have one change written on a branch of a repository of this project, opened as a pull request and merged once its checks pass; you hear its result.");
            }
        }
        assert!(found, "change offered");
    }
    let mut found = false;
    for tool in tools(Duration::from_secs(10)) {
        if tool.name.as_ref() == b"change" {
            found = true;
            assert_eq!(tool.schema.as_ref(), plan);
            crate::json::parse(&tool.schema).expect("plan schema");
        }
    }
    assert!(found, "complete registry includes plan schema");
}

#[test]
fn change_refuses_missing_words() {
    assert_eq!(decided(b"change", b"{}"), Err(Problem::Missing(b"words")));
}

#[test]
fn change_refuses_empty_words() {
    assert_eq!(decided(b"change", br#"{"words":""}"#), Err(Problem::Range(b"words")));
}

#[test]
fn change_refuses_non_string_words() {
    assert_eq!(decided(b"change", br#"{"words":1}"#), Err(Problem::Type(b"words")));
}

#[test]
fn change_refuses_malformed_repository_names() {
    for name in ["", "hello", "/hello", "work/", "work/hello/other"] {
        let input = format!(r#"{{"words":"edit README","repository":"{name}"}}"#);
        assert_eq!(decided(b"change", input.as_bytes()), Err(Problem::Range(b"repository")));
    }
}

#[test]
fn a_delegate_executor_or_grant_denial_suggests_change() {
    use jig_domain_authority::{Answer, Finding, Source};
    for finding in [Finding::Executor { source: Source::Task }, Finding::Grant { source: Source::Task }] {
        let rendered =
            answer(&engine::CallAnswer::DelegationDenied { answer: Answer::Refuse, findings: Box::new([finding]) });
        assert!(rendered.error());
        assert!(rendered.text().ends_with(b"For a change to a repository, call change with what to change."));
        assert!(rendered.text().len() <= run::HostAnswer::CAPACITY);
    }
}

#[test]
fn change_repository_refusals_render_the_three_choices() {
    struct Case {
        named: Option<&'static [u8]>,
        repositories: &'static [&'static [u8]],
        expected: &'static [u8],
    }
    for case in [
        Case { named: None, repositories: &[], expected: b"this project has no repository to change" },
        Case {
            named: None,
            repositories: &[b"work/hello", b"work/other"],
            expected: b"name the repository: work/hello, work/other",
        },
        Case {
            named: Some(b"work/x"),
            repositories: &[b"work/hello"],
            expected: b"no repository work/x here; this project has: work/hello",
        },
    ] {
        let mut repositories = skein_lib::List::with_capacity(2);
        for name in case.repositories {
            repositories.push(Box::from(*name)).expect("fixture repositories");
        }
        let requested = match case.named {
            Some(name) => Some(Box::from(name)),
            None => None,
        };
        let value = engine::CallAnswer::ChangeRefused {
            problem: Box::new(tasks::Problem::new(None, tasks::Refusal::Unknown)),
            named: requested,
            repositories: repositories.into_boxed(),
        };
        let rendered = answer(&value);
        assert!(rendered.error());
        assert_eq!(rendered.text(), case.expected);
        assert!(rendered.text().len() <= run::HostAnswer::CAPACITY);
    }
}

mod brief_fit;
mod docs;
mod json;
mod reference;
mod room;

#[test]
fn flat_forge_reads_decode_the_repository_name_and_name_bad_members() {
    assert_eq!(
        decided(b"read_branch", br#"{"branch":"main"}"#).unwrap().tool,
        engine::Tool::ReadNamed {
            repository: None,
            read: skein_forge::api::Read::Branch { branch: b"main".as_slice().into() }
        }
    );
    assert_eq!(
        decided(b"read_pull_for", br#"{"repository":"qa/pilot","head":"temper/1","base":"main"}"#).unwrap().tool,
        engine::Tool::ReadNamed {
            repository: Some(b"qa/pilot".as_slice().into()),
            read: skein_forge::api::Read::PullFor {
                head: b"temper/1".as_slice().into(),
                base: b"main".as_slice().into()
            }
        }
    );
    for skip in [0, 5, u32::MAX] {
        for (tool, members, read) in [
            (b"read_reviews".as_slice(), "\"number\":1", skein_forge::api::Read::Reviews { number: 1, page: 3, skip }),
            (
                b"read_remarks",
                "\"number\":1,\"review\":2",
                skein_forge::api::Read::Remarks { number: 1, review: 2, page: 3, skip },
            ),
            (
                b"read_pull_files",
                "\"number\":1,\"head\":\"0000000000000000000000000000000000000000000000000000000000000000\"",
                skein_forge::api::Read::PullFiles { number: 1, head: [0; 32], page: 3, skip, sides: true },
            ),
        ] {
            let input = format!("{{{members},\"page\":3,\"skip\":{skip}}}");
            assert_eq!(
                decided(tool, input.as_bytes()).unwrap().tool,
                engine::Tool::ReadNamed { repository: None, read }
            );
        }
    }
    for (tool, input, problem) in [
        (b"read_branch".as_slice(), b"{}".as_slice(), Problem::Missing(b"branch")),
        (b"read_branch", br#"{"branch":1}"#, Problem::Type(b"branch")),
        (b"read_reviews", br#"{"number":1,"page":4294967296}"#, Problem::Range(b"page")),
        (b"read_reviews", br#"{"number":1,"page":0}"#, Problem::Range(b"page")),
        (b"read_reviews", br#"{"number":1,"skip":4294967296}"#, Problem::Range(b"skip")),
        (b"read_checks", br#"{"commit":"bad"}"#, Problem::Range(b"full commit identifier")),
    ] {
        assert_eq!(decided(tool, input), Err(problem));
    }
    assert_eq!(
        decided(b"read_items", b"{}").unwrap().tool,
        engine::Tool::ReadNamed {
            repository: None,
            read: skein_forge::api::Read::Items { single: false, since: skein_lib::Time::ZERO, page: 1, kind: None }
        }
    );
    let engine::Tool::ReadNamed { read: skein_forge::api::Read::Checks { commit }, .. } =
        decided(b"read_checks", br#"{"commit":"abcdef0123456789abcdef0123456789abcdef01"}"#).unwrap().tool
    else {
        panic!("commit read");
    };
    assert_eq!(commit[20..], [0; 12]);
}

#[test]
fn every_forge_read_declares_its_flat_members_as_json() {
    let members: &[(&[u8], &[&[u8]])] = &[
        (b"read_items", &[b"since", b"page", b"item_kind"]),
        (b"read_item", &[b"number", b"after"]),
        (b"read_pull", &[b"number"]),
        (b"read_pull_for", &[b"head", b"base"]),
        (b"read_reviews", &[b"number", b"page", b"skip"]),
        (b"read_statuses", &[b"commit", b"page"]),
        (b"read_remarks", &[b"number", b"review", b"page", b"skip"]),
        (b"read_branch", &[b"branch"]),
        (b"read_branches", &[b"prefix"]),
        (b"read_pull_files", &[b"number", b"head", b"page", b"skip"]),
        (b"read_compare", &[b"before", b"after"]),
        (b"read_checks", &[b"commit"]),
        (b"read_file", &[b"head", b"path", b"max_bytes"]),
        (b"read_job", &[b"head", b"run", b"job", b"attempt", b"max_bytes"]),
        (b"read_protection", &[b"branch"]),
        (b"read_settings", &[]),
        (b"read_collaborators", &[b"page"]),
        (b"read_permission", &[b"user"]),
    ];
    let mut count = 0;
    for tool in tools(Duration::from_secs(1)) {
        let mut names = None;
        for (name, fields) in members {
            if *name == tool.name.as_ref() {
                names = Some(fields);
                break;
            }
        }
        let Some(names) = names else {
            continue;
        };
        count += 1;
        let schema = crate::json::parse(&tool.schema).expect("JSON schema");
        assert!(!schema.required(b"additionalProperties").unwrap().boolean(b"additionalProperties").unwrap());
        let properties = schema.required(b"properties").unwrap();
        let crate::json::Value::Object(fields) = properties else {
            panic!("properties object");
        };
        assert_eq!(fields.len(), names.len() + 1);
        assert_eq!(
            properties.required(b"repository").unwrap().required(b"type").unwrap().text(b"type").unwrap().as_ref(),
            b"string"
        );
        let description =
            properties.required(b"repository").unwrap().required(b"description").unwrap().text(b"description").unwrap();
        assert!(
            skein_lib::bytes::find(&description, b"leave it out, or empty").is_some()
                || skein_lib::bytes::find(&description, b"omit or empty").is_some()
        );
        for name in *names {
            let property = properties.required(name).unwrap();
            assert!(!property.required(b"description").unwrap().text(b"description").unwrap().is_empty());
            assert!(property.field(b"type").unwrap().is_some());
        }
    }
    assert_eq!(count, members.len());
}

#[test]
fn a_missing_job_refusal_names_the_asked_head_run_job_and_attempt_with_recovery() {
    for (head, digits, run, job, attempt) in
        [([0; 32], "0".repeat(40), 2, 0, 1), ([0xab; 32], "ab".repeat(32), 8, 15, 3)]
    {
        let pin = skein_forge::api::JobAttempt { head, run, job, attempt };
        let expected = format!(
            "Forge read failed: job missing. No job attempt at head {digits}, run {run}, job {job}, attempt {attempt}, as asked. read_checks at this head names each Actions job's run, job and attempt as read_job takes them; a link's run number and its index of the job are not those ids."
        );
        let rendered = crate::answer_for_job(
            &engine::CallAnswer::ForgeRead(Box::new(Err(skein_forge::api::Error::MissingJob))),
            b"read_job",
            1024,
            crate::ForgeReadBounds::STOCK,
            pin,
        );
        assert!(rendered.error());
        assert_eq!(rendered.text(), expected.as_bytes());
        for cap in [0, 32, u32::try_from(expected.len()).unwrap() - 1] {
            let clipped = crate::answer_for_job(
                &engine::CallAnswer::ForgeRead(Box::new(Err(skein_forge::api::Error::MissingJob))),
                b"read_job",
                cap,
                crate::ForgeReadBounds::STOCK,
                pin,
            );
            assert!(clipped.error());
            assert!(clipped.text().len() <= usize::try_from(cap).unwrap());
            assert!(expected.as_bytes().starts_with(clipped.text()));
        }
    }
}

#[test]
fn job_and_attempt_zero_decode_but_run_and_explicit_byte_limit_remain_positive() {
    for attempt_number in [0, 1] {
        let input = format!(
            r#"{{"head":"0000000000000000000000000000000000000000","run":1,"job":0,"attempt":{attempt_number}}}"#
        );
        let call = decided(b"read_job", input.as_bytes()).expect("zero job or attempt");
        let engine::Tool::ReadNamed { read: skein_forge::api::Read::Job { attempt, max_bytes, .. }, .. } = call.tool
        else {
            panic!("decoded job read")
        };
        assert_eq!((attempt.job, attempt.attempt, max_bytes), (0, attempt_number, 0));
    }
    for (input, field) in [
        (
            br#"{"head":"0000000000000000000000000000000000000000","run":0,"job":0,"attempt":0}"#.as_slice(),
            b"run".as_slice(),
        ),
        (
            br#"{"head":"0000000000000000000000000000000000000000","run":1,"job":0,"attempt":4294967296}"#.as_slice(),
            b"attempt".as_slice(),
        ),
        (
            br#"{"head":"0000000000000000000000000000000000000000","run":1,"job":0,"attempt":0,"max_bytes":0}"#
                .as_slice(),
            b"max_bytes".as_slice(),
        ),
    ] {
        assert_eq!(decided(b"read_job", input).err(), Some(Problem::Range(field)));
    }
    let tools = tools(Duration::from_secs(10));
    let mut schema = None;
    for tool in &tools {
        if tool.name.as_ref() == b"read_job" {
            schema = Some(&tool.schema);
        }
    }
    let schema = schema.expect("read_job tool");
    let schema = crate::json::parse(schema).expect("job schema");
    let properties = schema.required(b"properties").unwrap();
    for (name, minimum) in [(b"run".as_slice(), 1), (b"job".as_slice(), 0), (b"attempt".as_slice(), 0)] {
        let property = properties.required(name).unwrap();
        assert_eq!(property.required(b"minimum").unwrap().number(b"minimum").unwrap(), minimum);
        if name != b"attempt" {
            let words = property.required(b"description").unwrap().text(b"description").unwrap();
            assert!(skein_lib::bytes::find(&words, b"read_checks").is_some(), "{words:?}");
            assert!(skein_lib::bytes::find(&words, b"not a link's").is_some());
            assert!(!words.contains(&b'0'), "no job-zero invitation");
        }
    }
}

#[test]
fn goal_decodes_words_and_optional_advisor_and_agree() {
    for (input, advisor, agree) in [
        (br#"{"words":"plan"}"#.as_slice(), false, false),
        (br#"{"words":"plan","advisor":true}"#, true, false),
        (br#"{"words":"plan","advisor":false}"#, false, false),
        (br#"{"words":"plan","agree":true}"#, false, true),
        (br#"{"words":"plan","agree":false}"#, false, false),
        (br#"{"words":"plan","advisor":true,"agree":true}"#, true, true),
    ] {
        let call = decided(b"goal", input).expect("goal declared and decoded");
        assert_eq!(call.tool, engine::Tool::Goal { words: Box::from(b"plan".as_slice()), advisor, agree });
    }
    assert_eq!(decided(b"goal", b"{}"), Err(Problem::Missing(b"words")));
    assert_eq!(decided(b"goal", br#"{"words":""}"#), Err(Problem::Range(b"words")));
    assert_eq!(decided(b"goal", br#"{"words":"plan","advisor":1}"#), Err(Problem::Type(b"advisor")));
    assert_eq!(decided(b"goal", br#"{"words":"plan","agree":1}"#), Err(Problem::Type(b"agree")));
}

#[path = "tests/document.rs"]
mod document;
mod kinds;
#[path = "tests/settings.rs"]
mod settings;
#[test]
fn a_proposed_release_names_the_placed_hold_and_its_revision() {
    let body =
        decided(b"propose", br#"{"action":"release","task":41,"hold":12,"revision":1,"reason":"please resume"}"#)
            .expect("placed release");
    assert_eq!(
        body.tool,
        engine::Tool::Propose {
            action: engine::ProposedAction::Release { task: 41, hold: 12, revision: 1 },
            reason: b"please resume".as_slice().into(),
            as_holder: false,
        }
    );
    for (json, field) in [
        (br#"{"action":"release","task":41,"revision":1,"reason":"x"}"#.as_slice(), b"hold".as_slice()),
        (br#"{"action":"release","task":41,"hold":12,"reason":"x"}"#.as_slice(), b"revision".as_slice()),
    ] {
        assert_eq!(decided(b"propose", json), Err(Problem::Missing(field)));
    }
    decided(b"propose", br#"{"action":"release","task":41,"hold":0,"revision":1,"reason":"x"}"#).unwrap_err();
    decided(b"propose", br#"{"action":"release","task":41,"hold":12,"revision":0,"reason":"x"}"#).unwrap_err();
}

#[test]
fn a_refused_release_names_its_owner_and_the_exact_proposal() {
    for (owner, words) in [
        (jig_domain_people::HoldOwner::Party { person: 3 }, "person 3; only they may release it"),
        (jig_domain_people::HoldOwner::Task { task: 9 }, "task 9; only it or a task above it may release it"),
    ] {
        let value = engine::CallAnswer::HoldOwner { task: 41, hold: 12, revision: 1, owner };
        let part = value.core_part(1);
        assert_eq!(engine::CallAnswer::from_core(&part), Some(value.clone()));
        let rendered = answer(&value);
        assert!(rendered.error());
        assert_eq!(rendered.text(), format!("Refused: task 41 is held by hold 12 (revision 1), placed by {words}. Propose it: {{\"action\": \"release\", \"task\": 41, \"hold\": 12, \"revision\": 1, \"reason\": \"...\"}}").as_bytes());
    }
}

#[test]
fn directory_root_spellings_decode_to_the_empty_path() {
    for path in [".", "./", ""] {
        let input = format!(r#"{{"head":"abcdef0123456789abcdef0123456789abcdef01","path":"{path}"}}"#);
        assert_eq!(
            decided(b"read_file", input.as_bytes()).unwrap().tool,
            engine::Tool::ReadNamed {
                repository: None,
                read: skein_forge::api::Read::File { head: SHA1_HEAD, path: Box::from([]), max_bytes: 0 }
            }
        );
    }
}
#[test]
fn directory_answers_render_entries_and_the_root_header() {
    use skein_forge::api::{Answer, Entry, EntryKind};
    for (path, header) in [(b"src".as_slice(), b"src".as_slice()), (b"", b".")] {
        for (truncated, tail) in [(false, b"".as_slice()), (true, b"\n[directory cut]".as_slice())] {
            let value = Answer::Directory {
                head: SHA1_HEAD,
                path: path.into(),
                entries: Box::new([
                    Entry { name: b"main.rs".as_slice().into(), kind: EntryKind::File },
                    Entry { name: b"lib".as_slice().into(), kind: EntryKind::Directory },
                    Entry { name: b"vendor".as_slice().into(), kind: EntryKind::Submodule },
                ]),
                truncated,
            };
            let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(value))));
            let body =
                b" at abcdef0123456789abcdef0123456789abcdef01 (a directory):\nmain.rs\nlib/\nvendor (submodule)";
            let length = header
                .len()
                .checked_add(body.len())
                .expect("fixture size")
                .checked_add(tail.len())
                .expect("fixture size");
            let mut expected = skein_lib::Writer::new(length);
            for piece in [header, body.as_slice(), tail] {
                expected.put(piece).expect("measured fixture");
            }
            assert!(!rendered.error());
            assert_eq!(rendered.text(), expected.finish().as_ref());
        }
    }
}

#[test]
fn a_chats_answer_refusal_names_its_persons_inbox_question() {
    refusal_words(
        tasks::Refusal::PersonAnswers { question: 42 },
        None,
        b"Refused: task 99 your person answers question 42 in their inbox",
    );
}

#[test]
fn a_no_change_verdict_preserves_its_explanation_and_cannot_claim_a_resource() {
    let answer = || {
        let declared = run::outcome::Declared::Verdict(run::outcome::Verdict {
            name: b"no_change".as_slice().into(),
            text: b"No widget UI exists".as_slice().into(),
            fields: Box::new([]),
            items: Box::new([]),
        });
        run::Answer::Accepted { outcome: declared, spent: spend(3), turns: 1 }
    };
    assert_eq!(
        result(answer(), None),
        Ok(crate::Terminal {
            cumulative: 3,
            turns: 1,
            end: tasks::End::Finished {
                result: tasks::TaskResult::NoChange { words: b"No widget UI exists".as_slice().into() },
                cancel_delegates: false,
            },
        })
    );
    assert_eq!(
        result(answer(), Some(ChangeResource { connector: 1, kind: 2, resource: 42 })),
        Err(Problem::Type(b"no_change"))
    );
}

#[test]
fn job_answers_fit_metadata_and_escaped_log_bytes_within_the_host_bound() {
    use skein_forge::api;
    let diagnostic = b"Missing changes/40.md with the line: opened 2026-11-02T14:01:43Z\n";
    for cap in [128, 1024, 4096] {
        for byte in [0, b'\t', 255, b'x'] {
            let mut log = vec![byte; 6000];
            log.extend_from_slice(diagnostic);
            let read = api::Answer::Job {
                attempt: api::JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 },
                step: Some(api::Step { number: 3, name: Box::from(*b"build\t") }),
                log: log.into_boxed_slice(),
                truncated: false,
            };
            let rendered = crate::answer_bounded(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))), cap);
            assert!(!rendered.error());
            assert!(rendered.text().len() <= usize::try_from(cap).unwrap());
            assert!(rendered.text().ends_with(diagnostic));
            assert!(skein_lib::bytes::find(rendered.text(), b"[log bytes cut]").is_some());
        }
    }
}

#[test]
fn job_metadata_refusals_fit_even_a_tiny_host_bound() {
    use skein_forge::api;
    for cap in [0, 1, 16, 32] {
        let read = api::Answer::Job {
            attempt: api::JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 },
            step: Some(api::Step { number: 3, name: vec![255; 64].into_boxed_slice() }),
            log: Box::from(*b"diagnostic"),
            truncated: false,
        };
        let rendered = crate::answer_bounded(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))), cap);
        assert!(rendered.error());
        assert!(rendered.text().len() <= usize::try_from(cap).unwrap());
    }
}

#[test]
fn document_sections_have_repository_path_and_heading_titles() {
    for (words, title) in [
        (b"org/rules docs/rules.md".as_slice(), b"Document org/rules docs/rules.md".as_slice()),
        (b"org/rules docs/rules.md, \"Safety\"".as_slice(), b"Document org/rules docs/rules.md, \"Safety\"".as_slice()),
        (b"docs/rules.md".as_slice(), b"Document docs/rules.md".as_slice()),
    ] {
        let section = crate::section(&engine::BriefSection {
            kind: engine::BriefKind::Forge(engine::ForgeBriefKind::Document(words.into())),
            body: engine::BriefBody::Missing(brief::GatherMissing::Failed),
        });
        assert_eq!(section.title.as_ref(), title);
        assert_eq!(section.text.as_ref(), b"[read failed]");
    }
}

/// temper/docs/design/domain/agent/tools.md, 1: merge accepts both commit widths and pads SHA-1.
#[test]
fn a_plan_change_reads_merge_as_a_commit() {
    for (digits, length) in [("aB".repeat(20), 20), ("aB".repeat(32), 32)] {
        let input = format!(r#"{{"words":"Catch up","merge":"{digits}"}}"#);
        let engine::Tool::Change { changes } = decided(b"change", input.as_bytes()).expect("catch-up").tool else {
            panic!("change")
        };
        let mut commit = [0_u8; 32];
        commit[..length].fill(0xab);
        assert_eq!(changes[0].merge, Some(commit));
        assert_eq!(changes[0].landing, None);
        let input = format!(r#"{{"changes":[{{"words":"Catch up","landing":"goal","merge":"{digits}"}}]}}"#);
        let engine::Tool::Change { changes } = decided(b"change", input.as_bytes()).expect("catch-up batch").tool
        else {
            panic!("change")
        };
        assert_eq!(changes[0].merge, Some(commit));
        assert_eq!(changes[0].landing, Some(engine::ChangeLanding::Goal));
    }
}

/// temper/docs/design/domain/agent/tools.md, 1: a malformed commit names the refused member.
#[test]
fn a_malformed_merge_is_refused_naming_it() {
    for (digit, length) in [("a", 0), ("a", 39), ("a", 41), ("a", 63), ("a", 65), ("g", 40), ("g", 64)] {
        let digits = digit.repeat(length);
        let input = format!(r#"{{"words":"Catch up","merge":"{digits}"}}"#);
        assert_eq!(decided(b"change", input.as_bytes()), Err(Problem::Range(b"merge")));
    }
    assert_eq!(decided(b"change", br#"{"words":"Catch up","merge":42}"#), Err(Problem::Type(b"merge")));
}

#[test]
fn a_refused_message_says_where_its_requester_is() {
    use jig_domain_authority::{Answer, Finding};
    for (findings, tool, expected) in [
        (
            vec![Finding::Reference],
            b"message".as_slice(),
            "Authority refused: task reference. Your brief's [Requested by ...] line names your requester; [This task: N] is you. If it says [Requested by person N], they have no task to message: ask with delegate, a person task addressed to person N.",
        ),
        (
            vec![Finding::Reference, Finding::Tool, Finding::Tool],
            b"message".as_slice(),
            "Authority refused: task reference; tool family message. Your brief's [Requested by ...] line names your requester; [This task: N] is you. If it says [Requested by person N], they have no task to message: ask with delegate, a person task addressed to person N.",
        ),
        (vec![Finding::Reference], b"read_task".as_slice(), "Authority refused: task reference"),
    ] {
        let denied = engine::CallAnswer::ToolDenied { answer: Answer::Refuse, findings: findings.into() };
        let rendered = crate::answer_for_tool(&denied, tool, 1024, crate::ForgeReadBounds::STOCK);
        assert!(rendered.error());
        assert_eq!(rendered.text(), expected.as_bytes());
    }
}

#[test]
fn an_authority_refusal_ends_with_no_separator() {
    use jig_domain_authority::{Answer, Finding, Source};
    for (findings, expected) in [
        (vec![Finding::Tool], "Authority refused: tool family control"),
        (
            vec![Finding::Tool, Finding::Tool, Finding::Reference, Finding::Tool],
            "Authority refused: tool family control; task reference",
        ),
    ] {
        let denied = engine::CallAnswer::ToolDenied { answer: Answer::Refuse, findings: findings.into() };
        let rendered = crate::answer_for_tool(&denied, b"release", 1024, crate::ForgeReadBounds::STOCK);
        assert!(rendered.error());
        assert_eq!(rendered.text(), expected.as_bytes());
    }
    let denied = engine::CallAnswer::DelegationDenied {
        answer: Answer::Refuse,
        findings: Box::new([Finding::Executor { source: Source::Task }]),
    };
    let rendered = crate::answer_for_tool(&denied, b"change", 1024, crate::ForgeReadBounds::STOCK);
    assert_eq!(rendered.text(), b"Authority refused: missing executor permission (task authority.delegation.kinds). For a change to a repository, call change with what to change.");
}

#[test]
fn a_denied_release_names_the_control_family_once() {
    use jig_domain_authority::{Answer, Finding};
    let denied = engine::CallAnswer::ToolDenied {
        answer: Answer::Refuse,
        findings: Box::new([Finding::Tool, Finding::Tool, Finding::Tool]),
    };
    let rendered = crate::answer_for_tool(&denied, b"release", 1024, crate::ForgeReadBounds::STOCK);
    assert!(rendered.error());
    assert_eq!(rendered.text(), b"Authority refused: tool family control");
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::format_push_string,
    reason = "the rendering table constructs readable expected UTF-8 text"
)]
fn comparisons_answer_names_counts_and_containment_with_omissions() {
    use skein_forge::{api, measure};
    for (ahead, behind, changed, paths) in [
        (2u64, 0u64, 3u64, vec![Box::from(*b"x"), Box::from(*b"y")]),
        (9, 0, 9, vec![Box::from(*b"x"); 8]),
        (22, 3, 1, vec![Box::from(*b"x")]),
    ] {
        for (after, digits) in COMMIT_SPELLINGS {
            let compare = api::Answer::Compare {
                before: SHA1_HEAD,
                after: *after,
                ahead,
                behind,
                changed,
                files: paths.clone().into_boxed_slice(),
            };
            assert!(measure::answer_rows(&compare, 8));
            assert!(measure::answer(&compare).unwrap() < 65_536);
            let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(compare))));
            let a = core::str::from_utf8(digits).unwrap();
            let b = "abcdef0123456789abcdef0123456789abcdef01";
            let contains = if behind == 0 { "contains" } else { "lacks" };
            let mut expected = format!(
                "{a} is {ahead} commits ahead of {b} and {behind} behind it.\n{a} {contains} {b}.\nChanged files: {changed}\n"
            );
            for path in &paths {
                expected.push_str(core::str::from_utf8(path.as_ref()).unwrap());
                expected.push('\n');
            }
            let left = changed - u64::try_from(paths.len()).unwrap();
            if left > 0 {
                expected.push_str(&format!(
                    "[{left} not listed: read_pull_files lists a pull's files; read_file reads a path at each commit]\n"
                ));
            }
            assert!(!rendered.error());
            assert_eq!(rendered.text(), expected.as_bytes());
        }
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test inspects the host guidance as UTF-8 text")]
fn read_compare_too_large_names_bounds_and_paged_fallback() {
    use skein_forge::api::Error;
    let refused = engine::CallAnswer::ForgeRead(Box::new(Err(Error::TooLarge(skein_forge::api::Excess::exact(
        skein_forge::api::Bound::Rows,
        8,
        22,
    )))));
    let rendered = crate::answer_for_tool(&refused, b"read_compare", 2048, crate::ForgeReadBounds::STOCK);
    assert!(rendered.error());
    let text = core::str::from_utf8(rendered.text()).expect("compare guidance is UTF-8");
    assert!(text.starts_with("Forge read failed: too large: past forge.client.rows (8): 22 rows.\n"));
    for guidance in [
        "commit ids were read",
        "another id won't help",
        "Never retry with a changed or guessed id",
        "read_pull_files",
        "read_file",
        "both endpoints",
        "max_bytes",
    ] {
        assert!(text.contains(guidance), "missing {guidance}: {text:?}");
    }
    assert!(!text.contains("unknown"));
    for cap in [0, 32, 512, 2048] {
        let bounded = crate::answer_for_tool(&refused, b"read_compare", cap, crate::ForgeReadBounds::STOCK);
        assert!(bounded.error());
        assert!(bounded.text().len() <= usize::try_from(cap).unwrap());
        assert_eq!(bounded.text(), &text.as_bytes()[..bounded.text().len()]);
    }
    let missing = engine::CallAnswer::ForgeRead(Box::new(Err(Error::Missing)));
    assert_eq!(
        crate::answer_for_tool(&missing, b"read_compare", 2048, crate::ForgeReadBounds::STOCK).text(),
        b"Forge read failed: missing"
    );
}

/// domain/agent/tools.md: a forge size refusal retains its bound, measure and read-on words.
#[test]
fn forge_read_too_large_names_configured_bounds_and_clips() {
    use skein_forge::api::{Bound, Error, Excess};
    for (excess, expected) in [
        (Excess::exact(Bound::Rows, 8, 22), "forge.client.rows (8): 22 rows"),
        (
            Excess::at_least(Bound::AnswerBytes, 65_536, 70_000),
            "forge.client.answer_bytes (65536): at least 70000 bytes",
        ),
        (Excess::exact(Bound::OpBytes, 32, 44), "forge.client.op_bytes (32): 44 bytes"),
        (Excess::exact(Bound::Forge, 4_294_967_295, 4_294_967_296), "a bound of the forge's (4294967295): 4294967296"),
        (Excess::at_least(Bound::Forge, 0, 0), "a bound of the forge's"),
        (Excess::unnamed(), "an unnamed bound"),
    ] {
        let refused = engine::CallAnswer::ForgeRead(Box::new(Err(Error::TooLarge(excess))));
        for (tool, recovery) in [
            (b"read_branches".as_slice(), "Use a longer prefix, or read_branch for one name."),
            (b"read_reviews", "Advance page to list the rest; a later page skips this oversized page."),
        ] {
            let expected = format!("Forge read failed: too large: past {expected}. {recovery}");
            // An unrelated renderer configuration cannot replace the excess's figures.
            let forge = crate::ForgeReadBounds { rows: 3, answer_bytes: 4096 };
            let whole = crate::answer_for_tool(&refused, tool, 2048, forge);
            assert!(whole.error());
            assert_eq!(whole.text(), expected.as_bytes());
            for cap in [0, 64, u32::try_from(expected.len()).unwrap() - 1] {
                let clipped = crate::answer_for_tool(&refused, tool, cap, forge);
                assert!(clipped.error());
                assert_eq!(clipped.text(), &expected.as_bytes()[..usize::try_from(cap).unwrap()]);
            }
        }
    }
}

/// domain/agent/tools.md: a refused branch read says what to read instead; every
/// other Missing and Empty refusal keeps its one line.
#[test]
fn a_refused_branch_read_says_what_to_read_instead() {
    use skein_forge::api::Error;
    const BRANCHES_EMPTY: &str = "Forge read failed: empty. read_branches needs a nonempty prefix, as temper/ for temper's branches. read_settings names the default branch, and read_branch reads one branch by its name.";
    const BRANCHES_TOO_LARGE: &str = "Forge read failed: too large: past forge.client.answer_bytes (65536): at least 65537 bytes. Use a longer prefix, or read_branch for one name.";
    const BRANCH_MISSING: &str = "Forge read failed: missing. The forge has no branch of that name in this repository. read_settings names its default branch, and read_branches lists the names under a prefix, as temper/. A goal's own branch is there only where its goal holds one, and a run branch from its first push.";
    for (tool, error, expected) in [
        (b"read_branches".as_slice(), Error::Empty, BRANCHES_EMPTY),
        (
            b"read_branches",
            Error::TooLarge(skein_forge::api::Excess::at_least(skein_forge::api::Bound::AnswerBytes, 65536, 65537)),
            BRANCHES_TOO_LARGE,
        ),
        (b"read_branch", Error::Missing, BRANCH_MISSING),
        (b"read_branch", Error::Empty, "Forge read failed: empty"),
        (b"read_branches", Error::Missing, "Forge read failed: missing"),
        (b"read_file", Error::Missing, "Forge read failed: missing"),
        (b"read_pull", Error::Empty, "Forge read failed: empty"),
    ] {
        let refused = engine::CallAnswer::ForgeRead(Box::new(Err(error)));
        let rendered = crate::answer_for_tool(&refused, tool, 2048, crate::ForgeReadBounds::STOCK);
        assert!(rendered.error());
        assert_eq!(rendered.text(), expected.as_bytes());
        for cap in [0, 32, 64] {
            let bounded = crate::answer_for_tool(&refused, tool, cap, crate::ForgeReadBounds::STOCK);
            assert!(bounded.error());
            assert!(bounded.text().len() <= usize::try_from(cap).unwrap());
            assert_eq!(bounded.text(), &expected.as_bytes()[..bounded.text().len()]);
        }
    }
}

#[test]
fn item_listings_report_the_answer_byte_cut() {
    use skein_forge::api;
    for cut in [false, true] {
        let item = api::Summary {
            number: 1,
            kind: api::Kind::Issue,
            state: api::State::Open,
            author: 2,
            labels: Box::new([]),
            key: None,
            title: Box::new([]),
            body: Box::new([]),
            updated: skein_lib::Time::ZERO,
        };
        let value = api::Answer::Items { items: Box::new([item]), more: true, cut, now: skein_lib::Time::ZERO };
        let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(value)));
        let rendered = crate::answer_for_tool(&recorded, b"read_items", 2048, crate::ForgeReadBounds::STOCK);
        assert!(!rendered.error());
        assert_eq!(skein_lib::bytes::find(rendered.text(), b"Page cut at forge.client.answer_bytes.").is_some(), cut);
        assert!(skein_lib::bytes::find(rendered.text(), b"More pages available.").is_some());
    }
    for cut in [false, true] {
        for (tool, value) in [
            (
                b"read_reviews".as_slice(),
                api::Answer::Reviews {
                    reviews: Box::new([api::Review {
                        id: 1,
                        author: 2,
                        verdict: api::Verdict::Comment,
                        commit: [0; 32],
                        body: Box::new([]),
                        key: None,
                        at: skein_lib::Time::ZERO,
                        official: false,
                        provenance: api::Provenance::Original,
                        revision: 0,
                    }]),
                    more: true,
                    cut,
                },
            ),
            (
                b"read_remarks",
                api::Answer::Remarks {
                    remarks: Box::new([api::Remark {
                        id: 1,
                        author: 2,
                        path: b"x".as_slice().into(),
                        line: 1,
                        body: Box::new([]),
                    }]),
                    more: true,
                    cut,
                },
            ),
            (
                b"read_pull_files",
                api::Answer::PullFiles {
                    head: [0; 32],
                    files: Box::new([api::File { path: b"x".as_slice().into(), before: None, after: None }]),
                    more: true,
                    cut,
                },
            ),
        ] {
            let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(value)));
            for (page, skip) in [(1, 0), (3, 5)] {
                let rendered = crate::answer_for_page(&recorded, tool, 2048, crate::ForgeReadBounds::STOCK, page, skip);
                assert!(!rendered.error());
                let ending: Box<[u8]> = if cut {
                    format!("Page cut: read on with page {page}, skip {}.\n", skip + 1).into_bytes().into()
                } else {
                    b"More pages available.\n".as_slice().into()
                };
                assert!(rendered.text().ends_with(&ending), "{:?}", rendered.text());
            }
        }
    }
}

#[test]
fn branch_lists_render_whole_within_the_host_byte_bound() {
    use skein_forge::api::Answer;
    for count in [9, 40, 600, 2000] {
        let mut names = skein_lib::List::with_capacity(count);
        let mut expected = skein_lib::Writer::new(usize::try_from(count).unwrap() * 12);
        for index in 0..count {
            let name: Box<[u8]> = format!("temper/{index:04}").into_bytes().into();
            expected.put(&name).unwrap();
            expected.put(b"\n").unwrap();
            names.push(name).expect("fixture names");
        }
        let expected = expected.finish();
        let value = Answer::Branches(names.into_boxed());
        assert!(skein_forge::measure::answer(&value).unwrap() <= u64::from(crate::ForgeReadBounds::STOCK.answer_bytes));
        let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(value)));
        let rendered = crate::answer_for_tool(
            &recorded,
            b"read_branches",
            crate::ForgeReadBounds::STOCK.answer_bytes,
            crate::ForgeReadBounds::STOCK,
        );
        assert!(!rendered.error(), "{count} names");
        assert_eq!(rendered.text(), expected.as_ref());
    }
}

/// domain/agent/tools.md: a declaration says what a run calls instead.
#[test]
fn declarations_say_when_to_call_goal_and_what_prefix_to_give() {
    let mut count = 0_usize;
    for tool in tools(Duration::from_secs(1)) {
        match tool.name.as_ref() {
            b"goal" => assert_eq!(tool.description.as_ref(), b"Have one goal tracked, only for work your person wants agreed first (set agree) or a survey that changes nothing. For a change to a repository, call change."),
            b"delegate" => assert!(tool.description.ends_with(b"For a change to a repository, call change.")),
            b"read_branches" => {
                assert_eq!(tool.description.as_ref(), b"List the branches whose names start with a prefix. The names fit forge.client.answer_bytes; a longer prefix reads fewer.");
                let schema = crate::json::parse(&tool.schema).expect("JSON schema");
                let prefix = schema.required(b"properties").unwrap().required(b"prefix").unwrap();
                let description = prefix.required(b"description").unwrap().text(b"description").unwrap();
                assert_eq!(description.as_ref(), b"A nonempty prefix of the names, as temper/.");
            }
            _ => continue,
        }
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn successful_compare_over_host_bound_keeps_measured_feedback() {
    use skein_forge::api;
    const CAP: u32 = 2048;
    const PATH_BYTES: usize = 3000; // The path alone exceeds CAP.
    let compare = api::Answer::Compare {
        before: SHA1_HEAD,
        after: SHA1_HEAD,
        ahead: 1,
        behind: 0,
        changed: 1,
        files: Box::new([vec![b'f'; PATH_BYTES].into_boxed_slice()]),
    };
    let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(compare)));
    let whole = answer(&recorded);
    let size = u32::try_from(whole.text().len()).expect("fixture size");
    assert_eq!(crate::answer_for_tool(&recorded, b"read_compare", size, crate::ForgeReadBounds::STOCK), whole);
    let rendered = crate::answer_for_tool(&recorded, b"read_compare", CAP, crate::ForgeReadBounds::STOCK);
    assert!(rendered.error());
    assert!(rendered.text().len() <= usize::try_from(CAP).unwrap());
    let size = skein_lib::Decimal::of(u64::from(size));
    assert!(skein_lib::bytes::find(rendered.text(), size.as_bytes()).is_some());
    for phrase in [
        "rendered size",
        "host answer limit 2048 bytes",
        "forge.client.rows (8)",
        "forge.client.answer_bytes (65536)",
        "read_pull",
        "read_pull_files",
        "read_file",
        "max_bytes",
        "core.call_answer_bytes",
        "agent.json",
    ] {
        assert!(skein_lib::bytes::find(rendered.text(), phrase.as_bytes()).is_some(), "missing {phrase}");
    }
}

#[test]
fn file_and_pull_files_over_host_bound_name_limit_and_recovery() {
    oversized_file_and_pull_feedback();
    fitting_file_and_pull_feedback();
    file_and_pull_rendering_boundaries();
    pull_inventory_limits();
    many_pull_paths_fit_the_text_fragments();
}

fn many_pull_paths_fit_the_text_fragments() {
    use skein_forge::api;
    let mut files = skein_lib::List::with_capacity(200);
    for n in 0_u32..200 {
        files
            .push(api::File { path: format!("{n}").into_bytes().into(), before: Some(vec![0; 32].into()), after: None })
            .expect("fixture files fit");
    }
    let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(api::Answer::PullFiles {
        head: [0x11; 32],
        files: files.into_boxed(),
        more: false,
        cut: false,
    })));
    for (cap, omitted) in [(4096, true), (10_240, false)] {
        let feedback = crate::answer_bounded(&recorded, cap);
        assert!(feedback.error());
        assert!(feedback.text().len() <= usize::try_from(cap).unwrap());
        assert_eq!(skein_lib::bytes::find(feedback.text(), b"[changed paths omitted:").is_some(), omitted);
        assert!(skein_lib::bytes::find(feedback.text(), b"list directories").is_some());
    }
}

fn oversized_file_and_pull_feedback() {
    use skein_forge::api;
    for (tool, input) in [
        (b"read_file".as_slice(), br#"{"head":"c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6","path":"docs/design/formats.md"}"#.as_slice()),
        (b"read_pull_files", br#"{"number":11,"head":"1111111111111111111111111111111111111111111111111111111111111111","page":2}"#),
    ] {
        let _call = decoded(tool, input).expect("recorded oversized read");
    }
    let file = api::Answer::File {
        head: [0xc6; 32],
        path: b"docs/design/formats.md".as_slice().into(),
        bytes: vec![b'x'; 12_870].into(),
        truncated: false,
    };
    let files = api::Answer::PullFiles {
        head: [0x11; 32],
        files: Box::new([
            api::File {
                path: b"docs/design/formats.md".as_slice().into(),
                before: Some(vec![b'a'; 6000].into()),
                after: Some(vec![b'b'; 6000].into()),
            },
            api::File { path: b"close-out.md".as_slice().into(), before: None, after: Some(vec![b'c'; 6000].into()) },
        ]),
        more: true,
        cut: false,
    };
    for (tool, value, size) in [(b"read_file".as_slice(), file, 12_962), (b"read_pull_files", files, 18_076)] {
        oversized_file_feedback(tool, value, size);
    }
}

fn oversized_file_feedback(tool: &[u8], value: skein_forge::api::Answer, size: usize) {
    let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(value)));
    let whole = answer(&recorded);
    assert_eq!(whole.text().len(), size);
    let feedback = crate::answer_for_tool(&recorded, tool, 10_240, crate::ForgeReadBounds::STOCK);
    assert!(feedback.text().len() <= 10_240, "rendered answer bypassed the host bound");
    if tool == b"read_file" {
        assert!(!feedback.error());
        assert!(feedback.text().ends_with(b"[file truncated]"));
        return;
    }
    assert!(feedback.error());
    let words = feedback.text();
    let size_text = skein_lib::Decimal::of(u64::try_from(size).expect("fixture size"));
    assert!(skein_lib::bytes::find(words, size_text.as_bytes()).is_some());
    for required in ["10240", "read_file", "max_bytes", "overhead"] {
        assert!(skein_lib::bytes::find(words, required.as_bytes()).is_some(), "missing {required}: {words:?}");
    }
    if tool == b"read_pull_files" {
        for required in [
            "docs/design/formats.md",
            "close-out.md",
            "[omitted]",
            "[absent]",
            "More files",
            "page",
            "Advancing page alone cannot recover",
            "base commit",
            "11111111",
        ] {
            assert!(
                skein_lib::bytes::find(words, required.as_bytes()).is_some(),
                "pull #11 page 2: missing {required}: {words:?}"
            );
        }
    }
}

fn fitting_file_and_pull_feedback() {
    use skein_forge::api;
    let input = br#"{"head":"c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6","path":"docs/design/formats.md","max_bytes":32}"#;
    let _call = decoded(b"read_file", input).expect("explicit prefix read");
    let small = api::Answer::File {
        head: [0xc6; 32],
        path: b"docs/design/formats.md".as_slice().into(),
        bytes: vec![b'x'; 32].into(),
        truncated: true,
    };
    let pull = api::Answer::PullFiles {
        head: [0x11; 32],
        files: Box::new([api::File {
            path: b"close-out.md".as_slice().into(),
            before: None,
            after: Some(b"closed".as_slice().into()),
        }]),
        more: false,
        cut: false,
    };
    for value in [small, pull] {
        let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(value)));
        let whole = answer(&recorded);
        let bounded = crate::answer_bounded(&recorded, 1024);
        assert!(!bounded.error());
        assert_eq!(bounded.text(), whole.text());
        if let engine::CallAnswer::ForgeRead(read) = &recorded
            && let Ok(api::Answer::File { .. }) = read.as_ref()
        {
            assert!(bounded.text().ends_with(b"[file truncated]"));
        }
    }
}

fn file_and_pull_rendering_boundaries() {
    use skein_forge::api;
    for byte in [0, 255, b'x'] {
        for value in [
            api::Answer::File {
                head: [1; 32],
                path: vec![byte; 8].into(),
                bytes: vec![byte; 64].into(),
                truncated: true,
            },
            api::Answer::PullFiles {
                head: [1; 32],
                files: Box::new([api::File {
                    path: vec![byte; 8].into(),
                    before: Some(vec![byte; 64].into()),
                    after: None,
                }]),
                more: true,
                cut: false,
            },
            api::Answer::Directory {
                head: [1; 32],
                path: vec![byte; 64].into(),
                entries: Box::new([]),
                truncated: true,
            },
        ] {
            let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(value)));
            let whole = answer(&recorded);
            let size = u32::try_from(whole.text().len()).unwrap();
            assert_eq!(crate::answer_bounded(&recorded, size).text(), whole.text());
            for cap in [0, 1, 32] {
                let bounded = crate::answer_bounded(&recorded, cap);
                assert!(bounded.error(), "metadata and escaping count toward the bound");
                assert!(bounded.text().len() <= usize::try_from(cap).unwrap());
            }
        }
    }
    for tool in [b"read_file".as_slice(), b"read_pull_files"] {
        let recorded = engine::CallAnswer::ForgeRead(Box::new(Err(api::Error::TooLarge(api::Excess::unnamed()))));
        let feedback = crate::answer_for_tool(&recorded, tool, 1024, crate::ForgeReadBounds::STOCK);
        let words = feedback.text();
        assert!(feedback.error());
        for required in ["Forge read failed: too large: past an unnamed bound.", "read_file", "max_bytes"] {
            assert!(skein_lib::bytes::find(words, required.as_bytes()).is_some(), "missing {required}: {words:?}");
        }
    }
}

fn pull_inventory_limits() {
    use skein_forge::api;
    let mut files = skein_lib::List::with_capacity(40);
    for n in 0_u32..40 {
        files
            .push(api::File {
                path: format!("path-{n}-{}", "z".repeat(60)).into_bytes().into(),
                before: Some(vec![0; 64].into()),
                after: None,
            })
            .expect("fixture files fit");
    }
    let files = files.into_boxed();
    for more in [false, true] {
        let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(api::Answer::PullFiles {
            head: [0x11; 32],
            files: files.clone(),
            more,
            cut: false,
        })));
        let feedback = crate::answer_bounded(&recorded, 1024);
        let words = feedback.text();
        assert!(feedback.error());
        assert!(feedback.text().len() <= 1024);
        for required in ["path-0-", "before [omitted], after [absent]", "[changed paths omitted:"] {
            assert!(skein_lib::bytes::find(words, required.as_bytes()).is_some(), "missing {required}: {words:?}");
        }
        assert!(
            skein_lib::bytes::find(
                words,
                if more { b"More files available on next page." } else { b"No more file pages." }
            )
            .is_some()
        );
    }
    let file = engine::CallAnswer::ForgeRead(Box::new(Ok(api::Answer::File {
        head: [1; 32],
        path: b"f".as_slice().into(),
        bytes: vec![0; 100_000].into(),
        truncated: false,
    })));
    let feedback = crate::answer_bounded(&file, 1024);
    assert!(!feedback.error());
    assert!(feedback.text().ends_with(b"[file truncated]"));
}

#[test]
fn change_decodes_branch_mine_with_questions() {
    for input in [
        br#"{"words":"a","branch":"mine","questions":"none"}"#.as_slice(),
        br#"{"changes":[{"words":"a","branch":"mine","questions":"none"}]}"#,
    ] {
        let engine::Tool::Change { changes } = decided(b"change", input).expect("branch and questions").tool else {
            panic!("change")
        };
        assert_eq!(
            changes.as_ref(),
            &[engine::ChangeCall {
                branch: Some(engine::ChangeBranch::Mine),
                questions: Some(engine::ChangeQuestions::None),
                ..engine::ChangeCall::words(b"a".as_slice().into(), None)
            }]
        );
    }
}

#[test]
fn commit_reads_refuse_abbreviated_or_non_hexadecimal_identifiers() {
    for (tool, input) in [
        (b"read_file".as_slice(), br#"{"head":"f471720","path":"README.md"}"#.as_slice()),
        (b"read_compare", br#"{"before":"f471720","after":"f471720"}"#),
        (b"read_checks", br#"{"commit":"ABCDEF0123456789abcdef0123456789abcdef01"}"#),
    ] {
        assert_eq!(decoded(tool, input).unwrap_err(), Problem::Range(b"full commit identifier"));
    }
    for (_, full) in COMMIT_SPELLINGS {
        let prefix = br#"{"head":""#;
        let suffix = br#"","path":"README.md"}"#;
        let bytes = prefix.len().checked_add(full.len()).unwrap().checked_add(suffix.len()).unwrap();
        let mut input = skein_lib::Writer::new(bytes);
        input.put(prefix).unwrap();
        input.put(full).unwrap();
        input.put(suffix).unwrap();
        decoded(b"read_file", &input.finish()).expect("full identifier accepted");
    }
}

#[test]
fn task_reads_declare_read_effect_and_decode_continuations() {
    for allowed in [false, true] {
        let authority = if allowed { jig_domain_core::ToolFamilies::JIG.read.0 } else { 0 };
        let tools =
            crate::calls::tools_for(Duration::from_secs(10), engine::RunPolicy::EVERY_FAMILY, tasks::Tools(authority));
        let mut found = false;
        for declaration in &tools {
            if declaration.name.as_ref() == b"read_task" {
                assert_eq!(declaration.effect, run::HostEffect::Read);
                found = true;
            }
        }
        assert_eq!(found, allowed);
    }
    let decoded = decided(b"read_task", br#"{"task":23,"from":17}"#).expect("task read decodes");
    assert_eq!(
        decoded.tool,
        engine::Tool::ReadTask { task: 23, field: jig_domain_core::reads::TaskTextField::Specification, from: 17 }
    );
    for (field, expected) in [
        ("specification", jig_domain_core::reads::TaskTextField::Specification),
        ("result", jig_domain_core::reads::TaskTextField::Result),
    ] {
        let input = format!("{{\"task\":23,\"field\":\"{field}\"}}");
        let decoded = decided(b"read_task", input.as_bytes()).expect("text selector");
        assert_eq!(decoded.tool, engine::Tool::ReadTask { task: 23, field: expected, from: 0 });
    }
    for field in ["instructions", "unknown"] {
        let input = format!("{{\"task\":23,\"field\":\"{field}\"}}");
        let _invalid = decided(b"read_task", input.as_bytes()).expect_err("only spec or result");
    }
    let _zero = decided(b"read_task", br#"{"task":0}"#).expect_err("positive task number");
    let _negative = decided(b"read_task", br#"{"task":23,"from":-1}"#).expect_err("nonnegative offset");
}

fn held_task_read() -> jig_domain_core::reads::run_task::TaskRead {
    jig_domain_core::reads::run_task::TaskRead {
        task: 23,
        verdict_code: None,
        result_pending: false,
        questions: Box::new([]),
        state: jig_domain_core::reads::run_task::TaskState::Held,
        spec_revision: 8,
        dependencies: Box::new([5]),
        words: jig_domain_core::reads::TaskText {
            task: 23,
            field: jig_domain_core::reads::TaskTextField::Specification,
            from: 0,
            total: 480,
            next: None,
            bytes: Box::new([b'x'; 480]),
        },
        hold: Some(jig_domain_core::reads::run_task::TaskHold {
            revision: 9,
            reason: tasks::Hold::Dependency(5),
            words: Some(b"restart vetoed; amend authority and release".as_slice().into()),
        }),
    }
}

#[test]
fn task_reads_fit_the_host_bound_and_continue_cut_words() {
    const WORDS: &[u8] = b"Words from 0:\n";
    const NEXT: &[u8] = b"\nnext: ";
    let mut read = held_task_read();
    read.words.total = skein_lib::len_u64(read.words.bytes.len());
    let answer = engine::CallAnswer::TaskRead(Box::new(Ok(read.clone())));
    let first = crate::answer_bounded(&answer, 256);
    assert!(!first.error());
    assert!(first.text().len() <= 256);
    let text = first.text();
    assert!(skein_lib::bytes::find(text, b"spec revision 8").is_some());
    assert!(skein_lib::bytes::find(text, b"Hold revision 9: dependency 5").is_some());
    assert!(skein_lib::bytes::find(text, b"restart vetoed; amend authority and release").is_some());
    let start = skein_lib::bytes::find(text, WORDS).expect("words").checked_add(WORDS.len()).expect("header");
    let body = text.get(start..).expect("words body");
    let cut = skein_lib::bytes::find(body, NEXT).expect("continuation");
    let prefix = body.get(..cut).expect("prefix");
    let offset = body.get(cut.checked_add(NEXT.len()).expect("next label")..).expect("offset");
    let next = prefix.len();
    let number = skein_lib::Decimal::of(u64::try_from(next).expect("source offset"));
    assert_eq!(offset, number.as_bytes());
    assert!(next > 0 && next < read.words.bytes.len());
    read.words.bytes = read.words.bytes.get(next..).expect("continuation offset").into();
    read.words.from = u64::try_from(next).expect("offset");
    let answer = engine::CallAnswer::TaskRead(Box::new(Ok(read)));
    assert!(!crate::answer_bounded(&answer, 1024).error());
    for maximum in [0, 1, 64] {
        let answer = crate::answer_bounded(&answer, maximum);
        assert!(answer.error());
        assert!(answer.text().len() <= usize::try_from(maximum).expect("u32 fits"));
    }
}

#[test]
fn escalations_name_the_subject_revision_and_task_read_recovery() {
    let word = tasks::Word {
        number: 1,
        from: tasks::Party::Task(23),
        at: skein_lib::Wall::EPOCH,
        hits: 1,
        eligible: true,
        kind: tasks::MessageKind::Escalation { task: 23, revision: 9 },
        words: b"Held task needs a decision".as_slice().into(),
    };
    let words = engine::inbox_words(1, &word);
    assert!(skein_lib::bytes::find(&words, b"Held task needs a decision").is_some());
    assert!(skein_lib::bytes::find(&words, b"Held task 23, hold revision 9").is_some());
    assert!(skein_lib::bytes::find(&words, b"Use read_task on that task").is_some());
}

#[test]
fn task_read_hold_reasons_keep_their_numbers_and_failure_classes() {
    for (reason, expected) in [
        (
            tasks::Hold::Exceeded(tasks::Exceeded { bound: tasks::Bound::Window, value: 1024 }),
            b"window tokens 1024".as_slice(),
        ),
        (tasks::Hold::Failures(tasks::Class::Permanent), b"permanent failures".as_slice()),
        (tasks::Hold::Placed { hold: 19 }, b"placed hold 19".as_slice()),
        (tasks::Hold::Uncertain { entry: 12 }, b"uncertain, entry 12".as_slice()),
        (tasks::Hold::PoolLost { pool: 6 }, b"pool lost 6".as_slice()),
    ] {
        let mut read = held_task_read();
        read.hold.as_mut().expect("held task").reason = reason;
        let answer = crate::answer_bounded(&engine::CallAnswer::TaskRead(Box::new(Ok(read))), 512);
        assert!(!answer.error());
        assert!(answer.text().len() <= 512);
        assert!(skein_lib::bytes::find(answer.text(), expected).is_some());
    }
}

#[test]
fn authority_findings_name_sources_and_dimensions() {
    use jig_domain_authority::{Answer, Finding, Source};
    let dimensions = [
        ("tool permission", "tools"),
        ("resource grant", "grants"),
        ("executor permission", "delegation.kinds"),
        ("task allowance", "delegation.tasks"),
        ("descendant depth", "delegation.depth"),
        ("spend allowance", "budget.spend"),
        ("deadline", "budget.deadline"),
        ("note scope", "notes"),
        ("model permission", "models"),
    ];
    for (source, setting) in [
        (Source::Task, "task authority"),
        (Source::Role, "project role authority"),
        (Source::Project, "project policy.ceiling"),
        (Source::Deployment, "rules.ceiling"),
    ] {
        for (index, (dimension, field)) in dimensions.iter().enumerate() {
            let value = engine::CallAnswer::DelegationDenied {
                answer: Answer::Refuse,
                findings: Box::new([Finding::Authority { source, lacks: authority_deficits(1 << index) }]),
            };
            let rendered = answer(&value);
            assert!(rendered.error());
            assert_eq!(
                rendered.text(),
                format!("Authority refused: missing {dimension} ({setting}.{field})").as_bytes()
            );
        }
        let value = engine::CallAnswer::DelegationDenied {
            answer: Answer::Refuse,
            findings: Box::new([Finding::Authority { source, lacks: authority_deficits(511) }]),
        };
        let rendered = answer(&value);
        for (dimension, field) in dimensions {
            let expected = format!("missing {dimension} ({setting}.{field})");
            assert!(skein_lib::bytes::find(rendered.text(), expected.as_bytes()).is_some(), "{:?}", rendered.text());
        }
        let value = engine::CallAnswer::DelegationDenied {
            answer: Answer::Refuse,
            findings: Box::new([Finding::Executor { source }]),
        };
        let rendered = answer(&value);
        let expected = format!("missing executor permission ({setting}.delegation.kinds)");
        assert!(skein_lib::bytes::find(rendered.text(), expected.as_bytes()).is_some());
    }
}

fn authority_deficits(bits: u16) -> jig_domain_authority::Lacks {
    jig_domain_authority::Lacks {
        tools: bits & 1 != 0,
        grants: bits & 2 != 0,
        executors: bits & 4 != 0,
        tasks: bits & 8 != 0,
        depth: bits & 16 != 0,
        spend: bits & 32 != 0,
        deadline: bits & 64 != 0,
        notes: bits & 128 != 0,
        models: bits & 256 != 0,
    }
}

#[test]
fn task_reads_name_durable_answer_targets_and_answerers() {
    let mut read = held_task_read();
    read.questions = Box::new([tasks::QuestionCredit {
        number: 103,
        answerer: tasks::Party::Task(18),
        asked: tasks::Party::Task(18),
        line: 106,
    }]);
    let rendered = answer(&engine::CallAnswer::TaskRead(Box::new(Ok(read))));
    assert!(
        skein_lib::bytes::find(rendered.text(), b"Answer target 23, question 103 answerer task 18, activity 106")
            .is_some()
    );
    let refused = answer(&engine::CallAnswer::MessageRefused(tasks::Problem::new(Some(23), tasks::Refusal::Reference)));
    assert!(refused.error());
    assert!(skein_lib::bytes::find(refused.text(), b"Use read_task on the target task").is_some());
    assert!(skein_lib::bytes::find(refused.text(), b"form=answer").is_some());
}

#[test]
fn verdict_reads_keep_the_code_when_reason_words_continue() {
    const START: &[u8] = b"Result from 17:\n";
    let mut read = held_task_read();
    read.verdict_code = Some(2);
    read.words.field = jig_domain_core::reads::TaskTextField::Result;
    read.words.from = 17;
    read.words.total = 497; // Fixture's 480 bytes after offset 17.
    let answer = engine::CallAnswer::TaskRead(Box::new(Ok(read)));
    let rendered = crate::answer_bounded(&answer, 256);
    assert!(!rendered.error());
    let text = rendered.text();
    assert!(skein_lib::bytes::find(text, b"Verdict: 2").is_some());
    let from = skein_lib::bytes::find(text, START).expect("reason header") + START.len();
    let body = text.get(from..).expect("reason body");
    let cut = skein_lib::bytes::find(body, b"\nnext: ").expect("reason continues");
    let offset = body.get(cut + b"\nnext: ".len()..).expect("next offset");
    let expected = skein_lib::Decimal::of(17 + u64::try_from(cut).expect("bounded reason"));
    assert_eq!(offset, expected.as_bytes(), "metadata never advances the reason offset");
}

#[test]
fn file_prefixes_fit_escaping_metadata_and_report_every_cut() {
    use skein_forge::api;
    for bytes in [vec![0; 600], vec![255; 600], vec![b'"'; 600], "λ".repeat(300).into_bytes()] {
        let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(api::Answer::File {
            head: [1; 32],
            path: b"f".as_slice().into(),
            bytes: bytes.into(),
            truncated: false,
        })));
        let whole = answer(&recorded);
        let size = u32::try_from(whole.text().len()).unwrap();
        for cap in [0, 1, 80, 100, 128, size - 1, size, size + 1] {
            let bounded = crate::answer_bounded(&recorded, cap);
            assert!(bounded.text().len() <= usize::try_from(cap).unwrap());
            if cap >= size {
                assert_eq!(bounded.text(), whole.text());
            } else if cap >= 100 {
                assert!(!bounded.error(), "a file prefix fits");
                assert!(bounded.text().starts_with(b"f at "));
                assert!(bounded.text().ends_with(b"\n[file truncated]"));
                assert!(bounded.text().len() > b"f at ".len() + 64 + 2 + 17);
            } else {
                assert!(bounded.error(), "metadata leaves no content room");
            }
        }
    }
    let recorded = engine::CallAnswer::ForgeRead(Box::new(Ok(api::Answer::File {
        head: [1; 32],
        path: vec![b'p'; 2048].into(),
        bytes: b"body".as_slice().into(),
        truncated: false,
    })));
    let bounded = crate::answer_bounded(&recorded, 512);
    assert!(bounded.error());
    for required in [b"metadata".as_slice(), b"core.call_answer_bytes", b"lowering max_bytes cannot fit"] {
        assert!(skein_lib::bytes::find(bounded.text(), required).is_some());
    }
}

#[path = "tests/decided.rs"]
mod decided_tool;

#[path = "tests/see.rs"]
mod see;

#[path = "tests/see_person.rs"]
mod see_person;

#[path = "tests/remember.rs"]
mod remember;

#[path = "tests/wire.rs"]
mod wire;
