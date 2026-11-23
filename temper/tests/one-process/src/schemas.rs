//! Immutable binary texts shared by store opens and the durable observer (store-check.md, 2).
use skein_codec::schema::{self, Schema};
use std::sync::OnceLock;
use temper_engine_service::store::OWNER_SETS;

pub(crate) fn binary_schemas() -> &'static [Schema] {
    static SCHEMAS: OnceLock<Vec<Schema>> = OnceLock::new();
    SCHEMAS.get_or_init(|| {
        let mut schemas = Vec::new();
        for family in OWNER_SETS.sets.iter().flat_map(|set| set.families) {
            keep(&mut schemas, family.current);
            if let Some(frames) = &family.frames {
                for text in frames.texts {
                    keep(&mut schemas, text);
                }
            }
        }
        schemas
    })
}

fn keep(schemas: &mut Vec<Schema>, text: &str) {
    if !text.is_empty() && !schemas.iter().any(|schema| schema.text() == text.as_bytes()) {
        schemas.push(schema::parse(text.as_bytes()).expect("valid binary schema"));
    }
}

/// Registry row plans resolve holds against current families, not historical frame versions.
pub(crate) fn current_schemas() -> &'static [&'static Schema] {
    static CURRENT: OnceLock<Vec<&'static Schema>> = OnceLock::new();
    CURRENT.get_or_init(|| {
        let binary = binary_schemas();
        OWNER_SETS
            .sets
            .iter()
            .flat_map(|set| set.families)
            .filter(|family| !family.current.is_empty())
            .map(|family| {
                binary.iter().find(|schema| schema.text() == family.current.as_bytes()).expect("current schema")
            })
            .collect()
    })
}
