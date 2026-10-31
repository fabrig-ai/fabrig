//! Wire projections of owner inbox facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{wire_count, wire_cursor, wire_proposer, wire_task_line, wire_task_link};
pub(in crate::codec) fn wire_inbox_page(value: &c::InboxPage, limits: &w::Limits) -> Result<w::InboxPage, w::Problem> {
    w::InboxPage::new(
        limits,
        w::InboxPageParts {
            selection: value
                .selection
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_inbox_selection(item, limits)),
                        w::InboxSelection::measure,
                        w::InboxSelection::encode,
                    )
                })
                .transpose()?,
            cut: value.cut,
            rows: pieces(
                value.rows.iter().map(|item| wire_inbox_entry(item, limits)),
                w::InboxEntry::measure,
                w::InboxEntry::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_inbox_selection(
    value: &c::InboxSelection,
    limits: &w::Limits,
) -> Result<w::InboxSelection, w::Problem> {
    w::InboxSelection::new(
        limits,
        w::InboxSelectionParts {
            cursor: wire_inbox_cursor(&value.cursor, limits)?,
            goals: list(value.goals.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
        },
    )
}

pub(in crate::codec) fn wire_inbox_cursor(
    value: &c::InboxCursor,
    limits: &w::Limits,
) -> Result<w::InboxCursor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::InboxCursor::Number { number } => {
            w::InboxCursor::Number(w::InboxCursorNumber::new(limits, w::InboxCursorNumberParts { number: *(number) })?)
        }
        c::InboxCursor::Inbox { task, object, at } => w::InboxCursor::Inbox(w::InboxCursorInbox::new(
            limits,
            w::InboxCursorInboxParts {
                task: *(task),
                object: wire_inbox_object(object, limits)?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_inbox_object(
    value: &c::InboxObject,
    limits: &w::Limits,
) -> Result<w::InboxObject, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::InboxObject::Question { message } => w::InboxObject::Question(w::InboxObjectQuestion::new(
            limits,
            w::InboxObjectQuestionParts { message: *(message) },
        )?),
        c::InboxObject::Proposal { proposer, number } => w::InboxObject::Proposal(w::InboxObjectProposal::new(
            limits,
            w::InboxObjectProposalParts { proposer: wire_proposer(proposer, limits)?, number: *(number) },
        )?),
        c::InboxObject::Escalation { revision } => w::InboxObject::Escalation(w::InboxObjectEscalation::new(
            limits,
            w::InboxObjectEscalationParts { revision: *(revision) },
        )?),
        c::InboxObject::PersonTask => w::InboxObject::PersonTask,
        c::InboxObject::Reply { message } => {
            w::InboxObject::Turn(w::InboxObjectTurn::new(limits, w::InboxObjectTurnParts { message: *(message) })?)
        }
        c::InboxObject::Result { position } => w::InboxObject::Result(w::InboxObjectResult::new(
            limits,
            w::InboxObjectResultParts { position: *(position) },
        )?),
    })
}

pub(in crate::codec) fn wire_inbox_entry(
    value: &c::InboxEntry,
    limits: &w::Limits,
) -> Result<w::InboxEntry, w::Problem> {
    w::InboxEntry::new(
        limits,
        w::InboxEntryParts {
            reply: value.reply.as_ref().map(|item| wire_task_line(item, limits)).transpose()?,
            task: wire_task_link(&value.task, limits)?,
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            object: wire_inbox_object(&value.object, limits)?,
            activity: value.activity.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            unread: value.unread.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            cursor: wire_inbox_cursor(&value.cursor, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_inbox(value: &c::Inbox, limits: &w::Limits) -> Result<w::Inbox, w::Problem> {
    w::Inbox::new(
        limits,
        w::InboxParts {
            party: value.party,
            needs: pieces(
                value.needs.iter().map(|item| wire_inbox_entry(item, limits)),
                w::InboxEntry::measure,
                w::InboxEntry::encode,
            )?,
            updates: pieces(
                value.updates.iter().map(|item| wire_inbox_entry(item, limits)),
                w::InboxEntry::measure,
                w::InboxEntry::encode,
            )?,
            next: value.next.as_ref().map(|item| wire_inbox_cursor(item, limits)).transpose()?,
            cached_needs: value.cached_needs,
            cached_updates: wire_count(&value.cached_updates, limits)?,
            results_seen: value.results_seen,
            updates_time_complete: value.updates_time_complete,
            selection: value
                .selection
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_inbox_selection(item, limits)),
                        w::InboxSelection::measure,
                        w::InboxSelection::encode,
                    )
                })
                .transpose()?,
        },
    )
}
