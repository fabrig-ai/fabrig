//! Specs and part digests (hearth/docs/design/domain/render/spec.md, 1.4, 1.5).
pub use hearth_domain_render_vocabulary::parts::{Part, PartRecord, Spec};
use hearth_lib::{encode::Digest, hash::H};
pub mod boot;
pub mod config;
pub mod data;
pub mod exports;
pub mod live;
pub mod meta;
pub mod packages;
pub mod secrets;
pub mod software;
pub use meta::Meta;

#[must_use]
pub(crate) fn spec_digest(meta: &Digest, parts: &[Part]) -> Digest {
    let mut hash = H::new(b"spec", 1);
    let record = hash.values();
    record.map(u64::try_from(parts.len()).expect("bounded parts").saturating_add(1));
    let mut ordered = skein_lib::Map::with_capacity(u32::try_from(parts.len()).expect("part count").saturating_add(1));
    ordered.insert(skein_lib::bytes::copy_of(b"meta"), *meta).expect("part count");
    for part in parts {
        ordered.insert(skein_lib::bytes::copy_of(part.record.name()), part.digest).expect("part count");
    }
    for (name, digest) in &ordered {
        record.key(name);
        record.digest(digest);
    }
    hash.finish()
}
