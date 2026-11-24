//! Records and bounds drift (skein's data/codec/golden.md, 1).
use jig_protocol_accounts::v1 as codec;
use skein_codegen::{bounds_text, generate, values};

#[path = "../../../../testing/codec-drift.rs"]
mod codec_drift;

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh codec drift files")]
fn moved_schema_matches_generated_records() {
    let source = jig_protocol_accounts::SCHEMA;

    let tree = skein_scratch::paths::tree();
    let schema = &values(source).schema;
    let output = generate(schema);
    let path = tree.join("crates/jig-protocol-accounts-codec/src/generated/moved.rs");
    let again =
        "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-protocol-accounts-world --test suite -E 'test(drift::)'";
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    codec_drift::check_text(&path, &output.records, write, again);
    let path = skein_scratch::paths::package().join("bounds/moved.bounds");
    let bounds = bounds_text(codec::DECLARATIONS, codec::CEILINGS.values().as_slice());
    codec_drift::check_text(&path, &bounds, write, again);
}
