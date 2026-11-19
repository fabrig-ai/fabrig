//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;
use jig_test_store::v2::Record as _;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_test_store::SCHEMA,
        jig_test_store::v2, lean,
        limits [
            segment_bytes,
            resource_path_segments,
            effect_resources,
            record_procedure_delegate_outcome_revisions,
            record_task_resources,
            record_made_resources,
            setting_set_value,
            record_setting_default_value,
            record_setting_change_value_some,
            record_setting_replaced_value,
        ],
        roots [
            Row,
        ]
    );
}
