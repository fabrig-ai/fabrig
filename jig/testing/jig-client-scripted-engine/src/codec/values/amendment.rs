//! Wire projections of owner amendment facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
use super::{
    wire_authority_document, wire_document_text, wire_person_address, wire_specification_document, wire_wake_policy,
};
pub(in crate::codec) fn wire_amendment_state(
    value: &c::AmendmentState,
    limits: &w::Limits,
) -> Result<w::AmendmentState, w::Problem> {
    Ok(w::AmendmentState {
        addressed: value.addressed.as_ref().map(wire_person_address),
        taken_by: value.taken_by,
        instructions: value.instructions.as_ref().map(wire_document_text),
        spec: wire_specification_document(&value.spec),
        authority: wire_authority_document(&value.authority, limits)?,
        wake: wire_wake_policy(&value.wake),
        dependencies: list(value.dependencies.iter().copied()),
        cut: value.cut,
    })
}
