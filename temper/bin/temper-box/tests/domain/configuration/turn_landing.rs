//! Turn-landing documents as committed bytes and rendered values.
use std::path::PathBuf;
use temper_box_kit::turn_landing_fixture::{engine, figures, json, place, rendered, root};
use temper_service::startup::{self, Shape};

#[test]
fn the_turn_landing_documents_check_as_their_shapes() {
    let directories = jig_protocol_config::Directories::working();
    let bytes = rendered("engine.toml");
    let bundle = place().value("bundle").expect("test place bundle");
    assert_eq!(json(&bytes)["application"]["clients"]["bundle"], std::str::from_utf8(&bundle).expect("bundle path"));
    assert_eq!(startup::shape(&bytes, &directories).expect("engine shape"), Shape::EngineAlone);
    let named = startup::named(&bytes, &directories)
        .unwrap_or_else(|why| panic!("engine: {}", String::from_utf8_lossy(&startup::lines(&why))));
    assert!(named.projects.is_empty(), "Hello is made by ctl, not seeded");
    let engine = engine(&bytes);
    assert!(engine.limits.core.tasks.spec_bytes >= 16384);
    assert!(engine.limits.core.tasks.result_bytes >= 4096);
    assert_eq!(engine.limits.core.fleet.engine_slots, 0);
    assert!(json(&bytes)["engine"]["hosts"].get("workers").is_none(), "workers are admitted through settings");

    let bytes = rendered("worker.toml");
    assert_eq!(startup::shape(&bytes, &directories).expect("worker shape"), Shape::WorkerAlone);
    let worker_directories =
        jig_protocol_config::Directories::given(b"/turn/config/worker", b"/turn/data/worker", b"/turn/cache/worker")
            .expect("worker's place directories");
    let part = startup::worker_part(&bytes, &worker_directories, Some(b"/turn/data/bin/git"), &[])
        .expect("checked worker alone");
    assert_eq!(part.worker.jig.slots, 1, "admission gives two places through the worker's setting");
    assert!(json(&bytes)["workers"][0].get("limits").is_none(), "the engine supplies the worker's limits");
    assert_eq!(part.worker.jig.secret.as_ref(), b"/turn/data/worker/secrets/worker.secret");
    assert!(part.worker.jig.agent.is_none(), "temper pins the engine's agent figures");
    assert_eq!(part.worker.git.as_deref().expect("fake git"), b"/turn/data/bin/git");

    let configuration =
        temper_worker_shell::agent::figures::configuration(&figures(), None, &PathBuf::from("/turn/data/worker"), &[])
            .expect("agent document from the engine's figures");
    let mut value = json(&configuration);
    // Carrier bounds depend on figures, not the machine's launchers.
    value["programs"] = serde_json::json!({});
    let agent = smith_agent_shell::config::from_value(value.clone()).expect("generated agent configuration");
    let turn = temper_playground::documents::turn_bounds::turn_body(&value);
    let figures = figures();
    let hub = figures.limits.iter().find(|limit| limit.name.as_ref() == b"hub.turn_bytes").expect("engine turn figure");
    assert!(hub.value >= u64::from(turn), "engine figures carry the generated agent's turn");
    assert!(agent.service.limits.channel.bodies.turn_body > 0);

    let bytes = rendered("forgejo.json");
    let forge = temper_fake::forge::Document::decode(&bytes).expect("fake Forgejo document");
    assert_eq!(forge.repositories(), []);
    assert_eq!(forge.admin_token(), b"/turn/data/forgejo/admin_token");
    let forge = json(&bytes);
    assert_eq!(
        forge["workflows"],
        serde_json::json!([
            {"file":"gate.yml","jobs":["gate"],"cue":{"declared":".fake-forge/gate"}}
        ])
    );
    assert_eq!(forge["manual_merges"], serde_json::json!({"allow":true,"autodetect":true}));
    assert_eq!(forge["made_ci"], serde_json::json!({"declared":".fake-forge/ci"}));
    assert!(forge.get("github").is_none());

    let model = temper_fake::llm::Document::decode(&rendered("model.json")).expect("fake model document");
    assert_eq!(model.key(), b"/turn/data/model/key");
    let stock = temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    assert_eq!(stock.models.len(), 7);
    for charter in &stock.charters {
        let resolved = jig_domain_core::settings::charters::resolved(charter, &stock.models, &engine.limits.core)
            .expect("stock charter resolves without a settings change");
        for configured in std::iter::once(&resolved.policy.model).chain(&resolved.policy.alternatives) {
            if configured.provider == 1 {
                assert_eq!(configured.endpoint, 1);
                assert!(model.models().iter().any(|model| model.name == configured.name));
            } else {
                assert_eq!(configured.provider, 2, "the other stock provider remains in the catalogue");
                assert!(!engine.llm.providers.iter().any(|provider| provider.number == configured.provider));
            }
        }
    }
}

#[test]
fn the_turn_landing_settings_select_turn_two_and_its_blocking_gate() {
    use jig_client::contract::{Setting, SettingEdit, SettingValue};
    use temper_engine_domain_forge::settings;
    let bytes = std::fs::read(root().join("files/landing.json")).expect("landing settings");
    let items = temper_client_tools::jig::settings_file::decode(&bytes, &temper_client_tools::limits::tools())
        .expect("checked settings file");
    assert_eq!(items.len(), 2);
    for (item, kind) in items.iter().zip([4, 5]) {
        assert_eq!(item.setting, Setting::Application { kind, scope: 1, number: 1 });
        let SettingEdit::Set(set) = &item.edit else { panic!("set value") };
        let SettingValue::Application(bytes) = &set.value else { panic!("application setting") };
        let decoded = temper_engine_smith::kinds::decode(
            kind,
            bytes,
            &temper_engine_smith::kinds::Bounds { segments: 8, segment_bytes: 64, gates: 4, approvals: 4 },
        )
        .expect("landing value");
        match decoded.value {
            settings::Value::Landing(landing) => {
                assert_eq!(landing.by, temper_engine_domain_forge_change::Way::Turn);
                assert_eq!(landing.batch, 2);
            }
            settings::Value::Gates(gates) => {
                let [repository] = gates.repositories.as_ref() else { panic!("one repository's gate") };
                assert_eq!((repository.repository.forge, repository.repository.repository), (1, 1));
                let [gate] = repository.gates.as_ref() else { panic!("one gate") };
                assert_eq!(gate.name.as_ref(), b"gate");
                assert_eq!(gate.workflow.as_ref(), b"gate.yml");
                assert!(gate.blocking);
            }
            settings::Value::AgentFigures(_)
            | settings::Value::Welcome(_)
            | settings::Value::Rule(_)
            | settings::Value::Charter(_)
            | settings::Value::Questions(_)
            | settings::Value::Ways(_)
            | settings::Value::Classes(_)
            | settings::Value::Limits(_)
            | settings::Value::CharterWords(_)
            | settings::Value::ProjectWords(_)
            | settings::Value::Goal(_)
            | settings::Value::Follow(_)
            | settings::Value::Dependency(_)
            | settings::Value::ProjectAreas(_)
            | settings::Value::ProjectTriager(_) => panic!("kinds 4 and 5"),
        }
    }
}

#[test]
fn the_turn_landing_engine_documents_pass_startup_root_checks() {
    use skein_lib::{List, Secret};
    use temper_service::settings::{Files, Seeds};

    // The landing's settings reach the engine through ctl (files/landing.json), never by patching its document.
    let bytes = rendered("engine.toml");
    let directories = jig_protocol_config::Directories::working();
    let seeds = Seeds {
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
        link_secret: None,
        local_secret: None,
    };
    // Engine-alone startup ignores the local agent argument: its agent runs on the worker.
    let agent = temper_playground::documents::agent();
    let named = startup::named(&bytes, &directories).expect("named setup files");
    let mut entries = List::with_capacity(u32::try_from(named.files.len()).expect("file count"));
    for file in named.files {
        entries.push((file.path, Secret::new(Box::new([b'0'; 64])))).expect("one entry per named file");
    }
    let problems = match startup::check(
        &bytes,
        &directories,
        &Files { entries },
        &agent,
        &seeds,
        None,
        List::with_capacity(0),
        None,
    ) {
        Ok(()) => startup::Problems::empty(),
        Err(problems) => problems,
    };
    let lines = startup::lines(&problems);
    assert!(lines.is_empty(), "{}", String::from_utf8_lossy(&lines));
}

#[test]
fn the_turn_landing_scripts_are_the_real_loops_bytes_in_scripts_form() {
    let scripts = std::fs::read(root().join("scripts.json")).expect("setup scripts");
    let fixture = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/real-loop/tests/fixtures/turn-landing/scripts.json"),
    )
    .expect("real-loop fixture");
    assert!(scripts == fixture, "one source, byte for byte");
    temper_fake::llm::Document::decode_with_scripts(&rendered("model.json"), Some(&scripts))
        .expect("fake model --scripts form");
}
