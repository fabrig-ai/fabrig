//! Runs that park, resume and move (`temper/docs/design/testing/workloads.md`, 2.3).

use skein_world::faults::Kind;

pub const FAMILY: &str = "runs that park, resume and move";

pub const A_CHANGE_WAITS_ON_ANOTHER_GOAL: Kind = Kind { family: FAMILY, name: "a change waits on another goal" };
pub const DELIVERED_THEN_PARKED: Kind =
    Kind { family: FAMILY, name: "a run that delivered parks and finishes in its next run" };
pub const RESUMED_ON_ANOTHER_WORKER: Kind = Kind { family: FAMILY, name: "a run resumes on another worker" };
