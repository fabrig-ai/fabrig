//! The one growth rule of lib's containers (lib.md, 5): a buffer allocates
//! as it fills, doubling, never past its container's cap.

/// The least a growing buffer allocates, so that the first few pushes do
/// not each allocate.
const FLOOR: usize = 4;

/// How many more items a buffer of `len` items, with room allocated for
/// `allocated`, reserves exactly so that it holds `needed`: none while it has
/// room; otherwise up to twice its allocation, at least `FLOOR` and at least
/// `needed`, and never past `cap`. Its caller has checked that `needed` is
/// within `cap`.
pub(crate) fn growth(len: usize, allocated: usize, needed: usize, cap: u32) -> usize {
    if needed <= allocated {
        return 0;
    }
    let cap = usize::try_from(cap).expect("a u32 fits in a usize");
    let target = allocated.saturating_mul(2).max(FLOOR).max(needed).min(cap);
    assert!(needed <= target, "a container grows only within its cap");
    target.checked_sub(len).expect("a buffer holds no more than it has allocated")
}
