//! Findings on the forge family (temper/docs/design/client/forge/records.md).
use super::{Limits, wire_limits};
use alloc::boxed::Box;
use jig_client::{Ask, Family, Watchable, contract as c};
use skein_lib::{List, bytes};
use temper_client_protocol_wire::{self as wire, v1 as w};

/// A finding named across the client's forge and pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    pub project: u32,
    pub number: u32,
}

#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    watching(project, Some(w::FindingState::New), true, limits)
}

/// A findings subject with the caller's state and ownership filter.
#[must_use]
pub fn watching(project: u32, state: Option<w::FindingState>, mine: bool, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::FindingsSubject::new(
        &bounds,
        w::FindingsSubjectParts {
            project,
            filter: w::FindingsFilter::Triage(
                w::FindingsTriage::new(&bounds, w::FindingsTriageParts { state, mine }).ok()?,
            ),
        },
    )
    .ok()?;
    let watching = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Findings(subject) }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) },
    })
}

/// Where a finding stands (temper/docs/design/domain/red-findings.md, 10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stands {
    Cleared { tier: Box<[u8]>, run: u64 },
    Waiting,
    Repairing(Option<u64>),
    Backlog,
    Dropped(Box<[u8]>),
}

/// Clearing takes precedence over the triage state.
#[must_use]
pub fn stands(row: &w::FindingRow) -> Stands {
    if let Some(cleared) = row.cleared() {
        return Stands::Cleared { tier: bytes::copy_of(cleared.tier()), run: cleared.run() };
    }
    match row.state() {
        w::FindingState::New => Stands::Waiting,
        w::FindingState::Routed => Stands::Repairing(*row.goal()),
        w::FindingState::Backlog => Stands::Backlog,
        w::FindingState::Dropped => Stands::Dropped(match row.reason() {
            Some(reason) => reason.clone(),
            None => Box::default(),
        }),
    }
}

/// Uncleared findings first, keeping the subject's order within each group.
#[must_use]
pub fn uncleared_first(rows: &[w::FindingRow]) -> Box<[w::FindingRow]> {
    let mut ordered = List::uncapped();
    for cleared in [false, true] {
        for row in rows {
            if row.cleared().is_some() == cleared {
                ordered.push(row.clone()).expect("the findings subject's rows fit a list");
            }
        }
    }
    ordered.into_boxed()
}

#[must_use]
pub fn document(record: &Family, limits: &Limits) -> Option<w::Findings> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_findings(&record.bytes, &wire_limits(limits)?)
}

#[must_use]
pub fn request(project: u32, request: w::Request, limits: Limits) -> Option<Ask> {
    let bounds = wire_limits(&limits)?;
    let record = w::Requesting::new(&bounds, w::RequestingParts { request }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}

#[must_use]
pub fn outcome(outcome: &jig_client::Outcome, limits: Limits) -> Option<w::Outcome> {
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

#[cfg(test)]
#[path = "../tests/findings.rs"]
mod tests;
