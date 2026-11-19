//! Lean records and their boundary checks (data/codec/README.md, 4).

use alloc::boxed::Box;
use skein_lib::{Reader, Writer, usize_of};

use crate::Reason;

/// A declaration and one of its parts, in schema order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Path {
    pub declaration: u16,
    pub part: u16,
}

/// A refusal at a field, version, tail or tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Problem {
    pub path: Path,
    pub reason: Reason,
}

impl Path {
    /// Adds the reason for a refusal at this path.
    #[must_use]
    pub const fn problem(self, reason: Reason) -> Problem {
        Problem { path: self, reason }
    }
}

/// A generated declaration's measured, bounded codec.
/// Nested reads do not check a tail; only the outermost `decode` does.
pub trait Record: Sized {
    type Limits;
    const TAIL: Path;

    fn measure(&self, limits: &Self::Limits) -> Result<u32, Problem> {
        let mut out = Writer::measure(usize_of(u32::MAX));
        self.write(limits, &mut out)?;
        Ok(u32::try_from(out.written()).expect("measuring writer is bounded by u32"))
    }

    fn encode(&self, limits: &Self::Limits) -> Result<Box<[u8]>, Problem> {
        let length = self.measure(limits)?;
        let mut out = Writer::new(usize_of(length));
        self.write(limits, &mut out)?;
        Ok(out.finish())
    }

    fn decode(limits: &Self::Limits, reader: &mut Reader<'_>) -> Result<Self, Problem> {
        let result = Self::read(limits, reader);
        if result.is_ok() && !reader.is_empty() {
            return Err(Self::TAIL.problem(Reason::Trailing));
        }
        result
    }

    fn encoded_bound(limits: &Self::Limits) -> Option<u64>;
    fn decoded_bound(limits: &Self::Limits) -> Option<u64>;

    /// Writes checked fields, also used with a measuring writer.
    fn write(&self, limits: &Self::Limits, out: &mut Writer) -> Result<(), Problem>;
    /// Reads a nested declaration, leaving the enclosing fields unread.
    fn read(limits: &Self::Limits, reader: &mut Reader<'_>) -> Result<Self, Problem>;
}
