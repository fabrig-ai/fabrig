//! Wire projections of owner kind order facts (client/domain/testing.md, 4).
use super::super::{c, w};
pub(in crate::codec) fn wire_owner_kind_implication(
    value: &c::OwnerKindImplication,
    limits: &w::Limits,
) -> Result<w::OwnerKindImplication, w::Problem> {
    w::OwnerKindImplication::new(
        limits,
        w::OwnerKindImplicationParts { connector: value.connector, kind: value.kind, implies: value.implies },
    )
}
