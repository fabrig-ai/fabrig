//! Largest values at the ceilings, or at the largest clamp under the cap.

use super::codec_memory::family;
use jig_protocol_engine_clients::cursor_v2::Record as _;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_protocol_engine_clients::SCHEMA,
        jig_protocol_engine_clients::cursor_v2,
        roots [
            Held,
        ]
    );
}
