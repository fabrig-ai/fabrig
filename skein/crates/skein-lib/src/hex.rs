//! Hexadecimal digits (lib.md, 6.3): a nibble or a byte written as digits,
//! and a digit read back, for every crate that spells bytes in hex.

/// The case of the letters `a` to `f` a writer uses.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Case {
    /// `0`–`9`, `a`–`f`: digests, object ids, UUIDs.
    Lower,
    /// `0`–`9`, `A`–`F`: percent-encoding (RFC 3986, 2.1).
    Upper,
}

/// The digit of `nibble`, in `case`. A nibble past 15 is the caller's bug,
/// asserted.
#[must_use]
pub fn digit(nibble: u8, case: Case) -> u8 {
    let digits = match case {
        Case::Lower => b"0123456789abcdef",
        Case::Upper => b"0123456789ABCDEF",
    };
    *digits.get(usize::from(nibble)).expect("a nibble is below sixteen")
}

/// The two digits of `byte`, the high nibble's first.
#[must_use]
pub fn pair(byte: u8, case: Case) -> [u8; 2] {
    [digit(byte >> 4_u8, case), digit(byte & 0xF, case)]
}

/// The value of a digit of either case, or `None` for any other byte.
#[must_use]
pub const fn read(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(digit.wrapping_sub(b'a').wrapping_add(10)),
        b'A'..=b'F' => Some(digit.wrapping_sub(b'A').wrapping_add(10)),
        0..b'0' | b':'..b'A' | b'G'..b'a' | b'g'..=u8::MAX => None,
    }
}

/// The value of a digit in lowercase, or `None` for any other byte, an
/// uppercase letter among them: for a spelling that has one form, such as
/// an object id.
#[must_use]
pub const fn read_lower(digit: u8) -> Option<u8> {
    match digit {
        b'A'..=b'F' => None,
        0..b'A' | b'G'..=u8::MAX => read(digit),
    }
}
