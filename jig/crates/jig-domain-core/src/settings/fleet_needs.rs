//! The core's configured charter requirements for fleet admission (domain/engine/fleet.md).

use crate::Limits;
pub use crate::below::settings::fleet_needs::*;
use jig_domain_fleet::Needs;
use skein_lib::List;
pub(crate) fn current(core: &crate::Core, limits: &Limits) -> Needs {
    let capacity = limits.tasks.charters.checked_add(limits.settings.versions).expect("charter version bound");
    let mut charters = List::with_capacity(capacity);
    for setting in &core.current.offered() {
        let super::Setting::Charter(number) = *setting else { continue };
        let charter = core.charter(number).expect("offered charter");
        let (_, origin) = core.current.lookup(*setting);
        let (_, revision) = origin.expect("offered charter version");
        charters.push(one(charter, revision)).expect("configured charter count");
    }
    for (&key, value) in &core.held.versions {
        if let Some(charter) = &value.charter {
            charters.push(one(charter, key.revision)).expect("held charter version count");
        }
    }
    Needs {
        charters: charters.into_boxed(),
        spend: core.current.rules().maximum_run_spend,
        message_bytes: limits.tasks.message_bytes,
        start_messages: limits.tasks.run_inbox,
        unacknowledged: core.hub_turns,
    }
}
