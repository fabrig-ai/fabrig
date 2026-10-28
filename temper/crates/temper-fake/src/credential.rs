//! The document form draws its own reproducible credentials (protocol/setup.md, 8.4).
use skein_lib::Rng;

/// The model key's stream, apart from the domain and the admin token.
pub const KEY: u64 = 0x7465_6d70_6572_6b79;
/// The forge handover's stream, apart from the domain and the model key.
pub const ADMIN_TOKEN: u64 = 0x7465_6d70_6572_6164;
/// Further accepted model keys; the first key retains credential index zero.
pub const ADDED_KEYS: u32 = 15;

/// A stream for the zero-based position in the document's added keys.
#[must_use]
pub fn added_key(index: u32) -> u64 {
    assert!(index < ADDED_KEYS, "checked added key index");
    KEY.checked_add(1).expect("added key streams").checked_add(u64::from(index)).expect("fifteen added keys")
}

/// Twenty seeded bytes as forty lowercase hex digits. Never draws from a domain.
#[must_use]
pub fn credential(seed: u64, stream: u64) -> [u8; 40] {
    let mut rng = Rng::new(seed ^ stream);
    let mut hex = [0_u8; 40];
    for pair in hex.chunks_exact_mut(2) {
        let byte = u8::try_from(rng.below(256)).expect("byte draw");
        let high = byte.wrapping_shr(4);
        let low = byte & 15;
        *pair.first_mut().expect("two digits") = digit(high);
        *pair.last_mut().expect("two digits") = digit(low);
    }
    hex
}
const fn digit(n: u8) -> u8 {
    if n < 10 { b'0'.wrapping_add(n) } else { b'a'.wrapping_add(n.wrapping_sub(10)) }
}
