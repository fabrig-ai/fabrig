//! Shared deterministic codec inputs (skein's data/codec/golden.md, section 1).
pub use skein_checks::MutationBatch;
use skein_lib::Rng;

/// Draw arbitrary bytes of any length through the given maximum.
#[must_use]
pub fn drawn(rng: &mut Rng, most: u32) -> Vec<u8> {
    let length = rng.between(0, u64::from(most));
    (0..length).map(|_| u8::try_from(rng.below(256)).expect("one byte")).collect()
}

/// Flip, remove or insert one byte of a fixed-rule value value.
pub fn mutated(rng: &mut Rng, fixed_value: &[u8]) -> skein_checks::Mutation<Vec<u8>> {
    let mut bytes = Vec::new();
    write_mutation(rng, fixed_value, &mut bytes);
    skein_checks::mutation(bytes)
}

fn write_mutation(rng: &mut Rng, fixed_value: &[u8], bytes: &mut Vec<u8>) {
    bytes.clear();
    bytes.extend_from_slice(fixed_value);
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
