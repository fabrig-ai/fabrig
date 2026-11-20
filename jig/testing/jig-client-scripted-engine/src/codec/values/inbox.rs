//! Wire projections of owner inbox facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{wire_count, wire_cursor, wire_proposer, wire_task_line, wire_task_link};
pub(in crate::codec) fn wire_inbox_page(value: &c::InboxPage, limits: &w::Limits) -> Result<w::InboxPage, w::Problem> {
    Ok(w::InboxPage {
        selection: value
            .selection
            .as_ref()
            .map(|item| pieces((item).iter().map(|item| wire_inbox_selection(item, limits)), limits))
            .transpose()?,
        cut: value.cut,
        rows: pieces(value.rows.iter().map(|item| wire_inbox_entry(item, limits)), limits)?,
        before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
    })
}

pub(in crate::codec) fn wire_inbox_selection(
    value: &c::InboxSelection,
    limits: &w::Limits,
) -> Result<w::InboxSelection, w::Problem> {
    Ok(w::InboxSelection {
        cursor: wire_inbox_cursor(&value.cursor, limits)?,
        goals: list(value.goals.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
    })
}

pub(in crate::codec) fn wire_inbox_cursor(
    value: &c::InboxCursor,
    limits: &w::Limits,
) -> Result<w::InboxCursor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::InboxCursor::Number { number } => w::InboxCursor::Number(w::InboxCursorNumber { number: *(number) }),
        c::InboxCursor::Inbox { task, object, at } => w::InboxCursor::Inbox(w::InboxCursorInbox {
            task: *(task),
            object: wire_inbox_object(object, limits)?,
            at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        }),
    })
}

pub(in crate::codec) fn wire_inbox_object(
    value: &c::InboxObject,
    limits: &w::Limits,
) -> Result<w::InboxObject, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::InboxObject::Question { message } => {
            w::InboxObject::Question(w::InboxObjectQuestion { message: *(message) })
        }
        c::InboxObject::Proposal { proposer, number } => w::InboxObject::Proposal(w::InboxObjectProposal {
            proposer: wire_proposer(proposer, limits)?,
            number: *(number),
        }),
        c::InboxObject::Escalation { revision } => {
            w::InboxObject::Escalation(w::InboxObjectEscalation { revision: *(revision) })
        }
        c::InboxObject::PersonTask => w::InboxObject::PersonTask,
        c::InboxObject::Reply { message } => w::InboxObject::Turn(w::InboxObjectTurn { message: *(message) }),
        c::InboxObject::Result { position } => w::InboxObject::Result(w::InboxObjectResult { position: *(position) }),
    })
}

pub(in crate::codec) fn wire_inbox_entry(
    value: &c::InboxEntry,
    limits: &w::Limits,
) -> Result<w::InboxEntry, w::Problem> {
    Ok(w::InboxEntry {
        projects: value.projects.clone(),
        reply: value.reply.as_ref().map(|item| wire_task_line(item, limits)).transpose()?,
        task: wire_task_link(&value.task, limits)?,
        project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        object: wire_inbox_object(&value.object, limits)?,
        activity: value.activity.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        unread: value.unread.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        cursor: wire_inbox_cursor(&value.cursor, limits)?,
    })
}

pub(in crate::codec) fn wire_inbox(value: &c::Inbox, limits: &w::Limits) -> Result<w::Inbox, w::Problem> {
    Ok(w::Inbox {
        party: value.party,
        needs: pieces(value.needs.iter().map(|item| wire_inbox_entry(item, limits)), limits)?,
        updates: pieces(value.updates.iter().map(|item| wire_inbox_entry(item, limits)), limits)?,
        next: value.next.as_ref().map(|item| wire_inbox_cursor(item, limits)).transpose()?,
        cached_needs: value.cached_needs,
        cached_updates: wire_count(&value.cached_updates, limits)?,
        results_seen: value.results_seen,
        updates_time_complete: value.updates_time_complete,
        selection: value
            .selection
            .as_ref()
            .map(|item| pieces((item).iter().map(|item| wire_inbox_selection(item, limits)), limits))
            .transpose()?,
    })
}
