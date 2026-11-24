//! Largest values at the ceilings, or at the largest clamp under the cap.

use super::codec_memory::family;
use skein_oauth::v3::Record;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        skein_oauth::SCHEMA,
        skein_oauth::v3,
        roots [
            SavedToken,
        ]
    );
}
