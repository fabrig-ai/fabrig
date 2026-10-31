//! Wire projections of owner budget facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::wire_funder;
pub(in crate::codec) fn wire_owner_budget(
    value: &c::OwnerBudget,
    limits: &w::Limits,
) -> Result<w::OwnerBudget, w::Problem> {
    w::OwnerBudget::new(
        limits,
        w::OwnerBudgetParts {
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            left: value.left,
            funder: wire_funder(&value.funder, limits)?,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_numbers(
    value: &c::OwnerNumbers,
    limits: &w::Limits,
) -> Result<w::OwnerNumbers, w::Problem> {
    w::OwnerNumbers::new(
        limits,
        w::OwnerNumbersParts {
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
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            left: value.left,
            closed: value.closed,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}
