//! Boot parameters and mutability (hearth/docs/design/domain/render/spec.md, 1.3).
use alloc::boxed::Box;
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub before: Boot,
    pub after: Boot,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Boot {
    pub params: Box<[Box<[u8]>]>,
    pub mutable: bool,
}
