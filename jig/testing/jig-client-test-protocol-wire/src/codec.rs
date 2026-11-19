//! The application's own wire, at the engine's protocol boundary (client/domain/client/README.md, 8).
use crate::v2::{self as w, Record};
use alloc::boxed::Box;
use jig_test_connector as d;
use skein_lib::Reader;
mod documents;
mod facts;
pub mod family;
mod parts;
use parts::{decode_application_request, encode_application_request};

/// A record outside this family's version, shape or configured bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Undecodable;
/// Scope carried by the record, beside the complete connector value.
#[derive(Debug, PartialEq, Eq)]
pub struct DecodedRequest {
    pub project: u32,
    pub value: d::ApplicationRequest,
}
/// The family checks each field and count; the connector owns admission sums.
#[must_use]
pub fn limits(limits: &d::Limits) -> Option<w::Limits> {
    let mut bounds = w::CEILINGS;
    bounds.segment_bytes = limits.segment_bytes;
    bounds.resource_path_segments = limits.path_segments;
    bounds.effect_resources = limits.resources_per_effect;
    bounds.description_resources = limits.resources_per_effect;
    bounds.client_document_panel_resources = limits.resources_per_task;
    bounds.client_document_hold_resources = limits.resources_per_task;
    bounds.client_document_project_resources = limits.resources;
    bounds.client_document_panel_effects = limits.entries;
    bounds.client_document_hold_effects = limits.entries;
    bounds.client_document_deployment_outbox = limits.entries;
    w::check_limits(&bounds).ok()?;
    Some(bounds)
}
/// Encode only a connector request; no digest is reconstructed from its fields.
#[must_use]
pub fn request_record(project: u32, value: &d::ApplicationRequest) -> Box<[u8]> {
    let record = w::Request { project, value: encode_application_request(value) };
    record.encode(&w::CEILINGS).expect("admitted request fits schema")
}
/// Decode before constructing the root's input; its supplied digest stays untouched.
pub fn request(bytes: &[u8], bound: &d::Limits) -> Result<DecodedRequest, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Request::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    let w::Request { project, value } = record;
    Ok(DecodedRequest { project, value: decode_application_request(value)? })
}
#[must_use]
pub fn outcome(value: &d::ApplicationOutcome) -> Box<[u8]> {
    let record = w::Outcome { value: parts::encode_application_outcome(value) };
    record.encode(&w::CEILINGS).expect("admitted outcome fits schema")
}
pub fn outcome_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ApplicationOutcome, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Outcome::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(parts::decode_application_outcome(record.value, bound))
}
#[must_use]
pub fn subject(value: &d::ClientSubject) -> Box<[u8]> {
    let record = w::Subject { value: parts::encode_client_subject(value) };
    record.encode(&w::CEILINGS).expect("admitted subject fits schema")
}
pub fn subject_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ClientSubject, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Subject::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    parts::decode_client_subject(record.value)
}
#[must_use]
pub fn read(value: &d::ClientSubject) -> Box<[u8]> {
    let record = w::Read { value: parts::encode_client_subject(value) };
    record.encode(&w::CEILINGS).expect("admitted read fits schema")
}
pub fn read_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ClientSubject, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Read::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    parts::decode_client_subject(record.value)
}
#[must_use]
pub fn document(value: &d::ClientDocument) -> Box<[u8]> {
    let record = w::Document { value: documents::encode_client_document(value) };
    record.encode(&w::CEILINGS).expect("admitted document fits schema")
}
pub fn document_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ClientDocument, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Document::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    documents::decode_client_document(record.value, bound)
}

/// A family's read terminal retains its exact absence reason in its own record.
#[must_use]
pub fn read_answer(answer: &Result<d::ClientDocument, d::DocumentMissing>) -> Box<[u8]> {
    let value = match answer {
        Ok(value) => w::ReadResult::Value(Box::new(w::ReadValue { value: documents::encode_client_document(value) })),
        Err(d::DocumentMissing::Unknown) => w::ReadResult::Unknown,
        Err(d::DocumentMissing::NotAllowed) => w::ReadResult::NotAllowed,
    };
    let record = w::ReadAnswer { value };
    record.encode(&w::CEILINGS).expect("bounded read answer")
}
pub fn read_answer_value(
    bytes: &[u8],
    bound: &d::Limits,
) -> Result<Result<d::ClientDocument, d::DocumentMissing>, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::ReadAnswer::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(match record.value {
        w::ReadResult::Value(value) => Ok(documents::decode_client_document(value.value, bound)?),
        w::ReadResult::Unknown => Err(d::DocumentMissing::Unknown),
        w::ReadResult::NotAllowed => Err(d::DocumentMissing::NotAllowed),
    })
}
