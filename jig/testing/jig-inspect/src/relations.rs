//! The words shared with smith and the links, and the host's side of each relation
//! (jig/docs/design/domain/testing-fakes.md, 1.1).

use jig_domain_core::CallKey;
use jig_domain_host::Hub;
use skein_lib::Token;
use skein_world::inspect::{Inspection, Related, Relation, Rule, Side};

const OPEN_CALL: Side = Side { component: "smith", holds: "an open call" };
pub(crate) const CALL_RECORD: Side = Side { component: "jig", holds: "a call record" };
pub(crate) const REPLAY: Side = Side { component: "jig", holds: "an answer retained for replay" };
pub(crate) const CANCELLED: Side = Side { component: "jig", holds: "a cancelled run's call" };
pub(crate) const FLEET_RELAY: Side = Side { component: "jig", holds: "a relay in flight" };
const HOST_RELAY: Side = Side { component: "jig host", holds: "a relay in flight" };
const LINK_RELAY: Side = Side { component: "links", holds: "a relay in flight" };

/// Match named calls across smith, the engine, the host and their links.
pub const HOST_CALLS: Relation = Relation {
    name: "a run's host calls",
    sides: &[OPEN_CALL, CALL_RECORD, REPLAY, CANCELLED, FLEET_RELAY, HOST_RELAY, LINK_RELAY],
    rules: &[
        Rule { each: OPEN_CALL, by: &[CALL_RECORD, REPLAY, FLEET_RELAY, HOST_RELAY, LINK_RELAY] },
        Rule { each: CALL_RECORD, by: &[OPEN_CALL, FLEET_RELAY, HOST_RELAY, LINK_RELAY] },
    ],
};

pub(crate) const PLACEMENT: Side = Side { component: "jig", holds: "a placement" };
const HOSTED_SLOT: Side = Side { component: "jig host", holds: "a hosted slot" };
const TERMINAL: Side = Side { component: "jig host", holds: "a terminal awaiting its acknowledgement" };

/// Match the fleet's placements with the slots and terminals they have on the host.
pub const PLACEMENTS: Relation = Relation {
    name: "a run's placement",
    sides: &[PLACEMENT, HOSTED_SLOT, TERMINAL, LINK_RELAY],
    rules: &[
        Rule { each: PLACEMENT, by: &[HOSTED_SLOT, TERMINAL, LINK_RELAY] },
        Rule { each: HOSTED_SLOT, by: &[PLACEMENT, LINK_RELAY] },
    ],
};

pub(crate) fn placement_key(run: Token, attempt: Token) -> String {
    format!("run {} attempt {}", run.raw(), attempt.raw())
}

pub(crate) fn host_call(into: &mut Inspection, key: CallKey, side: Side) {
    let CallKey { task, attempt, completion, position } = key;
    into.related(Related {
        relation: HOST_CALLS.name,
        key: format!("task {task} attempt {attempt} completion {completion} position {position}"),
        holds: side.holds,
    });
}

pub(crate) fn relay(into: &mut Inspection, run: Token, attempt: Token, name: &[u8], side: Side) {
    let attempt = attempt.raw();
    if let Some(key) = jig_protocol_charter::call_key(name, attempt) {
        host_call(into, CallKey { task: run.raw(), attempt, completion: key.completion, position: key.position }, side);
    }
}

/// Read a host's slots, retained terminals and relays into the `jig host` inspection.
pub fn host(hub: &Hub, into: &mut Inspection) {
    let mut from = 0;
    while let Some((next, run, attempt, answered)) = hub.inspection_slot(from) {
        into.related(Related {
            relation: PLACEMENTS.name,
            key: placement_key(run, attempt),
            holds: if answered { TERMINAL.holds } else { HOSTED_SLOT.holds },
        });
        from = next;
    }
    let mut from = 0;
    while let Some((next, run, attempt, name)) = hub.inspection_relay(from) {
        relay(into, run, attempt, name, HOST_RELAY);
        from = next;
    }
}
