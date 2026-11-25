//! Read the engine's structured waits as supplied (jig/docs/design/client/domain/contract/documents.md, 1).
use super::{Limits, w};
use crate::generated::client;
use jig_client_domain::contract as c;
use skein_lib::List;

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
