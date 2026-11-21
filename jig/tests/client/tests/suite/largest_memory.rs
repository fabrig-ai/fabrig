//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;
use jig_client_view_codec::records_v1::Record as _;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_client_view::SCHEMA,
        jig_client_view_codec::records_v1,
        limits [
            draft_words,
            stored_record_record,
        ],
        roots [
            ViewRecord,
            StoredRecord,
        ]
    );
}
