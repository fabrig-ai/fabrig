//! Wire projections of owner inbox facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, try_pieces, w};
use super::{wire_count, wire_cursor, wire_proposer, wire_task_line, wire_task_link};
pub(in crate::codec) fn wire_inbox_page(value: &c::InboxPage, limits: &w::Limits) -> Result<w::InboxPage, w::Problem> {
    Ok(w::InboxPage {
        selection: value
            .selection
            .as_ref()
            .map(|item| pieces((item).iter().map(wire_inbox_selection), limits))
            .transpose()?,
        cut: value.cut,
        rows: try_pieces(value.rows.iter().map(|item| wire_inbox_entry(item, limits)), limits)?,
        before: value.before.as_ref().map(wire_cursor),
        after: value.after.as_ref().map(wire_cursor),
    })
}

pub(in crate::codec) fn wire_inbox_selection(value: &c::InboxSelection) -> w::InboxSelection {
    w::InboxSelection { cursor: wire_inbox_cursor(&value.cursor), goals: list(value.goals.iter().copied()) }
}

pub(in crate::codec) fn wire_inbox_cursor(value: &c::InboxCursor) -> w::InboxCursor {
    match value {
        c::InboxCursor::Number { number } => w::InboxCursor::Number(w::InboxCursorNumber { number: *(number) }),
        c::InboxCursor::Inbox { task, object, at } => w::InboxCursor::Inbox(w::InboxCursorInbox {
            task: *(task),
            object: wire_inbox_object(object),
            at: (at).as_ref().map(|item| (item).as_nanos()),
        }),
    }
}

pub(in crate::codec) fn wire_inbox_object(value: &c::InboxObject) -> w::InboxObject {
    match value {
        c::InboxObject::Question { message } => {
            w::InboxObject::Question(w::InboxObjectQuestion { message: *(message) })
        }
        c::InboxObject::Proposal { proposer, number } => {
            w::InboxObject::Proposal(w::InboxObjectProposal { proposer: wire_proposer(proposer), number: *(number) })
        }
        c::InboxObject::Escalation { revision } => {
            w::InboxObject::Escalation(w::InboxObjectEscalation { revision: *(revision) })
        }
        c::InboxObject::PersonTask => w::InboxObject::PersonTask,
        c::InboxObject::Reply { message } => w::InboxObject::Turn(w::InboxObjectTurn { message: *(message) }),
        c::InboxObject::Result { position } => w::InboxObject::Result(w::InboxObjectResult { position: *(position) }),
    }
}

pub(in crate::codec) fn wire_inbox_entry(
    value: &c::InboxEntry,
    limits: &w::Limits,
) -> Result<w::InboxEntry, w::Problem> {
    Ok(w::InboxEntry {
        projects: value.projects.clone(),
        reply: value.reply.as_ref().map(|item| wire_task_line(item, limits)).transpose()?,
        task: wire_task_link(&value.task, limits)?,
        project: value.project,
        object: wire_inbox_object(&value.object),
        activity: value.activity,
        at: value.at.as_ref().map(|item| (item).as_nanos()),
        unread: value.unread,
        cursor: wire_inbox_cursor(&value.cursor),
    })
}

pub(in crate::codec) fn wire_inbox(value: &c::Inbox, limits: &w::Limits) -> Result<w::Inbox, w::Problem> {
    Ok(w::Inbox {
        party: value.party,
        needs: try_pieces(value.needs.iter().map(|item| wire_inbox_entry(item, limits)), limits)?,
        updates: try_pieces(value.updates.iter().map(|item| wire_inbox_entry(item, limits)), limits)?,
        next: value.next.as_ref().map(wire_inbox_cursor),
        cached_needs: value.cached_needs,
        cached_updates: wire_count(&value.cached_updates),
        results_seen: value.results_seen,
        updates_time_complete: value.updates_time_complete,
        selection: value
            .selection
            .as_ref()
            .map(|item| pieces((item).iter().map(wire_inbox_selection), limits))
            .transpose()?,
    })
}
