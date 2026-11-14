//! One read per exchange with opaque cursor bytes (protocol/client/README.md, 4).
use super::asks::{wire_note_scope, wire_proposal_key};
use super::{Undecodable, documents, objects, pages};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::Readable;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::VERSION;
use jig_client_protocol_wire::v22 as w;
use skein_lib::{Reader, Writer, bytes, length::len_u64};

pub fn read(read: &c::Read, limits: &w::Limits) -> Result<w::Read, w::Problem> {
    Ok(match read {
        c::Read::Settings => w::Read::Settings,
        c::Read::Setting(value) => w::Read::Setting(super::settings::wire_read_setting(value, limits)?),
        c::Read::Revision(value) => w::Read::Revision(super::settings::wire_read_revision(value, limits)?),
        c::Read::Revisions(value) => w::Read::Revisions(super::settings::wire_read_revisions(value, limits)?),
        c::Read::Tasks { query, live, ended } => w::Read::Tasks(w::ReadTasks::new(
            limits,
            w::ReadTasksParts {
                query: super::queries::query(*query, limits)?,
                live: cursor(live.as_ref(), limits)?,
                ended: cursor(ended.as_ref(), limits)?,
            },
        )?),
        c::Read::Turn { task, attempt, turn } => {
            w::Read::Turn(w::ReadTurn::new(limits, w::ReadTurnParts { task: *task, attempt: *attempt, turn: *turn })?)
        }
        c::Read::TaskText { task, field, from } => w::Read::TaskText(w::ReadTaskText::new(
            limits,
            w::ReadTaskTextParts {
                task: *task,
                field: match field {
                    c::TaskTextField::Specification => w::TaskTextField::Specification,
                    c::TaskTextField::Instructions => w::TaskTextField::Instructions,
                    c::TaskTextField::Result => w::TaskTextField::Result,
                },
                from: *from,
            },
        )?),
        c::Read::ActivityBeside { task, position, side } => w::Read::ActivityBeside(w::ReadActivityBeside::new(
            limits,
            w::ReadActivityBesideParts {
                task: *task,
                position: *position,
                side: match side {
                    c::Side::Earlier => w::ActivitySide::Earlier,
                    c::Side::Later => w::ActivitySide::Later,
                },
            },
        )?),
        c::Read::Me => w::Read::Me,
        c::Read::Task { task } => w::Read::Task(w::ReadTask::new(limits, w::ReadTaskParts { task: *task })?),
        c::Read::Proposal { proposal } => w::Read::Proposal(w::ReadProposal::new(
            limits,
            w::ReadProposalParts { proposal: wire_proposal_key(proposal, limits)? },
        )?),
        c::Read::Escalation { task } => {
            w::Read::Escalation(w::ReadEscalation::new(limits, w::ReadEscalationParts { task: *task })?)
        }
        c::Read::Question { task, message } => {
            w::Read::Question(w::ReadQuestion::new(limits, w::ReadQuestionParts { task: *task, message: *message })?)
        }
        c::Read::PersonTask { task } => {
            w::Read::PersonTask(w::ReadPersonTask::new(limits, w::ReadPersonTaskParts { task: *task })?)
        }
        c::Read::Result { task } => w::Read::Result(w::ReadResult::new(limits, w::ReadResultParts { task: *task })?),
        c::Read::Ended { task, after } => w::Read::Ended(w::ReadEnded::new(
            limits,
            w::ReadEndedParts { task: *task, after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::History { task, subtree, after } => w::Read::History(w::ReadHistory::new(
            limits,
            w::ReadHistoryParts { task: *task, subtree: *subtree, after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::Activity { task, before } => w::Read::Activity(w::ReadActivity::new(
            limits,
            w::ReadActivityParts { task: *task, before: cursor(before.as_ref(), limits)? },
        )?),
        c::Read::Inbox { group, project, kind, goal, after } => w::Read::Inbox(w::ReadInbox::new(
            limits,
            w::ReadInboxParts {
                group: inbox_group(*group),
                project: *project,
                kind: inbox_kind(*kind),
                goal: *goal,
                after: cursor(after.as_ref(), limits)?,
            },
        )?),
        c::Read::Spend { project, after } => w::Read::Spend(w::ReadSpend::new(
            limits,
            w::ReadSpendParts { project: *project, after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::Project { project } => {
            w::Read::Project(w::ReadProject::new(limits, w::ReadProjectParts { project: *project })?)
        }
        c::Read::Notes { scope, description, after } => {
            let description = match description {
                Some(value) => Some(bytes::copy_of(value.bytes())),
                None => None,
            };
            w::Read::Notes(w::ReadNotes::new(
                limits,
                w::ReadNotesParts {
                    scope: wire_note_scope(scope, limits)?,
                    description,
                    after: cursor(after.as_ref(), limits)?,
                },
            )?)
        }
        c::Read::Deployment => w::Read::Deployment,
    })
}
fn cursor(value: Option<&c::Cursor>, limits: &w::Limits) -> Result<Option<w::Cursor>, w::Problem> {
    match value {
        Some(value) => Ok(Some(w::Cursor::new(limits, w::CursorParts { value: bytes::copy_of(value.value.bytes()) })?)),
        None => Ok(None),
    }
}
fn inbox_group(value: c::InboxReadGroup) -> w::InboxReadGroup {
    match value {
        c::InboxReadGroup::Needs => w::InboxReadGroup::Needs,
        c::InboxReadGroup::Updates => w::InboxReadGroup::Updates,
    }
}
fn inbox_kind(value: Option<c::InboxGroup>) -> Option<w::InboxGroup> {
    match value {
        Some(value) => Some(match value {
            c::InboxGroup::Questions => w::InboxGroup::Questions,
            c::InboxGroup::Proposals => w::InboxGroup::Proposals,
            c::InboxGroup::Escalations => w::InboxGroup::Escalations,
            c::InboxGroup::PersonTasks => w::InboxGroup::PersonTasks,
            c::InboxGroup::Results => w::InboxGroup::Results,
            c::InboxGroup::Replies => w::InboxGroup::Replies,
        }),
        None => None,
    }
}

/// The read exchange's owner envelope; family records remain opaque.
pub fn owned(value: Readable, limits: &w::Limits) -> Result<w::Owned, w::Problem> {
    let (owner, version, record) = match value {
        Readable::Jig(value) => {
            let value = read(&value, limits)?;
            let mut writer = Writer::new(skein_lib::length::usize_of(value.measure()));
            value.encode(&mut writer).expect("measured read");
            (w::Owner::Jig, VERSION, writer.finish())
        }
        Readable::Family { scope, window, record: value } => {
            let owner = w::Family::new(limits, w::FamilyParts { number: value.family })?;
            (w::Owner::Family(owner), value.version, super::envelope::record(scope, window, value, limits)?)
        }
    };
    w::Owned::new(limits, w::OwnedParts { owner, version, record })
}
#[must_use]
pub fn none(value: &w::ReadNone) -> c::ReadNone {
    match value {
        w::ReadNone::TooLarge(value) => c::ReadNone::TooLarge { bytes: value.bytes() },
        w::ReadNone::TooLinked(value) => c::ReadNone::TooLinked { joins: value.joins() },
        w::ReadNone::Malformed => c::ReadNone::Malformed,
        w::ReadNone::Unknown => c::ReadNone::Unknown,
        w::ReadNone::Refused => c::ReadNone::Refused,
        w::ReadNone::Gone => c::ReadNone::Gone,
        w::ReadNone::Busy => c::ReadNone::Busy,
        w::ReadNone::SignedOut => c::ReadNone::SignedOut,
    }
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
            c::ReadValue::Settings(Box::new(match super::settings::restore_settings_document(&v, limits) {
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
            c::ReadValue::Revision(Box::new(match super::settings::restore_revision_answer(v.value(), limits) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
        w::ReadValue::Revisions(v) => {
            c::ReadValue::Revisions(Box::new(match super::settings::restore_revisions_page(&v) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
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
    let value = value.into_parts();
    let Ok(text) = c::Text::new(value.words, limits.client().text) else { return Err(Undecodable::TooLarge) };
    let end = value.from.checked_add(len_u64(text.bytes().len())).ok_or(Undecodable::Malformed)?;
    let next = if end < value.total { Some(end) } else { None };
    if end > value.total || value.next != next || (value.next.is_some() && text.bytes().is_empty()) {
        return Err(Undecodable::Malformed);
    }
    Ok(c::TaskText {
        task: value.task,
        field: match value.field {
            w::TaskTextField::Specification => c::TaskTextField::Specification,
            w::TaskTextField::Instructions => c::TaskTextField::Instructions,
            w::TaskTextField::Result => c::TaskTextField::Result,
        },
        from: value.from,
        total: value.total,
        next: value.next,
        text,
    })
}
