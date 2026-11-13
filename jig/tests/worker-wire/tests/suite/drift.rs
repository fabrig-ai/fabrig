use skein_codegen::{generate, parse, tests_main};
use std::fs;

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh generated worker codecs and goldens"
)]
fn worker_wire_schema_matches_generated_code_and_goldens() {
    let root = skein_scratch::paths::tree().join("crates/jig-protocol-worker-wire");
    let source = fs::read_to_string(root.join("schema/worker-wire-v14.schema")).expect("schema");
    let schema = parse(&source).expect("schema valid");
    let output = generate(&schema);
    let codec = skein_scratch::paths::tree().join("crates/jig-protocol-worker-wire");
    let goldens = root.join("golden");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-worker-wire-world --test suite -E 'test(drift::)'";
    for (file, text) in [
        ("src/generated/v14.rs", output.records),
        ("tests/generated/v14.rs", output.tests.text("jig_protocol_worker_wire::v14")),
        ("tests/generated/main.rs", tests_main(&["v14"])),
    ] {
        let path = codec.join(file);
        if write {
            fs::create_dir_all(path.parent().expect("generated parent")).expect("create generated directory");
            fs::write(&path, text).expect("write generated text");
        } else {
            assert_eq!(fs::read_to_string(&path).expect("generated text"), text, "{file}: {again}");
        }
    }
    if write {
        fs::create_dir_all(&goldens).expect("create golden directory");
    }
    for golden in output.goldens {
        let path = goldens.join(&golden.name);
        if write {
            fs::write(&path, golden.bytes).expect("write golden bytes");
        } else {
            assert_eq!(fs::read(&path).expect("golden bytes"), golden.bytes, "golden {} drifted: {again}", golden.name);
        }
    }
}
