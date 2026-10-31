//! The committed codecs and golden bytes against their schemas (skein's codec.md, 6).
use std::{fmt::Write as _, fs, path::PathBuf};
#[path = "../../../codec-families.rs"]
mod families;

fn store() -> PathBuf {
    skein_scratch::paths::tree().join("crates/jig-protocol-store")
}

#[test]
fn codec_families_match_their_schemas_and_goldens() {
    families::check();
}

/// The layouts' fingerprints the store keeps (store.md 5.3), from the schemas.
fn layouts() -> String {
    let mut code = String::from(
        "//! Each kept family's layout fingerprint in this build's schemas (store.md 5.3).\n\
         //! Written and checked by `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world --test drift`.\n\
         use crate::codec::component::Layouts;\n\
         \n\
         pub(crate) const LAYOUTS: Layouts = Layouts {\n",
    );
    for family in families::FAMILIES.iter().filter(|family| family.root == "crates/jig-protocol-store") {
        let field = family.name;
        let source = fs::read(
            skein_scratch::paths::tree().join(family.root).join(family.schemas.first().expect("store schema")),
        )
        .expect("schema readable");
        let digest = skein_codec::schema::parse(&source).expect("schema valid").fingerprints().family().digest();
        let bytes: Vec<String> = digest.iter().map(|byte| format!("0x{byte:02x}")).collect();
        writeln!(code, "    {field}: [{}],", bytes.join(", ")).expect("a String takes the line");
    }
    let smith = fs::read(store().join("src/tests/fixtures/smith-transcript-v6.schema")).expect("checked smith schema");
    let smith = skein_codec::schema::parse(&smith).expect("smith schema valid").fingerprints().family().digest();
    for (field, digest) in [("skein_oauth", skein_oauth::FINGERPRINT), ("smith_transcript", smith)] {
        let bytes: Vec<String> = digest.iter().map(|byte| format!("0x{byte:02x}")).collect();
        writeln!(code, "    {field}: [{}],", bytes.join(", ")).expect("a String takes the line");
    }
    code.push_str("};\n");
    code
}

#[test]
fn the_stores_layouts_have_no_drift() {
    let path = store().join("src/generated/layouts.rs");
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        fs::write(&path, layouts()).expect("write layouts");
    } else {
        assert_eq!(
            fs::read_to_string(&path).expect("layouts committed"),
            layouts(),
            "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world --test drift"
        );
    }
}
