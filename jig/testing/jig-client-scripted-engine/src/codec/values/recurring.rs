//! Wire projections of recurring facts (client/domain/testing.md, 4).
use super::super::{c, w};
pub(in crate::codec) fn wire_recurring_overlap(
    value: &c::RecurringOverlap,
    _limits: &w::Limits,
) -> Result<w::RecurringOverlap, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RecurringOverlap::Skip => w::RecurringOverlap::Skip,
        c::RecurringOverlap::Wait => w::RecurringOverlap::Wait,
    })
}
