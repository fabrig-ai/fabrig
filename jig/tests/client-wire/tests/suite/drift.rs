//! The committed client codec and bytes against its schema (skein's codec.md, 6).
use skein_codegen::{generate, parse};
use std::{fs, path::Path};

#[test]
fn client_wire_schema_matches_generated_code_and_goldens() {
    check_client(19);
}

fn check_client(version: u16) {
    let root = skein_scratch::paths::tree().join("crates/jig-client-protocol-wire");
    let mut source = fs::read_to_string(root.join(format!("schema/client-v{version}.schema"))).expect("client schema");
    source.push_str(
        &fs::read_to_string(root.join(format!("schema/client-v{version}-part1.schema")))
            .expect("client schema continuation"),
    );
    check(&root, &source, &format!("v{version}"), version == 19);
}
#[test]
fn view_records_schema_matches_generated_code_and_goldens() {
    let root = skein_scratch::paths::tree().join("crates/jig-client-view");
    let source = fs::read_to_string(root.join("schema/records-v1.schema")).expect("view records schema");
    check(&root, &source, "records_v1", true);
}
#[test]
fn ops_client_wire_schema_matches_generated_code_and_goldens() {
    let root = skein_scratch::paths::tree().join("examples/ops/client-protocol-wire");
    let source = fs::read_to_string(root.join("schema/ops-client-v1.schema")).expect("ops client schema");
    check(&root, &source, "v1", true);
}
fn check(root: &Path, source: &str, version: &str, check_goldens: bool) {
    let output = generate(&parse(source).expect("schema valid"));
    let code = root.join(format!("src/generated/{version}.rs"));
    let goldens = root.join("golden");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test drift";
    if write {
        fs::create_dir_all(code.parent().expect("generated code directory")).expect("create code directory");
        fs::write(&code, &output.rust).expect("write generated code");
        fs::create_dir_all(&goldens).expect("create golden directory");
    } else {
        assert_eq!(fs::read_to_string(&code).expect("generated code committed"), output.rust, "{again}");
    }
    // Historical client schemas share golden names with v19, which owns them.
    if !check_goldens {
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
fn client_wire_v12_schema_matches_generated_code() {
    check_client(12);
}

#[test]
fn client_wire_v13_schema_matches_generated_code() {
    check_client(13);
}

#[test]
fn client_wire_v14_schema_matches_generated_code() {
    check_client(14);
}

#[test]
fn client_wire_v15_schema_matches_generated_code() {
    check_client(15);
}

#[test]
fn client_wire_v16_schema_matches_generated_code() {
    check_client(16);
}

#[test]
fn client_wire_v17_schema_matches_generated_code() {
    check_client(17);
}

#[test]
fn client_wire_v18_schema_matches_generated_code() {
    check_client(18);
}
