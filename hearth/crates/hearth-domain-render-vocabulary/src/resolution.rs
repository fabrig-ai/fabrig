//! A resolution run's decoded install identity (hearth/docs/design/domain/render/spec.md, 2.5).
use alloc::boxed::Box;
/// A package's identity, ordered by name, architecture, then version (hearth/docs/design/domain/render/spec.md, 2.3).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Inst {
    pub name: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub version: Box<[u8]>,
}
