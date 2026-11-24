//! Durable forge and playground rows (temper/docs/design/protocol/store.md, 2).
use super::{FITS, Undecodable, w};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode(row: &d::ForgeRecord) -> w::Forge {
    crate::generated::store::forge_to_record(row).expect(FITS)
}

pub(super) fn decode(row: w::Forge) -> Result<d::ForgeRecord, Undecodable> {
    crate::generated::store::forge_to_domain(row).or(Err(Undecodable::Malformed))
}

pub(super) fn encode_playground(row: &d::Playground) -> w::Playground {
    crate::generated::store::playground_to_record(row)
}

pub(super) fn decode_playground(row: w::Playground) -> d::Playground {
    crate::generated::store::playground_to_domain(row)
}

/// Owner-shaped `ForgeLink` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn forge_link_to_record(value: &d::Link) -> w::ForgeLink {
    w::ForgeLink { user: value.user, person: value.person }
}
pub fn forge_link_to_domain(value: w::ForgeLink) -> Result<d::Link, w::Problem> {
    if value.user == 0 || value.person == 0 {
        Err(Undecodable::Malformed)
    } else {
        Ok(d::Link { user: value.user, person: value.person })
    }
    .or(Err(w::paths::FORGE_LINK_USER.problem(skein_codec::Reason::Malformed)))
}
