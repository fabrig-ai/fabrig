//! Wire projections of turn facts (client/domain/testing.md, 4).
use super::super::{bytes, c, empty_pieces, list, w};
use super::wire_object_key;
pub(in crate::codec) fn wire_call(value: &c::Call, limits: &w::Limits) -> Result<w::Call, w::Problem> {
    w::Call::new(
        limits,
        w::CallParts {
            number: value.number,
            tool: bytes::copy_of(value.tool.bytes()),
            input: bytes::copy_of(value.input.bytes()),
            outcome: value
                .outcome
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
            failed: value.failed,
            objects: list(value.objects.iter().map(|item| wire_object_key(item, limits)))?,
            parts: empty_pieces(&value.parts)?,
        },
    )
}
