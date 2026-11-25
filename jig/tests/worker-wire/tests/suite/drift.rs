//! Records, bounds and translations drift (skein's data/codec/golden.md, 1).
use jig_protocol_worker_wire::v22 as codec;
use skein_codegen::{bounds_text, generate, translate, values};

#[path = "../../../../testing/codec-drift.rs"]
mod codec_drift;

const AGAIN: &str =
    "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-worker-wire-world --test suite -E 'test(drift::)'";

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh codec drift files")]
fn channel_translations_match_their_maps() {
    let schema = &values(smith_channel::CHANNEL_SCHEMA).schema;
    let crates = skein_scratch::paths::tree().join("crates");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    for name in ["jig-protocol-charter", "jig-protocol-worker-agents"] {
        let root = crates.join(name);
        let maps = std::fs::read_to_string(root.join("schema/channel.maps")).expect("read channel maps");
        let output = translate(schema, &maps).unwrap_or_else(|error| panic!("{name}: {error:?}"));
        codec_drift::check_text(&root.join("src/generated/channel.rs"), &output, write, AGAIN);
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh codec drift files")]
fn worker_wire_schema_matches_generated_records() {
    let source = jig_protocol_worker_wire::SCHEMA;

    let crates = skein_scratch::paths::tree().join("crates");
    let schema = &values(source).schema;
    let output = generate(schema);
    let path = crates.join("jig-protocol-worker-wire/src/generated/worker_wire.rs");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    codec_drift::check_text(&path, &output.records, write, AGAIN);
    for (end, maps) in [
        (
            "jig-protocol-engine-workers",
            include_str!("../../../../crates/jig-protocol-engine-workers/schema/worker_wire.maps"),
        ),
        (
            "jig-protocol-worker-link",
            include_str!("../../../../crates/jig-protocol-worker-link/schema/worker_wire.maps"),
        ),
    ] {
        let output = translate(schema, maps).unwrap_or_else(|error| panic!("{end}: {error}"));
        let path = crates.join(end).join("src/generated/worker_wire.rs");
        codec_drift::check_text(&path, &output, write, AGAIN);
    }
    let path = skein_scratch::paths::package().join("bounds/worker_wire.bounds");
    let bounds = bounds_text(codec::DECLARATIONS, codec::CEILINGS.values().as_slice());
    codec_drift::check_text(&path, &bounds, write, AGAIN);
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
