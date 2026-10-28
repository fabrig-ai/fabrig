//! Stable names for requests (jig's client/domain/contract.md, 3.1): the same
//! derivation as `temper-client-tools`' `jig::key`, which stays jig's own; a
//! ctl test pins the first repository's keys to the same bytes.
use alloc::boxed::Box;
use skein_lib::{RequestKey, sha256::Sha256};
/// One to 64 lower-case letters, digits or hyphens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    bytes: Box<[u8]>,
}
impl Name {
    #[must_use]
    pub fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes.len() > 64 {
            return None;
        }
        for byte in bytes {
            if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
                return None;
            }
        }
        Some(Self { bytes: Box::from(bytes) })
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
/// First sixteen bytes of the salt, zero separator and name's digest.
#[must_use]
pub fn named(salt: &[u8], name: &Name) -> RequestKey {
    let mut hash = Sha256::new();
    hash.update(salt);
    hash.update(&[0]);
    hash.update(name.bytes());
    let digest = hash.finish();
    let mut key = [0; 16];
    for (slot, byte) in key.iter_mut().zip(&digest) {
        *slot = *byte;
    }
    key
}

/// Salt shared with named ctl requests (client/tools.md, 7).
pub const TOOLS: &[u8] = b"temper-tools";
