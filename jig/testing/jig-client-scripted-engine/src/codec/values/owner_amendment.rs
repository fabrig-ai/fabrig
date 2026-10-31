//! Wire projections of owner amendment facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
use super::{
    wire_document_text, wire_owner_authority_document, wire_owner_wake_policy, wire_person_address,
    wire_specification_document,
};
pub(in crate::codec) fn wire_owner_amendment_state(
    value: &c::OwnerAmendmentState,
    limits: &w::Limits,
) -> Result<w::OwnerAmendmentState, w::Problem> {
    w::OwnerAmendmentState::new(
        limits,
        w::OwnerAmendmentStateParts {
            addressed: value.addressed.as_ref().map(|item| wire_person_address(item, limits)).transpose()?,
            taken_by: value.taken_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            instructions: value.instructions.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            spec: wire_specification_document(&value.spec, limits)?,
            authority: wire_owner_authority_document(&value.authority, limits)?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            dependencies: list(value.dependencies.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            cut: value.cut,
        },
    )
}
