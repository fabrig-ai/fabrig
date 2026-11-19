//! Lean records and their boundary checks (data/codec/README.md, 4).

use alloc::boxed::Box;
use core::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use skein_lib::{Reader, Writer, usize_of};

use crate::Reason;

/// A declaration and one of its parts, in schema order.
#[expect(clippy::disallowed_types, reason = "PhantomData ties a path to its family's limits without storing them")]
#[expect(clippy::partial_pub_fields, reason = "positions are public; only the family's zero-sized marker is private")]
pub struct Path<L> {
    pub declaration: u16,
    pub part: u16,
    family: PhantomData<fn() -> L>,
}

/// A refusal at a field, version, tail or tag.
pub struct Problem<L> {
    pub path: Path<L>,
    pub reason: Reason,
}

impl<L> Path<L> {
    /// Names a declaration and one of its parts in this family.
    #[must_use]
    pub const fn new(declaration: u16, part: u16) -> Self {
        Self { declaration, part, family: PhantomData }
    }

    /// Adds the reason for a refusal at this path.
    #[must_use]
    pub const fn problem(self, reason: Reason) -> Problem<L> {
        Problem { path: self, reason }
    }
}

// Derives would require the family's Limits to implement these traits too.
impl<L> Clone for Path<L> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<L> Copy for Path<L> {}
impl<L> PartialEq for Path<L> {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration && self.part == other.part
    }
}
impl<L> Eq for Path<L> {}
impl<L> Hash for Path<L> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.declaration.hash(state);
        self.part.hash(state);
    }
}
impl<L> fmt::Debug for Path<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Path").field("declaration", &self.declaration).field("part", &self.part).finish()
    }
}

impl<L> Clone for Problem<L> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<L> Copy for Problem<L> {}
impl<L> PartialEq for Problem<L> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.reason == other.reason
    }
}
impl<L> Eq for Problem<L> {}
impl<L> Hash for Problem<L> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.reason.hash(state);
    }
}
impl<L> fmt::Debug for Problem<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Problem").field("path", &self.path).field("reason", &self.reason).finish()
    }
}

/// A generated declaration's measured, bounded codec.
/// Nested reads do not check a tail; only the outermost `decode` does.
pub trait Record: Sized {
    type Limits;
    const TAIL: Path<Self::Limits>;

    fn measure(&self, limits: &Self::Limits) -> Result<u32, Problem<Self::Limits>> {
        let mut out = Writer::measure(usize_of(u32::MAX));
        self.write(limits, &mut out)?;
        Ok(u32::try_from(out.written()).expect("measuring writer is bounded by u32"))
    }

    fn encode(&self, limits: &Self::Limits) -> Result<Box<[u8]>, Problem<Self::Limits>> {
        let length = self.measure(limits)?;
        let mut out = Writer::new(usize_of(length));
        self.write(limits, &mut out)?;
        Ok(out.finish())
    }

    fn decode(limits: &Self::Limits, reader: &mut Reader<'_>) -> Result<Self, Problem<Self::Limits>> {
        let value = Self::read(limits, reader)?;
        if !reader.is_empty() {
            return Err(Self::TAIL.problem(Reason::Trailing));
        }
        Ok(value)
    }

    fn encoded_bound(limits: &Self::Limits) -> Option<u64>;
    fn decoded_bound(limits: &Self::Limits) -> Option<u64>;

    /// Writes checked fields, also used with a measuring writer.
    fn write(&self, limits: &Self::Limits, out: &mut Writer) -> Result<(), Problem<Self::Limits>>;
    /// Reads a nested declaration, leaving the enclosing fields unread.
    fn read(limits: &Self::Limits, reader: &mut Reader<'_>) -> Result<Self, Problem<Self::Limits>>;
}
