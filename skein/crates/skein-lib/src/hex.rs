//! ASCII hexadecimal digits (programming-model.md, 10.2).

/// The lowercase hex digit of `nibble`'s low four bits.
#[must_use]
pub const fn digit(nibble: u8) -> u8 {
    let nibble = nibble & 0x0f;
    if nibble < 10 { b'0'.wrapping_add(nibble) } else { b'a'.wrapping_add(nibble).wrapping_sub(10) }
}

/// The value of an ASCII hex digit of either case, or `None`.
#[must_use]
pub const fn value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'F' => Some(byte.wrapping_sub(b'A').wrapping_add(10)),
        _ => lower_value(byte),
    }
}

/// The value of a lowercase ASCII hex digit, or `None` (uppercase too).
#[must_use]
pub const fn lower_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(byte.wrapping_sub(b'a').wrapping_add(10)),
        _ => None,
    }
}
