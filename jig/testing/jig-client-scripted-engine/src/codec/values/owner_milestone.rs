//! Wire projections of owner milestone facts (client/domain/testing.md, 4).
use super::super::{c, facts, w};
use super::{wire_closing_stage, wire_hold_category, wire_owner_part, wire_owner_prior_phase, wire_run_stage};
pub(in crate::codec) fn wire_owner_milestone_line(
    value: &c::OwnerMilestoneLine,
    limits: &w::Limits,
) -> Result<w::OwnerMilestoneLine, w::Problem> {
    w::OwnerMilestoneLine::new(
        limits,
        w::OwnerMilestoneLineParts {
            task: value.task,
            position: value.position,
            phase: wire_owner_milestone_phase(&value.phase, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_milestone_phase(
    value: &c::OwnerMilestonePhase,
    limits: &w::Limits,
) -> Result<w::OwnerMilestonePhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerMilestonePhase::Waiting => w::OwnerMilestonePhase::Waiting,
        c::OwnerMilestonePhase::Active(value) => w::OwnerMilestonePhase::Active(w::OwnerMilestonePhaseActive::new(
            limits,
            w::OwnerMilestonePhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::OwnerMilestonePhase::Closing { stage, ending } => {
            w::OwnerMilestonePhase::Closing(w::OwnerMilestonePhaseClosing::new(
                limits,
                w::OwnerMilestonePhaseClosingParts {
                    stage: wire_closing_stage(stage, limits)?,
                    ending: facts::owner_chip_ending(ending, limits)?,
                },
            )?)
        }
        c::OwnerMilestonePhase::Held { before, why, source, ending } => {
            w::OwnerMilestonePhase::Held(w::OwnerMilestonePhaseHeld::new(
                limits,
                w::OwnerMilestonePhaseHeldParts {
                    before: wire_owner_prior_phase(before, limits)?,
                    why: wire_hold_category(why, limits)?,
                    source: (source).as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
                    ending: (ending).as_ref().map(|item| facts::owner_chip_ending(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerMilestonePhase::Ended(value) => w::OwnerMilestonePhase::Ended(w::OwnerMilestonePhaseEnded::new(
            limits,
            w::OwnerMilestonePhaseEndedParts { value: facts::owner_chip_ending(value, limits)? },
        )?),
    })
}
