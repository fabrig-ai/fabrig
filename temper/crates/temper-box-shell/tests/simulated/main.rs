//! The shipped fake hosts over the simulated kernel (temper/docs/design/testing/box/README.md, 3.12).
mod disk;
mod forge;
mod llm;
mod wire;

use serde_json::Value;
use std::path::{Path, PathBuf};
use temper_box_shell::fake::{Arguments, Kind, Source};

const SEED: u64 = 23;
const CHECKPOINT: &[u8] = b"{\"observations_lost\":0}\n";
const DOCUMENT: &str = "/fixture/document.json";
const OBSERVATIONS: &str = "/fixture/observations";

fn arguments(kind: Kind, source: Source) -> Arguments {
    Arguments {
        kind,
        source,
        seed: SEED,
        ready: Some(b"ready ".as_slice().into()),
        observations: Some(PathBuf::from(OBSERVATIONS)),
        state: None,
        scripts: None,
        strict: false,
    }
}
fn document(tree: &mut disk::Tree, value: &Value) -> Source {
    tree.seed(DOCUMENT.as_ref(), &serde_json::to_vec(value).expect("fixture document"));
    Source::Document(DOCUMENT.into())
}
fn private_hex(tree: &disk::Tree, path: &Path, expected: &[u8]) {
    let bytes = tree.bytes(path);
    assert_eq!(bytes.len(), 40);
    assert!(bytes.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
    assert_eq!(bytes, expected);
    assert_eq!(tree.mode(path), 0o600);
}
fn observations(bytes: &[u8]) -> Vec<Value> {
    std::str::from_utf8(bytes)
        .expect("observations text")
        .lines()
        .map(|line| serde_json::from_str(line).expect("observation record"))
        .collect()
}
