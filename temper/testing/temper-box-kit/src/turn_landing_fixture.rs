//! Turn-landing documents shared by value and real-directory admission tests.
use crate::configuration_fixture::DOCUMENT;
use serde_json::Value;
use std::path::PathBuf;
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
