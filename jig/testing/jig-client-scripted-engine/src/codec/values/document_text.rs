//! Wire projections of document text facts (client/domain/testing.md, 4).
use super::super::{bytes, c, w};
pub(in crate::codec) fn wire_document_text(
    value: &c::DocumentText,
    _limits: &w::Limits,
) -> Result<w::DocumentText, w::Problem> {
    Ok(w::DocumentText { bytes: bytes::copy_of(value.bytes.bytes()), cut: value.cut })
}
