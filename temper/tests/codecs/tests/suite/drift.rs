//! The committed records and pinned constructs against their schemas (skein's data/codec/README.md, 6).
use skein_codegen::{generate, parse};
use std::fs;
use std::path::Path;

fn schema_matches(crate_name: &str, codec_crate: &str, family: &str, version: &str, module: &str) {
    let crates = skein_scratch::paths::tree().join("crates");
    let root = crates.join(crate_name);
    let source =
        fs::read_to_string(root.join("schema").join(format!("{family}-{version}.schema"))).expect("family schema");
    let unpinned = skein_codegen::unpinned(&source).expect("schema valid");
    assert!(unpinned.is_empty(), "{family}: unpinned constructs: {unpinned:?}");
    let output = generate(&parse(&source).expect("schema valid"));
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
fn worker_wire_schema_matches_generated_code_and_pinned_constructs() {
    schema_matches("temper-protocol-worker-wire", "temper-protocol-worker-wire", "worker-wire", "v1", "worker_wire");
}

#[test]
fn client_wire_schema_matches_generated_code_and_pinned_constructs() {
    client_layout("client-wire-v1.schema", "temper-client-wire");
    schema_matches("temper-client-protocol-wire", "temper-client-protocol-wire", "client-wire", "v1", "client_wire");
}

// Client layouts have no bridge; preserve every older line and append kept
// cumulative versions before the current fingerprint (protocol/wires.md, 9).
fn client_layout(source: &str, family: &str) {
    use std::{collections::BTreeMap, fmt::Write};
    let root = skein_scratch::paths::tree().join("crates/temper-client-protocol-wire/schema");
    let source = fs::read(root.join(source)).expect("client schema");
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
fn store_schema_matches_generated_code_and_pinned_constructs() {
    schema_matches("temper-protocol-store", "temper-protocol-store-codec", "store", "v25", "store");
}

#[test]
fn secrets_schema_matches_generated_code_and_pinned_constructs() {
    schema_matches("temper-protocol-records", "temper-protocol-records", "secrets", "v1", "secrets");
}

#[test]
fn public_wire_schema_matches_generated_code_and_pinned_constructs() {
    client_layout("public-v1.schema", "temper-public-wire");
    schema_matches("temper-client-protocol-wire", "temper-client-protocol-wire", "public", "v1", "public");
}
