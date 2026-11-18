//! Local-agent composition at the startup/store/wire boundaries (protocol/configuration/limits.md).
#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "ordinary tests edit configuration fixtures")]
#![expect(clippy::disallowed_macros, reason = "ordinary tests format explicit scalar overrides")]
use super::*;
use crate::tests::fixtures;

fn document(engine: Option<u32>) -> fixtures::Start {
    let mut input = fixtures::document();
    input.jig.core.call_answer_bytes = engine.unwrap_or(config::stock().core.call_answer_bytes);
    input
}

#[test]
fn omitted_call_answers_reach_the_core_store_and_both_wire_sides() {
    let directories = config::Directories::working();
    for (engine, payload, expected_engine) in
        [(None, 4096, 135_168), (Some(4096), 4096, 4096), (Some(32_768), 32_768, 32_768)]
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
    let mut input = fixtures::engine_alone_document();
    input.jig.core.call_answer_bytes = config::stock().core.call_answer_bytes;
    let agent = fixtures::startup_agent();
    let declaration = agent.declaration();
    for metadata in [None, Some(&declaration)] {
        assert_eq!(store_limits(&input, &directories, metadata).unwrap().store.core.call_answer_bytes, 135_168);
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

fn standalone_refusals(input: &(impl fixtures::Input + ?Sized), agent: &settings::Agent) -> [Problems; 3] {
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
    let mut short = fixtures::engine_alone_document();
    short.jig.core.call_answer_bytes = 2047;
    let mut agent = fixtures::startup_agent();
    agent.tool_payload = 1;
    let expected = answer_floor_line("engine.limits.overrides.core.call_answer_bytes", 2047);
    for found in standalone_refusals(&short, &agent) {
        assert_eq!(core::str::from_utf8(&lines(&found)).unwrap(), expected);
    }
    let found = store_limits(&short, &config::Directories::working(), None).unwrap_err();
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
    let mut agent = fixtures::startup_agent_with_spend(u64::MAX);
    agent.tool_payload = 4096;
    agent.bounds.message_bytes = 4094;
    let mut input = document(None);
    let sources = temper_service_core::derived::Sources::agent(&agent.declaration());
    let (jig, own) = crate::limits::base(40, 4096, sources).unwrap();
    input.jig.core.tasks.message_bytes = jig.core.tasks.message_bytes;
    input.jig.core.tasks.result_bytes = jig.core.tasks.result_bytes;
    input.jig.core.tasks.inbox_bytes = jig.core.tasks.inbox_bytes;
    input.own.checkout.message_bytes = own.checkout.message_bytes;
    input.own.forge.client.op_bytes = own.forge.client.op_bytes;
    input.own.forge_component.body_bytes = own.forge_component.body_bytes;
    input.jig.hub.event_bytes = jig.hub.event_bytes;
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
    assert_eq!(checked.engine.jig.limits.hub.event_bytes, 4094);
    let (worker, _) = checked.worker.as_ref().unwrap();
    assert_eq!(worker.jig.limits.as_ref().unwrap().hub.event_bytes, 4094);
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

fn checked_start(
    input: &(impl fixtures::Input + ?Sized),
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<CheckedStart, Problems> {
    let input = fixtures::Input::configured(input);
    checked_start_with_limits(
        &input,
        directories,
        files,
        agent,
        seeds,
        forges,
        setup,
        &crate::stock::Given::Stock,
        None,
        &input.jig,
        &input.own,
    )
}
#[expect(clippy::too_many_arguments, reason = "the same startup inputs with typed fixture figures")]
fn startup(
    input: &(impl fixtures::Input + ?Sized),
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: WorkerRuntime,
    forges: List<crate::ForgeRuntime>,
) -> Result<Service, Problems> {
    let checked = checked_start(input, directories, files, Some(agent), seeds, forges, None)?;
    from_checked(checked, files, Some(agent), seeds, accounts, Some(runtime), None, None, Box::new([]))
}
#[expect(clippy::too_many_arguments, reason = "the same startup checks with typed fixture figures")]
fn check(
    input: &(impl fixtures::Input + ?Sized),
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    _accounts: Option<&AccountsRuntime>,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<(), Problems> {
    checked_start(input, directories, files, Some(agent), seeds, forges, setup)?;
    Ok(())
}
fn store_limits(
    input: &(impl fixtures::Input + ?Sized),
    directories: &config::Directories,
    agent: Option<&Declaration>,
) -> Result<StoreLimits, Problems> {
    fixtures::document_store_limits(input, directories, agent)
}
fn parse(input: &(impl fixtures::Input + ?Sized), directories: &config::Directories) -> Result<Parsed, Problems> {
    let input = fixtures::Input::configured(input);
    Ok(load_with_limits(
        &input,
        directories,
        true,
        &crate::stock::Given::Stock,
        temper_service_core::derived::STOCK,
        Some((&input.jig, &input.own)),
    )?
    .1
    .unwrap())
}
