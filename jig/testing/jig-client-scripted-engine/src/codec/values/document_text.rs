//! Wire projections of document text facts (client/domain/testing.md, 4).
use super::super::{bytes, c, w};
pub(in crate::codec) fn wire_document_text(
    value: &c::DocumentText,
    limits: &w::Limits,
) -> Result<w::DocumentText, w::Problem> {
    w::DocumentText::new(limits, w::DocumentTextParts { bytes: bytes::copy_of(value.bytes.bytes()), cut: value.cut })
}
