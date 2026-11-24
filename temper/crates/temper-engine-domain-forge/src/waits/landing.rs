//! The waits of a landing (temper/docs/design/domain/forge/waits.md, 5).
use crate::{Domain, Limits, room};
use jig_domain_core_vocabulary::waits::Reported;
use skein_lib::{Env, List};
use temper_client_protocol_wire::waits::LANDING_ROOM;
use temper_engine_domain_forge_landing::Phase;

pub(super) fn waits(d: &Domain, env: &Env<Limits>, task: u64) -> List<Reported> {
    let mut waits = List::with_capacity(1);
    if !d.landing_held()
        && let Some(row) = d.landing.rows.get(&task)
        && let Phase::Waiting
        | Phase::Preparing
        | Phase::Proving
        | Phase::Proved { .. }
        | Phase::Turn { .. }
        | Phase::Landed
        | Phase::Cancelled = row.landing.phase
        && let Some(wait) = room::waited(d, env, task, LANDING_ROOM)
    {
        waits.push(wait).expect("one room wait");
    }
    waits
}
