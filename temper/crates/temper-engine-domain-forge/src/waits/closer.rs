//! The waits of a closer (temper/docs/design/domain/forge/waits.md, 6).
use crate::{Domain, Limits, room};
use jig_domain_core_vocabulary::waits::Reported;
use skein_lib::{Env, List};
use temper_client_protocol_wire::waits::{CLOSER_CLOSING, CLOSER_LANDING, CLOSER_MAKING, CLOSER_REPORTING};

pub(super) fn waits(d: &Domain, env: &Env<Limits>, task: u64) -> List<Reported> {
    let mut waits = List::with_capacity(1);
    if let Some(row) = d.closer(task)
        && row.closer.hold.is_none()
        && row.closer.running.is_none()
    {
        let step = if row.done || row.closer.landing.is_some() {
            CLOSER_REPORTING
        } else if row.making.is_some() {
            CLOSER_MAKING
        } else if row.closer.closed.is_some() {
            CLOSER_LANDING
        } else {
            CLOSER_CLOSING
        };
        if let Some(wait) = room::waited(d, env, task, step) {
            waits.push(wait).expect("one room wait");
        }
    }
    waits
}
