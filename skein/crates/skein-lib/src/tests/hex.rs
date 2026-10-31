//! Hex digits and values (programming-model.md, 10.2).

use crate::hex::{digit, lower_value, value};

#[test]
fn hex_digits_are_lowercase_and_use_only_the_low_nibble() {
    const HIGH_BITS: u8 = digit(0xfe);
    for (nibble, &expected) in (0_u8..=15).zip(b"0123456789abcdef") {
        assert_eq!(digit(nibble), expected);
    }
    assert_eq!(HIGH_BITS, b'e');
}

#[test]
fn hex_values_accept_exactly_their_ascii_digit_classes() {
    const UPPER: Option<u8> = value(b'F');
    const LOWER: Option<u8> = lower_value(b'f');
    assert_eq!(UPPER, Some(15));
    assert_eq!(LOWER, Some(15));
    for byte in 0_u8..=u8::MAX {
        let lower = match byte {
            b'0'..=b'9' => Some(byte.checked_sub(b'0').unwrap()),
            b'a'..=b'f' => Some(byte.checked_sub(b'a').unwrap().checked_add(10).unwrap()),
            _ => None,
        };
        let either = match byte {
            b'A'..=b'F' => Some(byte.checked_sub(b'A').unwrap().checked_add(10).unwrap()),
            _ => lower,
        };
        assert_eq!(value(byte), either, "byte {byte}");
        assert_eq!(lower_value(byte), lower, "byte {byte}");
    }
}
