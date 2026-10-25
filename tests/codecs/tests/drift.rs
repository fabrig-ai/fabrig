//! The committed codecs and golden bytes against their schemas (skein's codec.md, 6).
use skein_codegen::{generate, parse};
use std::fs;

#[test]
fn worker_wire_schema_matches_generated_code_and_goldens() {
    let root = skein_scratch::paths::workspace().join("crates/temper-protocol-worker-wire");
    let source = fs::read_to_string(root.join("schema/worker-wire-v1.schema")).expect("worker wire schema");
    let output = generate(&parse(&source).expect("schema valid"));
    let code = root.join("src/generated/v1.rs");
    let goldens = root.join("golden");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test drift";
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
fn client_wire_schema_matches_generated_code_and_goldens() {
    let root = skein_scratch::paths::workspace().join("crates/temper-client-protocol-wire");
    let source = fs::read_to_string(root.join("schema/client-wire-v1.schema")).expect("client wire schema");
    let output = generate(&parse(&source).expect("schema valid"));
    let code = root.join("src/generated/v1.rs");
    let goldens = root.join("golden");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test drift";
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
fn store_schema_matches_generated_code_and_goldens() {
    let root = skein_scratch::paths::workspace().join("crates/temper-protocol-store");
    let source = fs::read_to_string(root.join("schema/store-v10.schema")).expect("store schema");
    let output = generate(&parse(&source).expect("schema valid"));
    let code = root.join("src/generated/v10.rs");
    let goldens = root.join("golden");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test drift";
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
