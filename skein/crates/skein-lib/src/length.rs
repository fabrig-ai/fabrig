//! Lengths used in byte accounting (skein/docs/design/lib.md, 6).

/// A native length as a `u64`; supported targets have at most 64-bit addresses.
#[must_use]
pub fn len_u64(len: usize) -> u64 {
    u64::try_from(len).expect("a native length fits a u64")
}

/// A `u32` count as an index; supported targets have at least 32-bit addresses.
#[must_use]
pub fn usize_of(n: u32) -> usize {
    usize::try_from(n).expect("u32 fits usize")
}

/// A length added up with checks (skein/docs/design/lib.md, 6): past a `usize` it stays `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measure(Option<usize>);

impl Measure {
    pub const ZERO: Measure = Measure(Some(0));

    pub fn add(&mut self, len: usize) {
        let Some(sum) = self.0 else { return };
        self.0 = sum.checked_add(len);
    }

    #[must_use]
    pub const fn total(self) -> Option<usize> {
        self.0
    }

    #[must_use]
    pub fn total_u32(self) -> Option<u32> {
        u32::try_from(self.total()?).ok()
    }
}
