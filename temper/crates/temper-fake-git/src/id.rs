//! The forge's counted ids (protocol/forge.md, section 9).
/// The shared fake-forge object bytes, as forty lowercase hex digits.
#[must_use]
pub fn id(count: u64) -> [u8; 40] {
    let raw = skein_fake_forge_domain::object_bytes(count);
    let mut out = [b'0'; 40];
    for (byte, digits) in raw.iter().zip(out.chunks_exact_mut(2)) {
        digits[0] = b"0123456789abcdef"[usize::from(byte >> 4)];
        digits[1] = b"0123456789abcdef"[usize::from(byte & 15)];
    }
    out
}
