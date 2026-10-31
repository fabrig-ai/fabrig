//! Hexadecimal digits (skein/docs/foundation/programming-model.md, 8).

use crate::hex;

#[test]
fn digit_encodes_the_low_four_bits_in_lowercase() {
    const ZERO: u8 = hex::digit(0);
    assert_eq!(ZERO, b'0');
    for byte in u8::MIN..=u8::MAX {
        assert_eq!(hex::digit(byte), b"0123456789abcdef"[usize::from(byte & 0x0f)]);
    }
}

#[test]
fn value_decodes_exactly_the_ascii_hex_digits() {
    const UPPER_A: Option<u8> = hex::value(b'A');
    assert_eq!(UPPER_A, Some(10));
    for byte in u8::MIN..=u8::MAX {
        let mut expected = None;
        for nibble in 0..16 {
            let digit = hex::digit(nibble);
            if byte == digit || byte == digit.to_ascii_uppercase() {
                expected = Some(nibble);
            }
        }
        assert_eq!(hex::value(byte), expected, "byte {byte}");
    }
}
