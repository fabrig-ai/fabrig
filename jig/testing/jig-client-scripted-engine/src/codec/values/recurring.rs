//! Wire projections of recurring facts (client/domain/testing.md, 4).
use super::super::{c, w};
pub(in crate::codec) fn wire_recurring_overlap(value: &c::RecurringOverlap) -> w::RecurringOverlap {
    match value {
        c::RecurringOverlap::Skip => w::RecurringOverlap::Skip,
        c::RecurringOverlap::Wait => w::RecurringOverlap::Wait,
    }
}
