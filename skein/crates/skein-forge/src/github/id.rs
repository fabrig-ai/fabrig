//! Exact GitHub ids (forge-github.md, section 3).
use crate::api;
use skein_lib::hex::{self, Case};
/// Reads 40 lowercase hexadecimal digits, zero-padded to 32 bytes.
#[must_use]
pub fn commit(text: &[u8]) -> Option<api::Commit> {
    if text.len() != 40 {
        return None;
    }
    let mut bytes = [0; 32];
    for (pair, byte) in text.chunks_exact(2).zip(bytes.iter_mut()) {
        *byte = hex::read_lower(*pair.first()?)?.checked_mul(16)?.checked_add(hex::read_lower(*pair.get(1)?)?)?;
    }
    Some(bytes)
}
/// Writes the 40 digits of a commit whose last 12 bytes are zero.
#[must_use]
pub fn write_commit(commit: &api::Commit) -> Option<[u8; 40]> {
    for byte in commit.iter().skip(20) {
        if *byte != 0 {
            return None;
        }
    }
    let mut out = [0; 40];
    for (byte, pair) in commit.iter().take(20).zip(out.chunks_exact_mut(2)) {
        let [high, low] = hex::pair(*byte, Case::Lower);
        *pair.first_mut()? = high;
        *pair.get_mut(1)? = low;
    }
    Some(out)
}
/// Narrows a repository id to the vocabulary's 32 bits.
pub fn repository(id: u64) -> Result<u32, api::Error> {
    match u32::try_from(id) {
        Ok(id) => Ok(id),
        Err(_) => Err(api::Error::TooLarge),
    }
}
