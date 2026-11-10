//! Shared deterministic codec inputs (skein's data/codec/README.md, section 6).
use skein_lib::Rng;
use std::fs;

/// Read this directory's golden files in a stable order, relative to the tree root.
#[must_use]
pub fn goldens(dir: &str) -> Vec<Vec<u8>> {
    let root = skein_scratch::paths::tree().join(dir);
    let mut paths: Vec<_> = fs::read_dir(root)
        .expect("golden directory")
        .map(|entry| entry.expect("golden entry"))
        .filter(|entry| entry.file_type().expect("golden file type").is_file())
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    paths.into_iter().map(|path| fs::read(path).expect("golden bytes")).collect()
}

/// Draw arbitrary bytes of any length through the given maximum.
#[must_use]
pub fn drawn(rng: &mut Rng, most: u32) -> Vec<u8> {
    let length = rng.between(0, u64::from(most));
    (0..length).map(|_| u8::try_from(rng.below(256)).expect("one byte")).collect()
}

/// Flip, remove or insert one byte of a golden value.
#[must_use]
pub fn mutated(rng: &mut Rng, golden: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    mutated_into(rng, golden, &mut bytes);
    bytes
}

/// The same mutation, retaining scratch capacity between complete inputs.
pub fn mutated_into(rng: &mut Rng, golden: &[u8], bytes: &mut Vec<u8>) {
    bytes.clear();
    bytes.extend_from_slice(golden);
    let length = skein_lib::len_u64(bytes.len());
    match rng.below(3) {
        0 if !bytes.is_empty() => {
            let at = usize::try_from(rng.below(length)).expect("a byte position fits");
            bytes[at] ^= 1 << rng.below(8);
        }
        1 if !bytes.is_empty() => {
            let at = usize::try_from(rng.below(length)).expect("a byte position fits");
            bytes.remove(at);
        }
        _ => {
            let at = usize::try_from(rng.between(0, length)).expect("a byte position fits");
            let byte = u8::try_from(rng.below(256)).expect("one byte");
            bytes.insert(at, byte);
        }
    }
}
