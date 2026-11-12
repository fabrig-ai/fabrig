//! The binary's Sol model, used only by scripted workers.
pub fn box_model() -> temper_engine_domain::Model {
    let document = jig_protocol_config::document(
        include_bytes!("../../../../../crates/temper-init/template/bare.json"),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("bare document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("document object")
        .member(&document, b"engine")
        .expect("engine member");
    let engine = temper_engine_service::startup::engine(&document, at, &jig_protocol_config::Directories::working())
        .expect("stock engine");
    engine.llm.models.iter().find(|entry| entry.model.name.as_ref() == b"gpt-6.1-sol").expect("stock Sol").model.clone()
}
