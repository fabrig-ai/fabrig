//! Listed entries retain every field (hearth/docs/design/domain/render/README.md, 2.2).
use crate::{git_machine::listing, render::revision};
use skein_lib::bytes;
#[must_use]
pub fn entry(entry: listing::Entry) -> revision::Entry {
    let listing::Entry { path, mode, oid, size } = entry;
    revision::Entry { path, mode, oid: revision::Oid(bytes::copy_of(oid.hex())), size }
}
