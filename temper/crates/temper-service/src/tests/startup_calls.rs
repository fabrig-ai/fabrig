//! Local-agent composition at the startup/store/wire boundaries (protocol/configuration/limits.md).
#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "ordinary tests edit configuration fixtures")]
#![expect(clippy::disallowed_macros, reason = "ordinary tests format explicit scalar overrides")]
use super::*;
use crate::tests::fixtures;
use core::fmt::Write;

fn document(engine: Option<u32>) -> Box<[u8]> {
    let input = fixtures::document();
    let remaining = core::str::from_utf8(&input).unwrap();
    let (before, after) = remaining.split_once(",\"core.call_answer_bytes\":2048").unwrap();
    let mut text = String::from(before);
    if let Some(bytes) = engine {
        write!(text, ",\"core.call_answer_bytes\":{bytes}").unwrap();
    }
    text.push_str(after);
    text.into_bytes().into_boxed_slice()
}

#[test]
fn omitted_call_answers_reach_the_core_store_and_both_wire_sides() {
    let directories = config::Directories::working();
    for (engine, payload, expected_engine) in
        [(None, 4096, 69_632), (Some(4096), 4096, 4096), (Some(32_768), 32_768, 32_768)]
    {
        let expected_worker = expected_engine;
        let input = document(engine);
        let mut agent = fixtures::startup_agent();
        agent.tool_payload = payload;
        let checked = checked_start(
            &input,
            &directories,
            &fixtures::files(),
            Some(&agent),
            &fixtures::seeds(),
            fixtures::forge_runtime(),
            None,
        )
        .unwrap();
        assert_eq!(checked.engine.jig.limits.core.call_answer_bytes, expected_engine);
        let wire = crate::workers::limits(&checked.engine).unwrap();
        assert_eq!((wire.sizes.call_input, wire.sizes.answer), (expected_engine, expected_engine));
        let (worker, _) = checked.worker.as_ref().unwrap();
        let payload = crate::worker_settings::payload_limits(&crate::limits::worker(&checked.engine.limits)).unwrap();
        let wire =
            crate::worker_settings::link_limits(&worker.jig, &payload, fixtures::worker_limits().peer.forges).unwrap();
        assert_eq!((wire.sizes.call_input, wire.sizes.answer), (expected_worker, expected_worker));
        let service = startup(
            &input,
            &directories,
            &fixtures::files(),
            &agent,
            &fixtures::seeds(),
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        )
        .unwrap();
        let engine = service.engine.as_ref().unwrap();
        assert_eq!(engine.limits().call_answer_bytes, expected_engine);
        assert_eq!(engine.limits().host_reply_bytes, Some(agent.limits.bodies.host_reply_text));
        let opening = store_limits(&input, &directories, Some(&agent.declaration())).unwrap();
        assert_eq!(opening.store, engine.opening_limits());
        assert_eq!(opening.store.core.call_answer_bytes, expected_engine);
        check(
            &input,
            &directories,
            &fixtures::files(),
            &agent,
            &fixtures::seeds(),
            None,
            fixtures::forge_runtime(),
            None,
        )
        .unwrap();
    }
}

#[test]
fn store_readers_require_local_agent_metadata_but_leave_remote_engines_configured() {
    let directories = config::Directories::working();
    let found = store_limits(&document(None), &directories, None).unwrap_err();
    assert_eq!(found.problems.get(0).unwrap().path, indexed(0).member(b"agent"));
    assert_eq!(found.problems.get(0).unwrap().what, What::Half(EngineWhat::Agent));
    let input = fixtures::engine_alone_document();
    let input = core::str::from_utf8(&input).unwrap().replace(",\"core.call_answer_bytes\":2048", "");
    let agent = fixtures::startup_agent();
    let declaration = agent.declaration();
    for metadata in [None, Some(&declaration)] {
        assert_eq!(
            store_limits(input.as_bytes(), &directories, metadata).unwrap().store.core.call_answer_bytes,
            135_168
        );
    }
}

fn call_input_lines(path: &str, bytes: u32, input: u32, ask: u32, reply: u32) -> String {
    format!(
        "{path}: {bytes} bytes, short of the agent's call input (profile.declared.tool_payload), {input} bytes; raise it to at least {input} bytes, or omit it to follow the agent's call input\n{path}: call input {ask} bytes; wire answer {bytes} bytes; smith host_reply_bytes {reply} bytes\n"
    )
}

fn answer_floor_line(path: &str, bytes: u32) -> String {
    format!("{path}: {bytes} bytes, short of temper's documentation answer (read_docs), 2048 bytes\n")
}

fn standalone_refusals(input: &[u8], agent: &settings::Agent) -> [Problems; 3] {
    let directories = config::Directories::working();
    [
        startup(
            input,
            &directories,
            &fixtures::files(),
            agent,
            &fixtures::seeds(),
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        )
        .err()
        .unwrap(),
        store_limits(input, &directories, Some(&agent.declaration())).unwrap_err(),
        check(
            input,
            &directories,
            &fixtures::files(),
            agent,
            &fixtures::seeds(),
            None,
            fixtures::forge_runtime(),
            None,
        )
        .unwrap_err(),
    ]
}

fn core_answer_floor() {
    let agent = fixtures::startup_agent();
    let input = document(Some(2047));
    let path = "engine.limits.overrides.core.call_answer_bytes";
    let expected = call_input_lines(path, 2047, 2048, 4096, 65_536) + &answer_floor_line(path, 2047);
    for found in standalone_refusals(&input, &agent) {
        assert_eq!(core::str::from_utf8(&lines(&found)).unwrap(), expected);
    }
    let service = startup(
        &document(Some(2048)),
        &config::Directories::working(),
        &fixtures::files(),
        &agent,
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    )
    .unwrap();
    assert_eq!(service.engine.as_ref().unwrap().limits().call_answer_bytes, 2048);
}

#[test]
fn the_engines_call_answer_bound_must_hold_the_documentation_answer() {
    core_answer_floor();
}

#[test]
fn an_engine_without_a_local_agent_still_needs_the_documentation_answer_bound() {
    let input = fixtures::engine_alone_document();
    let short =
        core::str::from_utf8(&input).unwrap().replace("core.call_answer_bytes\":2048", "core.call_answer_bytes\":2047");
    let mut agent = fixtures::startup_agent();
    agent.tool_payload = 1;
    let expected = answer_floor_line("engine.limits.overrides.core.call_answer_bytes", 2047);
    for found in standalone_refusals(short.as_bytes(), &agent) {
        assert_eq!(core::str::from_utf8(&lines(&found)).unwrap(), expected);
    }
    let found = store_limits(short.as_bytes(), &config::Directories::working(), None).unwrap_err();
    assert_eq!(core::str::from_utf8(&lines(&found)).unwrap(), expected);
}

#[test]
fn explicit_short_call_inputs_refuse_every_standalone_entrance() {
    let directories = config::Directories::working();
    for (engine, payload) in [(4095, 4096), (2048, 4096), (32_768, 40_960)] {
        let input = document(Some(engine));
        let mut agent = fixtures::startup_agent();
        agent.tool_payload = payload;
        let expected =
            call_input_lines("engine.limits.overrides.core.call_answer_bytes", engine, payload, 4096, 65_536);
        let found = [
            startup(
                &input,
                &directories,
                &fixtures::files(),
                &agent,
                &fixtures::seeds(),
                None,
                fixtures::worker_runtime(),
                fixtures::forge_runtime(),
            )
            .err()
            .unwrap(),
            store_limits(&input, &directories, Some(&agent.declaration())).unwrap_err(),
            check(
                &input,
                &directories,
                &fixtures::files(),
                &agent,
                &fixtures::seeds(),
                None,
                fixtures::forge_runtime(),
                None,
            )
            .unwrap_err(),
        ];
        for problems in found {
            assert_eq!(core::str::from_utf8(&lines(&problems)).unwrap(), expected);
            assert_eq!(problems.problems.len(), 1);
        }
        // A refusal preserves each configured value, rather than silently deriving it.
        let mut parsed = parse(&input, &directories).unwrap();
        parsed.compose(&agent.declaration()).unwrap_err();
        assert_eq!(parsed.engine.limits.core.call_answer_bytes, engine);
    }
}

#[test]
fn an_engine_without_a_local_agent_keeps_its_explicit_call_bound() {
    let input = fixtures::engine_alone_document();
    let directories = config::Directories::working();
    let mut agent = fixtures::startup_agent();
    for payload in [1, u32::MAX] {
        agent.tool_payload = payload;
        let checked = checked_start(
            &input,
            &directories,
            &fixtures::files(),
            Some(&agent),
            &fixtures::seeds(),
            fixtures::forge_runtime(),
            None,
        )
        .unwrap();
        assert_eq!(checked.engine.jig.limits.core.call_answer_bytes, 2048);
        assert_eq!(checked.engine.jig.limits.core.tasks.message_bytes, 512);
        assert_eq!(
            store_limits(&input, &directories, Some(&agent.declaration())).unwrap().store.core.tasks.message_bytes,
            512
        );
        assert_eq!(
            store_limits(&input, &directories, Some(&agent.declaration())).unwrap().store.core.call_answer_bytes,
            2048
        );
    }
}

#[test]
fn a_local_agents_tool_payload_must_hold_the_documentation_answer() {
    let mut agent = fixtures::startup_agent();
    agent.tool_payload = 2047;
    let expected = answer_floor_line("workers[0].agent.profile.declared.tool_payload", 2047);
    for found in standalone_refusals(&document(Some(2048)), &agent) {
        assert_eq!(core::str::from_utf8(&lines(&found)).unwrap(), expected);
    }
    agent.tool_payload = 2048;
    let directories = config::Directories::working();
    startup(
        &document(Some(2048)),
        &directories,
        &fixtures::files(),
        &agent,
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    )
    .unwrap();
}

#[test]
fn a_local_worker_derives_from_its_agent_instead_of_the_stock_agent() {
    let mut input: serde_json::Value = serde_json::from_slice(&document(None)).unwrap();
    for limits in ["/engine/limits/overrides", "/workers/0/limits/overrides"] {
        let overrides = input.pointer_mut(limits).unwrap().as_object_mut().unwrap();
        for name in ["tasks.message_bytes", "tasks.result_bytes", "tasks.inbox_bytes", "hub.event_bytes"] {
            overrides.remove(name);
        }
    }
    let overrides = input["application"]["limits"]["overrides"].as_object_mut().unwrap();
    for name in ["forge.client.op_bytes", "forge_component.body_bytes", "checkout.message_bytes"] {
        overrides.remove(name);
    }
    // This story varies payload carriers, not the agent's spend admission.
    let mut agent = fixtures::startup_agent_with_spend(u64::MAX);
    agent.tool_payload = 4096;
    agent.bounds.message_bytes = 4094;
    let input = serde_json::to_vec(&input).unwrap();
    let checked = checked_start(
        &input,
        &config::Directories::working(),
        &fixtures::files(),
        Some(&agent),
        &fixtures::seeds(),
        fixtures::forge_runtime(),
        None,
    )
    .unwrap();
    assert_eq!(checked.engine.jig.limits.core.tasks.message_bytes, 4094);
    assert_eq!(checked.engine.limits.checkout.message_bytes, 4096);
    assert_eq!(
        store_limits(&input, &config::Directories::working(), Some(&agent.declaration()))
            .unwrap()
            .store
            .core
            .tasks
            .message_bytes,
        4094
    );
}
