//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        skein_oauth::SCHEMA,
        skein_oauth::v3, sealed,
        limits [
            saved_token_access_token,
            saved_token_refresh_token_some,
            saved_token_metadata_some,
            saved_token_issued_client_id_some,
            saved_token_granted_scopes_some,
            saved_token_subject_some,
            saved_token_id_token_some,
        ],
        roots [
            SavedToken,
        ]
    );
}
