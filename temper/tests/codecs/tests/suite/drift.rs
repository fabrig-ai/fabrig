//! The committed records against their schemas (skein's data/codec/README.md, 6).
use skein_codegen::{generate, values};
use std::fs;
use std::path::Path;

fn schema_matches(codec_crate: &str, module: &str, source: &'static str) {
    let crates = skein_scratch::paths::tree().join("crates");

    let schema = &values(source).schema;
    let output = generate(schema);
    let code = crates.join(codec_crate).join("src/generated").join(format!("{module}.rs"));
    check_text(&code, &output.records, temper_test_env::rewrite());
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
fn worker_wire_schema_matches_generated_code() {
    schema_matches("temper-protocol-worker-wire", "worker_wire", temper_protocol_worker_wire::SCHEMA);
}

#[test]
fn client_wire_schema_matches_generated_code() {
    client_layout(temper_client_protocol_wire::CLIENT_SCHEMA, "temper-client-wire");
    schema_matches("temper-client-protocol-wire", "client_wire", temper_client_protocol_wire::CLIENT_SCHEMA);
}

// Client layouts have no bridge; preserve every older line and append kept
// cumulative versions before the current fingerprint (protocol/wires.md, 9).
fn client_layout(source: &str, family: &str) {
    use std::{collections::BTreeMap, fmt::Write};
    let root = skein_scratch::paths::tree().join("crates/temper-client-protocol-wire/schema");
    let source = source.as_bytes().to_vec();
    let schema = skein_codec::schema::parse(&source).expect("client schema valid");
    let current = schema.version();
    let mut texts = BTreeMap::new();
    let kept = root.join("kept");
    if kept.exists() {
        for entry in fs::read_dir(kept).expect("kept client schemas") {
            let path = entry.expect("kept client schema").path();
            if path.extension().is_some_and(|extension| extension == "schema") {
                let text = fs::read(&path).expect("kept client schema text");
                let held = skein_codec::schema::parse(&text).expect("kept client schema valid");
                if held.spelled(held.family()) == family.as_bytes() && held.version() <= current {
                    texts.insert(held.version(), text);
                }
            }
        }
    }
    texts.insert(current, source);
    let path = root.join(format!("{family}.layouts"));
    let mut layouts = fs::read_to_string(&path).expect("client layouts");
    let write = temper_test_env::rewrite();
    for (version, text) in texts {
        let schema = skein_codec::schema::parse(&text).expect("client schema valid");
        let mut line = format!("{version} ");
        for byte in schema.fingerprints().family().digest() {
            write!(&mut line, "{byte:02x}").expect("fingerprint text");
        }
        line.push('\n');
        let prefix = format!("{version} ");
        if let Some(held) = layouts.lines().find(|held| held.starts_with(&prefix)) {
            assert_eq!(held, line.trim_end(), "a committed layout is immutable");
        } else if write {
            let previous = layouts
                .lines()
                .last()
                .and_then(|held| held.split_once(' '))
                .map(|(version, _)| version.parse::<u16>().expect("layout version"));
            assert_eq!(previous.and_then(|version| version.checked_add(1)), Some(version), "contiguous client layouts");
            layouts.push_str(&line);
        } else {
            panic!("missing client layout {version}: {AGAIN}");
        }
        if version == current {
            assert!(layouts.ends_with(&line), "current client layout: {AGAIN}");
        }
    }
    if write {
        fs::write(&path, layouts).expect("append client layouts");
    }
}

#[test]
fn store_schema_matches_generated_code() {
    let source = temper_protocol_store::OWNER_SET.families.first().expect("store family").current;
    schema_matches("temper-protocol-store-codec", "store", source);
    let maps = include_str!("../../../../crates/temper-protocol-store/schema/store.maps");
    let translation = skein_codegen::translate(&values(source).schema, maps).expect("store maps valid");
    let path = skein_scratch::paths::tree().join("crates/temper-protocol-store/src/generated/store.rs");
    check_text(&path, &translation, temper_test_env::rewrite());
}

#[test]
fn secrets_schema_matches_generated_code() {
    schema_matches("temper-protocol-records", "secrets", temper_protocol_records::SCHEMA);
}

#[test]
fn public_wire_schema_matches_generated_code() {
    client_layout(temper_client_protocol_wire::PUBLIC_SCHEMA, "temper-public-wire");
    schema_matches("temper-client-protocol-wire", "public", temper_client_protocol_wire::PUBLIC_SCHEMA);
}
