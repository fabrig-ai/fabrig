//! Checked translations reached by the link's maps (skein/docs/design/data/codec/maps.md, 4.2).
use super::facts;
use crate::Limits;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use skein_lib::bytes;

pub(crate) use super::asks::wire_decision as decision_to_record;
pub(crate) use super::facts::{retry_notice_to_domain, run_work_to_domain};
pub(crate) use super::llm::{model_effort_to_domain, model_effort_to_record};

pub(crate) fn cursor_to_record(value: &c::Cursor) -> w::Cursor {
    w::Cursor { value: bytes::copy_of(value.value.bytes()) }
}

pub(crate) fn document_text_to_domain(record: w::DocumentText, limits: &Limits) -> Result<c::DocumentText, w::Problem> {
    let w::DocumentText { bytes, cut } = record;
    match c::Text::new(bytes, limits.client().long_text) {
        Ok(bytes) => Ok(c::DocumentText { bytes, cut }),
        Err(_) => Err(super::text_problem()),
    }
}

pub(crate) fn hold_reason_to_domain(record: w::HoldReason, limits: &Limits) -> Result<c::HoldReason, w::Problem> {
    facts::hold_reason(&record, limits)
}

pub(crate) fn revision_to_domain(value: u64) -> c::Revision {
    c::Revision { value }
}
