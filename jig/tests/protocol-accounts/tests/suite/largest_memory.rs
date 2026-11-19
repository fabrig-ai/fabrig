//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;
use jig_protocol_accounts::v1::Record as _;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_protocol_accounts::SCHEMA,
        jig_protocol_accounts::v1, lean,
        limits [
            moved_application,
            moved_issuer,
            moved_client_id,
            moved_subject,
            moved_display,
            moved_record,
        ],
        roots [
            Moved,
        ]
    );
}
