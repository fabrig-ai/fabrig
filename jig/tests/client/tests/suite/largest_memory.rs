//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        include_str!("../../../../crates/jig-client-view/schema/records-v1.schema"),
        jig_client_view_codec::records_v1, sealed,
        limits [
            draft_words,
            record_record,
        ],
        roots [
            JigRecord,
            Record,
        ]
    );
}
