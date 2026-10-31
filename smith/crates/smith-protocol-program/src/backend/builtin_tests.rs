#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary backend harness, programming-model.md 10.2"
)]
use super::fixtures::records as active;
use super::fixtures::{env, request};
use super::*;
use crate::backend::fixtures::COMPLETION;
use alloc::{vec, vec::Vec};
use smith_domain::tools::{Effect, Grants};

const READ: &[u8] = br#"{"type":"assistant","message":{"id":"one","content":[{"type":"tool_use","id":"r","name":"Read","input":{"file_path":"a"}}]}}"#;

const RESULT: &[u8] = br#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"r","content":[{"type":"text","text":"one"},{"type":"image"},{"type":"text","text":"two"}],"is_error":true}]}}"#;

#[test]
fn a_built_in_tool_use_is_a_builtin_call() {
    for (name, effect) in [
        ("Read", Effect::Read),
        ("Glob", Effect::Read),
        ("Grep", Effect::Read),
        ("Write", Effect::Write),
        ("Edit", Effect::Write),
        ("Bash", Effect::Write),
        ("future", Effect::Write),
        ("finish", Effect::Write),
    ] {
        let mut backend = active();
        let mut up = Queue::with_capacity(32);
        let mut below = Queue::with_capacity(64);
        let mut record: serde_json::Value = serde_json::from_slice(READ).expect("JSON");
        record["message"]["content"][0]["name"] = serde_json::json!(name);
        let bytes = serde_json::to_vec(&record).expect("JSON");
        backend.line(&env(0), &bytes, &mut up, &mut below);
        let Some(ToDomain::Completed { completion, .. }) = up.pop() else { panic!("completion") };
        let llm::Said::ToolCall { name: got, call, .. } = &completion.content[0] else { panic!("tool call") };
        assert_eq!(got.as_ref(), name.as_bytes());
        assert_eq!(*call, llm::Decoded::Builtin { effect });
    }
}

#[test]
fn a_built_in_result_follows_its_completion() {
    let mut backend = active();
    backend.owner = None;
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0), READ, &mut up, &mut below);
    backend.line(&env(0), RESULT, &mut up, &mut below);
    assert!(up.is_empty(), "result waits behind queued completion");
    backend.resume(&env(0), request(&backend), &mut up, &mut below);
    completed(up.pop(), COMPLETION);
    let Some(ToDomain::BuiltinResult { owner, id, text, error }) = up.pop() else { panic!("builtin result") };
    assert_eq!(owner, COMPLETION);
    assert_eq!(id.as_ref(), b"r");
    assert_eq!(text.as_ref(), b"one\ntwo");
    assert!(error);
    backend.line(&env(0), RESULT, &mut up, &mut below);
    assert!(up.is_empty(), "repeated result is inert");
}

#[test]
fn a_complete_s_result_for_a_built_in_is_skipped() {
    let fixtures::Harness { mut backend, mut up, mut below } = fixtures::Harness::new(active());
    backend.line(&env(0), READ, &mut up, &mut below);
    up.pop();
    while backend.pop_measure().is_some() {}
    let mut next = request(&backend);
    let FromDomain::Complete { prompt, .. } = &mut next else { panic!("Complete") };
    prompt.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::ToolResult {
            id: b"r".as_slice().into(),
            result: llm::Returned::Text { text: Box::from([b'x'; 8192]), error: false, replay: None },
        }]),
    }]);
    backend.resume(&env(0), next, &mut up, &mut below);
    assert!(up.is_empty());
    assert!(backend.outgoing.is_none());
    assert!(backend.ledger.get(0).expect("call").answer.is_none());
    backend.line(&env(0), RESULT, &mut up, &mut below);
    result_id(up.pop(), COMPLETION, b"r");
    let later = read_record("later", "r2");
    backend.line(&env(0), &later, &mut up, &mut below);
    completed(up.pop(), COMPLETION);
    let mut next = request(&backend);
    let FromDomain::Complete { prompt, .. } = &mut next else { panic!("Complete") };
    prompt.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::ToolResult {
            id: b"r".as_slice().into(),
            result: llm::Returned::Text { text: Box::from([b'x'; 8192]), error: false, replay: None },
        }]),
    }]);
    backend.resume(&env(0), next, &mut up, &mut below);
    assert!(up.is_empty(), "historical built-in results also bypass MCP rendering");
}

#[test]
fn tools_flag_names_the_granted_families() {
    for (grants, flag) in [
        (Grants { inspect: false, modify: false, shell: false }, b"".as_slice()),
        (Grants { inspect: true, modify: false, shell: false }, b"Read,Glob,Grep"),
        (Grants { inspect: false, modify: true, shell: true }, b"Write,Edit,Bash"),
        (Grants { inspect: true, modify: true, shell: true }, b"Read,Glob,Grep,Write,Edit,Bash"),
    ] {
        assert_eq!(policy::tools_flag(grants).as_ref(), flag);
    }
}

fn init(tools: &[&str], granted: bool) -> (Backend, Queue<ToDomain>) {
    init_contract(tools, granted, false)
}

fn init_contract(tools: &[&str], granted: bool, contract: bool) -> (Backend, Queue<ToDomain>) {
    let fixtures::Harness { mut backend, mut up, mut below } = fixtures::Harness::new(active());
    if let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request {
        prompt.tools.inspect = granted;
        if contract {
            prompt.served = Box::new([llm::Served::Finish]);
        }
    }
    if contract {
        backend.request = Some(report_request(&backend));
    }
    let bytes = serde_json::to_vec(&serde_json::json!({"type":"system","subtype":"init","mcp_servers":[{"name":"smith","status":"connected"}],"tools":tools})).expect("init");
    backend.admit_init(&env(0), &bytes, &mut up, &mut below);
    (backend, up)
}

#[test]
fn a_contract_init_may_list_structured_output() {
    let (backend, up) = init_contract(&["StructuredOutput"], false, true);
    assert!(up.is_empty());
    assert_eq!(backend.state, State::Running);
}

#[test]
fn a_contract_init_may_omit_structured_output() {
    let (backend, up) = init_contract(&[], false, true);
    assert!(up.is_empty());
    assert_eq!(backend.state, State::Running);
}

#[test]
fn structured_output_without_a_contract_fails_init() {
    let (_, mut up) = init_contract(&["StructuredOutput"], false, false);
    failed(up.pop(), b"smith: the program offers a built-in tool the run did not grant: StructuredOutput");
}

#[test]
fn a_contract_init_refuses_duplicate_structured_output() {
    let (_, mut up) = init_contract(&["StructuredOutput", "StructuredOutput"], false, true);
    failed(up.pop(), b"smith: the program offers a built-in tool the run did not grant: StructuredOutput");
}

const STRUCTURED_CALL: &[u8] = br#"{"type":"assistant","message":{"id":"structured","content":[{"type":"tool_use","id":"s","name":"StructuredOutput","input":{"outcome":{"form":"report","text":"draft","fields":{}}}}],"usage":{"input_tokens":13,"output_tokens":7}}}"#;

fn held_structured_output() -> (Backend, Queue<ToDomain>, Queue<Below>) {
    let mut backend = active();
    backend.request = Some(report_request(&backend));
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0), STRUCTURED_CALL, &mut up, &mut below);
    (backend, up, below)
}

#[test]
fn structured_output_is_the_finish_from_its_result() {
    use smith_domain::run::outcome::{Declared, Report};

    let (mut backend, mut up, mut below) = held_structured_output();
    assert!(up.is_empty(), "the outcome waits for its turn result");
    backend.line(&env(0), br#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"s","content":"accepted","is_error":false}]}}"#, &mut up, &mut below);
    assert!(up.is_empty(), "the program-owned result creates no domain event");
    backend.line(&env(0), br#"{"type":"result","subtype":"success","is_error":false,"structured_output":{"outcome":{"form":"report","text":"done","fields":{}}}}"#, &mut up, &mut below);
    let Some(ToDomain::Completed { owner, completion }) = up.pop() else { panic!("structured finish") };
    assert_eq!(owner, COMPLETION);
    let [llm::Said::ToolCall { name, input, call, .. }] = completion.content.as_ref() else { panic!("finish alone") };
    assert_eq!(name.as_ref(), b"finish");
    assert_eq!(input.as_ref(), br#"{"form":"report","text":"done","fields":{}}"#);
    assert_eq!(completion.usage.input_tokens, Some(13));
    assert_eq!(completion.usage.output_tokens, Some(7));
    assert_eq!(
        *call,
        llm::Decoded::Served {
            ask: smith_domain::run::Ask::Finish {
                outcome: Declared::Report(Report { text: b"done".as_slice().into(), fields: Box::default() })
            }
        }
    );
    assert!(backend.ledger.is_empty(), "no workspace call or result is owed");
    assert!(up.is_empty(), "no BuiltinResult or second completion");
    assert!(below.is_empty(), "neither a permission prompt nor an MCP answer");
    assert_eq!(backend.state, State::Running);
}

#[test]
fn a_structured_call_without_its_wrapped_outcome_fails() {
    for result in [
        serde_json::json!({"type":"result","is_error":false}),
        serde_json::json!({"type":"result","is_error":false,"structured_output":null}),
        serde_json::json!({"type":"result","is_error":false,"structured_output":{}}),
        serde_json::json!({"type":"result","is_error":false,"structured_output":{"outcome":{},"extra":true}}),
    ] {
        let (mut backend, mut up, mut below) = held_structured_output();
        let line = serde_json::to_vec(&result).expect("result JSON");
        backend.line(&env(0), &line, &mut up, &mut below);
        let Some(ToDomain::Failed { .. }) = up.pop() else { panic!("missing outcome fails") };
        assert_eq!(backend.state, State::Ending);
        assert!(up.is_empty(), "the held call was never delivered as a built-in");
    }
}

fn report_request(backend: &Backend) -> FromDomain {
    let mut request = request(backend);
    let FromDomain::Complete { prompt, outcome, .. } = &mut request else { panic!("complete") };
    prompt.served = Box::new([llm::Served::Finish]);
    // Same report text cap as the backend harness's max_tokens.
    outcome.report = Some(smith_domain::run::outcome::TextSpec { max: 128, fields: Box::default() });
    request
}

#[test]
fn an_init_with_an_ungranted_built_in_fails() {
    let (_, mut up) = init(&["mcp__smith__permission_prompt", "Read"], false);
    failed(up.pop(), b"smith: the program offers a built-in tool the run did not grant: Read");
}

#[test]
fn an_init_without_a_granted_built_in_fails() {
    let (_, mut up) = init(&["mcp__smith__permission_prompt", "Read", "Glob"], true);
    failed(up.pop(), b"smith: the program lacks a granted built-in tool: Grep");
    let (backend, up) = init(&["mcp__smith__permission_prompt", "Read", "Glob", "Grep"], true);
    assert!(up.is_empty());
    assert_eq!(backend.state, State::Running);
}

#[test]
fn the_server_keeps_only_remove_when_modify_is_granted() {
    for modify in [false, true] {
        let mut backend = active();
        let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request else { panic!("request") };
        prompt.tools = Grants { inspect: true, modify, shell: true };
        prompt.served = Box::new([llm::Served::Finish, llm::Served::Wait]);
        let expected = protocol::schemas_for_contract(
            prompt,
            &backend.settings.adapter.stated,
            &smith_domain::run::outcome::OutcomeSpec {
                change: None,
                verdicts: Box::default(),
                report: None,
                failure: None,
            },
            None,
            backend.settings.contract_bytes,
        )
        .expect("schemas");
        let tools = backend.tools();
        assert_eq!(tools.len(), if modify { 3 } else { 2 });
        assert!(tools.iter().all(|tool| [b"remove".as_slice(), b"finish", b"wait"].contains(&tool.name.as_ref())));
        let remove = tools.iter().find(|tool| tool.name.as_ref() == b"remove");
        assert_eq!(remove.is_some(), modify);
        if let Some(remove) = remove {
            let descriptor = expected.iter().find(|tool| tool.name.as_ref() == b"remove").expect("remove descriptor");
            assert_eq!(remove.description, descriptor.description);
            assert_eq!(remove.input_schema, descriptor.schema);
        }
    }
}

fn host_tool(schema: &[u8]) -> llm::Served {
    llm::Served::Host(smith_domain::run::HostTool {
        name: b"host_action".as_slice().into(),
        description: b"Host action".as_slice().into(),
        schema: schema.into(),
        effect: smith_domain::run::HostEffect::Write,
        timeout: Duration::from_secs(1),
    })
}

#[test]
fn every_charter_inventory_serves_only_object_input_schemas() {
    use smith_domain::run::outcome::{ChangeSpec, FieldRule, ItemSpec, OutcomeSpec, TextSpec, VerdictRule};
    // Same text/field cap as the backend harness's max_tokens.
    const CONTRACT_VALUE_BYTES: u32 = 128;
    let empty = OutcomeSpec { change: None, verdicts: Box::default(), report: None, failure: None };
    // Exercise every final form, including the nested verdict oneOf.
    let contract = OutcomeSpec {
        change: Some(ChangeSpec {
            checks_must_pass: true,
            fields: Box::new([FieldRule { name: b"title".as_slice().into(), max: CONTRACT_VALUE_BYTES }]),
        }),
        verdicts: Box::new([VerdictRule {
            name: b"approved".as_slice().into(),
            text_max: CONTRACT_VALUE_BYTES,
            fields: Box::default(),
            items: ItemSpec { min: 0, max: 0, kinds: Box::default() },
        }]),
        report: Some(TextSpec { max: CONTRACT_VALUE_BYTES, fields: Box::default() }),
        failure: Some(TextSpec { max: CONTRACT_VALUE_BYTES, fields: Box::default() }),
    };
    let delivery = Some(ChangeSpec { checks_must_pass: true, fields: Box::default() });
    // Open does not offer finish; Finite does. A delivery grant is independent.
    for (served, outcome, deliver, expected_names) in [
        (
            vec![llm::Served::Wait, host_tool(br#"{"properties":{"choice":{"anyOf":[{"type":"object"}]}}}"#)],
            empty.clone(),
            None,
            vec!["wait", "host_action", "permission_prompt"],
        ),
        (
            vec![llm::Served::Wait, llm::Served::Deliver],
            empty.clone(),
            delivery.clone(),
            vec!["wait", "deliver", "permission_prompt"],
        ),
        (vec![llm::Served::Wait], contract.clone(), None, vec!["wait", "permission_prompt"]),
        (vec![llm::Served::Finish, llm::Served::Wait], empty, None, vec!["finish", "wait", "permission_prompt"]),
        (vec![llm::Served::Finish, llm::Served::Wait], contract.clone(), None, vec!["wait", "permission_prompt"]),
        (
            vec![llm::Served::Finish, llm::Served::Deliver, host_tool(b"{}")],
            contract,
            delivery,
            vec!["deliver", "host_action", "permission_prompt"],
        ),
    ] {
        let mut backend = active();
        let Some(FromDomain::Complete { prompt, outcome: result, deliver: grant, .. }) = &mut backend.request else {
            panic!("request")
        };
        prompt.served = served.into_boxed_slice();
        *result = outcome.clone();
        *grant = deliver;
        let tools = backend.tools();
        let list = mcp::json::tool_list(&tools, backend.settings.mcp.answer_bytes, backend.settings.mcp.schema_bytes)
            .expect("inventory");
        let list: serde_json::Value = serde_json::from_slice(&list).expect("MCP list");
        assert_eq!(
            list["tools"]
                .as_array()
                .expect("tools")
                .iter()
                .map(|tool| tool["name"].as_str().expect("name"))
                .collect::<Vec<_>>(),
            expected_names
        );
        for tool in list["tools"].as_array().expect("tools") {
            assert!(tool["inputSchema"].is_object());
            for key in ["oneOf", "allOf", "anyOf"] {
                assert!(tool["inputSchema"].get(key).is_none());
            }
            assert_eq!(tool["inputSchema"]["type"], "object", "{}", tool["name"]);
        }
        let expected: serde_json::Value = serde_json::from_slice(
            &protocol::finish_schema(&outcome, backend.settings.contract_bytes).expect("finish schema"),
        )
        .expect("schema JSON");
        for tool in list["tools"].as_array().expect("tools").iter().filter(|tool| tool["name"] == "finish") {
            assert_eq!(tool["inputSchema"]["properties"]["outcome"]["oneOf"], expected["oneOf"]);
        }
    }
}

#[test]
fn a_host_schema_refusal_names_the_tool_at_startup() {
    let mut backend = active();
    let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request else { panic!("request") };
    prompt.served = Box::new([host_tool(br#"{"type":"string"}"#)]);
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.prepare_invocation(&env(0), &mut up, &mut below);
    failed(up.pop(), b"smith: MCP tool input schema is not an object: host_action");
    assert!(backend.server.is_none());
    for request in &below {
        if let Below::Io(io::Request::Listen { .. }) = request {
            panic!("schema refusal starts no listener");
        }
    }
}

#[test]
fn queued_built_in_results_keep_their_message_owners() {
    let mut backend = active();
    backend.owner = None;
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0), READ, &mut up, &mut below);
    backend.line(&env(0), RESULT, &mut up, &mut below);
    let next_call = read_record("second", "r2");
    let mut record: serde_json::Value = serde_json::from_slice(RESULT).expect("JSON");
    record["message"]["content"][0]["tool_use_id"] = serde_json::json!("r2");
    let next_result = serde_json::to_vec(&record).expect("JSON");
    backend.line(&env(0), &next_call, &mut up, &mut below);
    backend.line(&env(0), &next_result, &mut up, &mut below);
    backend.resume(&env(0), request(&backend), &mut up, &mut below);
    completed(up.pop(), COMPLETION);
    result_id(up.pop(), COMPLETION, b"r");
    assert!(up.is_empty());
    while backend.pop_measure().is_some() {}
    let mut next = request(&backend);
    let FromDomain::Complete { owner, .. } = &mut next else { panic!("Complete") };
    *owner = Token::new(81);
    backend.resume(&env(0), next, &mut up, &mut below);
    completed(up.pop(), Token::new(81));
    result_id(up.pop(), Token::new(81), b"r2");
    assert!(up.is_empty());
}

fn completed(event: Option<ToDomain>, expected: Token) {
    let Some(ToDomain::Completed { owner, .. }) = event else { panic!("completion") };
    assert_eq!(owner, expected);
}

fn result_id(event: Option<ToDomain>, expected: Token, expected_id: &[u8]) {
    let Some(ToDomain::BuiltinResult { owner, id, .. }) = event else { panic!("builtin result") };
    assert_eq!(owner, expected);
    assert_eq!(id.as_ref(), expected_id);
}

fn failed(event: Option<ToDomain>, words: &[u8]) {
    let Some(ToDomain::Failed { failure, detail, .. }) = event else { panic!("failure") };
    assert_eq!(failure, llm::Failure::Program { exit: None });
    assert_eq!(detail.as_ref(), words);
}

#[test]
fn an_open_catalog_has_wait_and_refuses_unoffered_finish() {
    let mut backend = active();
    let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request else { panic!("request") };
    prompt.served = Box::new([llm::Served::Wait]);
    let tools = backend.tools();
    let [waiting] = tools.as_ref() else { panic!("one offered tool") };
    assert_eq!(waiting.name.as_ref(), b"wait");
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0), br#"{"type":"assistant","message":{"id":"one","content":[{"type":"tool_use","id":"f","name":"mcp__smith__finish","input":{}}]}}"#, &mut up, &mut below);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else { panic!("observed completion") };
    let [llm::Said::ToolCall { call, .. }] = completion.content.as_ref() else { panic!("one unoffered call") };
    assert_eq!(*call, llm::Decoded::Invalid { problem: llm::Problem::UnknownTool });
}

fn structured_finish(output: serde_json::Value) -> llm::Decoded {
    let mut backend = active();
    backend.request = Some(report_request(&backend));
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    let line = serde_json::to_vec(
        &serde_json::json!({"type":"result","subtype":"success","is_error":false,"structured_output":output}),
    )
    .expect("result JSON");
    backend.line(&env(0), &line, &mut up, &mut below);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else { panic!("finish completion") };
    let mut blocks = completion.content.into_vec();
    let llm::Said::ToolCall { call, .. } = blocks.pop().expect("finish call") else { panic!("finish call") };
    assert!(blocks.is_empty());
    call
}

#[test]
fn a_wrapped_change_decodes_as_the_finish_declaration() {
    use smith_domain::run::{
        Ask,
        outcome::{Change, Declared},
    };
    assert_eq!(
        structured_finish(serde_json::json!({"outcome":{"form":"change","fields":{}}})),
        llm::Decoded::Served { ask: Ask::Finish { outcome: Declared::Change(Change { fields: Box::default() }) } }
    );
}

#[test]
fn a_wrapped_failure_decodes_as_the_finish_declaration() {
    use smith_domain::run::{
        Ask,
        outcome::{Declared, DeclaredFailure},
    };
    assert_eq!(
        structured_finish(serde_json::json!({"outcome":{"form":"failure","reason":"cannot proceed","fields":{}}})),
        llm::Decoded::Served {
            ask: Ask::Finish {
                outcome: Declared::Failure(DeclaredFailure {
                    reason: b"cannot proceed".as_slice().into(),
                    fields: Box::default()
                })
            }
        }
    );
}

#[test]
fn a_structured_result_requires_exactly_its_outcome_member() {
    for output in [
        serde_json::json!(null),
        serde_json::json!([]),
        serde_json::json!({}),
        serde_json::json!({"form":"report"}),
        serde_json::json!({"outcome":{},"extra":true}),
        serde_json::json!({"extra":true,"outcome":{}}),
    ] {
        let mut backend = active();
        backend.request = Some(report_request(&backend));
        let mut up = Queue::with_capacity(32);
        let mut below = Queue::with_capacity(64);
        let line =
            serde_json::to_vec(&serde_json::json!({"type":"result","is_error":false,"structured_output":output}))
                .expect("result JSON");
        backend.line(&env(0), &line, &mut up, &mut below);
        let Some(ToDomain::Failed { .. }) = up.pop() else { panic!("invalid wrapper fails") };
        assert_eq!(backend.state, State::Ending);
    }
}

#[test]
fn a_no_contract_mcp_finish_unwraps_its_outcome() {
    use smith_domain::run::{
        Ask,
        outcome::{Declared, Report},
    };
    let mut backend = active();
    let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request else { panic!("request") };
    prompt.served = Box::new([llm::Served::Finish]);
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0), br#"{"type":"assistant","message":{"id":"open","content":[{"type":"tool_use","id":"f","name":"mcp__smith__finish","input":{"outcome":{"form":"report","text":"done","fields":{}}}}]}}"#, &mut up, &mut below);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else { panic!("MCP finish") };
    let [llm::Said::ToolCall { call, .. }] = completion.content.as_ref() else { panic!("finish") };
    assert_eq!(
        *call,
        llm::Decoded::Served {
            ask: Ask::Finish {
                outcome: Declared::Report(Report { text: b"done".as_slice().into(), fields: Box::default() })
            }
        }
    );
}

#[test]
fn a_host_combinator_is_named_before_listening() {
    for key in ["oneOf", "allOf", "anyOf"] {
        let mut backend = active();
        let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request else { panic!("request") };
        let schema = serde_json::to_vec(&serde_json::json!({"type":"object",key:[]})).expect("schema");
        prompt.served = Box::new([host_tool(&schema)]);
        let mut up = Queue::with_capacity(32);
        let mut below = Queue::with_capacity(64);
        backend.prepare_invocation(&env(0), &mut up, &mut below);
        failed(up.pop(), b"smith: MCP tool input schema is not an object: host_action");
        assert!(backend.server.is_none());
    }
}

#[test]
fn launch_wraps_the_unchanged_contract_within_its_bound() {
    let mut backend = fixtures::resumable();
    backend.resuming = false;
    backend.request = Some(report_request(&backend));
    let spawn = backend.launch_spec().expect("launch");
    let flag = spawn.args.iter().position(|argument| argument.as_ref() == b"--json-schema").expect("schema flag");
    let schema: serde_json::Value = serde_json::from_slice(&spawn.args[flag + 1]).expect("schema");
    let Some(FromDomain::Complete { outcome, .. }) = &backend.request else { panic!("request") };
    let contract: serde_json::Value =
        serde_json::from_slice(&protocol::finish_schema(outcome, backend.settings.contract_bytes).expect("contract"))
            .expect("schema");
    assert_eq!(
        schema,
        serde_json::json!({"type":"object","properties":{"outcome":contract},"required":["outcome"],"additionalProperties":false})
    );
    let bound = u32::try_from(spawn.args[flag + 1].len()).expect("wrapped schema length");
    backend.settings.contract_bytes = bound;
    assert!(backend.launch_spec().is_some());
    backend.settings.contract_bytes = bound - 1;
    assert!(backend.launch_spec().is_none());
}

// Native snapshots are supplied by the file boundary, independently of the
// program's result text and the private startup files.
fn native_call(name: &str, input: serde_json::Value) -> Backend {
    let mut backend = active();
    backend.mounts = Box::new([Mount { root: Token::new(40), host: b"/work/repo".as_slice().into(), writable: true }]);
    if let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request {
        prompt.tools = Grants { inspect: true, modify: true, shell: false };
    }
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    let message = serde_json::json!({"type":"assistant","message":{"id":"native","content":[
        {"type":"tool_use","id":"n","name":name,"input":input}
    ]}});
    backend.line(&env(0), &serde_json::to_vec(&message).expect("native call"), &mut up, &mut below);
    completed(up.pop(), COMPLETION);
    while backend.pop_measure().is_some() {}
    backend.resume(&env(0), request(&backend), &mut up, &mut below);
    assert!(up.is_empty());
    if let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request {
        prompt.tools = Grants { inspect: true, modify: true, shell: false };
    }
    fixtures::connect_server(&mut backend);
    native_prompt(&mut backend, "n", name, input);
    backend
}

fn native_prompt(backend: &mut Backend, id: &str, name: &str, input: serde_json::Value) {
    let prompt = serde_json::json!({"jsonrpc":"2.0","id":1_u32,"method":"tools/call","params":{
        "name":"permission_prompt","arguments":{"tool_name":name,"input":input,"tool_use_id":id}
    }});
    backend.server.as_mut().expect("server").from_io(
        env(0).now,
        io::Event::Stream {
            owner: Token::new(101),
            up: skein_lib::stream::Up::Bytes(mcp::json::join(&[&serde_json::to_vec(&prompt).expect("prompt"), b"\n"])),
        },
        &mut backend.io,
    );
    let prompt = backend.server.as_mut().expect("server").up().expect("prompt");
    backend.permission(prompt);
}

fn allow_native(backend: &mut Backend, input: &serde_json::Value) {
    let reply = fixtures::server_reply(backend);
    let verdict: serde_json::Value =
        serde_json::from_str(reply["result"]["content"][0]["text"].as_str().expect("text")).expect("verdict");
    assert_eq!(verdict["behavior"], "allow");
    assert_eq!(&verdict["updatedInput"], input);
}

// lifecycle_tests supplies this stated whole-file limit; oversized is one byte past it.
const NATIVE_FILE_BYTES: u32 = 128;
const OVERSIZED_FILE_BYTES: u64 = 129;

fn native_load(backend: &mut Backend) {
    let mut below = Queue::with_capacity(64);
    backend.flush(&mut below);
    let loads: Vec<_> = below
        .iter()
        .filter_map(|request| match request {
            Below::File { request, .. } => Some(request),
            Below::Io(_) => None,
        })
        .collect();
    assert_eq!(loads.len(), 1, "serialized snapshot");
    assert_eq!(
        loads[0],
        &file::Request::Load {
            owner: backend.token(Owner::Native),
            root: Token::new(40),
            path: b"a".as_slice().into(),
            max: NATIVE_FILE_BYTES,
            no_follow: false
        }
    );
    assert!(backend.owns(backend.token(Owner::Native)));
}

fn native_version(event: Option<ToDomain>, owner: Token, bytes: &[u8]) {
    let Some(ToDomain::ReadVersion { owner: got, place, version }) = event else { panic!("version observation") };
    assert_eq!(got, owner);
    assert_eq!(place, smith_domain::tools::Place { root: Token::new(40), path: b"a".as_slice().into() });
    assert_eq!(version, smith_domain::tools::Version::new(skein_io::digest::digest(bytes).0));
}

fn native_result(backend: &mut Backend, error: bool, up: &mut Queue<ToDomain>) {
    let result = serde_json::json!({"type":"user","message":{"content":[
        {"type":"tool_result","tool_use_id":"n","content":"program text","is_error":error}
    ]}});
    backend.line(&env(0), &serde_json::to_vec(&result).expect("result"), up, &mut Queue::with_capacity(64));
}

#[test]
fn a_whole_native_read_snapshots_before_allow() {
    let input = serde_json::json!({"file_path":"./x/../a"});
    let mut backend = native_call("Read", input.clone());
    for request in &backend.io {
        if let io::Request::Output { .. } = request {
            panic!("no Allow before file terminal");
        }
    }
    native_load(&mut backend);
    let mut up = Queue::with_capacity(32);
    backend.native_event(
        file::Event::Loaded { owner: backend.token(Owner::Native), bytes: b"file bytes".as_slice().into() },
        &mut up,
    );
    native_version(up.pop(), COMPLETION, b"file bytes");
    assert!(up.is_empty());
    allow_native(&mut backend, &input);
}

#[test]
fn a_partial_native_read_never_snapshots_even_with_null_fields() {
    for field in ["offset", "limit"] {
        for value in [serde_json::json!(1_u32), serde_json::Value::Null] {
            let mut input = serde_json::json!({"file_path":"a"});
            input[field] = value;
            let mut backend = native_call("Read", input.clone());
            assert!(backend.native.is_none());
            allow_native(&mut backend, &input);
            let mut up = Queue::with_capacity(32);
            native_result(&mut backend, false, &mut up);
            result_id(up.pop(), COMPLETION, b"n");
            assert!(up.is_empty(), "no observation that could replace earlier knowledge");
        }
    }
}

#[test]
fn native_writes_and_edits_snapshot_after_their_results_including_errors() {
    for name in ["Write", "Edit"] {
        for error in [false, true] {
            let input = serde_json::json!({"file_path":"a"});
            let mut backend = native_call(name, input.clone());
            assert!(backend.native.is_none(), "no snapshot before result");
            allow_native(&mut backend, &input);
            let mut up = Queue::with_capacity(32);
            native_result(&mut backend, error, &mut up);
            assert!(up.is_empty(), "result waits for snapshot");
            native_load(&mut backend);
            backend.native_event(
                file::Event::Loaded { owner: backend.token(Owner::Native), bytes: b"stored bytes".as_slice().into() },
                &mut up,
            );
            native_version(up.pop(), COMPLETION, b"stored bytes");
            let Some(ToDomain::BuiltinResult { text, error: got, .. }) = up.pop() else { panic!("result") };
            assert_eq!(text.as_ref(), b"program text");
            assert_eq!(got, error);
            assert!(up.is_empty());
        }
    }
}

const SNAPSHOT_ERRORS: [kernel::Error; 3] =
    [kernel::Error::NotFound, kernel::Error::Permission, kernel::Error::IsADirectory];

fn fail_native_load(backend: &mut Backend, failure: kernel::Error, up: &mut Queue<ToDomain>) {
    native_load(backend);
    backend.native_event(
        file::Event::Failed { owner: backend.token(Owner::Native), error: failure, committed: false, residue: None },
        up,
    );
}

#[test]
fn failed_native_read_snapshots_preserve_allow() {
    for failure in SNAPSHOT_ERRORS {
        let input = serde_json::json!({"file_path":"a"});
        let mut backend = native_call("Read", input.clone());
        let mut up = Queue::with_capacity(32);
        fail_native_load(&mut backend, failure, &mut up);
        assert!(up.is_empty());
        allow_native(&mut backend, &input);
    }
    let input = serde_json::json!({"file_path":"a"});
    let mut backend = native_call("Read", input.clone());
    native_load(&mut backend);
    let mut up = Queue::with_capacity(32);
    backend.native_event(
        file::Event::TooLarge { owner: backend.token(Owner::Native), size: OVERSIZED_FILE_BYTES },
        &mut up,
    );
    assert!(up.is_empty());
    allow_native(&mut backend, &input);
}

#[test]
fn failed_native_write_snapshots_preserve_result_text() {
    for failure in SNAPSHOT_ERRORS {
        let input = serde_json::json!({"file_path":"a"});
        let mut backend = native_call("Write", input.clone());
        allow_native(&mut backend, &input);
        let mut up = Queue::with_capacity(32);
        native_result(&mut backend, true, &mut up);
        fail_native_load(&mut backend, failure, &mut up);
        let Some(ToDomain::BuiltinResult { text, error, .. }) = up.pop() else { panic!("unchanged result") };
        assert_eq!(text.as_ref(), b"program text");
        assert!(error);
        assert!(up.is_empty());
    }
}

#[test]
fn closing_settles_the_native_load_before_private_cleanup() {
    let mut backend = native_call("Read", serde_json::json!({"file_path":"a"}));
    native_load(&mut backend);
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.close(&env(0), &mut up, &mut below);
    let Some(ToDomain::Cancelled { .. }) = up.pop() else { panic!("cancel terminal") };
    backend.settings.directory = None;
    backend.cleanup(env(0).now, &mut below);
    assert!(!backend.closed(), "snapshot still owes its terminal");
    backend.native_event(
        file::Event::Loaded { owner: backend.token(Owner::Native), bytes: b"cancelled".as_slice().into() },
        &mut up,
    );
    assert!(up.is_empty(), "cancelled snapshot teaches nothing");
    backend.cleanup(env(0).now, &mut below);
    assert!(backend.closed());
}

#[test]
fn queued_native_snapshots_keep_result_order_and_message_owners() {
    let input = serde_json::json!({"file_path":"a"});
    let mut backend = native_call("Write", input.clone());
    allow_native(&mut backend, &input);
    let mut second = reply_tests::call(b"second");
    second.builtin = true;
    second.owner = Some(Token::new(81));
    backend.ledger.push(second).ok().expect("second call");
    backend
        .native_writes
        .insert(
            b"second".as_slice().into(),
            smith_domain::tools::Place { root: Token::new(40), path: b"a".as_slice().into() },
        )
        .expect("second allowed write");
    let results = serde_json::json!({"type":"user","message":{"content":[
        {"type":"tool_result","tool_use_id":"n","content":"first text"},
        {"type":"tool_result","tool_use_id":"second","content":"second text","is_error":true}
    ]}});
    let mut up = Queue::with_capacity(32);
    backend.line(&env(0), &serde_json::to_vec(&results).expect("results"), &mut up, &mut Queue::with_capacity(64));
    assert!(up.is_empty());
    native_load(&mut backend);
    backend.native_event(
        file::Event::Loaded { owner: backend.token(Owner::Native), bytes: b"first version".as_slice().into() },
        &mut up,
    );
    native_version(up.pop(), COMPLETION, b"first version");
    result_id(up.pop(), COMPLETION, b"n");
    assert!(up.is_empty(), "the second result waits behind its own snapshot");
    native_load(&mut backend);
    backend.native_event(
        file::Event::Loaded { owner: backend.token(Owner::Native), bytes: b"second version".as_slice().into() },
        &mut up,
    );
    native_version(up.pop(), Token::new(81), b"second version");
    let Some(ToDomain::BuiltinResult { owner, id, text, error }) = up.pop() else { panic!("second result") };
    assert_eq!(owner, Token::new(81));
    assert_eq!(id.as_ref(), b"second");
    assert_eq!(text.as_ref(), b"second text");
    assert!(error);
    assert!(up.is_empty());
}

fn server_call(backend: &mut Backend, id: u32) {
    let call =
        serde_json::json!({"jsonrpc":"2.0", "id":id, "method":"tools/call", "params":{"name":"foo", "arguments":{}}});
    backend.server.as_mut().expect("server").from_io(
        env(0).now,
        io::Event::Stream {
            owner: Token::new(101),
            up: skein_lib::stream::Up::Bytes(mcp::json::join(&[&serde_json::to_vec(&call).expect("RPC"), b"\n"])),
        },
        &mut backend.io,
    );
    let call = backend.server.as_mut().expect("server").up().expect("RPC");
    backend.calls.push(call).expect("held RPC");
    backend.match_calls();
}

fn after_partial_result() -> Backend {
    let input = serde_json::json!({"file_path":"a", "limit":1_u32});
    let mut backend = native_call("Read", input.clone());
    allow_native(&mut backend, &input);
    let mut up = Queue::with_capacity(32);
    native_result(&mut backend, false, &mut up);
    server_call(&mut backend, 2);
    backend
}

#[test]
fn an_owned_remove_result_reaches_its_mcp_caller() {
    use smith_domain::tools::Outcome;
    for (outcome, expected, error) in [
        (Outcome::Removed, "removed", false),
        (Outcome::NotRead, "read the file whole", true),
        (Outcome::Stale, "file changed", true),
    ] {
        let mut backend = active();
        let mut call = reply_tests::call(b"owned");
        call.name = b"foo".as_slice().into();
        backend.ledger.push(call).ok().expect("owned call");
        fixtures::connect_server(&mut backend);
        server_call(&mut backend, 1);
        let mut next = request(&backend);
        let FromDomain::Complete { prompt, .. } = &mut next else { panic!("complete") };
        prompt.messages = Box::new([llm::Message {
            role: llm::Role::User,
            content: Box::new([llm::Block::ToolResult {
                id: b"owned".as_slice().into(),
                result: llm::Returned::Owned { outcome },
            }]),
        }]);
        backend.resume(&env(0), next, &mut Queue::with_capacity(32), &mut Queue::with_capacity(64));
        let reply = fixtures::server_reply(&mut backend);
        assert!(reply["result"]["content"][0]["text"].as_str().expect("result text").contains(expected));
        assert_eq!(reply["result"]["isError"], error);
    }
}

#[test]
fn an_mcp_successor_may_arrive_before_its_next_assistant_record() {
    let mut backend = after_partial_result();
    let mut up = Queue::with_capacity(32);
    assert_eq!(backend.calls.len(), 1, "RPC waits for its next assistant record");
    backend.line(&env(0), br#"{"type":"assistant","message":{"id":"next-message","content":[{"type":"tool_use","id":"next","name":"mcp__smith__foo","input":{}}]}}"#,
        &mut up, &mut Queue::with_capacity(64));
    assert!(backend.calls.is_empty(), "the announcement correlates the RPC");
    completed(up.pop(), COMPLETION);
    let mut next = request(&backend);
    let FromDomain::Complete { prompt, .. } = &mut next else { panic!("complete") };
    prompt.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::ToolResult {
            id: b"next".as_slice().into(),
            result: llm::Returned::Text { text: b"found".as_slice().into(), error: false, replay: None },
        }]),
    }]);
    backend.resume(&env(0), next, &mut up, &mut Queue::with_capacity(64));
    let reply = fixtures::server_reply(&mut backend);
    assert_eq!(reply["id"], 2_u32);
    assert_eq!(reply["result"]["content"][0]["text"], "found");
    assert_eq!(reply["result"]["isError"], false);
}

#[test]
fn an_unannounced_mcp_successor_is_refused_when_the_turn_ends() {
    let mut backend = after_partial_result();
    let mut up = Queue::with_capacity(32);
    assert_eq!(backend.calls.len(), 1);
    backend.line(&env(0), br#"{"type":"result","is_error":false}"#, &mut up, &mut Queue::with_capacity(64));
    assert!(backend.calls.is_empty());
    let reply = fixtures::server_reply(&mut backend);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["content"][0]["text"], "smith: this call was not in the model's message");
}

fn read_record(message: &str, call: &str) -> Vec<u8> {
    let mut record: serde_json::Value = serde_json::from_slice(READ).expect("JSON");
    record["message"]["id"] = serde_json::json!(message);
    record["message"]["content"][0]["id"] = serde_json::json!(call);
    serde_json::to_vec(&record).expect("JSON")
}
