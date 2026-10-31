//! Hex digits and counts read from text (lib.md, 6; programming-model.md, 8).

use crate::{Decimal, hex};

#[test]
fn hex_digits_are_lowercase_ascii() {
    for (nibble, expected) in [(0, b'0'), (9, b'9'), (10, b'a'), (15, b'f')] {
        assert_eq!(hex::digit(nibble), expected);
    }
}

#[test]
#[should_panic(expected = "a nibble is at most 15")]
fn a_hex_nibble_above_fifteen_is_a_bug() {
    let _digit = hex::digit(16);
}

#[test]
fn hex_values_accept_decimal_digits() {
    assert_eq!(hex::value(b'0'), Some(0));
    assert_eq!(hex::value(b'9'), Some(9));
}

#[test]
fn hex_values_accept_both_letter_cases() {
    for (byte, expected) in [(b'a', 10), (b'f', 15), (b'A', 10), (b'F', 15)] {
        assert_eq!(hex::value(byte), Some(expected));
    }
}

#[test]
fn hex_values_refuse_other_bytes() {
    for byte in [0, b'/', b':', b'@', b'G', b'`', b'g', u8::MAX] {
        assert_eq!(hex::value(byte), None);
    }
}

#[test]
fn decimal_parses_zero() {
    assert_eq!(Decimal::parse(b"0"), Some(0));
}

#[test]
fn decimal_parses_the_largest_u64() {
    assert_eq!(Decimal::parse(b"18446744073709551615"), Some(u64::MAX));
}

#[test]
fn decimal_refuses_overflow() {
    assert_eq!(Decimal::parse(b"18446744073709551616"), None);
    assert_eq!(Decimal::parse(b"184467440737095516150"), None);
}

#[test]
fn decimal_refuses_empty_input() {
    assert_eq!(Decimal::parse(b""), None);
}

#[test]
fn decimal_refuses_signs() {
    assert_eq!(Decimal::parse(b"+1"), None);
    assert_eq!(Decimal::parse(b"-1"), None);
}

#[test]
fn decimal_refuses_spaces() {
    for digits in [b" 1", b"1 "] {
        assert_eq!(Decimal::parse(digits), None);
    }
}

#[test]
fn decimal_accepts_leading_zeros() {
    assert_eq!(Decimal::parse(b"00010"), Some(10));
}

#[test]
fn decimal_refuses_non_ascii_bytes() {
    assert_eq!(Decimal::parse(&[b'1', 0xff]), None);
}
