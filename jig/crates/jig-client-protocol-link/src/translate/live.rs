//! Provisional current run documents (client/domain/contract/watches.md, 1.3).
use super::Undecodable;
use super::classify;
use super::documents::{current_call_fields, live_call_fields, run_state_fields, run_text_fields};
use crate::Limits;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
pub fn run_state(record: w::RunState, limits: &Limits) -> Result<c::RunState, Undecodable> {
    if u64::from(match record.measure(limits.wire()) {
        Ok(size) => size,
        Err(problem) => return Err(classify(problem)),
    }) > limits.client().document_bytes
    {
        return Err(Undecodable::TooLarge);
    }
    match run_state_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}
pub fn run_text(record: w::RunText, limits: &Limits) -> Result<c::RunText, Undecodable> {
    if u64::from(match record.measure(limits.wire()) {
        Ok(size) => size,
        Err(problem) => return Err(classify(problem)),
    }) > limits.client().document_bytes
    {
        return Err(Undecodable::TooLarge);
    }
    match run_text_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}
pub fn current_call(record: w::CurrentCall, limits: &Limits) -> Result<c::CurrentCall, Undecodable> {
    if u64::from(match record.measure(limits.wire()) {
        Ok(size) => size,
        Err(problem) => return Err(classify(problem)),
    }) > limits.client().document_bytes
    {
        return Err(Undecodable::TooLarge);
    }
    match current_call_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}
pub fn live_call(record: w::LiveCall, limits: &Limits) -> Result<c::LiveCall, Undecodable> {
    if u64::from(match record.measure(limits.wire()) {
        Ok(size) => size,
        Err(problem) => return Err(classify(problem)),
    }) > limits.client().document_bytes
    {
        return Err(Undecodable::TooLarge);
    }
    match live_call_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}
