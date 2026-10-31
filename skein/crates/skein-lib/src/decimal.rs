//! Counts written as text (programming-model.md, section 8): the decimal digits
//! of a number, for a line a step writes into its own bytes without formatting.

/// The decimal digits of a `u64`, without leading zeros, `0` for zero: what a
/// step puts into a [`Writer`](crate::Writer) to write a count, after
/// measuring the text by their length.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Decimal {
    /// The digits, right-aligned, with zeros before the first.
    digits: [u8; 20],
    /// Where the first digit is.
    first: usize,
}

impl Decimal {
    /// The value of ASCII decimal digits: empty, a non-digit or overflow is `None`;
    /// no sign; leading zeros are accepted (HTTP's Content-Length, codec patches).
    #[must_use]
    pub const fn parse(digits: &[u8]) -> Option<u64> {
        if digits.is_empty() {
            return None;
        }
        let mut rest = digits;
        let mut value = 0_u64;
        // Bounded by the input's length: one digit is taken each time.
        while let Some((byte, tail)) = rest.split_first() {
            if !byte.is_ascii_digit() {
                return None;
            }
            let digit = digit_value(*byte);
            value = match value.checked_mul(10) {
                Some(value) => match value.checked_add(digit) {
                    Some(value) => value,
                    None => return None,
                },
                None => return None,
            };
            rest = tail;
        }
        Some(value)
    }

    /// `n` in decimal digits.
    #[must_use]
    pub fn of(n: u64) -> Decimal {
        let mut digits = [b'0'; 20];
        let mut rest = n;
        // The leftmost digit that is not a leading zero; the last, for zero.
        let mut first = 19;
        for (index, digit) in digits.iter_mut().enumerate().rev() {
            let value = u8::try_from(rest.checked_rem(10).unwrap_or(0)).expect("a digit fits in a byte");
            *digit = b'0'.saturating_add(value);
            if value != 0 {
                first = index;
            }
            rest = rest.checked_div(10).unwrap_or(0);
        }
        Decimal { digits, first }
    }

    /// The digits, most significant first.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.digits.get(self.first..).expect("the first digit is within the digits")
    }
}

// `From<u8>` is not const; the byte is widened without a cast.
const fn digit_value(byte: u8) -> u64 {
    let digit = byte.saturating_sub(b'0');
    u64::from_le_bytes([digit, 0, 0, 0, 0, 0, 0, 0])
}
