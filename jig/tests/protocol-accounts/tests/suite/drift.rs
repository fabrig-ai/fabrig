//! Records' drift and pinned constructs (skein's data/codec/README.md, 6).
use skein_codegen::{generate, unpinned, values};
use std::fs;

#[test]
#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh generated records")]
fn moved_schema_matches_generated_records_and_is_pinned() {
    let source = jig_protocol_accounts::SCHEMA;
    let unpinned = unpinned(source).expect("schema valid");
    assert!(unpinned.is_empty(), "moved: unpinned constructs: {unpinned:?}");
    let tree = skein_scratch::paths::tree();
    let schema = &values(source).schema;
    let output = generate(schema);
    let path = tree.join("crates/jig-protocol-accounts-codec/src/generated/moved.rs");
    let again =
        "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-protocol-accounts-world --test suite -E 'test(drift::)'";
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
