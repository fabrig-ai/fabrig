//! ASCII hexadecimal digits for bytes carried as text (lib.md, 6).

/// The lowercase ASCII digit of a value below sixteen; a larger one is a bug (`expect`).
#[must_use]
pub fn digit(nibble: u8) -> u8 {
    *b"0123456789abcdef".get(usize::from(nibble)).expect("a hexadecimal nibble is below sixteen")
}

/// Two lowercase digits, high nibble first.
#[must_use]
pub fn pair(byte: u8) -> [u8; 2] {
    [digit(byte >> 4), digit(byte & 0x0f)]
}

/// The value of an ASCII hex digit of either case, or `None`.
#[must_use]
pub fn value(byte: u8) -> Option<u8> {
    if (b'A'..=b'F').contains(&byte) { byte.checked_sub(b'A')?.checked_add(10) } else { lower(byte) }
}

/// The value of a lowercase ASCII hex digit, or `None` (uppercase included).
#[must_use]
pub fn lower(byte: u8) -> Option<u8> {
    if byte.is_ascii_digit() {
        byte.checked_sub(b'0')
    } else if (b'a'..=b'f').contains(&byte) {
        byte.checked_sub(b'a')?.checked_add(10)
    } else {
        None
    }
}
