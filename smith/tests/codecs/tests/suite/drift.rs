//! Generated records and pinned constructs follow skein/docs/design/data/codec/README.md, section 6.

use std::{fs, path::Path};

use skein_codegen::{generate, parse, unpinned};

const REGENERATE: &str =
    "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p smith-codecs-world --test suite -E 'test(/^drift::/)'";

#[test]
fn charter_schema_matches_generated_code_and_pinned_constructs() {
    assert_family("charter", 5);
}

#[test]
fn transcript_schema_matches_generated_code_and_pinned_constructs() {
    assert_family("transcript", 7);
}

#[test]
fn channel_schema_matches_generated_code_and_pinned_constructs() {
    assert_family("channel", smith_channel::wire::VERSION);
}

fn assert_family(family: &str, version: u16) {
    let root = skein_scratch::paths::tree().join(format!("crates/smith-{family}"));
    let source = fs::read_to_string(root.join(format!("schema/{family}-v{version}.schema"))).expect("schema source");
    check_records(&root, family, family, &source);
}

use crate::support::events;

#[test]
fn event_records_and_every_listed_value_match_frozen_version_four_goldens() {
    let root = skein_scratch::paths::tree().join("crates/smith-events/golden/v4");
    let limits = smith_events::Limits { items: 8, ..smith_events_fixtures::SMALL };
    let samples = events::samples();
    assert_eq!(fs::read_dir(&root).expect("event goldens").count(), samples.len(), "every golden is covered");
    check_event_goldens(&root, &limits, &samples);
}

fn check_event_goldens(
    root: &Path,
    limits: &smith_events::Limits,
    rows: &[(&str, smith_events::Record, smith_events::Capture)],
) {
    for (name, record, capture) in rows {
        let path = root.join(format!("{name}.jsonl"));
        if std::env::var_os("SKEIN_CODEGEN_WRITE").as_deref() == Some(std::ffi::OsStr::new("1")) {
            let encoded = smith_events::write(record, capture, limits).expect("bounded fixture").expect("event");
            check_artifact(&path, &encoded);
        }
        let frozen = fs::read(&path).expect("frozen event golden");
        let ceiling = u32::try_from(frozen.len()).expect("golden length");
        let encoded = smith_events::write_with_ceiling(record, capture, limits, ceiling)
            .expect("bounded fixture")
            .expect("event");
        assert_eq!(encoded.as_ref(), frozen, "event {name} drifted");
        let decoded =
            smith_events::read_with_ceiling(&frozen, limits, ceiling).expect("valid golden").expect("known record");
        let rewritten = smith_events::write_with_ceiling(&decoded, capture, limits, ceiling)
            .expect("bounded decoded fixture")
            .expect("event");
        assert_eq!(rewritten.as_ref(), frozen, "golden {name} reads back");
    }
}

#[test]
fn local_store_schema_matches_generated_code_and_pinned_constructs() {
    let crates = skein_scratch::paths::tree().join("crates");
    let root = crates.join("smith-local-protocol");
    let source = fs::read_to_string(root.join("schema/local-store-v1.schema")).expect("local store schema");
    check_records(&crates.join("smith-local-protocol-codec"), "local_store", "local-store", &source);
}

fn check_records(root: &Path, file: &str, family: &str, source: &str) {
    let missing = unpinned(source).expect("valid family schema");
    assert!(missing.is_empty(), "{family} uses unpinned constructs: {missing:?}");
    let output = generate(&parse(source).expect("valid family schema"));
    check_artifact(&root.join(format!("src/generated/{file}.rs")), output.records.as_bytes());
}

fn check_artifact(path: &Path, expected: &[u8]) {
    if std::env::var_os("SKEIN_CODEGEN_WRITE").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("artifact directory")).expect("artifact directory");
        fs::write(path, expected).expect("write generated artifact");
    }
    assert!(fs::read(path).ok().as_deref() == Some(expected), "{} drifted; {REGENERATE}", path.display());
}
