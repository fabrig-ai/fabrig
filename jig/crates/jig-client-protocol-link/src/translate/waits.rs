//! Read the engine's structured waits as supplied (jig/docs/design/client/domain/contract/documents.md, 1).
use super::{Limits, documents, w};
use crate::generated::client;
use jig_client_domain::contract as c;
use skein_lib::List;

pub(super) fn page(value: w::WaitPage, limits: &Limits) -> Result<c::WaitPage, w::Problem> {
    if value.rows.is_empty() && value.after.is_some() {
        return Err(w::paths::WAIT_PAGE_AFTER.problem(skein_codec::Reason::Malformed));
    }
    let mut rows = List::with_capacity(value.rows.len());
    for row in value.rows.into_boxed() {
        let chip = documents::chip_fields(&row.chip, limits)?;
        let wait = client::wait_to_domain(row.wait, limits)?;
        if chip.task != wait.task {
            return Err(w::paths::WAIT_ROW_WAIT.problem(skein_codec::Reason::Malformed));
        }
        rows.push(c::WaitRow { chip, wait }).expect("supplied rows fit");
    }
    let after = match value.after {
        Some(cursor) => Some(documents::cursor_fields(&cursor, limits)?),
        None => None,
    };
    Ok(c::WaitPage { now: skein_lib::Wall::from_nanos(value.now), rows, after })
}

pub(super) fn first(value: Option<w::Wait>, limits: &Limits) -> Result<Option<c::Wait>, w::Problem> {
    match value {
        Some(value) => Ok(Some(client::wait_to_domain(value, limits)?)),
        None => Ok(None),
    }
}

pub(super) fn nearest(value: Option<w::Nearest>) -> Option<c::Nearest> {
    Some(client::nearest_to_domain(value?))
}

pub(super) fn list(value: List<w::Wait>, limits: &Limits) -> Result<List<c::Wait>, w::Problem> {
    let mut waits = List::with_capacity(value.len());
    for row in value.into_boxed() {
        waits.push(client::wait_to_domain(row, limits)?).expect("supplied waits fit");
    }
    Ok(waits)
}
