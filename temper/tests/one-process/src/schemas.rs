//! Immutable binary texts shared by store opens and the durable observer (store-check.md, 2).
use skein_codec::schema::Schema;
use std::sync::OnceLock;
use temper_engine_service::store::OWNER_SETS;

pub(crate) fn binary_schemas() -> &'static [Schema] {
    temper_playground::programs::prepared_schemas()
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
