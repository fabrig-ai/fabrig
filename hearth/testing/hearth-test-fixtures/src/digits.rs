//! Digits a test writes as text.
use skein_lib::bytes;

/// The bytes `text` spells in pairs of hex digits, of either case; spaces don't count.
#[must_use]
pub fn hex(text: &str) -> Box<[u8]> {
    let digits: Vec<_> =
        text.bytes().filter(|&byte| byte != b' ').map(|byte| bytes::hex_value(byte).expect("a hex digit")).collect();
    assert_eq!(digits.len() % 2, 0, "hex pairs are whole");
    digits.as_chunks::<2>().0.iter().map(|pair| pair[0] * 16 + pair[1]).collect()
}
