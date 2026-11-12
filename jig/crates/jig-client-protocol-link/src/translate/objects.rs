//! Single read documents (client/domain/contract/documents.md and client/domain/contract/README.md, 4).
use super::pages::{ended_fields, escalation_fields, person_task_fields, proposal_fields, question_fields};
use super::{Undecodable, classify};
use crate::Limits;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v20 as w;
pub fn proposal(record: w::Proposal, limits: &Limits) -> Result<c::Proposal, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Proposal::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match proposal_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn escalation(record: w::Escalation, limits: &Limits) -> Result<c::Escalation, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Escalation::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match escalation_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn question(record: w::Question, limits: &Limits) -> Result<c::Question, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Question::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match question_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn person_task(record: w::PersonTask, limits: &Limits) -> Result<c::PersonTask, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::PersonTask::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match person_task_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn result(record: w::ResultDocument, limits: &Limits) -> Result<c::Result, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::ResultDocument::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match crate::translate::documents::result_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn ended(record: w::Ended, limits: &Limits) -> Result<c::Ended, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Ended::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match ended_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}
