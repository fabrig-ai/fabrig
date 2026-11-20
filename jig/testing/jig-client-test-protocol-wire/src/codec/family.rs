//! The scripted family and its complete connector values share one wire.
use super::{Record, Undecodable, d, documents, limits, parts, w};
use alloc::boxed::Box;
use skein_lib::Reader;

#[must_use]
pub fn document(value: w::FamilyDocument) -> Box<[u8]> {
    let record = w::FamilyDocumentRecord { value };
    record.encode(&w::CEILINGS).expect("admitted family document")
}
/// # Errors
/// Refuses malformed records or records outside this family's configured limits.
pub fn document_value(bytes: &[u8], bound: &d::Limits) -> Result<w::FamilyDocument, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    if w::check_limits(&bounds).is_err() {
        return Err(Undecodable);
    }
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::FamilyDocumentRecord::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(record.value)
}
#[must_use]
pub fn connector_document(value: &d::ClientDocument) -> w::ClientDocument {
    documents::encode_client_document(value)
}
/// # Errors
/// Refuses nested connector values outside the supplied limits.
pub fn connector_document_value(value: w::ClientDocument, bound: &d::Limits) -> Result<d::ClientDocument, Undecodable> {
    documents::decode_client_document(value, bound)
}

#[must_use]
pub fn answer(value: w::FamilyAnswer) -> Box<[u8]> {
    let record = w::FamilyAnswerRecord { value };
    record.encode(&w::CEILINGS).expect("admitted family answer")
}
/// # Errors
/// Refuses malformed records or records outside this family's configured limits.
pub fn answer_value(bytes: &[u8], bound: &d::Limits) -> Result<w::FamilyAnswer, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::FamilyAnswerRecord::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(record.value)
}
#[must_use]
pub fn connector_answer(value: &d::ApplicationOutcome) -> w::FamilyConnectorAnswer {
    w::FamilyConnectorAnswer { value: parts::encode_application_outcome(value) }
}
#[must_use]
pub fn connector_answer_value(value: w::FamilyConnectorAnswer, bound: &d::Limits) -> d::ApplicationOutcome {
    parts::decode_application_outcome(value.value, bound)
}

/// The complete application request, with optional parameters kept optional.
#[must_use]
pub fn request_record(project: u32, value: w::ApplicationRequest) -> Box<[u8]> {
    let record = w::Request { project, value };
    record.encode(&w::CEILINGS).expect("fixed family request")
}
/// # Errors
/// Refuses malformed records or records outside this family's configured limits.
pub fn request_value(bytes: &[u8], bound: &d::Limits) -> Result<w::Request, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Request::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(record)
}
#[must_use]
pub fn subject(value: w::FamilySubject) -> Box<[u8]> {
    let record = w::Subject { value };
    record.encode(&w::CEILINGS).expect("fixed family subject")
}
/// # Errors
/// Refuses malformed records or records outside this family's configured limits.
pub fn subject_value(bytes: &[u8], bound: &d::Limits) -> Result<w::FamilySubject, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Subject::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(record.value)
}
#[must_use]
pub fn read(value: w::FamilySubject) -> Box<[u8]> {
    let record = w::Read { value };
    record.encode(&w::CEILINGS).expect("fixed family read")
}
/// # Errors
/// Refuses malformed records or records outside this family's configured limits.
pub fn read_value(bytes: &[u8], bound: &d::Limits) -> Result<w::FamilySubject, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Read::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(record.value)
}
/// A fixed-value connector may receive only a request with a supplied value.
/// # Errors
/// Refuses absent values without manufacturing connector parameters.
pub fn connector_request(value: w::ApplicationRequest) -> Result<d::ApplicationRequest, Undecodable> {
    parts::decode_application_request(value)
}
/// # Errors
/// Refuses subjects for which this connector has no scoped representation.
pub fn connector_subject_value(value: w::FamilySubject) -> Result<d::ClientSubject, Undecodable> {
    parts::decode_client_subject(value)
}
