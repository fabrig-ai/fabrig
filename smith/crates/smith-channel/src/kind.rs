//! Version-six kind identities (smith/docs/design/protocol/channel.md, sections 3–4).

use alloc::boxed::Box;

/// The `Start` body kind.
pub const START: u16 = 0x0100;

/// The `Message` body kind.
pub const MESSAGE: u16 = 0x0101;

/// The `HostAnswer` body kind.
pub const HOST_ANSWER: u16 = 0x0102;

/// The `Acknowledge` body kind.
pub const ACKNOWLEDGE: u16 = 0x0103;

/// The `GrantRefresh` body kind.
pub const GRANT_REFRESH: u16 = 0x0104;

/// The `Cancel` body kind.
pub const CANCEL: u16 = 0x0105;

/// The `Admitted` body kind.
pub const ADMITTED: u16 = 0x0106;

/// The `Call` body kind.
pub const CALL: u16 = 0x0107;

/// The `Withdraw` body kind.
pub const WITHDRAW: u16 = 0x0108;

/// The `Turn` body kind.
pub const TURN: u16 = 0x0109;

/// The `Waiting` body kind.
pub const WAITING: u16 = 0x010a;

/// The `Long` body kind.
pub const LONG: u16 = 0x010b;

/// The `LongDone` body kind.
pub const LONG_DONE: u16 = 0x010c;

/// The `Unauthorized` body kind.
pub const UNAUTHORIZED: u16 = 0x010d;

/// The `Limited` body kind.
pub const LIMITED: u16 = 0x010e;

/// The `Fact` body kind.
pub const FACT: u16 = 0x010f;

/// The `Answer` body kind.
pub const ANSWER: u16 = 0x0110;

/// The `MessageRefused` body kind.
pub const MESSAGE_REFUSED: u16 = 0x0111;

/// The `Live` body kind.
pub const LIVE: u16 = 0x0112;

/// The `AccountRefused` body kind.
pub const ACCOUNT_REFUSED: u16 = 0x0113;

/// The `Drain` body kind.
pub const DRAIN: u16 = 0x0114;

/// The supported channel version.
pub const VERSION: u16 = crate::wire::VERSION;

/// A kind's name in a refusal's text: `kind 0x` and its four lower-case hex digits.
#[must_use]
pub fn text(kind: u16) -> Box<[u8]> {
    let bytes = kind.to_be_bytes();
    Box::from([
        b'k',
        b'i',
        b'n',
        b'd',
        b' ',
        b'0',
        b'x',
        skein_lib::bytes::hex_digit(bytes[0] >> 4),
        skein_lib::bytes::hex_digit(bytes[0] & 15),
        skein_lib::bytes::hex_digit(bytes[1] >> 4),
        skein_lib::bytes::hex_digit(bytes[1] & 15),
    ])
}

#[cfg(test)]
mod tests {
    use super::text;

    #[test]
    fn text_has_four_lower_case_hex_digits() {
        for (kind, expected) in [(0x0100, b"kind 0x0100"), (0xabcd, b"kind 0xabcd")] {
            assert_eq!(text(kind).as_ref(), expected);
        }
    }
}
