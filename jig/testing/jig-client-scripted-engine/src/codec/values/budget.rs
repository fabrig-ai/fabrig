//! Wire projections of owner budget facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::wire_funder;
pub(in crate::codec) fn wire_budget(value: &c::Budget) -> w::Budget {
    w::Budget {
        numbers: wire_numbers(&value.numbers),
        left: value.left,
        funder: wire_funder(&value.funder),
        deadline: value.deadline.as_ref().map(|item| (item).as_nanos()),
    }
}

pub(in crate::codec) fn wire_numbers(value: &c::Numbers) -> w::Numbers {
    w::Numbers { budget: value.budget, spent: value.spent, spent_below: value.spent_below, reserved: value.reserved }
}

pub(in crate::codec) fn wire_pool_ledger(value: &c::PoolLedger) -> w::PoolLedger {
    w::PoolLedger {
        source: wire_funder(&value.source),
        parent: value.parent.as_ref().map(wire_funder),
        numbers: wire_numbers(&value.numbers),
        left: value.left,
        closed: value.closed,
        began: value.began.as_ref().map(|item| (item).as_nanos()),
        next: value.next.as_ref().map(|item| (item).as_nanos()),
    }
}
