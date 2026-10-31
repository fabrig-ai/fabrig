//! Single read documents (client/domain/contract.md, 5–6).
use super::pages::{
    owner_escalation_fields, owner_historical_fields, owner_person_task_fields, owner_proposal_fields, question_fields,
};
use super::{Undecodable, classify};
use crate::Limits;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
pub fn owner_proposal(record: w::OwnerProposal, limits: &Limits) -> Result<c::OwnerProposal, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerProposal::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_proposal_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn owner_escalation(record: w::OwnerEscalation, limits: &Limits) -> Result<c::OwnerEscalation, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerEscalation::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_escalation_fields(&record, limits) {
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

pub fn owner_person_task(record: w::OwnerPersonTask, limits: &Limits) -> Result<c::OwnerPersonTask, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerPersonTask::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_person_task_fields(&record, limits) {
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

pub fn owner_historical(record: w::OwnerHistorical, limits: &Limits) -> Result<c::OwnerHistorical, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerHistorical::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_historical_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}
