//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::classify;
use super::documents::{
    budget_fields, chip_fields, cursor_fields, ending_fields, field_rule_fields, hold_fields, inbox_entry_piece,
    object_key_fields, offered_piece, owner_authority_document_fields, owner_authority_fields,
    owner_chip_ending_fields, owner_task_link_fields, part_piece, passed_piece, proposed_task_piece, recurring_piece,
    result_fields, specification_document_fields, task_contract_document_fields, task_fields,
};
use super::{Undecodable, piece_problem, too_large};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
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

#[path = "pages_part0.rs"]
mod part0;
pub(crate) use part0::*;
#[path = "pages_part1.rs"]
mod part1;
pub(crate) use part1::*;
#[path = "pages_part2.rs"]
mod part2;
pub(crate) use part2::*;
#[path = "pages_part3.rs"]
mod part3;
pub(crate) use part3::*;
#[path = "pages_part4.rs"]
mod part4;
pub(crate) use part4::*;
#[path = "pages_part5.rs"]
mod part5;
pub(crate) use part5::*;
#[path = "pages_part6.rs"]
mod part6;
pub(crate) use part6::*;
#[path = "pages_part7.rs"]
mod part7;
pub(crate) use part7::*;
#[path = "pages_part8.rs"]
mod part8;
pub(crate) use part8::*;

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

pub fn object_page(record: w::ObjectPage, limits: &Limits) -> Result<c::ObjectPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::ObjectPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match object_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn owner_ended_page(record: w::OwnerEndedPage, limits: &Limits) -> Result<c::OwnerEndedPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerEndedPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_ended_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn owner_inbox_page(record: w::OwnerInboxPage, limits: &Limits) -> Result<c::OwnerInboxPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerInboxPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_inbox_page_fields(&record, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

pub fn owner_spend_page(record: w::OwnerSpendPage, limits: &Limits) -> Result<c::OwnerSpendPage, Undecodable> {
    if u64::from(record.measure()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::OwnerSpendPage::new(limits.receiving(), record.into_parts()) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match owner_spend_page_fields(&record, limits) {
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

pub(crate) fn turn_fields(value: &[u8], limits: &Limits) -> c::TurnRead {
    let bytes = u32::try_from(limits.client().document_bytes)
        .expect("validated document bytes")
        .min(limits.receiving().activity_turn_turn);
    super::turn::decode(value, limits, bytes)
}

/// An archived turn its page carries, within the archive's bound (`turn::archive`).
pub(crate) fn carried_turn_fields(value: &[u8], limits: &Limits) -> c::TurnRead {
    let bytes = u32::try_from(limits.client().document_bytes)
        .expect("validated document bytes")
        .min(limits.receiving().turn_carried_bytes);
    super::turn::decode(value, limits, bytes)
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
