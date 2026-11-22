use crate::{agent, worker};
use serde_json::json;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
fn stock_figures() -> jig_protocol_worker_wire::Figures {
    let input: serde_json::Value =
        serde_json::from_slice(&temper_config_shell::init_document(false)).expect("stock document");
    configured_figures(&input)
}
fn configured_figures(input: &serde_json::Value) -> jig_protocol_worker_wire::Figures {
    configured_figures_with(input, &temper_service::stock::Given::Stock)
}
fn configured_figures_with(
    input: &serde_json::Value,
    given: &temper_service::stock::Given,
) -> jig_protocol_worker_wire::Figures {
    let input = serde_json::to_vec(&input["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(
        &input,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
    )
    .expect("engine document");
    let engine =
        temper_service::startup::engine_with(&document, 0, &jig_protocol_config::Directories::working(), given)
            .expect("stock engine");
    let own = temper_limits_fixtures::small();
    temper_engine_service_vocabulary::figures::Composer::new(&engine, &own).compose(&temper_service::stock::agents())
}
#[test]
fn stock_stop_line_uses_the_applied_workers_bounds() {
    let scratch = skein_scratch::Scratch::new("worker-stop-line");
    let (_, document) = super::worker_document(scratch.path());
    let directories = super::configuration::directories(scratch.path());
    let mut part = temper_worker_service::startup::worker_part(
        &super::worker_start(&document),
        &temper_service_shell::jig_directories(&directories),
        None,
        &[],
    )
    .expect("worker part");
    temper_worker_service::figures::apply(&mut part, stock_figures()).expect("applied figures");
    assert_eq!(
        worker::stop_line(part.worker.jig.stop_bound, part.worker.jig.graces.drain),
        "stop bound 326 s, drain 180 s\n"
    );
}

#[test]
fn stop_line_rounds_both_bounds_up_to_whole_seconds() {
    use skein_lib::Duration;
    assert_eq!(
        worker::stop_line(Duration::from_millis(326_001), Duration::from_millis(180_001)),
        "stop bound 327 s, drain 181 s\n"
    );
    assert_eq!(
        worker::stop_line(Duration::from_nanos(326_000_000_001), Duration::from_nanos(180_000_000_001)),
        "stop bound 327 s, drain 181 s\n"
    );
}

#[test]
fn toml_worker_git_wins_over_path_and_absent_key_uses_path() {
    let scratch = skein_scratch::Scratch::new("worker-program-git");
    let root = scratch.path();
    let directories = super::configuration::directories(root);
    let named = root.join("named-git");
    let found = root.join("git");
    fs::copy("/bin/sh", &named).expect("named executable");
    fs::copy("/bin/sh", &found).expect("PATH executable");
    for explicit in [true, false] {
        let text = if explicit {
            format!("engine = \"127.0.0.1:7200\"\n[programs]\ngit = {named:?}\n")
        } else {
            "engine = \"127.0.0.1:7200\"\n".into()
        };
        fs::write(root.join(temper_config_shell::FILE), text).expect("worker file");
        for environment in [Vec::new(), vec![(Box::from(b"PATH".as_slice()), Box::from(root.as_os_str().as_bytes()))]] {
            let read = temper_service_shell::configuration::read(&directories, temper_config_shell::Shape::WorkerAlone)
                .expect("worker read");
            let part =
                worker::part(&directories, &read.document, &environment, &read.programs().expect("checked programs"))
                    .expect("worker part");
            let expected = if explicit {
                Some(&named)
            } else if environment.is_empty() {
                None
            } else {
                Some(&found)
            };
            assert_eq!(part.worker.git.as_deref(), expected.map(|path| path.as_os_str().as_bytes()));
        }
    }
}

#[test]
fn toml_worker_claude_wins_then_path_then_fetched_in_written_configuration() {
    let scratch = skein_scratch::Scratch::new("worker-program-claude");
    let root = scratch.path();
    let directories = super::configuration::directories(root);
    let named = root.join("named-claude");
    let found = root.join("claude");
    fs::copy("/bin/sh", &named).expect("named executable");
    fs::copy("/bin/sh", &found).expect("PATH executable");
    for explicit in [true, false] {
        let text = if explicit {
            format!("engine = \"127.0.0.1:7200\"\n[programs]\nclaude = {named:?}\n")
        } else {
            "engine = \"127.0.0.1:7200\"\n".into()
        };
        fs::write(root.join(temper_config_shell::FILE), text).expect("worker file");
        for environment in [Vec::new(), vec![(Box::from(b"PATH".as_slice()), Box::from(root.as_os_str().as_bytes()))]] {
            let read = temper_service_shell::configuration::read(&directories, temper_config_shell::Shape::WorkerAlone)
                .expect("worker read");
            let mut part =
                worker::part(&directories, &read.document, &environment, &read.programs().expect("checked programs"))
                    .expect("worker part");
            let unknown = temper_worker_service::figures::apply(&mut part, stock_figures()).expect("applied figures");
            assert!(unknown.is_empty());
            let bytes = worker::configuration(&part, &directories, &environment).expect("agent configuration");
            let written: serde_json::Value = serde_json::from_slice(&bytes).expect("agent JSON");
            let expected = if explicit {
                named.to_str().expect("named path")
            } else if environment.is_empty() {
                "fetched"
            } else {
                found.to_str().expect("PATH program")
            };
            assert_eq!(written["programs"]["claude_code"]["path"], expected);
        }
    }
}

#[test]
fn the_workers_watchdog_covers_stock_agent_figures_and_a_sixty_minute_tool() {
    use jig_protocol_worker_wire as w;
    use skein_lib::Duration;
    let scratch = skein_scratch::Scratch::new("derived-agent-watchdog");
    let mut input: serde_json::Value =
        serde_json::from_slice(&temper_config_shell::init_document(true)).expect("stock box document");
    input["engine"]["clients"]["provider"] = serde_json::Value::Null;
    let given = temper_service::stock::Given::Stock.with_providers(
        temper_service::stock::providers(),
        Box::new([temper_service::stock::scripted(([127, 0, 0, 1], 8370).into())]),
    );
    for (override_ms, expected_seconds) in [(None, 1205), (Some(3_600_000), 3605)] {
        let mut parts = configured_figures_with(&input, &given);
        if let Some(value) = override_ms {
            let mut agents = parts.agents;
            let mut profile = skein_lib::List::with_capacity(agents.profile.len() + 1);
            for figure in agents.profile.into_boxed() {
                profile.push(figure).expect("existing profile figure");
            }
            profile.push(w::Limit { name: b"policy.tool_deadline".as_slice().into(), value }).expect("profile room");
            agents.profile = profile;
            parts.agents = agents;
        }
        let figures = parts;
        let bytes =
            agent::figures::configuration(&figures, Some(std::path::Path::new("/fixture/claude")), scratch.path(), &[])
                .expect("agent document from engine figures");
        let configured =
            agent::decoded_bytes(&bytes).expect("decode document").resolve(&mut Vec::new()).expect("derive agent");
        let tool_deadline = configured.service.limits.domain.session.tools.tool_deadline;
        let endpoints = configured.service.domain.limits;
        let prepared = agent::prepare_bytes(bytes).expect("worker agent settings");
        let host = prepared.agent.host_limits;
        assert_eq!(host.no_progress, Duration::from_secs(expected_seconds));
        assert!(host.no_progress >= tool_deadline);
        assert_eq!(endpoints.len(), 4, "all stock endpoints participate");
        for endpoint in &endpoints {
            assert!(host.no_progress >= endpoint.head.checked_add(endpoint.idle).expect("endpoint turn"));
        }
        assert_eq!(smith_host_domain::check(&host), Ok(()));
    }
}

#[test]
fn a_boxs_demo_endpoint_keeps_its_name_and_uses_smiths_openai_provider() {
    let mut input: serde_json::Value =
        serde_json::from_slice(&temper_config_shell::init_document(true)).expect("box document");
    input["engine"]["clients"]["provider"] = serde_json::Value::Null;
    let given = temper_service::stock::Given::Stock.with_providers(
        temper_service::stock::providers(),
        Box::new([temper_service::stock::scripted(([127, 0, 0, 1], 8370).into())]),
    );
    let figures = configured_figures_with(&input, &given);
    assert_eq!(figures.agents.endpoints.get(3).expect("demo endpoint").provider.as_ref(), b"demo");
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
        &super::worker_start(&document),
        &temper_service_shell::jig_directories(&directories),
        None,
        &[],
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
    let mut parts = stock_figures();
    let mut agents = parts.agents;
    let endpoint = agents.endpoints.get_mut(0).expect("endpoint");
    let mut value = endpoint.clone();
    value.host = b"::1".as_slice().into();
    value.port = 1234;
    value.program = Some(b"claude_code".as_slice().into());
    value.headers = skein_lib::List::with_capacity(1);
    value
        .headers
        .push(w::Header { name: b"x-fixture".as_slice().into(), value: b"given".as_slice().into() })
        .expect("one header");
    value.transport = w::Transport::Tls(w::Tls {
        server_name: b"skein.test".as_slice().into(),
        trust: Some(b"/fixture/root.der".as_slice().into()),
    });
    let mut limits = value.limits;
    limits.oversized = w::Oversized::Drop;
    value.limits = limits;
    *endpoint = value;
    parts.agents = agents;
    let bytes =
        agent::figures::configuration(&parts, None, std::path::Path::new("/fixture"), &[]).expect("written agent");
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
