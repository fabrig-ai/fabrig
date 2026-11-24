//! Largest values at the ceilings, or at the largest clamp under the cap.

use super::codec_memory::family;
use crate::records::Record;

const CAP: u64 = 64 * 1024 * 1024;

#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary Rust codec memory harness embeds its schema and reports family/root diagnostics"
)]
#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        skein_ui_example_view_codec::SCHEMA,
        crate::records,
        roots [
            Add,
            Notes,
        ]
    );
}
