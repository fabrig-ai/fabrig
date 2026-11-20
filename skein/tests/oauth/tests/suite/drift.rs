//! The kept record's schema and generated code (data/codec/README.md, 6).

use std::fs;
use std::path::Path;

const REGENERATE: &str =
    "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-oauth-world --test suite -E 'test(/^drift::/)'";

fn check(path: &Path, bytes: &[u8], write: bool) {
    if write {
        fs::create_dir_all(path.parent().expect("output has a parent")).expect("output directory");
        fs::write(path, bytes).expect("write generated output");
    } else if fs::read(path).ok().as_deref() != Some(bytes) {
        let made = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("oauth-drift")
            .join(path.file_name().expect("output has a name"));
        fs::create_dir_all(made.parent().expect("output parent")).expect("failure directory");
        fs::write(&made, bytes).expect("write generated output for inspection");
        panic!("{} drifted; output at {}; {REGENERATE}", path.display(), made.display());
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "skein's round: reads SKEIN_CODEGEN_WRITE, the codegen rewrite switch")]
fn the_saved_token_schema_has_no_drift() {
    let source = skein_oauth::SCHEMA;
    let unpinned = skein_codegen::unpinned(source).expect("schema valid");
    assert!(unpinned.is_empty(), "oauth-v3: unpinned constructs: {unpinned:?}");
    let schema = skein_codegen::values(source).schema.clone();
    let owner = skein_scratch::paths::tree().join("crates/skein-oauth");
    let form = skein_codegen::form_of(&owner.join("schema"), &schema.family).expect("owner's lean mark readable");
    let output = skein_codegen::generate_as(&schema, form);
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let codec = skein_scratch::paths::tree().join("crates/skein-oauth-codec");
    check(&codec.join("src/generated/oauth.rs"), output.records.as_bytes(), write);
}
