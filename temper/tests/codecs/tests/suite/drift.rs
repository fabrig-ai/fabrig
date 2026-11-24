//! The committed records and bounds (skein/docs/design/data/codec/golden.md, 1).
use skein_codec::bounds::Declaration;
use skein_codegen::{bounds_text, generate, values};
use std::fs;
use std::path::Path;

fn schema_matches(
    codec_crate: &str,
    module: &str,
    source: &'static str,
    declarations: &[Declaration],
    ceilings: &[u32],
) {
    let crates = skein_scratch::paths::tree().join("crates");
    let write = temper_test_env::rewrite();

    let schema = &values(source).schema;
    let output = generate(schema);
    let code = crates.join(codec_crate).join("src/generated").join(format!("{module}.rs"));
    check_text(&code, &output.records, write);

    let path = skein_scratch::paths::package().join("bounds").join(format!("{module}.bounds"));
    check_text(&path, &bounds_text(declarations, ceilings), write);
}

const AGAIN: &str =
    "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'";

fn check_text(path: &Path, text: &str, write: bool) {
    if write {
        fs::create_dir_all(path.parent().expect("generated file has a parent")).expect("create generated directory");
        fs::write(path, text).expect("write generated text");
    } else {
        let committed = fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}; {AGAIN}", path.display()));
        assert_eq!(committed, text, "{}: {AGAIN}", path.display());
    }
}

#[test]
fn worker_wire_schema_matches_generated_code() {
    use temper_protocol_worker_wire::{SCHEMA, v1 as codec};
    schema_matches(
        "temper-protocol-worker-wire",
        "worker_wire",
        SCHEMA,
        codec::DECLARATIONS,
        codec::CEILINGS.values().as_slice(),
    );
}

#[test]
fn client_wire_schema_matches_generated_code() {
    use temper_client_protocol_wire::{CLIENT_SCHEMA, v1 as codec};
    schema_matches(
        "temper-client-protocol-wire",
        "client_wire",
        CLIENT_SCHEMA,
        codec::DECLARATIONS,
        codec::CEILINGS.values().as_slice(),
    );
}

#[test]
fn store_schema_matches_generated_code() {
    use temper_protocol_store::{OWNER_SET, v25 as codec};
    let source = OWNER_SET.families.first().expect("store family").current;
    schema_matches(
        "temper-protocol-store-codec",
        "store",
        source,
        codec::DECLARATIONS,
        codec::CEILINGS.values().as_slice(),
    );
    let maps = include_str!("../../../../crates/temper-protocol-store/schema/store.maps");
    let translation = skein_codegen::translate(&values(source).schema, maps).expect("store maps valid");
    let path = skein_scratch::paths::tree().join("crates/temper-protocol-store/src/generated/store.rs");
    check_text(&path, &translation, temper_test_env::rewrite());
}

#[test]
fn secrets_schema_matches_generated_code() {
    use temper_protocol_records::{SCHEMA, v1 as codec};
    schema_matches(
        "temper-protocol-records",
        "secrets",
        SCHEMA,
        codec::DECLARATIONS,
        codec::CEILINGS.values().as_slice(),
    );
}

#[test]
fn public_wire_schema_matches_generated_code() {
    use temper_client_protocol_wire::{PUBLIC_SCHEMA, public_v1 as codec};
    schema_matches(
        "temper-client-protocol-wire",
        "public",
        PUBLIC_SCHEMA,
        codec::DECLARATIONS,
        codec::CEILINGS.values().as_slice(),
    );
}
