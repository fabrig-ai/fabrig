//! Records, bounds and translation drift (skein's data/codec/golden.md, 1).
use jig_protocol_worker_wire::v22 as codec;
use skein_codegen::{bounds_text, generate, translate, values};

#[path = "../../../../testing/codec-drift.rs"]
mod codec_drift;

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh codec drift files")]
fn worker_wire_schema_matches_generated_records() {
    let source = jig_protocol_worker_wire::SCHEMA;

    let root = skein_scratch::paths::tree().join("crates/jig-protocol-worker-wire");
    let schema = &values(source).schema;
    let output = generate(schema);
    let path = root.join("src/generated/worker_wire.rs");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-worker-wire-world --test suite -E 'test(drift::)'";
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    codec_drift::check_text(&path, &output.records, write, again);
    let path = skein_scratch::paths::package().join("bounds/worker_wire.bounds");
    let bounds = bounds_text(codec::DECLARATIONS, codec::CEILINGS.values().as_slice());
    codec_drift::check_text(&path, &bounds, write, again);
}

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh codec drift files")]
fn charter_translation_matches_generated_code() {
    let root = skein_scratch::paths::tree().join("crates/jig-protocol-charter");
    let maps_path = root.join("schema/charter.maps");
    let maps = std::fs::read_to_string(&maps_path).expect("charter translation maps");
    let code = translate(&values(smith_charter::SCHEMA).schema, &maps)
        .unwrap_or_else(|error| panic!("{}: {error}", maps_path.display()));
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-worker-wire-world --test suite -E 'test(=drift::charter_translation_matches_generated_code)'";
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    codec_drift::check_text(&root.join("src/generated/charter.rs"), &code, write, again);
}
