//! Jig's waits and bounded tables (jig/docs/design/domain/testing.md, 7.1).
//! These reads run at a world's quiet point and change none of the inspected state.
mod engine_clients;
pub use engine_clients::{ENGINE_CLIENTS_KINDS, engine_clients};

mod tables;
mod waits;

use jig_domain_core::Core;
use jig_domain_tasks::Domain;
use skein_world::inspect::{Inspection, Wait, Wake};

/// Every wait named by jig's design; missing reasons and missing alarms are not kinds.
pub const KINDS: &[&str] = &[
    "on its dependencies",
    "in a resource's queue",
    "on its delegates",
    "on its requester",
    "on a watch",
    "on a timer",
    "on its party",
    "on a person's answer",
    "on its proposal's decision",
    "held: exceeded",
    "held: failures",
    "held: stalled",
    "held: dependency",
    "held: stopped",
    "held: placed",
    "held: drift",
    "held: effects",
    "held: effect failed",
    "held: uncertain",
    "held: budget",
    "held: deadline",
    "held: holds waited",
    "held: resource unavailable",
    "held: resource access",
    "held: pool lost",
    "held: procedure",
    "held: circle",
    "closing on its run",
    "closing on its effects",
    "closing on its releases",
    "backing off",
    "suspended",
    "paced",
    "a batch awaiting its creation reports",
];

/// Read the task child's waits and bounded tables, without a core's placements or flights.
pub fn tasks(tasks: &Domain, into: &mut Inspection) {
    tables::tasks(tasks, None, into);
}

/// Read the core and its tasks, including parked batches and retained people answers.
pub fn core(core: &Core, into: &mut Inspection) {
    tables::tasks(&core.tasks, Some(core), into);
    for (owner, names) in core.tasks.inspection_report_pins() {
        if let Some((number, _, deadline)) = core.inspection_batch(*owner) {
            let mut wakes = Vec::new();
            for name in names {
                if core.tasks.resource_hold(name).is_none() {
                    let wake = Wake::Party(format!("connector {}", name.connector));
                    if !wakes.contains(&wake) {
                        wakes.push(wake);
                    }
                }
            }
            wakes.push(Wake::Timer(deadline));
            into.wait(Wait {
                waiter: format!("batch {number}"),
                kind: "a batch awaiting its creation reports".into(),
                wakes,
                bound: Some(deadline),
            });
        }
    }
    tables::answers(core, into);
}
