//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        include_str!("../../../../crates/jig-protocol-accounts/schema/moved-v1.schema"),
        jig_protocol_accounts::v1, sealed,
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
