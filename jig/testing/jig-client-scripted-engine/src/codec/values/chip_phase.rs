//! Wire projections of chip phase facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
use super::{wire_closing_stage, wire_hold_category, wire_person_address, wire_run_stage};
pub(in crate::codec) fn wire_chip_phase(value: &c::ChipPhase, limits: &w::Limits) -> Result<w::ChipPhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ChipPhase::Waiting { dependencies, more, holds } => w::ChipPhase::Waiting(w::ChipPhaseWaiting::new(
            limits,
            w::ChipPhaseWaitingParts {
                dependencies: list((dependencies).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                more: *(more),
                holds: *(holds),
            },
        )?),
        c::ChipPhase::Active(value) => w::ChipPhase::Active(w::ChipPhaseActive::new(
            limits,
            w::ChipPhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::ChipPhase::Person { address, taken_by, since } => w::ChipPhase::Person(w::ChipPhasePerson::new(
            limits,
            w::ChipPhasePersonParts {
                address: wire_person_address(address, limits)?,
                taken_by: (taken_by).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                since: (since).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::ChipPhase::Closing(value) => w::ChipPhase::Closing(w::ChipPhaseClosing::new(
            limits,
            w::ChipPhaseClosingParts { value: wire_closing_stage(value, limits)? },
        )?),
        c::ChipPhase::Held { reason, revision } => w::ChipPhase::Held(w::ChipPhaseHeld::new(
            limits,
            w::ChipPhaseHeldParts { reason: wire_hold_category(reason, limits)?, revision: *(revision) },
        )?),
        c::ChipPhase::Ended => w::ChipPhase::Ended,
    })
}
