//! Restart request bytes follow the store writer bounds (protocol/store.md, 7.2).
use super::fixtures;

#[test]
fn restore_bytes_prices_two_rows_without_u32_truncation() {
    let mut limits = fixtures::store_limits();
    assert_eq!(
        crate::sizes::restore_bytes(&limits),
        Some(2 * (u64::from(limits.store.key) + u64::from(limits.store.value)))
    );
    limits.store.key = u32::MAX;
    limits.store.value = u32::MAX;
    assert_eq!(crate::sizes::restore_bytes(&limits), Some(17_179_869_180));
}
