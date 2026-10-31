//! Wire projections of historical pages facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, pieces, w};
use super::{wire_cursor, wire_funder, wire_owner_numbers};
pub(in crate::codec) fn wire_owner_ended_page(
    value: &c::OwnerEndedPage,
    limits: &w::Limits,
) -> Result<w::OwnerEndedPage, w::Problem> {
    w::OwnerEndedPage::new(
        limits,
        w::OwnerEndedPageParts {
            rows: pieces(
                value.rows.iter().map(|item| wire_owner_ended_delegate(item, limits)),
                w::OwnerEndedDelegate::measure,
                w::OwnerEndedDelegate::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_owner_ended_delegate(
    value: &c::OwnerEndedDelegate,
    limits: &w::Limits,
) -> Result<w::OwnerEndedDelegate, w::Problem> {
    w::OwnerEndedDelegate::new(
        limits,
        w::OwnerEndedDelegateParts {
            chip: facts::chip(&value.chip, limits)?,
            ended: facts::owner_chip_ending(&value.ended, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_spend_page(
    value: &c::OwnerSpendPage,
    limits: &w::Limits,
) -> Result<w::OwnerSpendPage, w::Problem> {
    w::OwnerSpendPage::new(
        limits,
        w::OwnerSpendPageParts {
            rows: list(value.rows.iter().map(|item| wire_owner_funding_record(item, limits)))?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_owner_funding_record(
    value: &c::OwnerFundingRecord,
    limits: &w::Limits,
) -> Result<w::OwnerFundingRecord, w::Problem> {
    w::OwnerFundingRecord::new(
        limits,
        w::OwnerFundingRecordParts {
            period: value.period.as_ref().map(|item| wire_owner_period_times(item, limits)).transpose()?,
            funder: wire_funder(&value.funder, limits)?,
            parent: value.parent.as_ref().map(|item| wire_funder(item, limits)).transpose()?,
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            made: value.made,
            closed: value.closed,
        },
    )
}

pub(in crate::codec) fn wire_owner_period_times(
    value: &c::OwnerPeriodTimes,
    limits: &w::Limits,
) -> Result<w::OwnerPeriodTimes, w::Problem> {
    w::OwnerPeriodTimes::new(
        limits,
        w::OwnerPeriodTimesParts {
            began: value.began.as_nanos(),
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}
