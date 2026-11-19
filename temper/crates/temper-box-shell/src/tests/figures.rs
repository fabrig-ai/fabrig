//! The box builds from its engine and its durable agents copy.
use crate::{serve, served};
use jig_client_protocol_wire::Record as _;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
use temper_client_protocol_wire::v1 as w;
fn document(root: &std::path::Path) -> (crate::Directories, serde_json::Value) {
    fs::set_permissions(root, fs::Permissions::from_mode(0o700)).expect("private fixture root");
    let dirs = super::configuration::directories(root);
    let files = temper_init::bare(
        &temper_init::Bare { binary: b"/fixture/temper-box", git: None, claude: None },
        &temper_init::Directories { config: root.as_os_str().as_bytes(), data: root.as_os_str().as_bytes() },
        &[0; 32],
    )
    .expect("init files");
    for file in files {
        assert_ne!(file.name, b"agent.json");
        let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("bare bytes") };
        let path = root.join(std::ffi::OsStr::from_bytes(file.name));
        fs::write(&path, bytes).expect("private fixture");
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private");
    }
    let document = serde_json::from_slice(&temper_config_shell::init_document(false)).expect("init start");
    let mut providers = temper_service::stock::providers();
    for value in &mut providers {
        value.provider.signed_in_plan = false;
        value.provider.sign_in = None;
        for endpoint in &mut value.endpoints {
            endpoint.address = jig_protocol_config::EndpointAddress::Socket(([127, 0, 0, 1], 9).into());
            endpoint.transport = jig_protocol_config::Transport::Plaintext;
        }
    }
    temper_service_shell::providers::write(&root.join("providers.setting"), &providers).expect("world providers");
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
        4,
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
    let mut default = temper_agent_fixtures::at(&document, root);
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
            served::write_setting(&copy, contents).expect("settings copy");
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
            None,
            &temper_config_shell::Programs::default(),
        )
        .map(|(service, _, _)| service.operations());
        crate::close(owned);
        let operations = checked.unwrap_or_else(|why| panic!("{case}: {why}"));
        assert!(operations > 0);
        let pinned = root.join("workspaces/agent/agent.json");
        let configuration: serde_json::Value =
            serde_json::from_slice(&fs::read(&pinned).expect("pinned agent")).expect("agent JSON");
        let mut expected = expected.clone();
        // The selected port is pinned against the retained listener in the box-start test below.
        expected["endpoints"]
            .as_array_mut()
            .expect("expected endpoints")
            .iter_mut()
            .find(|endpoint| endpoint["name"] == "demo")
            .expect("scripted endpoint")["address"] = configuration["endpoints"]
            .as_array()
            .expect("actual endpoints")
            .iter()
            .find(|endpoint| endpoint["name"] == "demo")
            .expect("scripted endpoint")["address"]
            .clone();
        assert_eq!(configuration, expected, "{case}");
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
    served::write_setting(&path, &encoded(&agents(900))).expect("replace bad copy");
    assert_eq!(serve::agents_setting(&path, &mut Vec::new()), Some(agents(900)));
    assert!(!path.with_extension("setting.new").exists());
    let mut trailing = encoded(&agents(900)).into_vec();
    trailing.push(0);
    fs::write(&path, trailing).expect("trailing bytes");
    assert!(serve::agents_setting(&path, &mut Vec::new()).is_none());
}
#[test]
fn local_copy_builds_the_next_start_at_its_places_and_bad_bytes_log_once() {
    use jig_client_protocol_wire::WorkerSetting;
    let scratch = skein_scratch::Scratch::new("local-copy");
    let (dirs, document) = document(scratch.path());
    let bytes = serde_json::to_vec(&document).expect("JSON");
    let given = stock_defaults(&bytes, &dirs);
    let path = scratch.path().join("local.setting");
    let program = scratch.path().join("fixture-agent");
    fs::write(&program, b"box startup program fixture\n").expect("fixture program");
    let local =
        WorkerSetting { number: 1, name: b"local".as_slice().into(), places: 2, digest: None, withdrawn: false };
    let mut writer = skein_lib::Writer::new(
        usize::try_from(local.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("bounded codec"),
    );
    local.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("local bytes");
    let value = writer.finish();
    for (contents, places, warning) in [
        (None, 4, ""),
        (Some(value.as_ref()), 2, ""),
        (Some(b"bad".as_slice()), 4, "local.setting: cannot decode; using the default\n"),
    ] {
        if let Some(contents) = contents {
            served::write_setting(&path, contents).expect("whole local copy");
            assert_eq!(fs::read(&path).expect("copy"), contents);
            assert!(!path.with_extension("setting.new").exists());
        }
        let mut log = Vec::new();
        let initial = serve::local_setting(&path, &mut log);
        assert_eq!(String::from_utf8(log).expect("log"), warning);
        let part = temper_service::startup::box_worker(
            &bytes,
            &temper_service_shell::jig_directories(&dirs),
            &given,
            &temper_service::stock::agents(),
            initial.as_ref().map_or(4, |value| value.places),
            &[],
        )
        .expect("box start")
        .expect("worker");
        assert_eq!(part.worker.jig.slots, places);
        assert_eq!(part.figures.as_ref().expect("figures").places, places);
        assert_eq!(part.limits.checkout.workspaces, places);
        assert_eq!(part.limits.git_component.operations, places);
        let expected = temper_service::startup::operations_with_worker(
            &bytes,
            &temper_service_shell::jig_directories(&dirs),
            &given,
            None,
            Some(&part),
        )
        .expect("applied worker credit");
        assert_eq!(
            serve::operations_in_with("temper-box", &[], b"/", &dirs, &bytes, &given).expect("ring credit"),
            expected
        );
        if places == 2 {
            let mut owned = Vec::new();
            let started = serve::prepare_with_limits(
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
                None,
                &temper_config_shell::Programs::default(),
            )
            .map(|(service, _, _)| service.operations());
            crate::close(owned);
            assert_eq!(started.expect("next start at copied places"), expected);
        }
    }
    let mut trailing = value.into_vec();
    trailing.push(0);
    fs::write(&path, trailing).expect("trailing bytes");
    let mut log = Vec::new();
    assert!(serve::local_setting(&path, &mut log).is_none());
    assert_eq!(String::from_utf8(log).expect("log"), "local.setting: cannot decode; using the default\n");
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
    let why = serve::prepare(
        "temper-box",
        &[],
        b"/",
        &dirs,
        &serde_json::to_vec(&original).expect("JSON"),
        &mut Vec::new(),
        &temper_config_shell::Programs::default(),
    )
    .err()
    .expect("old file refused");
    assert_eq!(why, "agent.json: moved: temper writes the agent's configuration under the worker's root; remove it");
}
#[test]
fn box_refuses_sixty_five_endpoints_before_composing() {
    let scratch = skein_scratch::Scratch::new("box-endpoints");
    let (dirs, document) = document(scratch.path());
    let mut providers = temper_service::stock::providers().into_vec();
    providers.truncate(1);
    let first = providers[0].endpoints[0].clone();
    providers[0].endpoints = (1..=65)
        .map(|number| {
            let mut endpoint = first.clone();
            endpoint.number = number;
            endpoint.name = if number == 1 {
                b"openai".as_slice().into()
            } else {
                format!("endpoint-{number}").into_bytes().into_boxed_slice()
            };
            endpoint
        })
        .collect();
    let given = temper_service::stock::Given::Stock.with_providers(providers.into_boxed_slice(), Box::new([]));
    let why = temper_service::startup::box_worker(
        &serde_json::to_vec(&document).expect("JSON"),
        &temper_service_shell::jig_directories(&dirs),
        &given,
        &temper_service::stock::agents(),
        4,
        &[],
    )
    .expect_err("endpoint bound");
    assert!(
        String::from_utf8_lossy(&temper_service::startup::lines(&why))
            .contains("providers.setting: 65 endpoints, at most 64, the workers' figures carry no more")
    );
}

#[test]
fn every_box_pins_the_chosen_scripted_listener_in_its_fixed_provider_and_worker() {
    let preferred = std::net::TcpListener::bind(crate::scripted::PREFERRED).ok();
    let listener = crate::scripted::choose().unwrap();
    let address = listener.local_addr().unwrap();
    assert!(address.ip().is_loopback());
    assert_ne!(address.port(), 0);
    if preferred.is_some() {
        assert_ne!(address.port(), 8370);
    }
    let scratch = skein_scratch::Scratch::new("scripted-start");
    let (dirs, document) = document(scratch.path());
    let given = crate::scripted::given(&dirs, &temper_service::stock::Given::Stock, address);
    let bytes = serde_json::to_vec(&document).unwrap();
    let jig = temper_engine_shell::serve::settings(&bytes, &dirs, &given);
    let core = jig_protocol_config::core(&jig, [1; 16], 1);
    assert_eq!(core.fixed_providers.as_ref(), &[temper_service::stock::scripted(address)]);
    assert!(jig.llm.providers.iter().any(|provider| provider.number == 3));
    let demo =
        crate::setup::check_with(&bytes, &temper_service_shell::jig_directories(&dirs), &given).unwrap().demo.unwrap();
    assert_eq!(demo.llm().listen(), address);
    let environment = [(Box::from(crate::scripted::ADDRESS), address.to_string().into_bytes().into_boxed_slice())];
    let mut owned = Vec::new();
    let prepared = crate::prepare(&environment, b"/", &dirs, &bytes, &mut owned).unwrap();
    drop(prepared);
    crate::close(owned);
    let actual: serde_json::Value =
        serde_json::from_slice(&fs::read(scratch.path().join("workspaces/agent/agent.json")).unwrap()).unwrap();
    let endpoint = actual["endpoints"].as_array().unwrap().iter().find(|endpoint| endpoint["name"] == "demo").unwrap();
    assert_eq!(endpoint["number"], 3);
    assert_eq!(endpoint["dialect"], 3);
    assert_eq!(endpoint["address"], address.to_string());
    assert_eq!(endpoint["transport"], "plaintext");
    assert_eq!(endpoint["connect"], 10000);
    assert_eq!(endpoint["handshake"], 10000);
    let copy =
        temper_service_shell::providers::read(&scratch.path().join("providers.setting"), Box::new([]), &mut Vec::new());
    assert!(copy.iter().all(|provider| provider.provider.number != 3));
}
