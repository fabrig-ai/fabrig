//! The committed codecs and golden bytes against their schemas (skein's codec.md, 6).
use skein_codegen::{Names, generate_named, parse};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

fn store() -> PathBuf {
    skein_scratch::paths::tree().join("crates/jig-protocol-store")
}

fn check(family: &str, version: u16, names: Names) {
    let source =
        fs::read_to_string(store().join(format!("schema/{family}-v{version}.schema"))).expect("schema readable");
    let output = generate_named(&parse(&source).expect("schema valid"), names);
    let code = store().join(format!("src/generated/{family}_v{version}.rs"));
    let goldens = store().join(format!("golden/{family}"));
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world --test drift";
    if write {
        fs::write(&code, &output.rust).expect("write generated code");
        fs::create_dir_all(&goldens).expect("create golden directory");
    } else {
        assert_eq!(fs::read_to_string(&code).expect("generated code committed"), output.rust, "{again}");
    }
    // Historical schemas share golden names with the current family version.
    // Their writer owns only code; the current version owns the golden directory.
    if matches!(
        (family, version),
        ("accounts", 1) | ("component", 2) | ("core", 4 | 5) | ("people", 7..=12) | ("tasks", 7)
    ) {
        return;
    }
    for golden in output.goldens {
        let path = goldens.join(golden.name);
        if write {
            fs::write(&path, golden.bytes).expect("write golden bytes");
        } else {
            assert_eq!(fs::read(&path).expect("golden bytes committed"), golden.bytes, "{again}");
        }
    }
}

#[test]
fn the_notes_familys_code_and_goldens_have_no_drift() {
    check("notes", 1, Names::Bounds);
}

#[test]
fn the_component_familys_code_and_goldens_have_no_drift() {
    check("component", 3, Names::Bounds);
}

#[test]
fn the_accounts_familys_code_and_goldens_have_no_drift() {
    check("accounts", 2, Names::Bounds);
}

#[test]
fn the_tasks_familys_code_and_goldens_have_no_drift() {
    check("tasks", 9, Names::Bounds);
}

#[test]
fn the_people_familys_code_and_goldens_have_no_drift() {
    check("people", 13, Names::Both);
}

#[test]
fn the_core_familys_code_and_goldens_have_no_drift() {
    check("core", 6, Names::Both);
}

#[test]
fn the_test_connectors_code_and_goldens_have_no_drift() {
    check_test_connector(2, Names::Bounds);
}

#[test]
fn the_test_connector_v1_code_has_no_drift() {
    check_test_connector(1, Names::Bounds);
}

fn check_test_connector(version: u16, names: Names) {
    let root = skein_scratch::paths::tree().join("testing/jig-test-store");
    let source =
        fs::read_to_string(root.join(format!("schema/test-connector-v{version}.schema"))).expect("schema readable");
    let output = generate_named(&parse(&source).expect("schema valid"), names);
    let code = root.join(format!("src/generated/v{version}.rs"));
    let goldens = root.join("golden");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    if write {
        fs::write(code, &output.rust).expect("write code");
        fs::create_dir_all(&goldens).expect("golden directory");
    } else {
        assert_eq!(fs::read_to_string(code).expect("code committed"), output.rust);
    }
    // Both versions use the same golden names; v2 owns those bytes.
    if version == 1 {
        return;
    }
    for golden in output.goldens {
        let path = goldens.join(golden.name);
        if write {
            fs::write(path, golden.bytes).expect("write golden");
        } else {
            assert_eq!(fs::read(path).expect("golden committed"), golden.bytes);
        }
    }
}

#[test]
fn the_settings_familys_code_and_goldens_have_no_drift() {
    check("settings", 1, Names::Bounds);
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
    for (field, family, version) in [
        ("component", "component", 3),
        ("core", "core", 6),
        ("tasks", "tasks", 9),
        ("people", "people", 13),
        ("notes", "notes", 1),
        ("accounts", "accounts", 2),
        ("settings", "settings", 1),
        ("secrets", "secrets", 1),
    ] {
        let source = fs::read(store().join(format!("schema/{family}-v{version}.schema"))).expect("schema readable");
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

#[test]
fn the_secrets_familys_code_and_goldens_have_no_drift() {
    check("secrets", 1, Names::Both);
}

#[test]
fn the_accounts_v1_code_has_no_drift() {
    check("accounts", 1, Names::Bounds);
}

#[test]
fn the_component_v2_code_has_no_drift() {
    check("component", 2, Names::Bounds);
}

#[test]
fn the_core_v4_code_has_no_drift() {
    check("core", 4, Names::Bounds);
}

#[test]
fn the_core_v5_code_has_no_drift() {
    check("core", 5, Names::Bounds);
}

#[test]
fn the_people_v7_code_has_no_drift() {
    check("people", 7, Names::Bounds);
}

#[test]
fn the_people_v8_code_has_no_drift() {
    check("people", 8, Names::Bounds);
}

#[test]
fn the_people_v9_code_has_no_drift() {
    check("people", 9, Names::Bounds);
}

#[test]
fn the_people_v10_code_has_no_drift() {
    check("people", 10, Names::Bounds);
}

#[test]
fn the_people_v11_code_has_no_drift() {
    check("people", 11, Names::Bounds);
}

#[test]
fn the_people_v12_code_has_no_drift() {
    check("people", 12, Names::Bounds);
}

#[test]
fn the_tasks_v7_code_has_no_drift() {
    check("tasks", 7, Names::Bounds);
}
