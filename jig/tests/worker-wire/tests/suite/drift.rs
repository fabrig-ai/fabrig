//! Records' drift (skein's data/codec/golden.md, 1).
use skein_codegen::{generate, values};
use std::fs;

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh generated records")]
fn worker_wire_schema_matches_generated_records() {
    let source = jig_protocol_worker_wire::SCHEMA;

    let root = skein_scratch::paths::tree().join("crates/jig-protocol-worker-wire");
    let schema = &values(source).schema;
    let output = generate(schema);
    let path = root.join("src/generated/worker_wire.rs");
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
