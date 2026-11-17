//! Checkpoint names and bounds (protocol/store/checkpoints.md, 2.1, 2.3 and 7).
pub(crate) mod files;
pub(crate) mod named;
pub(crate) mod retention;
pub(crate) mod selection;
pub(crate) mod sweep;
pub(crate) mod taking;
use skein_lib::{Decimal, bytes};

/// The checkpoint copy chunk bound (protocol/store/checkpoints.md, 4.5).
pub use files::CHUNK;

/// The checkpoint record's file-list ceiling (protocol/store/checkpoints.md, 2.3).
pub const FILES: u32 = 65_536;
pub(crate) const DIRECTORY: &[u8] = b"checkpoints";

/// Retention of completed checkpoints (protocol/limits.md, Checkpoints kept).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Checkpoints {
    pub keep: u32,
}

/// A commit and the first half of its draw (protocol/store/checkpoints.md, 2.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CheckpointName {
    pub store: u64,
    pub suffix: u64,
}
impl CheckpointName {
    #[must_use]
    pub fn from_draw(store: u64, draw: [u8; 16]) -> (CheckpointName, u64) {
        let (first, last) = draw.split_at_checked(8).expect("sixteen draw bytes");
        let suffix = u64::from_be_bytes(first.try_into().expect("eight draw bytes"));
        let temporary = u64::from_be_bytes(last.try_into().expect("eight draw bytes"));
        (Self { store, suffix }, temporary)
    }

    #[must_use]
    pub fn bytes(self) -> [u8; 37] {
        let mut bytes = [b'0'; 37];
        let decimal = Decimal::of(self.store);
        let start = 20_usize.checked_sub(decimal.as_bytes().len()).expect("u64 has at most twenty digits");
        put(bytes.get_mut(start..20).expect("decimal field"), decimal.as_bytes());
        *bytes.get_mut(20).expect("separator") = b'-';
        put(bytes.get_mut(21..).expect("suffix field"), &hex(self.suffix));
        bytes
    }

    #[must_use]
    pub fn temporary(self, suffix: u64) -> [u8; 60] {
        let mut bytes = [0; 60];
        put(bytes.get_mut(..37).expect("name field"), &self.bytes());
        put(bytes.get_mut(37..44).expect("temporary marker"), b".skein-");
        put(bytes.get_mut(44..).expect("temporary suffix"), &hex(suffix));
        bytes
    }

    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 37 || bytes.get(20) != Some(&b'-') {
            return None;
        }
        let store = bytes::decimal(bytes.get(..20)?)?;
        Some(Self { store, suffix: unhex(bytes.get(21..)?)? })
    }

    #[must_use]
    pub fn parse_temporary(bytes: &[u8]) -> Option<(Self, u64)> {
        if bytes.len() != 60 || bytes.get(37..44)? != b".skein-" {
            return None;
        }
        Some((Self::parse(bytes.get(..37)?)?, unhex(bytes.get(44..)?)?))
    }
}
fn hex(number: u64) -> [u8; 16] {
    let mut bytes = [0; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let shift = 15_usize.checked_sub(index).expect("hex digit index").checked_mul(4).expect("hex shift");
        let digit = u8::try_from((number >> shift) & 15).expect("a nibble");
        *byte = bytes::hex_digit(digit);
    }
    bytes
}
fn unhex(bytes: &[u8]) -> Option<u64> {
    if bytes.len() != 16 {
        return None;
    }
    let mut number = 0_u64;
    for byte in bytes {
        if byte.is_ascii_uppercase() {
            return None;
        }
        let digit = bytes::hex_value(*byte)?;
        number = number.checked_mul(16)?.checked_add(u64::from(digit))?;
    }
    Some(number)
}

fn put(target: &mut [u8], bytes: &[u8]) {
    assert_eq!(target.len(), bytes.len(), "fixed name field");
    for (target, byte) in target.iter_mut().zip(bytes) {
        *target = *byte;
    }
}
