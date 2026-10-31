//! Wire projections of owner phase counts facts (client/domain/testing.md, 4).
use super::super::{c, w};
pub(in crate::codec) fn wire_phase_counts(
    value: &c::PhaseCounts,
    limits: &w::Limits,
) -> Result<w::PhaseCounts, w::Problem> {
    w::PhaseCounts::new(
        limits,
        w::PhaseCountsParts {
            waiting: value.waiting,
            idle: value.idle,
            starting: value.starting,
            running: value.running,
            backing_off: value.backing_off,
            closing: value.closing,
            held: value.held,
            done: value.done,
            failed: value.failed,
            cancelled: value.cancelled,
        },
    )
}
