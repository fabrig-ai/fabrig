//! Decimal parsing (skein/docs/design/lib.md, 6).

use crate::Decimal;

#[test]
fn parse_round_trips_formatted_counts() {
    for number in [0, 1, u64::MAX] {
        assert_eq!(Decimal::parse(Decimal::of(number).as_bytes()), Some(number));
    }
}

#[test]
fn parse_accepts_leading_zeros() {
    assert_eq!(Decimal::parse(b"00018446744073709551615"), Some(u64::MAX));
}

#[test]
fn parse_refuses_empty_input() {
    assert_eq!(Decimal::parse(b""), None);
}

#[test]
fn parse_refuses_a_sign() {
    assert_eq!(Decimal::parse(b"+1"), None);
    assert_eq!(Decimal::parse(b"-1"), None);
}

#[test]
fn parse_refuses_a_space() {
    assert_eq!(Decimal::parse(b" 1"), None);
    assert_eq!(Decimal::parse(b"1 "), None);
    assert_eq!(Decimal::parse(b"1 2"), None);
}

#[test]
fn parse_refuses_overflow() {
    assert_eq!(Decimal::parse(b"18446744073709551616"), None);
    assert_eq!(Decimal::parse(b"184467440737095516150"), None);
}

#[test]
fn parse_refuses_every_byte_other_than_an_ascii_digit() {
    for byte in u8::MIN..=u8::MAX {
        if !byte.is_ascii_digit() {
            assert_eq!(Decimal::parse(&[b'1', byte, b'2']), None);
        }
    }
}
