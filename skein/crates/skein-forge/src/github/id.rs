//! Exact GitHub ids (forge-github.md, section 3).
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
    for (pair, byte) in hex.chunks_exact(2).zip(out.iter_mut()) {
        *byte = bytes::hex_value(*pair.first()?)?.checked_mul(16)?.checked_add(bytes::hex_value(*pair.get(1)?)?)?;
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
    for (byte, pair) in commit.iter().take(20).zip(out.chunks_exact_mut(2)) {
        *pair.first_mut()? = bytes::hex_digit(byte / 16);
        *pair.get_mut(1)? = bytes::hex_digit(byte % 16);
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
