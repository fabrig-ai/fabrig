//! The directory's sign-in note (temper/docs/design/client/tui/README.md, 2).
use alloc::boxed::Box;
use skein_lib::{
    Decimal, Wall,
    bytes::{concat, decimal},
};

/// DIRECTORY/sign-in for one address (temper/docs/design/client/tui/README.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Note {
    Until(Wall),
    SignedOut,
}
impl Note {
    pub const MOST: u32 = 64;

    /// `HOST:PORT until NANOS\n` or `HOST:PORT signed-out\n`.
    #[must_use]
    pub fn print(self, address: &[u8]) -> Box<[u8]> {
        match self {
            Note::Until(wall) => concat(&[address, b" until ", Decimal::of(wall.as_nanos()).as_bytes(), b"\n"]),
            Note::SignedOut => concat(&[address, b" signed-out\n"]),
        }
    }

    /// The address's note, or none for an unreadable or oversized line.
    #[must_use]
    pub fn read(bytes: &[u8], address: &[u8]) -> Option<Note> {
        if bytes.len() > skein_lib::usize_of(Self::MOST) {
            return None;
        }
        let line = bytes.strip_suffix(b"\n").unwrap_or(bytes);
        let rest = line.strip_prefix(address)?.strip_prefix(b" ")?;
        if rest == b"signed-out" {
            return Some(Note::SignedOut);
        }
        let nanos = rest.strip_prefix(b"until ")?;
        if nanos.len() > 1 && nanos.first() == Some(&b'0') {
            return None;
        }
        Some(Note::Until(Wall::from_nanos(decimal(nanos)?)))
    }
}
