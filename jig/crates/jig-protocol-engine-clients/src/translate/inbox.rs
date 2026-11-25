//! Whole live document facts (protocol/client/wire.md, 1, and protocol/client/README.md, 5).
use super::{live, pieces, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};
#[must_use]
pub fn object(value: d::InboxObject) -> w::InboxObject {
    mapped::inbox_object_to_record(&value)
}
#[must_use]
pub fn cursor(value: d::Cursor) -> w::InboxCursor {
    mapped::inbox_cursor_to_record(&value)
}
pub fn entry(value: &d::InboxEntry, limits: &w::Limits) -> Result<w::InboxEntry, w::Problem> {
    mapped::inbox_entry_to_record(value, &(&[], limits))
}
pub fn selection(
    value: Option<&List<d::InboxSelection>>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Option<Box<[u8]>>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    let mut rows = List::with_capacity(value.len());
    for row in value {
        rows.push(mapped::inbox_selection_to_record(row)).expect("supplied selections fit");
    }
    Ok(Some(selection_pieces(&rows, bound, path, limits)?))
}
pub fn document(value: &d::Inbox, limits: &w::Limits) -> Result<w::Inbox, w::Problem> {
    let mut needs = List::with_capacity(value.needs.len());
    for row in &value.needs {
        needs.push(entry(row, limits)?).expect("supplied rows fit");
    }
    let needs = entry_pieces(&needs, limits.inbox_needs, w::paths::INBOX_NEEDS, limits)?;
    let mut updates = List::with_capacity(value.updates.len());
    for row in &value.updates {
        updates.push(entry(row, limits)?).expect("supplied rows fit");
    }
    let updates = entry_pieces(&updates, limits.inbox_updates, w::paths::INBOX_UPDATES, limits)?;

    Ok(w::Inbox {
        party: value.party,
        needs,
        updates,
        next: match value.next {
            Some(item) => Some(cursor(item)),
            None => None,
        },
        cached_needs: value.cached_needs,
        cached_updates: live::count(value.cached_updates),
        results_seen: value.results_seen,
        updates_time_complete: value.updates_time_complete,
        selection: selection(value.selection.as_ref(), limits, limits.inbox_selection_some, w::paths::INBOX_SELECTION)?,
    })
}
fn entry_pieces(
    rows: &List<w::InboxEntry>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn selection_pieces(
    rows: &List<w::InboxSelection>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
