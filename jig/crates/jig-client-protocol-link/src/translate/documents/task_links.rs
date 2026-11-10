//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{Limits, List, Reader, Wall, c, chip_fields, too_large, w};

pub(crate) fn task_status_fields(value: &w::TaskStatus) -> c::TaskStatus {
    match value {
        w::TaskStatus::Done => c::TaskStatus::Done,
        w::TaskStatus::Failed => c::TaskStatus::Failed,
        w::TaskStatus::Cancelled => c::TaskStatus::Cancelled,
    }
}

pub(crate) fn task_link_fields(value: &w::TaskLink, limits: &Limits) -> Result<c::TaskLink, w::Problem> {
    Ok(match value {
        w::TaskLink::Live(row) => c::TaskLink::Live(chip_fields(row.value(), limits)?),
        w::TaskLink::Retained(row) => {
            let opt0 = match row.at() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::TaskLink::Retained { chip: chip_fields(row.chip(), limits)?, at: opt0 }
        }
        w::TaskLink::Stored(row) => {
            let opt0 = match row.status() {
                Some(item) => Some(task_status_fields(item)),
                None => None,
            };
            c::TaskLink::Stored { number: row.number(), status: opt0 }
        }
    })
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

pub(crate) fn subscription_kind_fields(value: &w::SubscriptionKind) -> c::SubscriptionKind {
    match value {
        w::SubscriptionKind::Task(row) => {
            c::SubscriptionKind::Task { target: row.target(), held: row.held(), result: row.result() }
        }
        w::SubscriptionKind::Timer(row) => {
            let opt0 = match row.period() {
                Some(item) => Some(*item),
                None => None,
            };
            c::SubscriptionKind::Timer { at: Wall::from_nanos(row.at()), period: opt0 }
        }
        w::SubscriptionKind::Topic(row) => {
            c::SubscriptionKind::Topic { connector: row.connector(), topic: row.topic() }
        }
    }
}

pub(crate) fn subscription_fields(value: &w::Subscription) -> c::Subscription {
    c::Subscription { number: value.number(), kind: subscription_kind_fields(value.kind()) }
}

pub(crate) fn subscriptions_fields(value: &w::Subscriptions, limits: &Limits) -> Result<c::Subscriptions, w::Problem> {
    if value.entries().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.entries().len());
    for item in value.entries() {
        list0.push(subscription_fields(item)).expect("reserved supplied list");
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
        let row = crate::translate::facts::resource_line(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn live_delegate_fields(value: &w::LiveDelegate, limits: &Limits) -> Result<c::LiveDelegate, w::Problem> {
    Ok(c::LiveDelegate { chip: chip_fields(value.chip(), limits)?, takeover_budget: value.takeover_budget() })
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
        let row = live_delegate_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
