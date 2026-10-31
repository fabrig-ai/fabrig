//! Widening lengths and counts (skein/docs/design/lib.md, 6).

use crate::{len_u64, usize_of};

#[test]
fn lengths_include_zero_and_the_native_maximum() {
    assert_eq!(len_u64(0), 0);
    assert_eq!(len_u64(17), 17);
    assert_eq!(len_u64(usize::MAX), u64::try_from(usize::MAX).expect("native maximum fits"));
}

#[test]
fn counts_widen_through_the_u32_maximum() {
    assert_eq!(usize_of(0), 0);
    assert_eq!(usize_of(17), 17);
    assert_eq!(len_u64(usize_of(u32::MAX)), u64::from(u32::MAX));
}
