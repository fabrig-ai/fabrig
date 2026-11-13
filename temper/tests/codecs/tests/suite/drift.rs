//! The committed codecs and golden bytes against their schemas (skein's data/codec/README.md, 6).
use skein_codegen::{generate, parse};
use std::fs;
mod drafts;

fn schema_matches(crate_name: &str, codec_crate: &str, family: &str, version: &str) {
    let crates = skein_scratch::paths::tree().join("crates");
    let root = crates.join(crate_name);
    let source =
        fs::read_to_string(root.join("schema").join(format!("{family}-{version}.schema"))).expect("family schema");
    let mut output = generate(&parse(&source).expect("schema valid"));
    if crate_name == "temper-client-protocol-wire" && family == "client-wire" {
        output.goldens.extend(drafts::goldens());
    }
    let code_name = if family == "public" { format!("public_{version}.rs") } else { format!("{version}.rs") };
    let code = crates.join(codec_crate).join("src/generated").join(code_name);
    let goldens = match family {
        "store" => root.join("golden").join(version),
        "public" => root.join("golden/public"),
        _ => root.join("golden"),
    };
    let write = temper_test_env::rewrite();
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'";
    if write {
        fs::write(&code, &output.rust).expect("write generated code");
        fs::create_dir_all(&goldens).expect("create golden directory");
    } else {
        assert_eq!(fs::read_to_string(&code).expect("generated code committed"), output.rust, "{again}");
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
fn worker_wire_schema_matches_generated_code_and_goldens() {
    schema_matches("temper-protocol-worker-wire", "temper-protocol-worker-wire", "worker-wire", "v1");
}

#[test]
fn client_wire_schema_matches_generated_code_and_goldens() {
    schema_matches("temper-client-protocol-wire", "temper-client-protocol-wire", "client-wire", "v1");
}

#[test]
fn store_schema_matches_generated_code_and_goldens() {
    schema_matches("temper-protocol-store", "temper-protocol-store-codec", "store", "v21");
}

#[test]
fn secrets_schema_matches_generated_code_and_goldens() {
    schema_matches("temper-protocol-records", "temper-protocol-records", "secrets", "v1");
}

#[test]
fn public_wire_schema_matches_generated_code_and_goldens() {
    schema_matches("temper-client-protocol-wire", "temper-client-protocol-wire", "public", "v1");
}
