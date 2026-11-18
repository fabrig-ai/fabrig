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
    let document = document.expect("init document");
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
#[test]
fn startup_pins_composed_default_file_and_fallback_figures() {
    for case in ["absent", "file", "corrupt"] {
        let scratch = skein_scratch::Scratch::new(case);
        let root = scratch.path();
        let (dirs, document) = document(root);
        let mut part = temper_service::startup::box_worker(
            &serde_json::to_vec(&document).expect("JSON"),
            &temper_service_shell::jig_directories(&dirs),
            &temper_service::stock::Given::Stock,
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
        let copy = root.join("agents.setting");
        match case {
            "file" => served::write_agents(&copy, &encoded(&agents(900))).expect("atomic copy"),
            "corrupt" => fs::write(&copy, b"not a setting").expect("bad copy"),
            _ => {}
        }
        let mut owned = Vec::new();
        let checked = crate::prepare(&[], b"/", &dirs, &serde_json::to_vec(&document).expect("JSON"), &mut owned)
            .map(|service| service.operations());
        crate::close(owned);
        let operations = checked.unwrap_or_else(|why| panic!("{case}: {why}"));
        assert!(operations > 0);
        let pinned = root.join("workspaces/agent/agent.json");
        let actual: serde_json::Value =
            serde_json::from_slice(&fs::read(&pinned).expect("pinned agent")).expect("agent JSON");
        let mut expected = temper_agent_fixtures::at(&document, root);
        expected["programs"]["claude_code"]["directory"] = serde_json::json!(root.join("programs"));
        // The selected port is pinned against the retained listener in the box-start test below.
        expected["endpoints"]
            .as_array_mut()
            .expect("expected endpoints")
            .iter_mut()
            .find(|endpoint| endpoint["name"] == "demo")
            .expect("scripted endpoint")["address"] = actual["endpoints"]
            .as_array()
            .expect("actual endpoints")
            .iter()
            .find(|endpoint| endpoint["name"] == "demo")
            .expect("scripted endpoint")["address"]
            .clone();
        if case == "file" {
            expected["profile"]["policy"]["max_turns"] = 900.into();
        }
        assert_eq!(actual, expected, "{case}");
        assert!(
            fs::metadata(
                root.join("workspaces/agent").join(std::env::current_exe().expect("exe").file_name().expect("name"))
            )
            .expect("derived program pinned")
            .is_file()
        );
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
