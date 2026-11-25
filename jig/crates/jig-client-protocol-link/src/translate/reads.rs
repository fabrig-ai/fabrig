//! One read per exchange with opaque cursor bytes (protocol/client/README.md, 4).
use super::{Undecodable, documents, objects, pages};
use crate::Limits;
use crate::generated::client;
use alloc::boxed::Box;
use jig_client_domain::Readable;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::VERSION;
use skein_lib::{Reader, Writer, length::len_u64};

#[must_use]
pub fn read(read: &c::Read) -> w::Read {
    client::read_to_record(read)
}
/// The read exchange's owner envelope; family records remain opaque.
pub fn owned(value: Readable, limits: &w::Limits) -> Result<w::Owned, w::Problem> {
    let (owner, version, record) = match value {
        Readable::Jig(value) => {
            let value = read(&value);
            let mut writer = Writer::new(skein_lib::length::usize_of(value.measure(limits)?));
            value.write(limits, &mut writer).expect("measured read");
            (w::Owner::Jig, VERSION, writer.finish())
        }
        Readable::Family { scope, window, record: value } => {
            let owner = w::Family { number: value.family };
            (w::Owner::Family(owner), value.version, super::envelope::record(scope, window, value, limits)?)
        }
    };
    Ok(w::Owned { owner, version, record })
}
#[must_use]
pub fn none(value: &w::ReadNone) -> c::ReadNone {
    client::read_none_to_domain(value.clone())
}

/// Decodes a whole discriminated document against the original client limits.
pub fn value(bytes: &[u8], limits: &Limits) -> Result<c::ReadValue, Undecodable> {
    if len_u64(bytes.len()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let record = match w::ReadValue::decode(limits.receiving(), &mut Reader::new(bytes)) {
        Ok(record) => record,
        Err(problem) => return Err(super::classify(problem)),
    };
    Ok(match record {
        w::ReadValue::Settings(v) => {
            c::ReadValue::Settings(Box::new(match client::settings_document_to_domain(v, limits) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
        w::ReadValue::Setting(v) => {
            c::ReadValue::Setting(Box::new(match super::settings::restore_setting_document(&v, limits) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
        w::ReadValue::Revision(v) => {
            c::ReadValue::Revision(Box::new(match client::read_revision_answer_to_domain(v, limits) {
                Ok(value) => value.value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
        w::ReadValue::Revisions(v) => c::ReadValue::Revisions(Box::new(client::revisions_page_to_domain(v))),
        w::ReadValue::Project(record) => c::ReadValue::Project(Box::new(documents::project(record, limits)?)),
        w::ReadValue::Ended(record) => c::ReadValue::Ended(Box::new(objects::ended(*record, limits)?)),
        w::ReadValue::Tasks(record) => c::ReadValue::Tasks(Box::new(pages::task_page(record, limits)?)),
        w::ReadValue::Turn(record) => c::ReadValue::Turn(Box::new(super::turn::archive(record, limits)?)),
        w::ReadValue::TaskText(record) => c::ReadValue::TaskText(Box::new(task_text(record, limits)?)),
        w::ReadValue::Me(record) => c::ReadValue::Me(Box::new(documents::frame(record, limits)?)),
        w::ReadValue::Task(record) => c::ReadValue::Task(Box::new(documents::task(record, limits)?)),
        w::ReadValue::Proposal(record) => c::ReadValue::Proposal(Box::new(objects::proposal(record, limits)?)),
        w::ReadValue::Escalation(record) => c::ReadValue::Escalation(Box::new(objects::escalation(record, limits)?)),
        w::ReadValue::Question(record) => c::ReadValue::Question(Box::new(objects::question(record, limits)?)),
        w::ReadValue::PersonTask(record) => c::ReadValue::PersonTask(Box::new(objects::person_task(record, limits)?)),
        w::ReadValue::Result(record) => c::ReadValue::Result(Box::new(objects::result(record, limits)?)),
        w::ReadValue::EndedPage(record) => c::ReadValue::EndedPage(Box::new(pages::ended_page(record, limits)?)),
        w::ReadValue::History(record) => c::ReadValue::History(Box::new(pages::history_page(record, limits)?)),
        w::ReadValue::Activity(record) => c::ReadValue::Activity(Box::new(pages::activity_page(record, limits)?)),
        w::ReadValue::Inbox(record) => c::ReadValue::Inbox(Box::new(pages::inbox_page(record, limits)?)),
        w::ReadValue::Spend(record) => c::ReadValue::Spend(Box::new(pages::spend_page(record, limits)?)),
        w::ReadValue::Notes(record) => c::ReadValue::Notes(Box::new(pages::note_page(record, limits)?)),
        w::ReadValue::Deployment(record) => c::ReadValue::Deployment(Box::new(documents::deployment(record, limits)?)),
    })
}

fn task_text(value: w::TaskText, limits: &Limits) -> Result<c::TaskText, Undecodable> {
    let Ok(text) = c::Text::new(value.words, limits.client().text) else { return Err(Undecodable::TooLarge) };
    let end = value.from.checked_add(len_u64(text.bytes().len())).ok_or(Undecodable::Malformed)?;
    let next = if end < value.total { Some(end) } else { None };
    if end > value.total || value.next != next || (value.next.is_some() && text.bytes().is_empty()) {
        return Err(Undecodable::Malformed);
    }
    Ok(c::TaskText {
        task: value.task,
        field: client::task_text_field_to_domain(value.field),
        from: value.from,
        total: value.total,
        next: value.next,
        text,
    })
}
