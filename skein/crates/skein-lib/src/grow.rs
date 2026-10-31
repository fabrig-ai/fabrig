//! Container growth (skein/docs/design/lib.md, 3 and 5): double under the cap.

const GROWTH_FLOOR: usize = 4;

/// The next allocation, at least `needed`, doubling `allocated` under the cap.
pub(crate) fn target(allocated: usize, needed: usize, capacity: u32) -> usize {
    let cap = usize::try_from(capacity).expect("a u32 fits in a usize");
    allocated.saturating_mul(2).max(needed).max(GROWTH_FLOOR).min(cap)
}
