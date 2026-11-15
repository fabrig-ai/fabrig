//! Wire projections of attempts and holds (client/domain/testing.md, 4).
use super::super::{c, facts, w};
use super::{wire_hold_category, wire_part, wire_prior_phase, wire_run_end};
pub(in crate::codec) fn wire_attempt_line(
    value: &c::AttemptLine,
    limits: &w::Limits,
) -> Result<w::AttemptLine, w::Problem> {
    w::AttemptLine::new(
        limits,
        w::AttemptLineParts {
            task: value.task,
            attempt: value.attempt,
            claimed: value.claimed.map(skein_lib::Wall::as_nanos),
            started: value.started.map(skein_lib::Wall::as_nanos),
            ended: value.ended.map(skein_lib::Wall::as_nanos),
            end: value.end.as_ref().map(|item| wire_run_end(item, limits)).transpose()?,
            phases: value.phases,
        },
    )
}
pub(in crate::codec) fn wire_hold_line(value: &c::HoldLine, limits: &w::Limits) -> Result<w::HoldLine, w::Problem> {
    w::HoldLine::new(
        limits,
        w::HoldLineParts {
            task: value.task,
            position: value.position,
            at: value.at.as_nanos(),
            before: wire_prior_phase(&value.before, limits)?,
            why: wire_hold_category(&value.why, limits)?,
            source: value.source.as_ref().map(|item| wire_part(item, limits)).transpose()?,
            ending: value.ending.as_ref().map(|item| facts::chip_ending(item, limits)).transpose()?,
        },
    )
}
