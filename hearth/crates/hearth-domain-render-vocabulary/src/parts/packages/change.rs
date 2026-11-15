//! Typed packages differences (hearth/docs/design/domain/render/README.md, 4.1).
use super::{Apt, Initramfs, Preseed, Sources, Variant};
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub resolution: crate::diff::ResolutionChange,
    pub request: Names,
    pub seeds: SeedsChange,
    pub sources: Option<SourcesPair>,
    pub variant: Option<VariantPair>,
    pub preseeds: Option<PreseedsPair>,
    pub initramfs: Option<InitramfsPair>,
    pub apt: Option<AptPair>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Names {
    pub added: Box<[Box<[u8]>]>,
    pub removed: Box<[Box<[u8]>]>,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourcesPair {
    pub before: Sources,
    pub after: Sources,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VariantPair {
    pub before: Variant,
    pub after: Variant,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PreseedsPair {
    pub before: Box<[Preseed]>,
    pub after: Box<[Preseed]>,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InitramfsPair {
    pub before: Option<Initramfs>,
    pub after: Option<Initramfs>,
}
/// Before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AptPair {
    pub before: Apt,
    pub after: Apt,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedsChange {
    pub users: Box<[SeedUserKeyed]>,
    pub groups: Box<[SeedGroupKeyed]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedIds {
    pub uid: u32,
    pub gid: Option<u32>,
}
/// Key-ordered seed changes (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SeedUserKeyed {
    Added { key: Box<[u8]>, value: SeedIds },
    Removed { key: Box<[u8]>, value: SeedIds },
    Changed { key: Box<[u8]>, before: SeedIds, after: SeedIds },
}
/// Key-ordered seed changes (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SeedGroupKeyed {
    Added { key: Box<[u8]>, value: u32 },
    Removed { key: Box<[u8]>, value: u32 },
    Changed { key: Box<[u8]>, before: u32, after: u32 },
}
