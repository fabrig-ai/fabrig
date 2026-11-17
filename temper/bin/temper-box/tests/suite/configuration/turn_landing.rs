//! The deployment setup's documents (testing/qa/setups.md, 5–6).
use super::DOCUMENT;
use serde_json::Value;
use std::path::PathBuf;
use temper_client_journeys_tool::drive::deployment::{self, Place};
use temper_service::startup::{self, Shape};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups/turn-landing")
}
fn place() -> Place {
    Place::new(
        [7101, 7102, 7103, 7104, 7105, 7106, 7107, 7108, 7109],
        "/turn".into(),
        "/programs".into(),
        Some("/turn/bundle".into()),
    )
    .expect("test place")
}
fn rendered(name: &str) -> Box<[u8]> {
    let bytes = std::fs::read(root().join("config").join(name)).expect("setup document");
    deployment::render(name, &bytes, &place()).expect("every place name renders")
}
fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("rendered JSON")
}
fn engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    let bytes = serde_json::to_vec(&json(bytes)["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    startup::engine(&document, 0, &jig_protocol_config::Directories::working()).expect("stock engine")
}

#[test]
fn the_turn_landing_documents_check_as_their_shapes() {
    let directories = jig_protocol_config::Directories::working();
    let bytes = rendered("engine.json");
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

    let bytes = rendered("worker.json");
    assert_eq!(startup::shape(&bytes, &directories).expect("worker shape"), Shape::WorkerAlone);
    let part = startup::worker_part(&bytes, &directories, Some(b"/turn/data/bin/git")).expect("checked worker alone");
    assert!(part.worker.jig.slots >= 2);
    assert!(part.worker.jig.limits.hub.slots >= 2);
    assert_eq!(part.worker.jig.secret.as_ref(), b"/turn/data/worker/secrets/worker.secret");
    assert_eq!(part.worker.jig.agent.program.as_ref(), b"/programs/temper-agent");
    assert_eq!(part.worker.git.as_deref().expect("fake git"), b"/turn/data/bin/git");

    let value = json(&rendered("agent.json"));
    let agent = smith_agent_shell::config::from_value(value.clone()).expect("smith agent configuration");
    let turn = temper_playground::documents::turn_bounds::turn_body(&value);
    for name in ["engine.json", "worker.json"] {
        let document = json(&rendered(name));
        let mut derived = document.clone();
        temper_playground::documents::turn_bounds::apply(&mut derived, turn);
        assert_eq!(document, derived, "{name} carries the agent's turn bounds");
    }
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
    assert!(!stock.models.is_empty());
    for charter in &stock.charters {
        let resolved = jig_domain_core::settings::charters::resolved(charter, &stock.models, &engine.limits.core)
            .expect("stock charter resolves without a settings change");
        for configured in std::iter::once(&resolved.policy.model).chain(&resolved.policy.alternatives) {
            assert_eq!(configured.endpoint, 0);
            assert_eq!(configured.provider, 1);
            assert!(model.models().iter().any(|model| model.name == configured.name));
        }
    }
}

#[test]
fn the_turn_landing_engine_passes_every_startup_check() {
    let bytes = rendered("engine.json");
    let directories = jig_protocol_config::Directories::working();
    let named = startup::named(&bytes, &directories).expect("named setup files");
    admission(&bytes, named.files);
}

#[test]
fn the_turn_landing_worker_passes_every_startup_check() {
    let bytes = rendered("worker.json");
    let directories = jig_protocol_config::Directories::working();
    let part = startup::worker_part(&bytes, &directories, Some(b"/turn/data/bin/git")).expect("setup worker part");
    admission(&bytes, part.files);
}

fn admission(bytes: &[u8], files: Box<[jig_protocol_config::File]>) {
    use skein_lib::{List, Secret};
    use temper_service::settings::{Files, Seeds};

    let directories = jig_protocol_config::Directories::working();
    let mut entries = List::with_capacity(u32::try_from(files.len()).expect("file count"));
    for file in files {
        entries.push((file.path, Secret::new(Box::new([b'7'; 64])))).expect("named file count");
    }
    let scratch = crate::support::Scratch::startup("turn-landing-admission");
    let agent_path = scratch.path().join("agent.json");
    std::fs::write(&agent_path, rendered("agent.json")).expect("rendered agent document");
    let agent = temper_worker_shell::agent::read(&agent_path).expect("setup agent");
    let seeds = Seeds {
        link_secret: None,
        local_secret: None,
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
    };
    let result =
        startup::check(bytes, &directories, &Files { entries }, &agent, &seeds, None, List::with_capacity(0), None);
    if let Err(problems) = result {
        panic!("{}", String::from_utf8_lossy(&startup::lines(&problems)));
    }
    scratch.remove();
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
            settings::Value::Welcome(_)
            | settings::Value::Rule(_)
            | settings::Value::Charter(_)
            | settings::Value::Questions(_)
            | settings::Value::Ways(_)
            | settings::Value::Classes(_)
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
