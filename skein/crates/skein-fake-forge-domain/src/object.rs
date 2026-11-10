//! The fake's SHA-1 spelling (skein/docs/design/testing/fakes/forge.md, 4, and skein/docs/design/testing/fakes/github.md, 3), shared by its faces and queue.

#[cfg(test)]
#[path = "tests/object.rs"]
mod tests;

/// A `SplitMix64` finalizer of the count, then the count, big-endian, then four zero bytes.
#[must_use]
pub fn object_bytes(count: u64) -> [u8; 20] {
    let mut bytes = [0; 20];
    for (out, byte) in bytes.iter_mut().zip(mix(count).to_be_bytes().into_iter().chain(count.to_be_bytes())) {
        *out = byte;
    }
    bytes
}

/// A commit count's 32 bytes: its SHA-1 spelling, padded.
#[must_use]
pub fn commit_bytes(count: u64) -> [u8; 32] {
    let mut bytes = [0; 32];
    for (out, byte) in bytes.iter_mut().zip(object_bytes(count)) {
        *out = byte;
    }
    bytes
}

/// The count those bytes spell, if they're the fake's.
#[must_use]
pub fn commit_count(bytes: &[u8; 32]) -> Option<u64> {
    let mut count = 0u64;
    for byte in bytes.get(8..16)? {
        count = count.checked_mul(256)?.checked_add(u64::from(*byte))?;
    }
    if commit_bytes(count) == *bytes { Some(count) } else { None }
}

fn mix(count: u64) -> u64 {
    let z = count.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let z = (z ^ z.checked_shr(30).expect("fixed shift")).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let z = (z ^ z.checked_shr(27).expect("fixed shift")).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ z.checked_shr(31).expect("fixed shift")
}
