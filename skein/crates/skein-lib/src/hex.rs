//! Hexadecimal digits for protocol bytes (skein/docs/foundation/programming-model.md, 8).

/// The lowercase ASCII hexadecimal digit of the low four bits of `nibble`.
#[must_use]
pub const fn digit(nibble: u8) -> u8 {
    let value = nibble & 0x0f;
    if value < 10 { b'0'.saturating_add(value) } else { b'a'.saturating_add(value.saturating_sub(10)) }
}

/// The value of an ASCII hexadecimal digit in either case, or `None` for another byte.
#[must_use]
pub const fn value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte.saturating_sub(b'0')),
        b'a'..=b'f' => Some(byte.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Some(byte.saturating_sub(b'A').saturating_add(10)),
        0..=b'/' | b':'..=b'@' | b'G'..=b'`' | b'g'..=u8::MAX => None,
    }
}
