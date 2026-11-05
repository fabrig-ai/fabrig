//! A project's decisions (temper/docs/design/client/README.md, 4.6).
use super::{Limits, wire_limits};
use jig_client::{Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::DecidedSubject::new(&bounds, w::DecidedSubjectParts { project }).ok()?;
    let watching = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Decided(subject) }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) },
    })
}

#[must_use]
pub fn document(record: &Family, limits: Limits) -> Option<w::Decisions> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_decisions(&record.bytes, &wire_limits(&limits)?)
}
