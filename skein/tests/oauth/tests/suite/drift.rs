//! The kept record's schema, generated code, goldens and fingerprint (codec.md, 6).

use std::fs;
use std::path::Path;

const REGENERATE: &str =
    "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p skein-oauth-world --test suite -E 'test(/^drift::/)'";

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
fn the_saved_token_schema_has_no_drift() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/skein-oauth");
    let source = fs::read_to_string(root.join("schema/oauth-v3.schema")).expect("schema readable");
    let schema = skein_codegen::parse(&source).expect("schema valid");
    let output = skein_codegen::generate_named(&schema, skein_codegen::Names::Bounds);
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    check(&root.join("src/generated/v3.rs"), output.rust.as_bytes(), write);
    for golden in output.goldens {
        check(&root.join("golden/v3").join(golden.name), &golden.bytes, write);
    }
    let family = skein_codec::schema::parse(source.as_bytes()).expect("runtime schema valid");
    let fingerprint = family.fingerprints().family().digest();
    if write {
        let path = root.join("src/lib.rs");
        let code = fs::read_to_string(&path).expect("crate root readable");
        let (before, declaration) = code.split_once("pub const FINGERPRINT:").expect("fingerprint declaration");
        let (_, after) = declaration.split_once(";\n").expect("fingerprint declaration ends");
        let code = format!("{before}pub const FINGERPRINT: [u8; 32] = {fingerprint:?};\n{after}");
        fs::write(path, code).expect("write fingerprint");
    } else {
        assert_eq!(skein_oauth::FINGERPRINT, fingerprint, "{REGENERATE}");
    }
}
