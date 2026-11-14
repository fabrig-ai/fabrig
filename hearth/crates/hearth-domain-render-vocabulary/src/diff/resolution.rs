//! Recorded closures compared by name, then architecture (hearth/docs/design/domain/store/records.md, 4).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResolutionChange {
    Same,
    NewKey,
    Resolved { added: Box<[Listed]>, removed: Box<[Listed]>, versions_changed: Box<[Moved]> },
}
pub type Listed = crate::resolution::Inst;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Moved {
    pub name: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub before: Box<[u8]>,
    pub after: Box<[u8]>,
}
