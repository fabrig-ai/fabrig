//! The durable account of a named limit hold (protocol/configuration/limit-settings.md, 6).
use super::{Bounds, Undecodable, d, w};

pub(super) fn encode(row: &d::LimitHold) -> w::LimitHold {
    crate::generated::store::limit_hold_to_record(row, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn decode(value: w::LimitHold, bounds: &Bounds) -> Result<d::LimitHold, Undecodable> {
    crate::generated::store::limit_hold_to_domain(value, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))
}
