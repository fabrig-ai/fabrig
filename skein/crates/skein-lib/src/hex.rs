//! ASCII hex digits and their values (skein/docs/design/lib.md, 6).

/// A nibble's lowercase ASCII hex digit. A nibble above 15 is a bug.
#[must_use]
pub fn digit(nibble: u8) -> u8 {
    assert!(nibble <= 15, "a nibble is at most 15");
    if nibble < 10 {
        b'0'.checked_add(nibble).expect("one decimal digit")
    } else {
        b'a'.checked_add(nibble.saturating_sub(10)).expect("one hex digit")
    }
}

/// An ASCII hex digit's value, either case, or `None` for another byte.
#[must_use]
pub fn value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10),
        b'A'..=b'F' => byte.checked_sub(b'A')?.checked_add(10),
        0..=b'/' | b':'..=b'@' | b'G'..=b'`' | b'g'..=u8::MAX => None,
    }
}
