//! Fixed-size caller identity rendering; no clocks, randomness or allocations.
//! Contract: llm.md, sections 2.1 and 4.8.

use skein_lib::bytes;

pub(crate) fn uuid(key: &[u8; 16]) -> [u8; 36] {
    let mut out = [b'-'; 36];
    let mut at = 0_usize;
    for (index, byte) in key.iter().enumerate() {
        if index == 4 || index == 6 || index == 8 || index == 10 {
            at = at.checked_add(1).expect("fixed UUID separator");
        }
        *out.get_mut(at).expect("fixed UUID high nibble") = bytes::hex_digit(byte >> 4);
        at = at.checked_add(1).expect("fixed UUID width");
        *out.get_mut(at).expect("fixed UUID low nibble") = bytes::hex_digit(byte & 15);
        at = at.checked_add(1).expect("fixed UUID width");
    }
    out
}
