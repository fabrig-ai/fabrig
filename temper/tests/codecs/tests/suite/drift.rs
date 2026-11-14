//! The committed codecs and golden bytes against their schemas (skein's data/codec/README.md, 6).
use skein_codegen::{generate, parse, tests_main};
use std::fs;
use std::path::Path;
mod drafts;

fn schema_matches(
    crate_name: &str,
    codec_crate: &str,
    family: &str,
    version: &str,
    records: &str,
    module: &str,
    modules: &[&str],
) {
    let crates = skein_scratch::paths::tree().join("crates");
    let root = crates.join(crate_name);
    let source =
        fs::read_to_string(root.join("schema").join(format!("{family}-{version}.schema"))).expect("family schema");
    let mut output = generate(&parse(&source).expect("schema valid"));
    if crate_name == "temper-client-protocol-wire" && family == "client-wire" {
        output.goldens.extend(drafts::goldens());
    }
    let codec = crates.join(codec_crate);
    let code = codec.join("src/generated").join(format!("{module}.rs"));
    let tests = codec.join("tests/generated");
    let goldens = match family {
        "store" => root.join("golden").join(version),
        "public" => root.join("golden/public"),
        _ => root.join("golden"),
    };
    let write = temper_test_env::rewrite();
    check_text(&code, &output.records, write);
    check_text(&tests.join(format!("{module}.rs")), &output.tests.text(records), write);
    check_text(&tests.join("main.rs"), &tests_main(modules), write);
    check_goldens(&goldens, output.goldens, write);
}

fn check_goldens(directory: &Path, goldens: Vec<skein_codegen::Golden>, write: bool) {
    if write {
        fs::create_dir_all(directory).expect("create golden directory");
    }
    for golden in goldens {
        let path = directory.join(golden.name);
        if write {
            fs::write(&path, golden.bytes).expect("write golden bytes");
        } else {
            assert_eq!(fs::read(&path).expect("golden bytes committed"), golden.bytes, "{AGAIN}");
        }
    }
}

const AGAIN: &str =
    "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'";

fn check_text(path: &Path, text: &str, write: bool) {
    if write {
        fs::create_dir_all(path.parent().expect("generated file has a parent")).expect("create generated directory");
        fs::write(path, text).expect("write generated text");
    } else {
        let committed = fs::read_to_string(path).expect("generated text committed");
        assert_eq!(committed, text, "{}: {AGAIN}", path.display());
    }
}

#[test]
fn worker_wire_schema_matches_generated_code_and_goldens() {
    schema_matches(
        "temper-protocol-worker-wire",
        "temper-protocol-worker-wire",
        "worker-wire",
        "v1",
        "temper_protocol_worker_wire::v1",
        "v1",
        &["v1"],
    );
}

#[test]
fn client_wire_schema_matches_generated_code_and_goldens() {
    schema_matches(
        "temper-client-protocol-wire",
        "temper-client-protocol-wire",
        "client-wire",
        "v1",
        "temper_client_protocol_wire::v1",
        "v1",
        &["public_v1", "v1"],
    );
}

#[test]
fn store_schema_matches_generated_code_and_goldens() {
    schema_matches(
        "temper-protocol-store",
        "temper-protocol-store-codec",
        "store",
        "v23",
        "temper_protocol_store_codec::v23",
        "v23",
        &["v23"],
    );
}

#[test]
fn secrets_schema_matches_generated_code_and_goldens() {
    schema_matches(
        "temper-protocol-records",
        "temper-protocol-records",
        "secrets",
        "v1",
        "temper_protocol_records::v1",
        "v1",
        &["v1"],
    );
}

#[test]
fn public_wire_schema_matches_generated_code_and_goldens() {
    schema_matches(
        "temper-client-protocol-wire",
        "temper-client-protocol-wire",
        "public",
        "v1",
        "temper_client_protocol_wire::public_v1",
        "public_v1",
        &["public_v1", "v1"],
    );
}
