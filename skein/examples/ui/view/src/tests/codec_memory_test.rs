//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        include_str!("../../schema/notes-v1.schema"),
        crate::records, sealed,
        limits [
            note_title,
            note_body,
            note_due,
            add_title,
            add_body,
            add_due,
            notes_notes,
        ],
        roots [
            Add,
            Notes,
        ]
    );
}

