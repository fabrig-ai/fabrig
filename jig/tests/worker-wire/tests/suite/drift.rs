//! Records' drift and pinned constructs (skein's data/codec/README.md, 6).
use skein_codegen::{generate, parse, unpinned};
use std::fs;

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh generated records")]
fn worker_wire_schema_matches_generated_records_and_is_pinned() {
    let root = skein_scratch::paths::tree().join("crates/jig-protocol-worker-wire");
    let source = fs::read_to_string(root.join("schema/worker-wire-v18.schema")).expect("schema");
    let unpinned = unpinned(&source).expect("schema valid");
    assert!(unpinned.is_empty(), "worker_wire: unpinned constructs: {unpinned:?}");
    let output = generate(&parse(&source).expect("schema valid"));
    let path = skein_scratch::paths::tree().join("crates/jig-protocol-worker-wire/src/generated/worker_wire.rs");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-worker-wire-world --test suite -E 'test(drift::)'";
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        fs::write(&path, output.records).expect("write generated records");
    } else {
        assert_eq!(
            fs::read_to_string(&path).expect("generated records"),
            output.records,
            "{}: {again}",
            path.display()
        );
    }
}
