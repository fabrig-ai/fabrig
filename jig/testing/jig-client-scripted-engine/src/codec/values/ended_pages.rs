//! Wire projections of ended pages facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, pieces, w};
use super::{wire_cursor, wire_funder, wire_numbers};
pub(in crate::codec) fn wire_ended_page(value: &c::EndedPage, limits: &w::Limits) -> Result<w::EndedPage, w::Problem> {
    Ok(w::EndedPage {
            rows: pieces(value.rows.iter().map(|item| wire_ended_delegate(item, limits)), limits)?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            cut: value.cut,
        })
}

pub(in crate::codec) fn wire_ended_delegate(
    value: &c::EndedDelegate,
    limits: &w::Limits,
) -> Result<w::EndedDelegate, w::Problem> {
    Ok(w::EndedDelegate {
            chip: facts::chip(&value.chip, limits)?,
            ended: facts::chip_ending(&value.ended, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        })
}

pub(in crate::codec) fn wire_spend_page(value: &c::SpendPage, limits: &w::Limits) -> Result<w::SpendPage, w::Problem> {
    Ok(w::SpendPage {
            rows: list(value.rows.iter().map(|item| wire_funding_record(item, limits)))?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            cut: value.cut,
        })
}

pub(in crate::codec) fn wire_funding_record(
    value: &c::FundingRecord,
    limits: &w::Limits,
) -> Result<w::FundingRecord, w::Problem> {
    Ok(w::FundingRecord {
            period: value.period.as_ref().map(|item| wire_period_times(item, limits)).transpose()?,
            funder: wire_funder(&value.funder, limits)?,
            parent: value.parent.as_ref().map(|item| wire_funder(item, limits)).transpose()?,
            numbers: wire_numbers(&value.numbers, limits)?,
            made: value.made,
            closed: value.closed,
        })
}

pub(in crate::codec) fn wire_period_times(
    value: &c::PeriodTimes,
    _limits: &w::Limits,
) -> Result<w::PeriodTimes, w::Problem> {
    Ok(w::PeriodTimes {
            began: value.began.as_nanos(),
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        })
}
