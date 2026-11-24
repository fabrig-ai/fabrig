//! Composed turn-landing documents and agent (temper/docs/design/testing/qa/setups.md, 5–6).
use crate::configuration_fixture::DOCUMENT;
use serde_json::Value;
use std::path::{Path, PathBuf};
use temper_client_journeys_tool::drive::deployment::{self, Place};
use temper_service::startup::{self, Shape};

pub fn root() -> PathBuf {
    skein_scratch::paths::tree().join("docs/development/qa/setups/turn-landing")
}
pub fn place() -> Place {
    Place::new(
        [7101, 7102, 7103, 7104, 7105, 7106, 7107, 7108, 7109],
        "/turn".into(),
        "/programs".into(),
        Some("/turn/bundle".into()),
    )
    .expect("test place")
}
pub fn rendered(name: &str) -> Box<[u8]> {
    let bytes = std::fs::read(root().join("config").join(name)).expect("setup document");
    let rendered = deployment::render(name, &bytes, &place()).expect("every place name renders");
    let shape = match name {
        "engine.toml" => Some(Shape::EngineAlone),
        "worker.toml" => Some(Shape::WorkerAlone),
        _ => None,
    };
    match shape {
        Some(shape) => crate::configuration::parse(&rendered, shape).document,
        None => rendered,
    }
}
/// Render either landing setup's engine with fixed listener and bundle values for its turn check.
#[must_use]
pub fn engine_template(setup: &Path, root: &Path, listener: &str) -> Value {
    let bundle = root.join("bundle");
    crate::configuration::engine_template(
        &setup.join("config/engine.toml"),
        false,
        &[
            ("place.clients", listener),
            ("place.workers", listener),
            ("place.webhooks", listener),
            ("place.model", listener),
            ("place.bundle", bundle.to_str().expect("bundle path")),
        ],
    )
}

pub fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("rendered JSON")
}
pub fn engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    let bytes = serde_json::to_vec(&json(bytes)["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    let given = temper_service::stock::Given::Stock
        .with_providers(temper_agent_fixtures::providers::parse(&rendered("providers.json")), Box::new([]));
    startup::engine_with(&document, 0, &jig_protocol_config::Directories::working(), &given).expect("stock engine")
}

/// Prepare the agent from the engine's figures with the programs-directory verdict supplied.
pub fn agent(data: &Path, figures: &jig_protocol_worker_wire::Figures) -> temper_worker_shell::agent::Prepared {
    let configuration = temper_worker_shell::agent::figures::configuration(figures, None, data, &[])
        .expect("agent document from the engine's figures");
    temper_worker_shell::agent::prepare_bytes_with(configuration, &mut std::io::sink(), |_| Ok(()))
        .expect("setup agent")
}

pub fn figures() -> jig_protocol_worker_wire::Figures {
    let source = rendered("engine.toml");
    let engine = engine(&source);
    let document = jig_protocol_config::document(&source, &DOCUMENT).expect("engine document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("object")
        .member(&document, b"application")
        .expect("application");
    let mut problems = startup::Problems::empty();
    let application = startup::application::parse(
        &document,
        at,
        &engine,
        &jig_protocol_config::Directories::working(),
        &mut problems,
    )
    .expect("checked application");
    assert!(problems.is_empty());
    temper_engine_service::figures::Composer::new(&engine, &application.limits)
        .compose(&temper_service::stock::agents())
}
