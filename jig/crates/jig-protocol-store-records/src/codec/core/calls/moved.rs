//! Revision facts kept in the core family (protocol/store/records.md, 1).
use crate::core_v6 as wire;
use jig_domain_people as domain;
pub(super) fn encode_failure_class(value: domain::FailureClass) -> wire::FailureClass {
    match value {
        domain::FailureClass::Transient => wire::FailureClass::Transient,
        domain::FailureClass::Permanent => wire::FailureClass::Permanent,
        domain::FailureClass::Run => wire::FailureClass::Run,
        domain::FailureClass::Agent => wire::FailureClass::Agent,
        domain::FailureClass::Lost => wire::FailureClass::Lost,
        domain::FailureClass::Invalid => wire::FailureClass::Invalid,
        domain::FailureClass::Infrastructure => wire::FailureClass::Infrastructure,
    }
}
pub(super) fn decode_failure_class(value: wire::FailureClass) -> domain::FailureClass {
    match value {
        wire::FailureClass::Transient => domain::FailureClass::Transient,
        wire::FailureClass::Permanent => domain::FailureClass::Permanent,
        wire::FailureClass::Run => domain::FailureClass::Run,
        wire::FailureClass::Agent => domain::FailureClass::Agent,
        wire::FailureClass::Lost => domain::FailureClass::Lost,
        wire::FailureClass::Invalid => domain::FailureClass::Invalid,
        wire::FailureClass::Infrastructure => domain::FailureClass::Infrastructure,
    }
}
pub(super) fn encode_hold_reason(value: &domain::HoldReason, _limits: &wire::Limits) -> wire::HoldReason {
    match value {
        domain::HoldReason::Exceeded(value) => wire::HoldReason::Exceeded(wire::Exceeded {
            bound: match value.bound {
                domain::Bound::Reasoning => wire::Bound::Reasoning,
                domain::Bound::Metadata => wire::Bound::Metadata,
                domain::Bound::Items => wire::Bound::Items,
                domain::Bound::Answer => wire::Bound::Answer,
                domain::Bound::Tools => wire::Bound::Tools,
                domain::Bound::HistoryItems => wire::Bound::HistoryItems,
                domain::Bound::History => wire::Bound::History,
                domain::Bound::Window => wire::Bound::Window,
                domain::Bound::Output => wire::Bound::Output,
            },
            value: value.value,
        }),
        domain::HoldReason::Failures { class } => {
            wire::HoldReason::Failures(wire::HoldReasonFailures { class: encode_failure_class(*class) })
        }
        domain::HoldReason::Stalled => wire::HoldReason::Stalled,
        domain::HoldReason::Dependency { task } => {
            wire::HoldReason::Dependency(wire::HoldReasonDependency { task: *task })
        }
        domain::HoldReason::Stopped => wire::HoldReason::Stopped,
        domain::HoldReason::Placed { hold } => wire::HoldReason::Placed(wire::HoldReasonPlaced { hold: *hold }),
        domain::HoldReason::Drift => wire::HoldReason::Drift,
        domain::HoldReason::Effects => wire::HoldReason::Effects,
        domain::HoldReason::EffectFailed => wire::HoldReason::EffectFailed,
        domain::HoldReason::Uncertain { entry } => {
            wire::HoldReason::Uncertain(wire::HoldReasonUncertain { entry: *entry })
        }
        domain::HoldReason::Budget => wire::HoldReason::Budget,
        domain::HoldReason::Deadline => wire::HoldReason::Deadline,
        domain::HoldReason::HoldsWaited => wire::HoldReason::HoldsWaited,
        domain::HoldReason::ResourceUnavailable => wire::HoldReason::ResourceUnavailable,
        domain::HoldReason::ResourceAccess => wire::HoldReason::ResourceAccess,
        domain::HoldReason::PoolLost { pool } => wire::HoldReason::PoolLost(wire::HoldReasonPoolLost { pool: *pool }),
        domain::HoldReason::Procedure => wire::HoldReason::Procedure,
        domain::HoldReason::Circle => wire::HoldReason::Circle,
    }
}
pub(super) fn decode_hold_reason(value: wire::HoldReason) -> domain::HoldReason {
    match value {
        wire::HoldReason::Exceeded(value) => domain::HoldReason::Exceeded(domain::Exceeded {
            bound: match value.bound {
                wire::Bound::Reasoning => domain::Bound::Reasoning,
                wire::Bound::Metadata => domain::Bound::Metadata,
                wire::Bound::Items => domain::Bound::Items,
                wire::Bound::Answer => domain::Bound::Answer,
                wire::Bound::Tools => domain::Bound::Tools,
                wire::Bound::HistoryItems => domain::Bound::HistoryItems,
                wire::Bound::History => domain::Bound::History,
                wire::Bound::Window => domain::Bound::Window,
                wire::Bound::Output => domain::Bound::Output,
            },
            value: value.value,
        }),
        wire::HoldReason::Failures(value) => {
            let wire::HoldReasonFailures { class } = value;
            domain::HoldReason::Failures { class: decode_failure_class(class) }
        }
        wire::HoldReason::Stalled => domain::HoldReason::Stalled,
        wire::HoldReason::Dependency(value) => {
            let wire::HoldReasonDependency { task } = value;
            domain::HoldReason::Dependency { task }
        }
        wire::HoldReason::Stopped => domain::HoldReason::Stopped,
        wire::HoldReason::Placed(value) => {
            let wire::HoldReasonPlaced { hold } = value;
            domain::HoldReason::Placed { hold }
        }
        wire::HoldReason::Drift => domain::HoldReason::Drift,
        wire::HoldReason::Effects => domain::HoldReason::Effects,
        wire::HoldReason::EffectFailed => domain::HoldReason::EffectFailed,
        wire::HoldReason::Uncertain(value) => {
            let wire::HoldReasonUncertain { entry } = value;
            domain::HoldReason::Uncertain { entry }
        }
        wire::HoldReason::Budget => domain::HoldReason::Budget,
        wire::HoldReason::Deadline => domain::HoldReason::Deadline,
        wire::HoldReason::HoldsWaited => domain::HoldReason::HoldsWaited,
        wire::HoldReason::ResourceUnavailable => domain::HoldReason::ResourceUnavailable,
        wire::HoldReason::ResourceAccess => domain::HoldReason::ResourceAccess,
        wire::HoldReason::PoolLost(value) => {
            let wire::HoldReasonPoolLost { pool } = value;
            domain::HoldReason::PoolLost { pool }
        }
        wire::HoldReason::Procedure => domain::HoldReason::Procedure,
        wire::HoldReason::Circle => domain::HoldReason::Circle,
    }
}
pub(super) fn encode_moved(value: &domain::Moved, limits: &wire::Limits) -> wire::Moved {
    match value {
        domain::Moved::Hold { revision, reason } => {
            wire::Moved::Hold(wire::MovedHold { revision: *revision, reason: encode_hold_reason(reason, limits) })
        }
        domain::Moved::Spec { revision } => wire::Moved::Spec(wire::MovedSpec { revision: *revision }),
        domain::Moved::Person { revision } => wire::Moved::Person(wire::MovedPerson { revision: *revision }),
        domain::Moved::PlacedHold { revision } => {
            wire::Moved::PlacedHold(wire::MovedPlacedHold { revision: *revision })
        }
        domain::Moved::Note { revision } => wire::Moved::Note(wire::MovedNote { revision: *revision }),
    }
}
pub(super) fn decode_moved(value: wire::Moved) -> domain::Moved {
    match value {
        wire::Moved::Hold(value) => {
            let wire::MovedHold { revision, reason } = value;
            domain::Moved::Hold { revision, reason: decode_hold_reason(reason) }
        }
        wire::Moved::Spec(value) => {
            let wire::MovedSpec { revision } = value;
            domain::Moved::Spec { revision }
        }
        wire::Moved::Person(value) => {
            let wire::MovedPerson { revision } = value;
            domain::Moved::Person { revision }
        }
        wire::Moved::PlacedHold(value) => domain::Moved::PlacedHold { revision: value.revision },
        wire::Moved::Note(value) => {
            let wire::MovedNote { revision } = value;
            domain::Moved::Note { revision }
        }
    }
}

pub(super) fn encode_hold_owner(value: &domain::HoldOwner, _limits: &wire::Limits) -> wire::HoldOwner {
    match value {
        domain::HoldOwner::Party { person } => wire::HoldOwner::Party(wire::HoldOwnerParty { person: *person }),
        domain::HoldOwner::Task { task } => wire::HoldOwner::Task(wire::HoldOwnerTask { task: *task }),
    }
}
pub(super) fn decode_hold_owner(value: wire::HoldOwner) -> domain::HoldOwner {
    match value {
        wire::HoldOwner::Party(value) => domain::HoldOwner::Party { person: value.person },
        wire::HoldOwner::Task(value) => domain::HoldOwner::Task { task: value.task },
    }
}
