//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{Box, Limits, List, Reader, Wall, c, cursor_fields, task_link_fields, too_large, w};

pub(crate) fn inbox_object_fields(value: &w::InboxObject) -> c::InboxObject {
    match value {
        w::InboxObject::Question(row) => c::InboxObject::Question { message: row.message() },
        w::InboxObject::Proposal(row) => c::InboxObject::Proposal {
            proposer: crate::translate::answers::proposer(row.proposer()),
            number: row.number(),
        },
        w::InboxObject::Escalation(row) => c::InboxObject::Escalation { revision: row.revision() },
        w::InboxObject::PersonTask => c::InboxObject::PersonTask,
        w::InboxObject::Turn(row) => c::InboxObject::Reply { message: row.message() },
        w::InboxObject::Result(row) => c::InboxObject::Result { position: row.position() },
    }
}

pub(crate) fn inbox_cursor_fields(value: &w::InboxCursor) -> c::InboxCursor {
    match value {
        w::InboxCursor::Number(row) => c::InboxCursor::Number { number: row.number() },
        w::InboxCursor::Inbox(row) => {
            let opt0 = match row.at() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::InboxCursor::Inbox { task: row.task(), object: inbox_object_fields(row.object()), at: opt0 }
        }
    }
}

pub(crate) fn inbox_selection_fields(
    value: &w::InboxSelection,
    limits: &Limits,
) -> Result<c::InboxSelection, w::Problem> {
    if value.goals().len() > limits.wire().inbox_selection_goals {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.goals().len());
    for item in value.goals() {
        list0.push(*item).expect("reserved supplied list");
    }
    Ok(c::InboxSelection { cursor: inbox_cursor_fields(value.cursor()), goals: list0 })
}

pub(crate) fn inbox_selection_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::InboxSelection>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::InboxSelection::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = inbox_selection_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn inbox_entry_fields(value: &w::InboxEntry, limits: &Limits) -> Result<c::InboxEntry, w::Problem> {
    let opt0 = match value.reply() {
        Some(item) => Some(Box::new(crate::translate::queries::line(item, limits)?)),
        None => None,
    };
    let opt1 = match value.project() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match value.activity() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt4 = match value.unread() {
        Some(item) => Some(*item),
        None => None,
    };
    Ok(c::InboxEntry {
        projects: value.projects().clone(),
        reply: opt0,
        task: task_link_fields(value.task(), limits)?,
        project: opt1,
        object: inbox_object_fields(value.object()),
        activity: opt2,
        at: opt3,
        unread: opt4,
        cursor: inbox_cursor_fields(value.cursor()),
    })
}

pub(crate) fn inbox_entry_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::InboxEntry>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::InboxEntry::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = inbox_entry_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn inbox_page_fields(value: &w::InboxPage, limits: &Limits) -> Result<c::InboxPage, w::Problem> {
    let opt0 = match value.selection() {
        Some(item) => Some(inbox_selection_piece(item, limits, limits.client().rows)?),
        None => None,
    };
    let opt1 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt2 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::InboxPage {
        selection: opt0,
        cut: value.cut(),
        rows: inbox_entry_piece(value.rows(), limits, limits.client().rows)?,
        before: opt1,
        after: opt2,
    })
}
