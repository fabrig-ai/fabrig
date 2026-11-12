//! The binary's Sol model, used only by scripted workers.
pub fn box_model() -> temper_engine_domain::Model {
    let engine = engine();
    engine.llm.models.iter().find(|entry| entry.model.name.as_ref() == b"gpt-6.1-sol").expect("stock Sol").model.clone()
}

pub fn defaults(words: u32) -> jig_domain_core::settings::Defaults {
    temper_service::stock::with_models(engine().llm.models, words)
}

fn engine() -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        include_bytes!("../../../../../crates/temper-init/template/bare.json"),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("bare document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("document object")
        .member(&document, b"engine")
        .expect("engine member");
    temper_service::startup::engine(&document, at, &jig_protocol_config::Directories::working()).expect("stock engine")
}
