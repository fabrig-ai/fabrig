//! Wire projections of owner kind order facts (client/domain/testing.md, 4).
use super::super::{c, w};
pub(in crate::codec) fn wire_kind_implication(
    value: &c::KindImplication,
    limits: &w::Limits,
) -> Result<w::KindImplication, w::Problem> {
    w::KindImplication::new(
        limits,
        w::KindImplicationParts { connector: value.connector, kind: value.kind, implies: value.implies },
    )
}
