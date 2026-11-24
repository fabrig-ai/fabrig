//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1).
use super::{Limits, List, Reader, c, too_large, w};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn task_link_fields(value: &w::TaskLink, limits: &Limits) -> Result<c::TaskLink, w::Problem> {
    client::task_link_to_domain(value.clone(), limits)
}

pub(crate) fn task_link_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::TaskLink>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::TaskLink::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = task_link_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn subscriptions_fields(value: &w::Subscriptions, limits: &Limits) -> Result<c::Subscriptions, w::Problem> {
    if value.entries.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.entries.len());
    for item in &value.entries {
        list0.push(client::subscription_to_domain(item.clone())).expect("reserved supplied list");
    }
    Ok(c::Subscriptions { entries: list0 })
}

pub(crate) fn task_resource_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ResourceLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::ResourceLine::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = client::resource_line_to_domain(row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn live_delegate_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::LiveDelegate>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::LiveDelegate::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = client::live_delegate_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
