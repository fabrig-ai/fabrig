//! Parked-run workload faults (`jig/docs/design/domain/testing-workloads.md`, 2).
use skein_world::faults::Kind;

pub const FAMILY: &str = "runs that park, resume and move";
pub const ANSWERED_WHILE_PARKED: Kind =
    Kind { family: FAMILY, name: "a parked run's answer arrives while it is parked" };
pub const ANSWERED_AFTER_ITS_END: Kind = Kind { family: FAMILY, name: "a parked run's answer arrives after it ended" };
pub const ANSWERED_TO_ITS_SUCCESSOR: Kind =
    Kind { family: FAMILY, name: "a parked run's answer reaches its successor" };
pub const COORDINATOR_ENDS_WHILE_A_CHILD_WAITS: Kind =
    Kind { family: FAMILY, name: "a coordinator ends while a child waits for its answer" };
