//! Shared storage growth under a container's cap (lib.md, 3, 5 and 7).

pub(crate) const GROWTH_FLOOR: usize = 4;

/// The next storage size: double the allocated room, fit what is needed, and cap it.
pub(crate) fn growth(allocated: usize, needed: usize, capacity: u32) -> usize {
    let cap = usize::try_from(capacity).expect("a u32 fits in a usize");
    assert!(needed <= cap, "growth follows admission under the cap");
    allocated.saturating_mul(2).max(needed).max(GROWTH_FLOOR).min(cap)
}
