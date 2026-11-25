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
pub const PARKS_WITH_A_QUESTION_OPEN: Kind =
    Kind { family: FAMILY, name: "a run parks with a question to its requester open" };
pub const STUCK_IN_A_CIRCLE: Kind = Kind { family: FAMILY, name: "a circle of questions and watches" };
pub const PARKS_ON_NOTHING: Kind = Kind { family: FAMILY, name: "a run parks with nothing to await" };
pub const WATCHES_A_TASK_THAT_CAN_ANSWER: Kind =
    Kind { family: FAMILY, name: "a watch that can still bring the answer" };
