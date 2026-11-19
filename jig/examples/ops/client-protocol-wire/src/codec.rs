//! Whole, versioned records at both ends of the family wire (protocol/client/README.md, 8).
use crate::v1::{self as w, Record};
use alloc::boxed::Box;
use skein_lib::Reader;

/// A record with the wrong version, malformed fields, or trailing bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Undecodable;

pub mod infrastructure {
    use super::{Box, Reader, Record, Undecodable, w};

    /// Encode an admitted environment request.
    pub fn ask(value: &w::InfrastructureAskRecord) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }

    /// Decode a request before the engine admits it.
    pub fn ask_value(bytes: &[u8]) -> Result<w::InfrastructureAskRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureAskRecord::decode(&w::CEILINGS, &mut reader) else { return Err(Undecodable) };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode the connector's durable answer.
    pub fn answer_record(value: &w::InfrastructureAnswerRecord) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }

    /// Decode the connector's answer at the client.
    pub fn answer(bytes: &[u8]) -> Result<w::InfrastructureAnswerRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureAnswerRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode a subject kind for a watch or read.
    pub fn subject(value: &w::InfrastructureSubjectRecord) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }

    /// Decode the subject at the engine.
    pub fn subject_value(bytes: &[u8]) -> Result<w::InfrastructureSubjectRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureSubjectRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode the owner's whole document.
    pub fn document_record(value: &w::InfrastructureDocumentRecord) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }

    /// Decode the owner's whole document at the client.
    pub fn document(bytes: &[u8]) -> Result<w::InfrastructureDocumentRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureDocumentRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
    /// Encode the owner's read answer, including a refusal.
    pub fn read_answer(value: &w::InfrastructureReadAnswer) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }
    /// Decode a whole owner read answer.
    pub fn read_answer_value(bytes: &[u8]) -> Result<w::InfrastructureReadAnswer, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureReadAnswer::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
}

pub mod observability {
    use super::{Box, Reader, Record, Undecodable, w};

    /// Encode an incident, alert or panel subject kind.
    pub fn subject(value: &w::ObservabilitySubjectRecord) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }

    /// Decode the subject at the engine.
    pub fn subject_value(bytes: &[u8]) -> Result<w::ObservabilitySubjectRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::ObservabilitySubjectRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode the owner's whole incident, alert or panel document.
    pub fn document_record(value: &w::ObservabilityDocumentRecord) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }

    /// Decode the owner's whole document at the client.
    pub fn document(bytes: &[u8]) -> Result<w::ObservabilityDocumentRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::ObservabilityDocumentRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
    /// Encode the owner's read answer, including a refusal.
    pub fn read_answer(value: &w::ObservabilityReadAnswer) -> Result<Box<[u8]>, w::Problem> {
        value.encode(&w::CEILINGS)
    }
    /// Decode a whole owner read answer.
    pub fn read_answer_value(bytes: &[u8]) -> Result<w::ObservabilityReadAnswer, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::ObservabilityReadAnswer::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
}
