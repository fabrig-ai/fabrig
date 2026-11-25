//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{Limits, List, c, cursor_fields, too_large, w};
use crate::generated::client;

pub(crate) fn spend_page_fields(value: &w::SpendPage, limits: &Limits) -> Result<c::SpendPage, w::Problem> {
    if value.rows.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.rows.len());
    for item in &value.rows {
        list0.push(client::funding_record_to_domain(item.clone())).expect("reserved supplied list");
    }
    let opt1 = match &value.before {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt2 = match &value.after {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::SpendPage { rows: list0, before: opt1, after: opt2, cut: value.cut })
}
