//! Wire projections of chip phase facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
use super::{wire_closing_stage, wire_hold_category, wire_person_address, wire_run_stage};
pub(in crate::codec) fn wire_owner_chip_phase(
    value: &c::OwnerChipPhase,
    limits: &w::Limits,
) -> Result<w::OwnerChipPhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerChipPhase::Waiting { dependencies, more, holds } => {
            w::OwnerChipPhase::Waiting(w::OwnerChipPhaseWaiting::new(
                limits,
                w::OwnerChipPhaseWaitingParts {
                    dependencies: list((dependencies).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                    more: *(more),
                    holds: *(holds),
                },
            )?)
        }
        c::OwnerChipPhase::Active(value) => w::OwnerChipPhase::Active(w::OwnerChipPhaseActive::new(
            limits,
            w::OwnerChipPhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::OwnerChipPhase::Person { address, taken_by, since } => {
            w::OwnerChipPhase::Person(w::OwnerChipPhasePerson::new(
                limits,
                w::OwnerChipPhasePersonParts {
                    address: wire_person_address(address, limits)?,
                    taken_by: (taken_by).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                    since: (since).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
                },
            )?)
        }
        c::OwnerChipPhase::Closing(value) => w::OwnerChipPhase::Closing(w::OwnerChipPhaseClosing::new(
            limits,
            w::OwnerChipPhaseClosingParts { value: wire_closing_stage(value, limits)? },
        )?),
        c::OwnerChipPhase::Held { reason, revision } => w::OwnerChipPhase::Held(w::OwnerChipPhaseHeld::new(
            limits,
            w::OwnerChipPhaseHeldParts { reason: wire_hold_category(reason, limits)?, revision: *(revision) },
        )?),
        c::OwnerChipPhase::Ended => w::OwnerChipPhase::Ended,
    })
}
