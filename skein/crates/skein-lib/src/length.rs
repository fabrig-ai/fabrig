//! Lengths used in byte accounting (skein/docs/design/lib.md, 6).

/// A native length as a `u64`; supported targets have at most 64-bit addresses.
#[must_use]
pub fn len_u64(len: usize) -> u64 {
    u64::try_from(len).expect("a native length fits a u64")
}

/// A `u32` count as an index; supported targets have at least 32-bit addresses.
#[must_use]
pub fn usize_of(n: u32) -> usize {
    usize::try_from(n).expect("u32 fits usize")
}
