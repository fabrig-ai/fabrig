//! Exact GitHub ids (forge/github.md, section 3).
use crate::api;
use skein_lib::bytes;
/// Reads 40 lowercase hexadecimal digits, zero-padded to 32 bytes.
#[must_use]
pub fn commit(hex: &[u8]) -> Option<api::Commit> {
    if hex.len() != 40 {
        return None;
    }
    for byte in hex {
        if byte.is_ascii_uppercase() {
            return None;
        }
    }
    let mut out = [0; 32];
    for (&[high, low], byte) in hex.as_chunks::<2>().0.iter().zip(out.iter_mut()) {
        *byte = bytes::hex_value(high)?.checked_mul(16)?.checked_add(bytes::hex_value(low)?)?;
    }
    Some(out)
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
    for (byte, [high, low]) in commit.iter().take(20).zip(out.as_chunks_mut::<2>().0) {
        *high = bytes::hex_digit(byte / 16);
        *low = bytes::hex_digit(byte % 16);
    }
    Some(out)
}
/// Narrows a repository id to the vocabulary's 32 bits.
pub fn repository(id: u64) -> Result<u32, api::Error> {
    match u32::try_from(id) {
        Ok(id) => Ok(id),
        Err(_) => Err(api::Error::TooLarge(api::Excess::exact(api::Bound::Forge, u64::from(u32::MAX), id))),
    }
}
