//! Top-level forge routing and commit fence (jig's domain/connectors.md,
//! sections 2–5). Durable rows are held here and mirrored by root store
//! writes.
//! A requester's confirmed branch moves become its change's new head
use crate::Repository;
use alloc::boxed::Box;
use skein_lib::List;

/// The title for an empty explicit pull title (domain/forge/change/README.md, 1).
pub const DEFAULT_CHANGE_TITLE: &[u8] = b"Update repository";
/// Pack a repository's complete forge identity (domain/forge/adoption.md).
#[must_use]
pub const fn tag(forge: u16, repository: u32) -> u64 {
    let [forge_high, forge_low] = forge.to_be_bytes();
    let [one, two, three, four] = repository.to_be_bytes();
    u64::from_be_bytes([0, 0, forge_high, forge_low, one, two, three, four])
}

/// Decode a repository tag, rejecting bits outside the forge identity's width.
#[must_use]
pub const fn untag(tag: u64) -> Option<(u16, u32)> {
    let [high, low, forge_high, forge_low, one, two, three, four] = tag.to_be_bytes();
    if high != 0 || low != 0 {
        None
    } else {
        Some((u16::from_be_bytes([forge_high, forge_low]), u32::from_be_bytes([one, two, three, four])))
    }
}
pub fn branch_resource_name(repository: &Repository, branch: &[u8]) -> Option<Box<[Box<[u8]>]>> {
    let mut segments = List::uncapped();
    for segment in [b"forge".as_slice(), &repository.host, &repository.owner, &repository.name, b"branch"] {
        if segment.is_empty() {
            return None;
        }
        segments.push(Box::from(segment)).ok()?;
    }
    let mut start = 0;
    for at in 0..=branch.len() {
        if at == branch.len() || branch.get(at) == Some(&b'/') {
            let part = branch.get(start..at)?;
            if part.is_empty() {
                return None;
            }
            segments.push(Box::from(part)).ok()?;
            start = at.checked_add(1)?;
        }
    }
    Some(segments.into_boxed())
}
