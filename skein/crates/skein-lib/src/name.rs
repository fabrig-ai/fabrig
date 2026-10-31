//! Inline names checked at their entrance (skein/docs/design/lib.md, 4).

/// The spelling a name's maker admits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Grammar {
    /// ASCII lowercase letters, digits and hyphens.
    LowercaseDigitsHyphen,
    /// ASCII bytes from `!` through `~`, without spaces or controls.
    PrintableAscii,
}

impl Grammar {
    fn accepts(self, bytes: &[u8]) -> bool {
        for byte in bytes {
            let accepted = match self {
                Self::LowercaseDigitsHyphen => byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-',
                Self::PrintableAscii => (b'!'..=b'~').contains(byte),
            };
            if !accepted {
                return false;
            }
        }
        true
    }
}

/// A name of at most `N` bytes, checked once and copied freely after.
/// `N` must be at most 255; the constructors assert this at compile time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Name<const N: usize> {
    length: u8,
    bytes: [u8; N],
}

impl<const N: usize> Name<N> {
    /// Refuses empty names, names longer than `N`, and names outside the grammar.
    #[must_use]
    pub fn new(bytes: &[u8], grammar: Grammar) -> Option<Name<N>> {
        let name = Self::from_checked(bytes)?;
        if bytes.is_empty() || !grammar.accepts(bytes) {
            return None;
        }
        Some(name)
    }

    /// Reconstructs a stored name without judging its spelling, checking only
    /// that its length fits. The maker has already checked its grammar.
    #[must_use]
    pub fn from_checked(bytes: &[u8]) -> Option<Name<N>> {
        const { assert!(N <= 255, "a name's capacity fits its u8 length") };
        if bytes.len() > N {
            return None;
        }
        let length = u8::try_from(bytes.len()).ok()?;
        let mut name = Self { length, bytes: [0; N] };
        for (index, byte) in bytes.iter().enumerate() {
            *name.bytes.get_mut(index)? = *byte;
        }
        Some(name)
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.get(..usize::from(self.length)).expect("name fits its inline capacity")
    }
}

#[cfg(test)]
#[path = "tests/name.rs"]
mod tests;
