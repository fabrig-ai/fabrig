//! Wire projections of ended pages facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, try_pieces, w};
use super::{wire_cursor, wire_funder, wire_numbers};
pub(in crate::codec) fn wire_ended_page(value: &c::EndedPage, limits: &w::Limits) -> Result<w::EndedPage, w::Problem> {
    Ok(w::EndedPage {
        rows: try_pieces(value.rows.iter().map(|item| wire_ended_delegate(item, limits)), limits)?,
        before: value.before.as_ref().map(wire_cursor),
        after: value.after.as_ref().map(wire_cursor),
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
        at: value.at.as_ref().map(|item| (item).as_nanos()),
    })
}

pub(in crate::codec) fn wire_spend_page(value: &c::SpendPage) -> w::SpendPage {
    w::SpendPage {
        rows: list(value.rows.iter().map(wire_funding_record)),
        before: value.before.as_ref().map(wire_cursor),
        after: value.after.as_ref().map(wire_cursor),
        cut: value.cut,
    }
}

pub(in crate::codec) fn wire_funding_record(value: &c::FundingRecord) -> w::FundingRecord {
    w::FundingRecord {
        period: value.period.as_ref().map(wire_period_times),
        funder: wire_funder(&value.funder),
        parent: value.parent.as_ref().map(wire_funder),
        numbers: wire_numbers(&value.numbers),
        made: value.made,
        closed: value.closed,
    }
}

pub(in crate::codec) fn wire_period_times(value: &c::PeriodTimes) -> w::PeriodTimes {
    w::PeriodTimes { began: value.began.as_nanos(), next: value.next.as_ref().map(|item| (item).as_nanos()) }
}
