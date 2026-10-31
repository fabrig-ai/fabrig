//! ASCII hex digits (skein/docs/design/lib.md, 6).

/// The lowercase ASCII hex digit of `nibble`'s low four bits.
#[must_use]
pub const fn lower(nibble: u8) -> u8 {
    let nibble = nibble & 0x0f;
    if nibble < 10 { b'0'.saturating_add(nibble) } else { b'a'.saturating_add(nibble.saturating_sub(10)) }
}

/// The uppercase ASCII hex digit of `nibble`'s low four bits (percent-encoding).
#[must_use]
pub const fn upper(nibble: u8) -> u8 {
    let nibble = nibble & 0x0f;
    if nibble < 10 { b'0'.saturating_add(nibble) } else { b'A'.saturating_add(nibble.saturating_sub(10)) }
}

/// The value of an ASCII hex digit, either case.
#[must_use]
pub const fn value(byte: u8) -> Option<u8> {
    if byte >= b'A' && byte <= b'F' { Some(byte.saturating_sub(b'A').saturating_add(10)) } else { lower_value(byte) }
}

/// The value of a lowercase ASCII hex digit (git object names): uppercase is refused.
#[must_use]
pub const fn lower_value(byte: u8) -> Option<u8> {
    if byte >= b'0' && byte <= b'9' {
        Some(byte.saturating_sub(b'0'))
    } else if byte >= b'a' && byte <= b'f' {
        Some(byte.saturating_sub(b'a').saturating_add(10))
    } else {
        None
    }
}
