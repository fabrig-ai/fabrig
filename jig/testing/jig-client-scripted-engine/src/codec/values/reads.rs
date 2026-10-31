//! Wire projections of reads facts (client/domain/testing.md, 4).
use super::super::{bytes, c, pieces, w};
use super::{
    wire_archived_turn, wire_decoded_activity_line, wire_deployment, wire_ended, wire_ended_page, wire_escalation,
    wire_history_row, wire_inbox_page, wire_me, wire_note, wire_person_task, wire_project, wire_proposal,
    wire_question, wire_result_document, wire_revision_answer, wire_revisions_page, wire_setting_document,
    wire_settings_document, wire_spend_page, wire_task, wire_task_page,
};
pub(in crate::codec) fn wire_read_value(value: &c::ReadValue, limits: &w::Limits) -> Result<w::ReadValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ReadValue::Settings(value) => w::ReadValue::Settings(wire_settings_document(value, limits)?),
        c::ReadValue::Setting(value) => w::ReadValue::Setting(wire_setting_document(value, limits)?),
        c::ReadValue::Revision(value) => w::ReadValue::Revision(w::ReadRevisionAnswer::new(
            limits,
            w::ReadRevisionAnswerParts { value: wire_revision_answer(value, limits)? },
        )?),
        c::ReadValue::Revisions(value) => w::ReadValue::Revisions(wire_revisions_page(value, limits)?),
        c::ReadValue::Tasks(value) => w::ReadValue::Tasks(wire_task_page(value, limits)?),
        c::ReadValue::Project(value) => w::ReadValue::Project(wire_project(value, limits)?),
        c::ReadValue::Ended(value) => w::ReadValue::Ended(Box::new(wire_ended(value, limits)?)),
        c::ReadValue::Me(value) => w::ReadValue::Me(wire_me(value, limits)?),
        c::ReadValue::Turn(value) => w::ReadValue::Turn(wire_archived_turn(value, limits)?),
        c::ReadValue::Task(value) => w::ReadValue::Task(wire_task(value, limits)?),
        c::ReadValue::Proposal(value) => w::ReadValue::Proposal(wire_proposal(value, limits)?),
        c::ReadValue::Escalation(value) => w::ReadValue::Escalation(wire_escalation(value, limits)?),
        c::ReadValue::Question(value) => w::ReadValue::Question(wire_question(value, limits)?),
        c::ReadValue::PersonTask(value) => w::ReadValue::PersonTask(wire_person_task(value, limits)?),
        c::ReadValue::Result(value) => w::ReadValue::Result(wire_result_document(value, limits)?),
        c::ReadValue::EndedPage(value) => w::ReadValue::EndedPage(wire_ended_page(value, limits)?),
        c::ReadValue::History(value) => w::ReadValue::History(wire_history_page(value, limits)?),
        c::ReadValue::Activity(value) => w::ReadValue::Activity(wire_activity_page(value, limits)?),
        c::ReadValue::Inbox(value) => w::ReadValue::Inbox(wire_inbox_page(value, limits)?),
        c::ReadValue::Spend(value) => w::ReadValue::Spend(wire_spend_page(value, limits)?),
        c::ReadValue::Notes(value) => w::ReadValue::Notes(wire_note_page(value, limits)?),
        c::ReadValue::Deployment(value) => w::ReadValue::Deployment(wire_deployment(value, limits)?),
    })
}

pub(in crate::codec) fn wire_cursor(value: &c::Cursor, limits: &w::Limits) -> Result<w::Cursor, w::Problem> {
    w::Cursor::new(limits, w::CursorParts { value: bytes::copy_of(value.value.bytes()) })
}

pub(in crate::codec) fn wire_history_page(
    value: &c::HistoryPage,
    limits: &w::Limits,
) -> Result<w::HistoryPage, w::Problem> {
    w::HistoryPage::new(
        limits,
        w::HistoryPageParts {
            cut: value.cut,
            rows: pieces(
                value.rows.iter().map(|item| wire_history_row(item, limits)),
                w::HistoryRow::measure,
                w::HistoryRow::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_activity_page(
    value: &c::ActivityPage,
    limits: &w::Limits,
) -> Result<w::ActivityPage, w::Problem> {
    w::ActivityPage::new(
        limits,
        w::ActivityPageParts {
            cut: value.cut,
            rows: pieces(
                value.rows.iter().map(|item| wire_decoded_activity_line(item, limits)),
                w::DecodedActivityLine::measure,
                w::DecodedActivityLine::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_note_page(value: &c::NotePage, limits: &w::Limits) -> Result<w::NotePage, w::Problem> {
    w::NotePage::new(
        limits,
        w::NotePageParts {
            cut: value.cut,
            rows: pieces(value.rows.iter().map(|item| wire_note(item, limits)), w::Note::measure, w::Note::encode)?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}
