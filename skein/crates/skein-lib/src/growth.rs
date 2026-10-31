//! Capacity growth for lib's buffers (lib.md, 3, 5 and 7).

const GROWTH_FLOOR: usize = 4;

/// Doubles `len`, meeting `needed` and the floor, bounded by `cap`.
pub(crate) fn grown(len: usize, needed: usize, cap: usize) -> usize {
    len.saturating_mul(2).max(needed).max(GROWTH_FLOOR).min(cap)
}
