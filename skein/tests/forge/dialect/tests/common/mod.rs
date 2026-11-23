pub mod cases;
pub mod forgejo;
pub use skein_checks::MutationBatch;
use skein_lib::Rng;
pub mod admitted;
pub use crate::fixtures;
pub fn mutated(rng: &mut Rng, golden: &[u8]) -> skein_checks::Mutation<Vec<u8>> {
    let mut bytes = Vec::new();
    write_mutation(rng, golden, &mut bytes);
    skein_checks::mutation(bytes)
}
pub fn mutated_into<'a>(rng: &mut Rng, golden: &[u8], bytes: &'a mut Vec<u8>) -> skein_checks::Mutation<&'a [u8]> {
    write_mutation(rng, golden, bytes);
    skein_checks::mutation(bytes.as_slice())
}

fn write_mutation(rng: &mut Rng, golden: &[u8], bytes: &mut Vec<u8>) {
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

pub mod generated;

pub mod goldens;
pub mod json;
