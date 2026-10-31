//! Shared buffer growth (lib.md, 3, 5 and 7): floor, cap and batch admission.

use crate::growth::grown;

#[test]
fn growth_starts_at_the_floor() {
    assert_eq!(grown(0, 1, 100), 4);
    assert_eq!(grown(1, 2, 100), 4);
    assert_eq!(grown(4, 5, 100), 8);
}

#[test]
fn growth_stops_at_the_cap() {
    assert_eq!(grown(0, 0, 0), 0);
    assert_eq!(grown(0, 1, 3), 3);
    assert_eq!(grown(8, 9, 10), 10);
    assert_eq!(grown(usize::MAX, usize::MAX, usize::MAX), usize::MAX);
}

#[test]
fn growth_meets_a_need_above_double() {
    assert_eq!(grown(4, 11, 100), 11);
    assert_eq!(grown(4, 11, 10), 10);
}
