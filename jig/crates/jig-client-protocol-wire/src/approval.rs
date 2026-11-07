//! An approval's code (jig/docs/design/protocol/client.md, 8).

/// Twenty consonants, with no vowel or letter like a digit.
pub const CODE_ALPHABET: [u8; 20] = *b"BCDFGHJKLMNPQRSTVWXZ";
/// How many letters a code has.
pub const CODE_LETTERS: usize = 8;
/// The most bytes `ApprovalCode::parse` reads.
pub const CODE_INPUT_BYTES: usize = 32;

/// An approval's code: eight letters of the alphabet, upper case.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ApprovalCode([u8; 8]);

impl ApprovalCode {
    /// Read exactly eight alphabet letters, ignoring hyphens, spaces and ASCII case.
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > CODE_INPUT_BYTES {
            return None;
        }
        let mut letters = [0; CODE_LETTERS];
        let mut remaining = letters.iter_mut();
        for byte in bytes {
            if *byte == b'-' || *byte == b' ' {
                continue;
            }
            let letter = byte.to_ascii_uppercase();
            if !CODE_ALPHABET.contains(&letter) {
                return None;
            }
            *remaining.next()? = letter;
        }
        if remaining.next().is_some() {
            return None;
        }
        Some(Self(letters))
    }

    /// Read eight alphabet indexes, refusing any index past 19.
    #[must_use]
    pub fn from_indexes(indexes: [u8; 8]) -> Option<Self> {
        let mut letters = [0; CODE_LETTERS];
        for (letter, index) in letters.iter_mut().zip(indexes) {
            *letter = *CODE_ALPHABET.get(usize::from(index))?;
        }
        Some(Self(letters))
    }

    /// The eight letters, as the wire's `code` carries them.
    #[must_use]
    pub const fn letters(&self) -> &[u8; 8] {
        &self.0
    }

    /// Four letters, a hyphen and four letters, as shown and in an address.
    #[must_use]
    pub fn shown(&self) -> [u8; 9] {
        let [first, second, third, fourth, fifth, sixth, seventh, eighth] = self.0;
        [first, second, third, fourth, b'-', fifth, sixth, seventh, eighth]
    }
}
