//! Typed packages differences (hearth/docs/design/domain/render/README.md, 4.1).
use super::{Box, List, Packages};
pub use hearth_domain_render_vocabulary::parts::packages::{
    AptPair, Change, InitramfsPair, Names, PreseedsPair, SeedGroupKeyed, SeedIds, SeedUserKeyed, SeedsChange,
    SourcesPair, VariantPair,
};
use skein_lib::{Joined, Map, bytes};

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
        sources: if before.sources == after.sources {
            None
        } else {
            Some(SourcesPair { before: before.sources.clone(), after: after.sources.clone() })
        },
        variant: if before.variant == after.variant {
            None
        } else {
            Some(VariantPair { before: before.variant, after: after.variant })
        },
        preseeds: if before.preseeds == after.preseeds {
            None
        } else {
            Some(PreseedsPair { before: before.preseeds.clone(), after: after.preseeds.clone() })
        },
        initramfs: if before.initramfs == after.initramfs {
            None
        } else {
            Some(InitramfsPair { before: before.initramfs.clone(), after: after.initramfs.clone() })
        },
        apt: if before.apt == after.apt {
            None
        } else {
            Some(AptPair { before: before.apt.clone(), after: after.apt.clone() })
        },
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

fn seeds(before: &super::Seeds, after: &super::Seeds) -> SeedsChange {
    let mut old: Map<&[u8], &super::SeedUser> = Map::uncapped();
    let mut new: Map<&[u8], &super::SeedUser> = Map::uncapped();
    for row in &before.users {
        old.insert(&row.name, row).expect("an uncapped map");
    }
    for row in &after.users {
        new.insert(&row.name, row).expect("an uncapped map");
    }
    let mut users = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => {
                SeedUserKeyed::Removed { key: bytes::copy_of(key), value: SeedIds { uid: row.uid, gid: row.gid } }
            }
            Joined::Second(row) => {
                SeedUserKeyed::Added { key: bytes::copy_of(key), value: SeedIds { uid: row.uid, gid: row.gid } }
            }
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => SeedUserKeyed::Changed {
                key: bytes::copy_of(key),
                before: SeedIds { uid: was.uid, gid: was.gid },
                after: SeedIds { uid: now.uid, gid: now.gid },
            },
        };
        users.push(change).expect("an uncapped list");
    }
    let mut old: Map<&[u8], &super::SeedGroup> = Map::uncapped();
    let mut new: Map<&[u8], &super::SeedGroup> = Map::uncapped();
    for row in &before.groups {
        old.insert(&row.name, row).expect("an uncapped map");
    }
    for row in &after.groups {
        new.insert(&row.name, row).expect("an uncapped map");
    }
    let mut groups = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => SeedGroupKeyed::Removed { key: bytes::copy_of(key), value: row.gid },
            Joined::Second(row) => SeedGroupKeyed::Added { key: bytes::copy_of(key), value: row.gid },
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => {
                SeedGroupKeyed::Changed { key: bytes::copy_of(key), before: was.gid, after: now.gid }
            }
        };
        groups.push(change).expect("an uncapped list");
    }
    SeedsChange { users: users.into_boxed(), groups: groups.into_boxed() }
}
