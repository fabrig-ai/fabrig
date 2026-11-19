//! A project's landed drafts (temper/docs/design/client/forge/records.md; domain/forge/drafts.md, 2-3).
use temper_client_protocol_wire::v1::Record;
use super::{Limits, wire_limits};
use alloc::boxed::Box;
use jig_client::{Ask, Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::DraftsSubject { project };
    let watching = w::Watching { watch: w::Watch::Drafts(subject) };
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: watching.encode(&bounds).ok()? },
    })
}

#[must_use]
pub fn panel(record: &Family, limits: Limits) -> Option<w::DraftsPanel> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_drafts_panel(&record.bytes, &wire_limits(&limits)?)
}

#[must_use]
pub fn ask(
    project: u32,
    repository: Option<w::DraftRepository>,
    path: Box<[u8]>,
    change: w::DraftChange,
    limits: &Limits,
) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let value = w::SetDraft { project, repository, path, change };
    let record = w::Requesting { request: w::Request::SetDraft(value) };
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: record.encode(&bounds).ok()? },
    })
}
