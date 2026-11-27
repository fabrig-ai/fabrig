//! Places and continuations kept beside the task child (jig/docs/design/domain/testing-fakes.md, 1.1).

use jig_domain_core::{CallKey, CallPart, Core};
use jig_domain_tasks::{Ending, Phase, Was};
use skein_world::inspect::{Entry, Inspection, Related, Release, Table};

use crate::relations::{self, CALL_RECORD, CANCELLED, FLEET_RELAY, PLACEMENT, PLACEMENTS, REPLAY};

pub(crate) fn placements(core: &Core, into: &mut Inspection) {
    let mut entries = Vec::new();
    let mut from = 0;
    while let Some((next, run, attempt, claimed)) = core.fleet.inspection_placement(from) {
        let due = claimed
            && core
                .tasks
                .task(run.raw())
                .is_none_or(|row| row.last_answer == Some(attempt.raw()) || matches!(row.phase, Phase::Ended(_)));
        let key = relations::placement_key(run, attempt);
        into.related(Related { relation: PLACEMENTS.name, key: key.clone(), holds: PLACEMENT.holds });
        entries.push(Entry {
            key,
            rule: "its attempt's end",
            release: if due { Release::Due } else { Release::Pending },
        });
        from = next;
    }
    into.table(Table {
        name: "placements",
        capacity: u64::from(core.fleet.inspection_attempt_capacity()),
        entries,
        refused: Vec::new(),
    });
}

pub(crate) fn flights(core: &Core, into: &mut Inspection) {
    let mut entries = Vec::new();
    let mut from = 0;
    while let Some((next, flight, kind, owner)) = core.inspection_flight(from) {
        let (key, due) = if let Some(task) = owner {
            (
                format!("{kind} task {task}"),
                core.tasks.task(task).is_none_or(|row| matches!(row.phase, Phase::Ended(_))),
            )
        } else {
            (format!("{kind} flight {}", flight.raw()), false)
        };
        entries.push(Entry { key, rule: "its reply", release: if due { Release::Due } else { Release::Pending } });
        from = next;
    }
    into.table(Table {
        name: "flights",
        capacity: u64::from(core.inspection_flight_capacity()),
        entries,
        refused: Vec::new(),
    });
}

fn cancelled(core: &Core, key: CallKey) -> bool {
    core.tasks.task(key.task).is_some_and(|row| {
        row.attempt == key.attempt
            && match &row.phase {
                Phase::Closing(closing) | Phase::Held { was: Was::Closing(closing), .. } => {
                    matches!(closing.ending, Ending::Cancelled { .. })
                }
                Phase::Ended(ending) => matches!(ending, Ending::Cancelled { .. }),
                _ => false,
            }
    })
}

pub(crate) fn host_calls(core: &Core, into: &mut Inspection) {
    let mut after = None;
    while let Some(key) = core.inspection_pending_call_after(after) {
        relations::host_call(into, key, if cancelled(core, key) { CANCELLED } else { CALL_RECORD });
        after = Some(key);
    }
    for (&key, part) in &core.call_parts {
        let side = if cancelled(core, key) {
            CANCELLED
        } else if let CallPart::Effect { entry, .. } = part
            && core.inspection_effect_reply(*entry) == Some(key)
        {
            CALL_RECORD
        } else {
            REPLAY
        };
        relations::host_call(into, key, side);
    }
    let mut from = 0;
    while let Some((next, run, attempt, name)) = core.fleet.inspection_relay(from) {
        relations::relay(into, run, attempt, name, FLEET_RELAY);
        from = next;
    }
}
