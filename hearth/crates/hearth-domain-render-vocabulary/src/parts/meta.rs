//! Meta's canonical record (hearth/docs/design/domain/render/spec.md, 1.4; hearth/docs/design/domain/store/README.md, 3.1).
use crate::revision::{Firmware, Runs};
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Meta {
    pub target: Box<[u8]>,
    pub base: Box<[u8]>,
    pub member: Option<Box<[u8]>>,
    pub environment: Box<[u8]>,
    pub placement: Placement,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    pub kind: Runs,
    pub firmware: Option<Firmware>,
    pub hardware: Option<Box<[u8]>>,
    pub on: Option<Box<[u8]>>,
}
/// Each changed meta field, before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub base: Option<NamePair>,
    pub member: Option<MemberPair>,
    pub environment: Option<NamePair>,
    pub placement: Option<PlacementPair>,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NamePair {
    pub before: Box<[u8]>,
    pub after: Box<[u8]>,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MemberPair {
    pub before: Option<Box<[u8]>>,
    pub after: Option<Box<[u8]>>,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PlacementPair {
    pub before: Placement,
    pub after: Placement,
}
