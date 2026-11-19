//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        include_str!("../../../../crates/jig-protocol-engine-clients/schema/cursor-v2.schema"),
        jig_protocol_engine_clients::cursor_v2, sealed,
        limits [
            blob_value,
            owner_key_value,
            pattern_segments,
            pattern_segments_item,
            scoped_notes_description_some,
            query_parts,
        ],
        roots [
            Held,
        ]
    );
}

