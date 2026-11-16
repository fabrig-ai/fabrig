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
    assert_eq!(startup::shape(&bytes, &directories).expect("engine shape"), Shape::EngineAlone);
    let named = startup::named(&bytes, &directories)
        .unwrap_or_else(|why| panic!("engine: {}", String::from_utf8_lossy(&startup::lines(&why))));
    assert!(named.projects.is_empty(), "Hello is made by ctl, not seeded");
    let engine = engine(&bytes);
    assert!(engine.limits.core.tasks.spec_bytes >= 16384);
    assert!(engine.limits.core.tasks.result_bytes >= 4096);
    assert_eq!(engine.limits.core.fleet.engine_slots, 0);
    assert_eq!(engine.hosts.workers.len(), 1);
    assert_eq!(engine.hosts.workers[0].digest.as_ref(), b"/turn/data/engine/secrets/worker.digest");

    let bytes = rendered("worker.json");
    assert_eq!(startup::shape(&bytes, &directories).expect("worker shape"), Shape::WorkerAlone);
    let part = startup::worker_part(&bytes, &directories).expect("checked worker alone");
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
fn the_turn_landing_patch_selects_turn_two_and_its_blocking_gate() {
    let patch = std::fs::read(root().join("files/landing.json")).expect("landing patch");
    let bytes = deployment::merge(&rendered("engine.json"), &patch, &place()).expect("merge engine patch");
    let directories = jig_protocol_config::Directories::working();
    startup::named(&bytes, &directories).expect("patched engine checks as serve checks it");
    let engine = engine(&bytes);
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("patched document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("object")
        .member(&document, b"application")
        .expect("application");
    let mut problems = startup::Problems::empty();
    let application =
        startup::application::parse(&document, at, &engine, &directories, &mut problems).expect("checked application");
    assert!(problems.is_empty());
    let landing = application.landings.get(&1).expect("first project");
    assert_eq!(landing.by, temper_engine_domain::Way::Turn);
    assert_eq!(landing.batch, 2);
    let [repository] = landing.repositories.as_ref() else { panic!("one repository's gate") };
    assert_eq!((repository.repository.forge, repository.repository.repository), (1, 1));
    let [gate] = repository.gates.as_ref() else { panic!("one gate") };
    assert_eq!(gate.name.as_ref(), b"gate");
    assert_eq!(gate.workflow.as_ref(), b"gate.yml");
    assert!(gate.blocking);
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
