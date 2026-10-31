//! Buffer growth under a fixed cap (skein/docs/design/lib.md, 5).

const GROWTH_FLOOR: usize = 4;

/// Additional storage to reserve for at least `needed` items, under `cap`.
pub(crate) fn growth(held: usize, len: usize, needed: usize, cap: u32) -> usize {
    let cap = usize::try_from(cap).expect("a u32 fits in a usize");
    let target = held.saturating_mul(2).max(needed).max(GROWTH_FLOOR).min(cap);
    target.checked_sub(len).expect("growth keeps every held item")
}
