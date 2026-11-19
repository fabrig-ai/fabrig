//! Ops values mapped explicitly to their versioned family records (root.md, 7).
use alloc::boxed::Box;
use jig_ops_client_protocol_wire::{codec, v1 as w};
/// A value outside the family wire's bounds or vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
mod infrastructure_data;
mod infrastructure_values;
mod observability_data;
mod observability_values;
fn proposal_encode(value: &jig_client::contract::ProposalKey) -> w::DecisionKey {
    use jig_client::contract::Proposer;
    let maker = match value.proposer {
        Proposer::Task(value) => {
            let value = w::DecisionTask { value };
            w::DecisionMaker::Task(value)
        }
        Proposer::Party(value) => {
            let value = w::DecisionParty { value };
            w::DecisionMaker::Party(value)
        }
    };
    w::DecisionKey { maker, number: value.number }
}
fn proposal_decode(value: w::DecisionKey) -> jig_client::contract::ProposalKey {
    use jig_client::contract::Proposer;

    let proposer = match value.maker {
        w::DecisionMaker::Task(value) => Proposer::Task(value.value),
        w::DecisionMaker::Party(value) => Proposer::Party(value.value),
    };
    jig_client::contract::ProposalKey { proposer, number: value.number }
}
/// Infrastructure subjects and documents on the family wire.
pub mod infrastructure {
    use super::{Box, Invalid, codec, infrastructure_values as values, w};
    use crate::infrastructure as d;
    /// Encode a whole subject.
    pub fn subject(value: &d::Subject) -> Result<Box<[u8]>, Invalid> {
        let record = w::InfrastructureSubjectRecord { value: values::subject_encode(value).kind };
        match codec::infrastructure::subject(&record) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Invalid),
        }
    }
    /// Decode a subject kind in the scope supplied by jig.
    pub fn subject_value(bytes: &[u8], scope: jig_client::contract::Scope) -> Result<d::Subject, Invalid> {
        let Ok(record) = codec::infrastructure::subject_value(bytes) else { return Err(Invalid) };
        let scope = match scope {
            jig_client::contract::Scope::Project(project) => d::Scope::Project(project),
            jig_client::contract::Scope::Deployment => d::Scope::Deployment,
            jig_client::contract::Scope::Own => return Err(Invalid),
        };
        Ok(d::Subject { scope, kind: values::kind_decode(record.value) })
    }
    /// Encode a whole document.
    pub fn document(value: &d::Document) -> Result<Box<[u8]>, Invalid> {
        let record = w::InfrastructureDocumentRecord { value: values::document_encode(value)? };
        match codec::infrastructure::document_record(&record) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Invalid),
        }
    }
    /// Decode a whole document.
    pub fn document_value(bytes: &[u8]) -> Result<d::Document, Invalid> {
        let Ok(record) = codec::infrastructure::document(bytes) else { return Err(Invalid) };
        Ok(values::document_decode(record.value))
    }
    /// Encode an owner read result without losing its refusal.
    pub fn read_answer(value: &d::ReadResult) -> Result<Box<[u8]>, Invalid> {
        let value = match value {
            d::ReadResult::Value(value) => {
                let value = w::InfrastructureReadValue { value: values::document_encode(value)? };
                w::InfrastructureReadResult::Value(Box::new(value))
            }
            d::ReadResult::NotAllowed => w::InfrastructureReadResult::NotAllowed,
            d::ReadResult::Unknown | d::ReadResult::Busy | d::ReadResult::SignedOut => {
                w::InfrastructureReadResult::Unknown
            }
        };
        let record = w::InfrastructureReadAnswer { value };
        match codec::infrastructure::read_answer(&record) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Invalid),
        }
    }
    /// Decode the owner's read result; malformed records are unknown.
    #[must_use]
    pub fn read_answer_value(bytes: &[u8]) -> d::ReadResult {
        match codec::infrastructure::read_answer_value(bytes) {
            Ok(record) => match record.value {
                w::InfrastructureReadResult::Value(value) => {
                    d::ReadResult::Value(Box::new(values::document_decode(value.value)))
                }
                w::InfrastructureReadResult::NotAllowed => d::ReadResult::NotAllowed,
                w::InfrastructureReadResult::Unknown => d::ReadResult::Unknown,
            },
            Err(_) => d::ReadResult::Unknown,
        }
    }
}
/// Observability subjects and documents on the family wire.
pub mod observability {
    use super::{Box, Invalid, codec, observability_values as values, w};
    use crate::observability as d;
    /// Encode a whole subject.
    pub fn subject(value: &d::Subject) -> Result<Box<[u8]>, Invalid> {
        let record = w::ObservabilitySubjectRecord { value: values::subject_encode(value).kind };
        match codec::observability::subject(&record) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Invalid),
        }
    }
    /// Decode a subject kind in the scope supplied by jig.
    pub fn subject_value(bytes: &[u8], scope: jig_client::contract::Scope) -> Result<d::Subject, Invalid> {
        let Ok(record) = codec::observability::subject_value(bytes) else { return Err(Invalid) };
        let scope = match scope {
            jig_client::contract::Scope::Project(project) => d::Scope::Project(project),
            jig_client::contract::Scope::Deployment => d::Scope::Deployment,
            jig_client::contract::Scope::Own => return Err(Invalid),
        };
        Ok(d::Subject { scope, kind: values::kind_decode(record.value) })
    }
    /// Encode a whole document.
    pub fn document(value: &d::Document) -> Result<Box<[u8]>, Invalid> {
        let record = w::ObservabilityDocumentRecord { value: values::document_encode(value)? };
        match codec::observability::document_record(&record) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Invalid),
        }
    }
    /// Decode a whole document.
    pub fn document_value(bytes: &[u8]) -> Result<d::Document, Invalid> {
        let Ok(record) = codec::observability::document(bytes) else { return Err(Invalid) };
        Ok(values::document_decode(record.value))
    }
    /// Encode an owner read result without losing its refusal.
    pub fn read_answer(value: &d::ReadResult) -> Result<Box<[u8]>, Invalid> {
        let value = match value {
            d::ReadResult::Value(value) => {
                let value = w::ObservabilityReadValue { value: values::document_encode(value)? };
                w::ObservabilityReadResult::Value(Box::new(value))
            }
            d::ReadResult::NotAllowed => w::ObservabilityReadResult::NotAllowed,
            d::ReadResult::Unknown | d::ReadResult::Busy | d::ReadResult::SignedOut => {
                w::ObservabilityReadResult::Unknown
            }
        };
        let record = w::ObservabilityReadAnswer { value };
        match codec::observability::read_answer(&record) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Invalid),
        }
    }
    /// Decode the owner's read result; malformed records are unknown.
    #[must_use]
    pub fn read_answer_value(bytes: &[u8]) -> d::ReadResult {
        match codec::observability::read_answer_value(bytes) {
            Ok(record) => match record.value {
                w::ObservabilityReadResult::Value(value) => {
                    d::ReadResult::Value(Box::new(values::document_decode(value.value)))
                }
                w::ObservabilityReadResult::NotAllowed => d::ReadResult::NotAllowed,
                w::ObservabilityReadResult::Unknown => d::ReadResult::Unknown,
            },
            Err(_) => d::ReadResult::Unknown,
        }
    }
}
/// Encode an infrastructure ask.
pub fn ask(value: &crate::infrastructure::Ask) -> Result<Box<[u8]>, Invalid> {
    let record = w::InfrastructureAskRecord { value: infrastructure_values::ask_encode(value) };
    match codec::infrastructure::ask(&record) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(Invalid),
    }
}
/// Decode an infrastructure ask.
pub fn ask_value(bytes: &[u8]) -> Result<crate::infrastructure::Ask, Invalid> {
    let Ok(record) = codec::infrastructure::ask_value(bytes) else { return Err(Invalid) };
    Ok(infrastructure_values::ask_decode(record.value))
}
/// Encode an infrastructure answer.
pub fn answer(value: &crate::infrastructure::Answer) -> Result<Box<[u8]>, Invalid> {
    let record = w::InfrastructureAnswerRecord { value: infrastructure_values::answer_encode(value)? };
    match codec::infrastructure::answer_record(&record) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(Invalid),
    }
}
/// Decode an infrastructure answer.
pub fn answer_value(bytes: &[u8]) -> Result<crate::infrastructure::Answer, Invalid> {
    let Ok(record) = codec::infrastructure::answer(bytes) else { return Err(Invalid) };
    Ok(infrastructure_values::answer_decode(record.value))
}
