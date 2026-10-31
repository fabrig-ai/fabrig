//! Wire projections of owner budget facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::wire_funder;
pub(in crate::codec) fn wire_budget(value: &c::Budget, limits: &w::Limits) -> Result<w::Budget, w::Problem> {
    w::Budget::new(
        limits,
        w::BudgetParts {
            numbers: wire_numbers(&value.numbers, limits)?,
            left: value.left,
            funder: wire_funder(&value.funder, limits)?,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_numbers(value: &c::Numbers, limits: &w::Limits) -> Result<w::Numbers, w::Problem> {
    w::Numbers::new(
        limits,
        w::NumbersParts {
            budget: value.budget,
            spent: value.spent,
            spent_below: value.spent_below,
            reserved: value.reserved,
        },
    )
}

pub(in crate::codec) fn wire_pool_ledger(
    value: &c::PoolLedger,
    limits: &w::Limits,
) -> Result<w::PoolLedger, w::Problem> {
    w::PoolLedger::new(
        limits,
        w::PoolLedgerParts {
            source: wire_funder(&value.source, limits)?,
            parent: value.parent.as_ref().map(|item| wire_funder(item, limits)).transpose()?,
            numbers: wire_numbers(&value.numbers, limits)?,
            left: value.left,
            closed: value.closed,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}
