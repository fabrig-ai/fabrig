//! Whole document decoding (protocol/client.md, 3; client/domain/contract.md, 5–7).
use super::{Undecodable, classify, piece_problem, too_large};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
use skein_lib::{List, Reader, Wall, bytes};
#[path = "accounts.rs"]
mod accounts;
#[path = "catalogue.rs"]
mod catalogue;
#[path = "documents_part0.rs"]
mod part0;
pub(crate) use accounts::*;
pub(crate) use part0::*;
#[path = "documents_part1.rs"]
mod part1;
pub(crate) use part1::*;
#[path = "documents_part2.rs"]
mod part2;
pub(crate) use part2::*;
#[path = "documents_part3.rs"]
mod part3;
pub(crate) use part3::*;
#[path = "documents_part4.rs"]
mod part4;
pub(crate) use part4::*;
#[path = "documents_part5.rs"]
mod part5;
pub(crate) use part5::*;
#[path = "documents_part6.rs"]
mod part6;
pub(crate) use part6::*;
#[path = "documents_part7.rs"]
mod part7;
pub(crate) use part7::*;
#[path = "documents_part8.rs"]
mod part8;
pub(crate) use part8::*;

fn text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limits.client().text) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}
fn raw(value: &[u8], limits: &Limits) -> Result<Box<[u8]>, w::Problem> {
    if u64::try_from(value.len()).expect("usize fits u64") > u64::from(limits.client().text) {
        return Err(too_large());
    }
    Ok(bytes::copy_of(value))
}

pub fn frame(record: w::Me, limits: &Limits) -> Result<c::Me, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Me::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match frame_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn task(record: w::Task, limits: &Limits) -> Result<c::Task, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Task::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match task_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn tasks(record: w::Tasks, limits: &Limits) -> Result<c::Tasks, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Tasks::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match tasks_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn owner_deployment(record: w::OwnerDeployment, limits: &Limits) -> Result<c::OwnerDeployment, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerDeployment::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_deployment_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn owner_project(record: w::OwnerProject, limits: &Limits) -> Result<c::OwnerProject, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerProject::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_project_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn chip(record: w::Chip, limits: &Limits) -> Result<c::Chip, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    match chip_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

#[path = "me.rs"]
pub(crate) mod me;
