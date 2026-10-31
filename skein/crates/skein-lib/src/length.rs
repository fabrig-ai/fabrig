//! Lengths used in byte accounting (skein/docs/design/lib.md, 6).

/// A native length as a `u64`; supported targets have at most 64-bit addresses.
#[must_use]
pub fn len_u64(len: usize) -> u64 {
    u64::try_from(len).expect("a native length fits a u64")
}

#[cfg(test)]
mod tests {
    use super::len_u64;

    #[test]
    fn lengths_include_zero_and_the_native_maximum() {
        assert_eq!(len_u64(0), 0);
        assert_eq!(len_u64(17), 17);
        assert_eq!(len_u64(usize::MAX), u64::try_from(usize::MAX).expect("native maximum fits"));
    }
}
