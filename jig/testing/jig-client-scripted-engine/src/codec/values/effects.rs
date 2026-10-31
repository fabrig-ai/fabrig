//! Wire projections of effects facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::wire_impact;
pub(in crate::codec) fn wire_effect(value: &c::Effect, limits: &w::Limits) -> Result<w::Effect, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Effect::Release(value) => w::Effect::Release(w::EffectRelease::new(
            limits,
            w::EffectReleaseParts { what: wire_release_effect(value, limits)? },
        )?),
        c::Effect::Cancel(value) => w::Effect::Cancel(wire_impact(value, limits)?),
        c::Effect::Move { reservation } => {
            w::Effect::MoveTask(w::EffectMove::new(limits, w::EffectMoveParts { reservation: *(reservation) })?)
        }
    })
}

pub(in crate::codec) fn wire_release_effect(
    value: &c::ReleaseEffect,
    _limits: &w::Limits,
) -> Result<w::ReleaseEffect, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ReleaseEffect::Judged => w::ReleaseEffect::Judged,
        c::ReleaseEffect::RunAgain => w::ReleaseEffect::RunAgain,
        c::ReleaseEffect::Uncertain => w::ReleaseEffect::Uncertain,
    })
}
