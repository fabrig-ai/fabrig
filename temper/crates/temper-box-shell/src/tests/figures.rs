//! The box builds from its engine and its durable agents copy.
use crate::{serve, served};
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
use temper_client_protocol_wire::v1 as w;
fn document(root: &std::path::Path) -> (crate::Directories, serde_json::Value) {
    fs::set_permissions(root, fs::Permissions::from_mode(0o700)).expect("private fixture root");
    let dirs = super::configuration::directories(root);
    let files = temper_init::bare(
        &temper_init::Bare {
            binary: b"/fixture/temper-box",
            git: None,
            claude: None,
            clients: core::num::NonZeroU16::new(8367).expect("port"),
            webhooks: core::num::NonZeroU16::new(8371).expect("port"),
        },
        &temper_init::Directories { config: root.as_os_str().as_bytes(), data: root.as_os_str().as_bytes() },
        &[0; 32],
    )
    .expect("init files");
    let mut document = None;
    for file in files {
        assert_ne!(file.name, b"agent.json");
        let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("bare bytes") };
        if file.name == b"temper.json" {
            document = Some(serde_json::from_slice::<serde_json::Value>(&bytes).expect("init JSON"));
        } else {
            let path = root.join(std::ffi::OsStr::from_bytes(file.name));
            fs::write(&path, bytes).expect("private fixture");
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private");
        }
    }
    let mut document = document.expect("init document");
    for provider in document["engine"]["llm"]["providers"].as_array_mut().expect("providers") {
        provider["signed_in_plan"] = false.into();
        provider["sources"].as_object_mut().expect("sources").remove("sign_in");
    }
    for endpoint in document["engine"]["llm"]["endpoints"].as_array_mut().expect("endpoints") {
        endpoint["address"] = serde_json::json!({"socket":"127.0.0.1:9"});
        endpoint["transport"] = "plaintext".into();
        endpoint.as_object_mut().expect("endpoint").remove("server_name");
    }
    (dirs, document)
}
fn agents(turns: u64) -> w::AgentFigures {
    let mut parts = temper_service::stock::agents().into_parts();
    for row in parts.profile.iter_mut() {
        if row.name() == b"policy.max_turns" {
            *row = w::AgentFigure::new(
                &w::CEILINGS,
                w::AgentFigureParts { name: b"policy.max_turns".as_slice().into(), value: turns },
            )
            .expect("figure");
        }
    }
    w::AgentFigures::new(&w::CEILINGS, parts).expect("agents")
}
fn encoded(value: &w::AgentFigures) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(skein_lib::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured value");
    writer.finish()
}
fn stock_defaults(bytes: &[u8], dirs: &crate::Directories) -> temper_service::stock::Given {
    use jig_protocol_config::{DocumentLimits, read::Object};
    use temper_service::{limits, startup, stock};
    let document = jig_protocol_config::document(
        bytes,
        &DocumentLimits { bytes: crate::CONFIG_BYTES, tokens: crate::CONFIG_BYTES, depth: 32 },
    )
    .expect("fixture document");
    let at = Object::new(&document, 0).expect("document object").member(&document, b"engine").expect("engine");
    let engine = startup::engine(&document, at, &temper_service_shell::jig_directories(dirs)).expect("stock engine");
    let root = limits::engine(&engine.limits, &limits::stock()).expect("stock limits");
    stock::Given::Own {
        jig: stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes),
        forge: stock::forge(&root),
        ways: stock::ways(),
    }
}
#[test]
fn startup_pins_composed_default_file_and_fallback_figures() {
    const PROGRAM: &[u8] = b"box startup agent program fixture\n";
    let scratch = skein_scratch::Scratch::new("startup-figures");
    let root = scratch.path();
    let (dirs, document) = document(root);
    let bytes = serde_json::to_vec(&document).expect("JSON");
    // All three settings copies use the same stock catalogue, charters and limits.
    let given = stock_defaults(&bytes, &dirs);
    let mut part = temper_service::startup::box_worker(
        &bytes,
        &temper_service_shell::jig_directories(&dirs),
        &given,
        &temper_service::stock::agents(),
        &[],
    )
    .expect("box part")
    .expect("worker");
    serve::box_inputs_worker(&mut part, &dirs).expect("derived program");
    assert_eq!(part.named.program.as_ref(), std::env::current_exe().expect("exe").as_os_str().as_bytes());
    assert_eq!(
        part.named.arguments.as_ref(),
        [Box::from(b"internal".as_slice()), Box::from(b"agent".as_slice()), part.named.agent.clone()]
    );
    // Preparation pins a copy, but never runs it: its size is not part of the figures contract.
    let program = root.join("fixture-agent");
    fs::write(&program, PROGRAM).expect("fixture program");
    let setting = encoded(&agents(900));
    let mut default = temper_agent_fixtures::configuration(&document);
    default["programs"]["claude_code"]["directory"] = serde_json::json!(root.join("programs"));
    let mut custom = default.clone();
    custom["profile"]["policy"]["max_turns"] = 900.into();
    let copy = root.join("agents.setting");
    for (case, contents, expected) in [
        ("absent", None, &default),
        ("file", Some(setting.as_ref()), &custom),
        ("corrupt", Some(b"not a setting".as_slice()), &default),
    ] {
        if let Some(contents) = contents {
            served::write_agents(&copy, contents).expect("settings copy");
        }
        let mut owned = Vec::new();
        let checked = serve::prepare_with_limits(
            "temper-box",
            &[],
            b"/",
            &dirs,
            &bytes,
            &mut owned,
            false,
            false,
            None,
            None,
            &given,
            Some(&program),
            false,
            &[],
            None,
        )
        .map(|(service, _, _)| service.operations());
        crate::close(owned);
        let operations = checked.unwrap_or_else(|why| panic!("{case}: {why}"));
        assert!(operations > 0);
        let pinned = root.join("workspaces/agent/agent.json");
        let configuration: serde_json::Value =
            serde_json::from_slice(&fs::read(&pinned).expect("pinned agent")).expect("agent JSON");
        assert_eq!(&configuration, expected, "{case}");
        assert_eq!(fs::read(root.join("workspaces/agent/fixture-agent")).expect("pinned program"), PROGRAM, "{case}");
        assert!(!root.join("agent.json").exists());
    }
}
#[test]
fn corrupt_copy_is_logged_and_atomic_replacement_decodes_whole() {
    let scratch = skein_scratch::Scratch::new("agents-copy");
    let path = scratch.path().join("agents.setting");
    let mut log = Vec::new();
    assert!(serve::agents_setting(&path, &mut log).is_none());
    assert_eq!(log, [] as [u8; 0]);
    fs::write(&path, b"bad").expect("bad copy");
    assert!(serve::agents_setting(&path, &mut log).is_none());
    assert_eq!(String::from_utf8(log).expect("log"), "agents.setting: cannot decode; using the default\n");
    served::write_agents(&path, &encoded(&agents(900))).expect("replace bad copy");
    assert_eq!(serve::agents_setting(&path, &mut Vec::new()), Some(agents(900)));
    assert!(!path.with_extension("setting.new").exists());
    let mut trailing = encoded(&agents(900)).into_vec();
    trailing.push(0);
    fs::write(&path, trailing).expect("trailing bytes");
    assert!(serve::agents_setting(&path, &mut Vec::new()).is_none());
}
#[test]
fn box_refuses_moved_worker_members_and_agent_file_by_name() {
    let scratch = skein_scratch::Scratch::new("box-moved");
    let (dirs, original) = document(scratch.path());
    for (name, words) in [
        ("limits", "a worker takes its limits from its engine"),
        ("agent", "temper writes the agent's configuration from its engine's figures"),
    ] {
        let mut document = original.clone();
        document["workers"][0][name] = serde_json::json!({});
        let why = temper_service::startup::named(
            &serde_json::to_vec(&document).expect("JSON"),
            &temper_service_shell::jig_directories(&dirs),
        )
        .expect_err("moved member");
        assert_eq!(
            String::from_utf8_lossy(&temper_service::startup::lines(&why)),
            format!("workers[0].{name}: moved: {words}; remove it\n")
        );
    }
    fs::write(scratch.path().join("agent.json"), b"never read").expect("old file");
    let why =
        serve::prepare("temper-box", &[], b"/", &dirs, &serde_json::to_vec(&original).expect("JSON"), &mut Vec::new())
            .err()
            .expect("old file refused");
    assert_eq!(why, "agent.json: moved: temper writes the agent's configuration under the worker's root; remove it");
}
#[test]
fn box_refuses_sixty_five_endpoints_before_composing() {
    let scratch = skein_scratch::Scratch::new("box-endpoints");
    let (dirs, mut document) = document(scratch.path());
    let first = document["engine"]["llm"]["endpoints"][0].clone();
    document["engine"]["llm"]["endpoints"] = (1..=65)
        .map(|number| {
            let mut endpoint = first.clone();
            endpoint["number"] = number.into();
            endpoint["name"] = if number == 1 { "openai".into() } else { format!("endpoint-{number}").into() };
            endpoint
        })
        .collect();
    // Keep the program provider's named endpoints: the count check comes before composition.
    document["engine"]["llm"]["providers"].as_array_mut().expect("providers").truncate(1);
    let why = temper_service::startup::box_worker(
        &serde_json::to_vec(&document).expect("JSON"),
        &temper_service_shell::jig_directories(&dirs),
        &temper_service::stock::Given::Stock,
        &temper_service::stock::agents(),
        &[],
    )
    .expect_err("endpoint bound");
    assert!(
        String::from_utf8_lossy(&temper_service::startup::lines(&why))
            .contains("engine.llm: 65 endpoints, at most 64, the workers' figures carry no more")
    );
}
