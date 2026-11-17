use crate::{agent, worker};
use serde_json::json;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
fn stock_figures() -> jig_protocol_worker_wire::Figures {
    let input: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/bare.json")).expect("stock document");
    configured_figures(&input)
}
fn configured_figures(input: &serde_json::Value) -> jig_protocol_worker_wire::Figures {
    let input = serde_json::to_vec(&input["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(
        &input,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
    )
    .expect("engine document");
    let engine = temper_service::startup::engine(&document, 0, &jig_protocol_config::Directories::working())
        .expect("stock engine");
    let own = temper_limits_fixtures::small();
    temper_engine_service_vocabulary::figures::Composer::new(&engine, &own).compose(&temper_service::stock::agents())
}
#[test]
fn a_boxs_demo_endpoint_keeps_its_name_and_uses_smiths_openai_provider() {
    let mut input: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/temper.json")).expect("box document");
    input["engine"]["clients"]["provider"] = serde_json::Value::Null;
    input["engine"]["llm"]["endpoints"][3]["address"]["socket"] = "127.0.0.1:8370".into();
    let figures = configured_figures(&input);
    assert_eq!(figures.agents().endpoints().get(3).expect("demo endpoint").provider(), b"demo");
    let scratch = skein_scratch::Scratch::new("demo-agent-endpoint");
    let bytes = agent::figures::configuration(&figures, None, scratch.path(), &[]).expect("written agent");
    let written: serde_json::Value = serde_json::from_slice(&bytes).expect("agent JSON");
    assert_eq!(written["endpoints"][3]["name"], "demo");
    assert_eq!(written["endpoints"][3]["number"], 3);
    assert_eq!(written["endpoints"][3]["dialect"], 3);
    assert_eq!(written["endpoints"][3]["provider"], "openai");
    let _decoded = agent::decoded_bytes(&bytes).expect("smith reads every box endpoint");
}
#[test]
fn written_figures_decode_as_the_stock_agent_and_keep_machine_members() {
    let scratch = skein_scratch::Scratch::new("written-agent");
    let data = scratch.path();
    let figures = stock_figures();
    for explicit in [false, true] {
        let claude = explicit.then(|| data.join("claude"));
        let environment = if explicit {
            vec![(Box::from(b"WORKER_ENV".as_slice()), Box::from(b"given".as_slice()))]
        } else {
            Vec::new()
        };
        let bytes =
            agent::figures::configuration(&figures, claude.as_deref(), data, &environment).expect("written agent");
        let written: serde_json::Value = serde_json::from_slice(&bytes).expect("written JSON");
        let _decoded = agent::decoded_bytes(&bytes).expect("smith reads written bytes");
        assert_eq!(written["profile"]["policy"]["max_turns"], 800);
        assert_eq!(written["endpoints"][0]["name"], "openai");
        assert_eq!(
            written["programs"],
            json!({"claude_code":{"path":claude.as_ref().map_or("fetched",|path| path.to_str().expect("path")),"directory":data.join("programs")}})
        );
        assert_eq!(
            written["environment"],
            if explicit { json!([{"name":"WORKER_ENV","value":"given"}]) } else { json!([]) }
        );
    }
}
#[test]
fn worker_inputs_refuse_the_old_file_and_validate_derived_program_and_claude() {
    let scratch = skein_scratch::Scratch::new("worker-inputs");
    let data = scratch.path();
    let (_, document) = super::worker_document(data);
    let directories = super::configuration::directories(data);
    let mut part = temper_worker_service::startup::worker_part(
        &serde_json::to_vec(&document).expect("JSON"),
        &temper_service_shell::jig_directories(&directories),
        None,
    )
    .expect("worker part");
    let executable = data.join("temper-worker");
    let program = data.join("temper-agent");
    let why = worker::inputs(&mut part, &directories, &executable).expect_err("missing derived program");
    assert!(why.contains(program.to_str().expect("path")), "{why}");
    fs::write(&program, b"agent").expect("program");
    assert!(
        worker::inputs(&mut part, &directories, &executable).expect_err("not executable").contains("executable file")
    );
    fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).expect("executable");
    worker::inputs(&mut part, &directories, &executable).expect("derived agent");
    assert_eq!(part.named.program.as_ref(), program.as_os_str().as_bytes());
    assert_eq!(part.named.arguments.as_ref(), [part.named.agent.clone()]);
    part.claude = Some(data.join("missing-claude").as_os_str().as_bytes().into());
    assert!(
        worker::inputs(&mut part, &directories, &executable)
            .expect_err("missing claude")
            .starts_with("application.workers[0].claude:")
    );
    part.claude = Some(program.as_os_str().as_bytes().into());
    worker::inputs(&mut part, &directories, &executable).expect("explicit executable");
    fs::write(data.join("agent.json"), b"never read").expect("old file");
    assert_eq!(
        worker::inputs(&mut part, &directories, &executable).expect_err("moved file"),
        "agent.json: moved: temper writes the agent's configuration under the worker's root; remove it"
    );
}
#[test]
fn claude_lookup_requires_an_absolute_path_entry_and_executable_file() {
    let scratch = skein_scratch::Scratch::new("claude-path");
    let path = [(Box::from(b"PATH".as_slice()), Box::from(scratch.path().as_os_str().as_bytes()))];
    assert_eq!(worker::find_claude(&[]), None);
    let claude = scratch.path().join("claude");
    fs::write(&claude, b"claude").expect("claude");
    assert_eq!(worker::find_claude(&path), None);
    fs::set_permissions(&claude, fs::Permissions::from_mode(0o700)).expect("executable");
    assert_eq!(worker::find_claude(&path), Some(claude));
    assert_eq!(worker::find_claude(&[(Box::from(b"PATH".as_slice()), Box::from(b"relative".as_slice()))]), None);
}

#[test]
fn tls_figures_write_smiths_trust_and_keep_endpoint_details() {
    use jig_protocol_worker_wire as w;
    let mut parts = stock_figures().into_parts();
    let mut agents = parts.agents.into_parts();
    let endpoint = agents.endpoints.get_mut(0).expect("endpoint");
    let mut value = endpoint.clone().into_parts();
    value.host = b"::1".as_slice().into();
    value.port = 1234;
    value.program = Some(b"claude_code".as_slice().into());
    value.headers = skein_lib::List::with_capacity(1);
    value
        .headers
        .push(
            w::Header::new(
                &w::CEILINGS,
                w::HeaderParts { name: b"x-fixture".as_slice().into(), value: b"given".as_slice().into() },
            )
            .expect("header"),
        )
        .expect("one header");
    value.transport = w::Transport::Tls(
        w::Tls::new(
            &w::CEILINGS,
            w::TlsParts {
                server_name: b"skein.test".as_slice().into(),
                trust: Some(b"/fixture/root.der".as_slice().into()),
            },
        )
        .expect("TLS"),
    );
    let mut limits = value.limits.into_parts();
    limits.oversized = w::Oversized::Drop;
    value.limits = w::EndpointLimits::new(&w::CEILINGS, limits).expect("limits");
    *endpoint = w::AgentEndpoint::new(&w::CEILINGS, value).expect("endpoint");
    parts.agents = w::Agents::new(&w::CEILINGS, agents).expect("agents");
    let bytes = agent::figures::configuration(
        &w::Figures::new(&w::CEILINGS, parts).expect("figures"),
        None,
        std::path::Path::new("/fixture"),
        &[],
    )
    .expect("written agent");
    let _decoded = agent::decoded_bytes(&bytes).expect("smith reads endpoint trust");
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("JSON");
    let endpoint = &document["endpoints"][0];
    assert_eq!(endpoint["address"], "[::1]:1234");
    assert_eq!(endpoint["trust_der"], "/fixture/root.der");
    assert_eq!(endpoint["server_name"], "skein.test");
    assert_eq!(endpoint["headers"], json!([{"name":"x-fixture","value":"given"}]));
    assert_eq!(endpoint["program"], "claude_code");
    assert_eq!(endpoint["limits"]["oversized_reasoning"], "drop");
}
