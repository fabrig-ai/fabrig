//! How lib's growing containers grow (lib.md, 5 and 10): a queue, a list, a
//! stack, a slab and an intake allocate as they fill, doubling from a small
//! floor, and never past their cap.

/// The least a growing buffer holds once it allocates, so that its first
/// few items do not each reallocate.
const GROWTH_FLOOR: usize = 4;

/// What a buffer's storage grows to when it holds `len` and must hold
/// `needed`: twice `len`, at least `needed` and the floor, never past `cap`.
///
/// A queue, a list, a stack and a slab's lists grow by an item when full,
/// so `len` is their length; an intake grows by what arrives once that
/// outgrows its storage, so `len` is the storage's capacity.
pub(crate) fn grown(len: usize, needed: usize, cap: usize) -> usize {
    len.saturating_mul(2).max(needed).max(GROWTH_FLOOR).min(cap)
}
