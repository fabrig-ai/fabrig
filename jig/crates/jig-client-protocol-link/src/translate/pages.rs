//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::classify;
use super::documents::{
    authority_document_fields, authority_fields, chip_ending_fields, chip_fields, cursor_fields, ending_fields,
    object_key_fields, offered_piece, proposed_task_piece, result_fields, specification_document_fields,
    task_contract_document_fields, task_fields, task_link_fields,
};
use super::{Undecodable, too_large};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v21 as w;
use skein_lib::{List, Reader, Wall, bytes};
fn opaque(value: &[u8], limits: &Limits) -> Result<c::Bytes, w::Problem> {
    match c::Bytes::new(bytes::copy_of(value), limits.client().text) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}
fn text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limits.client().text) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}

/// A line's words, whole within `long_text` (client/domain/client/README.md, 11).
pub(super) fn long_text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limits.client().long_text) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}

mod activity;
pub(crate) use activity::*;
mod decisions;
pub(crate) use decisions::*;
mod ended;
pub(crate) use ended::*;
mod history;
pub(crate) use history::*;
mod inbox;
pub(crate) use inbox::*;
mod notes;
pub(crate) use notes::*;
mod person;
pub(crate) use person::*;
mod proposals;
pub(crate) use proposals::*;
mod spend;
pub(crate) use spend::*;

pub fn history_page(record: w::HistoryPage, limits: &Limits) -> Result<c::HistoryPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::HistoryPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match history_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn note_page(record: w::NotePage, limits: &Limits) -> Result<c::NotePage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::NotePage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match note_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn ended_page(record: w::EndedPage, limits: &Limits) -> Result<c::EndedPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::EndedPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match ended_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn inbox_page(record: w::InboxPage, limits: &Limits) -> Result<c::InboxPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::InboxPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match inbox_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn spend_page(record: w::SpendPage, limits: &Limits) -> Result<c::SpendPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::SpendPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match spend_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn activity_page(record: w::ActivityPage, limits: &Limits) -> Result<c::ActivityPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::ActivityPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match activity_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

/// An archived turn its page carries, within the archive's byte bound and the page's `turn_text`.
pub(crate) fn carried_turn_fields(value: &[u8], limits: &Limits) -> c::TurnRead {
    super::turn::decode(value, limits.archived_turn(), false)
}

pub(crate) fn task_page(value: w::TaskPage, limits: &Limits) -> Result<c::TaskPage, Undecodable> {
    if u64::from(value.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let value = match w::TaskPage::new(limits.receiving(), value.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match task_page_fields(&value, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}
fn task_page_fields(value: &w::TaskPage, limits: &Limits) -> Result<c::TaskPage, w::Problem> {
    let live = super::documents::task_line_piece(value.live(), limits, limits.client().rows)?;
    let live_after = match value.live_after() {
        Some(v) => Some(cursor_fields(v, limits)?),
        None => None,
    };
    let ended = match value.ended() {
        w::EndedGroup::None => c::EndedGroup::None,
        w::EndedGroup::Rows(v) => c::EndedGroup::Rows {
            rows: super::documents::task_line_piece(v.rows(), limits, limits.client().rows)?,
            after: match v.after() {
                Some(v) => Some(cursor_fields(v, limits)?),
                None => None,
            },
            cut: v.cut(),
        },
    };
    Ok(c::TaskPage { live, live_after, ended })
}
