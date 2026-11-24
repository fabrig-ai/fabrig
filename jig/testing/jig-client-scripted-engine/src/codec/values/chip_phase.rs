//! Wire projections of chip phase facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
use super::{wire_closing_stage, wire_hold_category, wire_person_address, wire_run_stage};
pub(in crate::codec) fn wire_chip_phase(value: &c::ChipPhase) -> w::ChipPhase {
    match value {
        c::ChipPhase::Waiting { dependencies, more, holds } => w::ChipPhase::Waiting(w::ChipPhaseWaiting {
            dependencies: list((dependencies).iter().copied()),
            more: *(more),
            holds: *(holds),
        }),
        c::ChipPhase::Active(value) => w::ChipPhase::Active(w::ChipPhaseActive { value: wire_run_stage(value) }),
        c::ChipPhase::Person { address, taken_by, since } => w::ChipPhase::Person(w::ChipPhasePerson {
            address: wire_person_address(address),
            taken_by: *taken_by,
            since: (since).as_ref().map(|item| (item).as_nanos()),
        }),
        c::ChipPhase::Closing(value) => w::ChipPhase::Closing(w::ChipPhaseClosing { value: wire_closing_stage(value) }),
        c::ChipPhase::Held { reason, revision } => {
            w::ChipPhase::Held(w::ChipPhaseHeld { reason: wire_hold_category(reason), revision: *(revision) })
        }
        c::ChipPhase::Ended => w::ChipPhase::Ended,
    }
}
