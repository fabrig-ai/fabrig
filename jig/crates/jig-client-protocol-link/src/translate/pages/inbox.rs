//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{Limits, List, Reader, c, cursor_fields, too_large, w};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn inbox_cursor_fields(value: &w::InboxCursor) -> c::InboxCursor {
    client::inbox_cursor_to_domain(value.clone())
}

pub(crate) fn inbox_selection_fields(
    value: &w::InboxSelection,
    limits: &Limits,
) -> Result<c::InboxSelection, w::Problem> {
    if value.goals.len() > limits.wire().inbox_selection_goals {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.goals.len());
    for item in &value.goals {
        list0.push(*item).expect("reserved supplied list");
    }
    Ok(c::InboxSelection { cursor: inbox_cursor_fields(&value.cursor), goals: list0 })
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

pub(crate) fn inbox_entry_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::InboxEntry>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::InboxEntry::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = client::inbox_entry_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn inbox_page_fields(value: &w::InboxPage, limits: &Limits) -> Result<c::InboxPage, w::Problem> {
    let opt0 = match &value.selection {
        Some(item) => Some(inbox_selection_piece(item, limits, limits.client().rows)?),
        None => None,
    };
    let opt1 = match &value.before {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt2 = match &value.after {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::InboxPage {
        selection: opt0,
        cut: value.cut,
        rows: inbox_entry_piece(&value.rows, limits, limits.client().rows)?,
        before: opt1,
        after: opt2,
    })
}
