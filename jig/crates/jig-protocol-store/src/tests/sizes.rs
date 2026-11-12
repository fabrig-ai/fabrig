//! Restart request bytes follow the store writer bounds (protocol/store/loads.md, 2).
use super::fixtures;

#[test]
fn settings_value_includes_the_generated_model_names_and_replaced_values() {
    let domain = fixtures::CORE_LIMITS;
    let codec = crate::codec::settings::limits(&domain).unwrap();
    let names = crate::settings_v3::ModelName::encoded_bound(&codec).unwrap() * u64::from(jig_domain_tasks::MODELS) + 4;
    assert_eq!(
        u64::from(crate::sizes::settings_value(&domain).unwrap()),
        jig_domain_core::settings::revision_bound(&domain).unwrap() + 16 + names
    );
}

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

#[test]
fn the_task_row_grows_with_the_batch_count_only_by_its_numbers() {
    let mut domain = fixtures::TASKS_LIMITS;
    domain.batch = 4;
    let small = crate::codec::tasks::limits(&domain).expect("small count");
    domain.batch = 160;
    let large = crate::codec::tasks::limits(&domain).expect("large count");
    let small = crate::tasks_v11::TaskRecord::encoded_bound(&small).expect("small row");
    let large = crate::tasks_v11::TaskRecord::encoded_bound(&large).expect("large row");
    // Last batch and result followups each retain 156 additional u64 numbers.
    assert_eq!(large - small, 2 * 156 * 8);
}
