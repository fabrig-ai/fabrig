//! A project's changes subject (temper/docs/design/client/forge/records.md).
use super::{Limits, wire_limits};
use jig_client::{Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::ChangesSubject::new(&bounds, w::ChangesSubjectParts { project }).ok()?;
    let watching = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Changes(subject) }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) },
    })
}

#[must_use]
pub fn panel(record: &Family, limits: Limits) -> Option<w::ChangesPanel> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_changes_panel(&record.bytes, &wire_limits(&limits)?)
}
