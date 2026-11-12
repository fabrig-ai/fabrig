//! Findings on the forge family (temper/docs/design/client/forge/records.md).
use super::{Limits, wire_limits};
use jig_client::{Ask, Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::FindingsSubject::new(
        &bounds,
        w::FindingsSubjectParts { project, state: Some(w::FindingState::New), mine: true },
    )
    .ok()?;
    let watching = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Findings(subject) }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) },
    })
}

#[must_use]
pub fn document(record: &Family, limits: &Limits) -> Option<w::Findings> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_findings(&record.bytes, &wire_limits(limits)?)
}

pub(crate) fn request(project: u32, request: w::Request, limits: Limits) -> Option<Ask> {
    let bounds = wire_limits(&limits)?;
    let record = w::Requesting::new(&bounds, w::RequestingParts { request }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}

pub(crate) fn outcome(outcome: &jig_client::Outcome, limits: Limits) -> Option<w::Outcome> {
    let jig_client::Outcome::Answered(jig_client::Answered::Family(record)) = outcome else { return None };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(&wire_limits(&limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(answer.outcome().clone())
}
