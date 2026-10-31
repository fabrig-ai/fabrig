//! Hexadecimal text carried as bytes (lib.md, 6).

use crate::hex;

#[test]
fn every_nibble_round_trips_through_its_lowercase_digit() {
    for (nibble, &digit) in (0_u8..16).zip(b"0123456789abcdef") {
        assert_eq!(hex::digit(nibble), digit);
        assert_eq!(hex::value(digit), Some(nibble));
        assert_eq!(hex::lower(digit), Some(nibble));
    }
}

#[test]
fn uppercase_hex_is_accepted_only_by_the_case_insensitive_parser() {
    for (nibble, &digit) in (10_u8..16).zip(b"ABCDEF") {
        assert_eq!(hex::value(digit), Some(nibble));
        assert_eq!(hex::lower(digit), None);
    }
}

#[test]
fn bytes_outside_the_hex_alphabets_are_refused() {
    for byte in [0, b'/', b':', b'@', b'G', b'`', b'g', u8::MAX] {
        assert_eq!(hex::value(byte), None, "byte {byte}");
        assert_eq!(hex::lower(byte), None, "byte {byte}");
    }
}

#[test]
fn a_pair_writes_the_high_nibble_first() {
    for (byte, expected) in [(0xa5, *b"a5"), (0, *b"00"), (u8::MAX, *b"ff")] {
        assert_eq!(hex::pair(byte), expected);
    }
}

#[test]
#[should_panic(expected = "a hexadecimal nibble is below sixteen")]
fn a_value_past_fifteen_is_not_a_nibble() {
    let _digit = hex::digit(16);
}
