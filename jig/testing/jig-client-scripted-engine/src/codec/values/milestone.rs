//! Wire projections of owner milestone facts (client/domain/testing.md, 4).
use super::super::{c, facts, w};
use super::{wire_closing_stage, wire_hold_category, wire_part, wire_prior_phase, wire_run_stage};
pub(in crate::codec) fn wire_milestone_line(
    value: &c::MilestoneLine,
    limits: &w::Limits,
) -> Result<w::MilestoneLine, w::Problem> {
    w::MilestoneLine::new(
        limits,
        w::MilestoneLineParts {
            task: value.task,
            position: value.position,
            phase: wire_milestone_phase(&value.phase, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_milestone_phase(
    value: &c::MilestonePhase,
    limits: &w::Limits,
) -> Result<w::MilestonePhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::MilestonePhase::Waiting => w::MilestonePhase::Waiting,
        c::MilestonePhase::Active(value) => w::MilestonePhase::Active(w::MilestonePhaseActive::new(
            limits,
            w::MilestonePhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::MilestonePhase::Closing { stage, ending } => w::MilestonePhase::Closing(w::MilestonePhaseClosing::new(
            limits,
            w::MilestonePhaseClosingParts {
                stage: wire_closing_stage(stage, limits)?,
                ending: facts::chip_ending(ending, limits)?,
            },
        )?),
        c::MilestonePhase::Held { before, why, source, ending } => w::MilestonePhase::Held(w::MilestonePhaseHeld::new(
            limits,
            w::MilestonePhaseHeldParts {
                before: wire_prior_phase(before, limits)?,
                why: wire_hold_category(why, limits)?,
                source: (source).as_ref().map(|item| wire_part(item, limits)).transpose()?,
                ending: (ending).as_ref().map(|item| facts::chip_ending(item, limits)).transpose()?,
            },
        )?),
        c::MilestonePhase::Ended(value) => w::MilestonePhase::Ended(w::MilestonePhaseEnded::new(
            limits,
            w::MilestonePhaseEndedParts { value: facts::chip_ending(value, limits)? },
        )?),
    })
}
