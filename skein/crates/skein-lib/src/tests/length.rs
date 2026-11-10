//! Widening lengths and counts (skein/docs/design/runtime/lib.md, 6).

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

#[test]
fn measures_check_native_sums_and_u32_totals() {
    let max_u32 = usize_of(u32::MAX);
    for (parts, total, total_u32) in [
        (&[][..], Some(0), Some(0)),
        (&[1, 2][..], Some(3), Some(3)),
        (&[max_u32, 1][..], max_u32.checked_add(1), None),
        (&[usize::MAX, 1][..], None, None),
        (&[usize::MAX, 1, 2][..], None, None),
    ] {
        let mut measure = crate::length::Measure::ZERO;
        for &part in parts {
            measure.add(part);
        }
        assert_eq!(measure.total(), total, "parts: {parts:?}");
        assert_eq!(measure.total_u32(), total_u32, "parts: {parts:?}");
    }
}
