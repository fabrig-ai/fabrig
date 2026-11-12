//! Whole document decoding (protocol/client/wire.md, 1; client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{Undecodable, classify, piece_problem, too_large};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v20 as w;
use skein_lib::{List, Reader, Wall, bytes, length::len_u64};
#[path = "accounts.rs"]
mod accounts;
#[path = "catalogue.rs"]
mod catalogue;
pub(crate) use accounts::*;

mod authority;
pub(crate) use authority::*;
mod capacity;
pub(crate) use capacity::*;
mod chips;
pub(crate) use chips::*;
mod deployment;
pub(crate) use deployment::*;
mod frame;
pub(crate) use frame::*;
mod hosts;
pub(crate) use hosts::*;
mod lines;
pub(crate) use lines::*;
mod offered;
pub(crate) use offered::*;
mod project;
pub(crate) use project::*;
mod results;
pub(crate) use results::*;
mod runs;
pub(crate) use runs::*;
mod specification;
pub(crate) use specification::*;
mod task;
pub(crate) use task::*;
mod task_links;
pub(crate) use task_links::*;

fn text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limits.client().text) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}
fn raw(value: &[u8], limits: &Limits) -> Result<Box<[u8]>, w::Problem> {
    if len_u64(value.len()) > u64::from(limits.client().text) {
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

pub fn deployment(record: w::Deployment, limits: &Limits) -> Result<c::Deployment, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Deployment::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match deployment_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn project(record: w::Project, limits: &Limits) -> Result<c::Project, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::Project::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match project_fields(&record, limits) {
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
