//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{Limits, List, Wall, c, cursor_fields, too_large, w};

pub(crate) fn period_times_fields(value: &w::PeriodTimes) -> c::PeriodTimes {
    let opt0 = match value.next() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    c::PeriodTimes { began: Wall::from_nanos(value.began()), next: opt0 }
}

pub(crate) fn funding_record_fields(value: &w::FundingRecord) -> c::FundingRecord {
    let opt0 = match value.period() {
        Some(item) => Some(period_times_fields(item)),
        None => None,
    };
    let opt1 = match value.parent() {
        Some(item) => Some(crate::translate::facts::funder(item)),
        None => None,
    };
    c::FundingRecord {
        period: opt0,
        funder: crate::translate::facts::funder(value.funder()),
        parent: opt1,
        numbers: crate::translate::facts::numbers(value.numbers()),
        made: value.made(),
        closed: value.closed(),
    }
}

pub(crate) fn spend_page_fields(value: &w::SpendPage, limits: &Limits) -> Result<c::SpendPage, w::Problem> {
    if value.rows().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.rows().len());
    for item in value.rows() {
        list0.push(funding_record_fields(item)).expect("reserved supplied list");
    }
    let opt1 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt2 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::SpendPage { rows: list0, before: opt1, after: opt2, cut: value.cut() })
}
