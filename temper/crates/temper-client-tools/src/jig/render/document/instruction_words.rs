//! Effective instructions, previewed by characters with the held byte count.
use super::Text;
use jig_client::contract::DocumentText;
pub(super) fn text(instructions: &DocumentText, out: &mut Text) {
    let bytes = instructions.bytes();
    let mut characters = 0_u8;
    let mut end = bytes.len();
    // Document text is valid UTF-8; every non-continuation byte starts one character.
    for (at, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' || (byte & 0xc0 != 0x80 && characters == 80) {
            end = at;
            break;
        }
        if byte & 0xc0 != 0x80 {
            characters = characters.checked_add(1).expect("at most 80 characters");
        }
    }
    let preview = end < bytes.len();
    out.put(b"\ninstructions: ");
    out.words(bytes.get(..end).expect("character boundary"));
    if preview || instructions.cut {
        if preview {
            out.put(b"\xe2\x80\xa6");
        }
        out.put(b" (");
        out.number(u64::try_from(bytes.len()).expect("document byte count fits"));
        out.put(if instructions.cut {
            b" bytes held, cut; --json prints what is held)"
        } else {
            b" bytes; --json prints them whole)"
        });
    }
}
