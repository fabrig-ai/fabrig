//! Issue projection entries use the store maps (skein/docs/design/data/codec/maps.md, 4).
use super::{Undecodable, w};
use temper_engine_domain_forge_vocabulary as d;
pub(super) fn encode(value: &d::IssueRow) -> w::IssueRow {
    crate::generated::store::issue_row_to_record(value).expect(super::FITS)
}
pub(super) fn decode(value: w::IssueRow) -> Result<d::IssueRow, Undecodable> {
    crate::generated::store::issue_row_to_domain(value).or(Err(Undecodable::Malformed))
}
