//! Typed packages differences (hearth/docs/design/domain/render/README.md, 4.1).
use super::{Apt, Box, Initramfs, List, Packages, Preseed, Sources, Variant};
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
pub type SourcesPair = crate::diff::Changed<Sources>;
pub type VariantPair = crate::diff::Changed<Variant>;
pub type PreseedsPair = crate::diff::Changed<Box<[Preseed]>>;
pub type InitramfsPair = crate::diff::Changed<Option<Initramfs>>;
pub type AptPair = crate::diff::Changed<Apt>;

#[must_use]
pub(crate) fn change(before: &Packages, after: &Packages) -> Change {
    Change {
        resolution: crate::diff::resolution(
            crate::keys::resolution(&super::inputs(before)),
            crate::keys::resolution(&super::inputs(after)),
            None,
            None,
        ),
        seeds: seeds(&before.seeds, &after.seeds),
        request: Names {
            added: difference(&after.request, &before.request),
            removed: difference(&before.request, &after.request),
        },
        sources: SourcesPair::between(&before.sources, &after.sources),
        variant: VariantPair::between(&before.variant, &after.variant),
        preseeds: PreseedsPair::between(&before.preseeds, &after.preseeds),
        initramfs: InitramfsPair::between(&before.initramfs, &after.initramfs),
        apt: AptPair::between(&before.apt, &after.apt),
    }
}
fn difference(names: &[Box<[u8]>], other: &[Box<[u8]>]) -> Box<[Box<[u8]>]> {
    let mut changed = List::with_capacity(u32::try_from(names.len()).expect("bounded request"));
    for name in names {
        if !other.contains(name) {
            changed.push(name.clone()).expect("request capacity");
        }
    }
    let mut changed = changed.into_boxed();
    changed.sort();
    changed
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
pub type SeedUserKeyed = crate::diff::Keyed<SeedIds>;
pub type SeedGroupKeyed = crate::diff::Keyed<u32>;

fn seeds(before: &super::Seeds, after: &super::Seeds) -> SeedsChange {
    SeedsChange {
        users: crate::diff::keyed(
            &before.users,
            &after.users,
            |row| &row.name,
            |change| change.map(|row| SeedIds { uid: row.uid, gid: row.gid }),
        ),
        groups: crate::diff::keyed(&before.groups, &after.groups, |row| &row.name, |change| change.map(|row| row.gid)),
    }
}
