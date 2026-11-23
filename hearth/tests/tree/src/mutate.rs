//! Eight mutations of frames obtained from the real record writers.
pub use skein_checks::MutationBatch;
use skein_lib::Rng;

pub fn frames(rng: &mut Rng, frame: &[u8]) -> impl Iterator<Item = skein_checks::Mutation<Vec<u8>>> + use<> {
    let mut changed = Vec::with_capacity(8);
    let at = usize::try_from(rng.below(u64::try_from(frame.len()).expect("frame size"))).expect("offset");
    let mut value = frame.to_vec();
    value[at] ^= u8::try_from(rng.between(1, 255)).expect("nonzero byte");
    changed.push(value);
    let mut value = frame.to_vec();
    value.remove(at);
    changed.push(value);
    let mut value = frame.to_vec();
    value.insert(at, u8::try_from(rng.below(256)).expect("byte"));
    changed.push(value);
    for difference in [-1_i64, 1] {
        let mut value = frame.to_vec();
        let length = u64::from_be_bytes(value[16..24].try_into().expect("header length"));
        let length = length.checked_add_signed(difference).expect("nonempty bounded body");
        value[16..24].copy_from_slice(&length.to_be_bytes());
        changed.push(value);
    }
    let mut value = frame.to_vec();
    value[9] ^= 1;
    changed.push(value);
    let mut value = frame.to_vec();
    value[11] ^= 1;
    changed.push(value);
    let cut = usize::try_from(rng.below(u64::try_from(frame.len()).expect("frame size"))).expect("cut");
    changed.push(frame[..cut].to_vec());
    changed.into_iter().map(skein_checks::mutation)
}
