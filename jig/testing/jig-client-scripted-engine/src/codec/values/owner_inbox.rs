//! Wire projections of owner inbox facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{wire_count, wire_cursor, wire_owner_task_link, wire_proposer, wire_task_line};
pub(in crate::codec) fn wire_owner_inbox_page(
    value: &c::OwnerInboxPage,
    limits: &w::Limits,
) -> Result<w::OwnerInboxPage, w::Problem> {
    w::OwnerInboxPage::new(
        limits,
        w::OwnerInboxPageParts {
            selection: value
                .selection
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_owner_inbox_selection(item, limits)),
                        w::OwnerInboxSelection::measure,
                        w::OwnerInboxSelection::encode,
                    )
                })
                .transpose()?,
            cut: value.cut,
            rows: pieces(
                value.rows.iter().map(|item| wire_owner_inbox_entry(item, limits)),
                w::OwnerInboxEntry::measure,
                w::OwnerInboxEntry::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_inbox_selection(
    value: &c::OwnerInboxSelection,
    limits: &w::Limits,
) -> Result<w::OwnerInboxSelection, w::Problem> {
    w::OwnerInboxSelection::new(
        limits,
        w::OwnerInboxSelectionParts {
            cursor: wire_owner_inbox_cursor(&value.cursor, limits)?,
            goals: list(value.goals.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
        },
    )
}

pub(in crate::codec) fn wire_owner_inbox_cursor(
    value: &c::OwnerInboxCursor,
    limits: &w::Limits,
) -> Result<w::OwnerInboxCursor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerInboxCursor::Number { number } => w::OwnerInboxCursor::Number(w::OwnerInboxCursorNumber::new(
            limits,
            w::OwnerInboxCursorNumberParts { number: *(number) },
        )?),
        c::OwnerInboxCursor::Inbox { task, object, at } => w::OwnerInboxCursor::Inbox(w::OwnerInboxCursorInbox::new(
            limits,
            w::OwnerInboxCursorInboxParts {
                task: *(task),
                object: wire_owner_inbox_object(object, limits)?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_inbox_object(
    value: &c::OwnerInboxObject,
    limits: &w::Limits,
) -> Result<w::OwnerInboxObject, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerInboxObject::Question { message } => w::OwnerInboxObject::Question(w::OwnerInboxObjectQuestion::new(
            limits,
            w::OwnerInboxObjectQuestionParts { message: *(message) },
        )?),
        c::OwnerInboxObject::Proposal { proposer, number } => {
            w::OwnerInboxObject::Proposal(w::OwnerInboxObjectProposal::new(
                limits,
                w::OwnerInboxObjectProposalParts { proposer: wire_proposer(proposer, limits)?, number: *(number) },
            )?)
        }
        c::OwnerInboxObject::Escalation { revision } => w::OwnerInboxObject::Escalation(
            w::OwnerInboxObjectEscalation::new(limits, w::OwnerInboxObjectEscalationParts { revision: *(revision) })?,
        ),
        c::OwnerInboxObject::PersonTask => w::OwnerInboxObject::PersonTask,
        c::OwnerInboxObject::Reply { message } => w::OwnerInboxObject::Turn(w::OwnerInboxObjectTurn::new(
            limits,
            w::OwnerInboxObjectTurnParts { message: *(message) },
        )?),
        c::OwnerInboxObject::Result { position } => w::OwnerInboxObject::Result(w::OwnerInboxObjectResult::new(
            limits,
            w::OwnerInboxObjectResultParts { position: *(position) },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_inbox_entry(
    value: &c::OwnerInboxEntry,
    limits: &w::Limits,
) -> Result<w::OwnerInboxEntry, w::Problem> {
    w::OwnerInboxEntry::new(
        limits,
        w::OwnerInboxEntryParts {
            reply: value.reply.as_ref().map(|item| wire_task_line(item, limits)).transpose()?,
            task: wire_owner_task_link(&value.task, limits)?,
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            object: wire_owner_inbox_object(&value.object, limits)?,
            activity: value.activity.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            unread: value.unread.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            cursor: wire_owner_inbox_cursor(&value.cursor, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_inbox(
    value: &c::OwnerInbox,
    limits: &w::Limits,
) -> Result<w::OwnerInbox, w::Problem> {
    w::OwnerInbox::new(
        limits,
        w::OwnerInboxParts {
            party: value.party,
            needs: pieces(
                value.needs.iter().map(|item| wire_owner_inbox_entry(item, limits)),
                w::OwnerInboxEntry::measure,
                w::OwnerInboxEntry::encode,
            )?,
            updates: pieces(
                value.updates.iter().map(|item| wire_owner_inbox_entry(item, limits)),
                w::OwnerInboxEntry::measure,
                w::OwnerInboxEntry::encode,
            )?,
            next: value.next.as_ref().map(|item| wire_owner_inbox_cursor(item, limits)).transpose()?,
            cached_needs: value.cached_needs,
            cached_updates: wire_count(&value.cached_updates, limits)?,
            results_seen: value.results_seen,
            updates_time_complete: value.updates_time_complete,
            selection: value
                .selection
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_owner_inbox_selection(item, limits)),
                        w::OwnerInboxSelection::measure,
                        w::OwnerInboxSelection::encode,
                    )
                })
                .transpose()?,
        },
    )
}
