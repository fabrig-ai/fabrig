//! Forge records and the bounded attempts owned by their routing (client/forge/records.md).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use crate::forge;
use crate::generated::client_wire as maps;
use alloc::boxed::Box;
use jig_client_protocol_link as link;
use skein_lib::{List, Reader};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};
#[must_use]
pub fn request(a: &forge::Adoption, limits: &w::Limits) -> Option<jig_client::Family> {
    if a.checks.len() > skein_lib::usize_of(limits.adopt_checks) {
        return None;
    }
    let record = maps::adopt_to_record(a).ok()?;
    Some(jig_client::Family {
        family: wire::number(wire::Owner::Forge),
        version: wire::CLIENT_VERSION,
        bytes: (w::Requesting { request: w::Request::Adopt(record) }).encode(limits).ok()?,
    })
}
#[must_use]
pub fn make_repository(request: &w::MakeRepository, limits: &w::Limits) -> Option<jig_client::Family> {
    let record = w::Requesting { request: w::Request::MakeRepository(request.clone()) };
    Some(jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: record.encode(limits).ok()? })
}
/// Read one picker page (temper/docs/design/client/forge/repositories.md, 1).
#[must_use]
pub fn reachable(forge: u16, installation: Option<u64>, page: u32, limits: &w::Limits) -> Option<jig_client::Family> {
    let request = w::Reachable { forge, installation, page };
    let record = w::Requesting { request: w::Request::Reachable(request) };
    Some(jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: record.encode(limits).ok()? })
}

/// Dismiss a failed making by name (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn dismiss_making(
    forge: u16,
    org: &[u8],
    name: &[u8],
    into: Option<u32>,
    limits: &w::Limits,
) -> Option<jig_client::Family> {
    let request = w::DismissMaking { forge, org: Box::from(org), name: Box::from(name), into };
    let record = w::Requesting { request: w::Request::DismissMaking(request) };
    Some(jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: record.encode(limits).ok()? })
}

#[must_use]
pub fn answer(record: &jig_client::Family, limits: &w::Limits, link: &link::Limits) -> Option<forge::Answer> {
    if record.family != wire::number(wire::Owner::Forge) {
        return None;
    }
    if record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = Reader::new(&record.bytes);
    let a = w::Answer::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match a.outcome {
        w::Outcome::Adopted(a) => forge::Answer::Adopted(maps::adopted_to_domain(a)),
        w::Outcome::CoreRefused(a) => forge::Answer::CoreRefused(core_refusal(&a.refusal, link)?),
        w::Outcome::ForgeRefused(a) => forge::Answer::ForgeRefused(maps::forge_reason_to_domain(a.reason)),
        w::Outcome::Unknown => forge::Answer::Unknown,
        w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Removed
        | w::Outcome::Looked
        | w::Outcome::Dismissed
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::Reset
        | w::Outcome::DeviceAdded(_)
        | w::Outcome::DeviceRemoved
        | w::Outcome::DeviceRefused(_)
        | w::Outcome::Away
        | w::Outcome::Tried
        | w::Outcome::Filed(_)
        | w::Outcome::Handed
        | w::Outcome::Triaged(_)
        | w::Outcome::FindingRefused(_)
        | w::Outcome::Drafted(_)
        | w::Outcome::DraftRefused(_)
        | w::Outcome::Follow(_)
        | w::Outcome::Opened(_)
        | w::Outcome::OpenRefused(_)
        | w::Outcome::Tier(_)
        | w::Outcome::NotTheirs
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::Checkpointed(_)
        | w::Outcome::CheckpointRefused(_)
        | w::Outcome::EndRefused(_) => return None,
    })
}
#[must_use]
pub fn core_refusal(bytes: &[u8], limits: &link::Limits) -> Option<jig_client::contract::Refusal> {
    use jig_client_protocol_wire as j;
    let mut reader = Reader::new(bytes);
    let refusal = j::Refusal::decode(limits.wire(), &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let final_answer = j::Final::Refused(j::Refused { refusal, at: None });
    let record = final_answer.encode(limits.wire()).ok()?;
    let owned = j::Owned { owner: j::Owner::Jig, version: j::VERSION, record };
    match link::translate::answers::answer(j::KeyedAnswer::FinalAnswer(owned), limits).ok()? {
        jig_client::contract::Answer::Refused { refusal, .. } => Some(refusal),
        jig_client::contract::Answer::Done(_) => None,
    }
}

#[must_use]
pub fn set_up(
    forge: u16,
    org: Box<[u8]>,
    repository: Box<[u8]>,
    playground: bool,
    limits: &w::Limits,
) -> Option<jig_client::Family> {
    let request = w::SetUp {
        first: w::SetUpFirst::Empty(w::FirstEmpty { org, repository, playground, sample: false }),
        forge: w::Making::Kept(w::Kept { forge }),
    };
    let record = w::Requesting { request: w::Request::SetUp(request) };
    Some(jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: record.encode(limits).ok()? })
}
#[must_use]
pub fn set_up_answer(record: &jig_client::Family, limits: &w::Limits) -> Option<forge::forges::SetUpOutcome> {
    use forge::forges::SetUpOutcome as O;
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = Reader::new(&record.bytes);
    let answer = w::Answer::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match answer.outcome {
        w::Outcome::SettingUp => O::SettingUp,
        w::Outcome::SetUpRefused(why) => O::Refused(maps::set_up_refusal_to_domain(why.why)),
        w::Outcome::Unknown => O::Unknown,
        w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Removed
        | w::Outcome::Looked
        | w::Outcome::Dismissed
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::Reset
        | w::Outcome::DeviceAdded(_)
        | w::Outcome::DeviceRemoved
        | w::Outcome::DeviceRefused(_)
        | w::Outcome::Away
        | w::Outcome::Tried
        | w::Outcome::Filed(_)
        | w::Outcome::Handed
        | w::Outcome::Triaged(_)
        | w::Outcome::FindingRefused(_)
        | w::Outcome::Drafted(_)
        | w::Outcome::DraftRefused(_)
        | w::Outcome::Follow(_)
        | w::Outcome::Opened(_)
        | w::Outcome::OpenRefused(_)
        | w::Outcome::Tier(_)
        | w::Outcome::NotTheirs
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::Checkpointed(_)
        | w::Outcome::CheckpointRefused(_)
        | w::Outcome::EndRefused(_) => return None,
    })
}
#[must_use]
pub fn forges(bytes: &[u8], limits: &w::Limits) -> Option<Box<[forge::forges::Row]>> {
    let (rows, _, _) = forges_document(bytes, limits)?;
    Some(rows)
}
/// The forge rows and whether the printed link signed in (client/forge/records.md).
#[must_use]
pub fn forges_document(bytes: &[u8], limits: &w::Limits) -> Option<forge::forges::Document> {
    use forge::forges::Row;
    let record = wire::decode_forges(bytes, limits)?;
    let mut rows = List::with_capacity(record.forges.len());
    for row in record.forges.into_boxed() {
        let fetched = fetched_from(&row.connection);
        let connection = connection_from(row.connection);
        let mut makings = List::with_capacity(row.makings.len());
        for making in row.makings.into_boxed() {
            makings
                .push(forge::making::Making {
                    org: making.org,
                    name: making.name,
                    from: making.from,
                    into: making.into,
                    state: making.state,
                })
                .ok()?;
        }
        rows.push(Row {
            catching_up: row.catching_up,
            makings: makings.into_boxed(),
            read_only: row.read_only,
            number: row.number,
            host: row.host,
            kind: maps::kind_to_domain(row.kind),
            mode: maps::mode_to_domain(row.mode),
            connection,
            fetched,
            offer: maps::offer_to_domain(row.offer),
            first: match row.first {
                Some(first) => Some(maps::first_repository_to_domain(first)),
                None => None,
            },
            starts_with: maps::starts_with_to_domain(row.starts_with),
            installations: installations_from(row.installations)?,
            ended: match row.ended {
                Some(ended) => Some(ended.why),
                None => None,
            },
            app: match row.app {
                Some(app) => Some(maps::app_way_to_domain(app)),
                None => None,
            },
            web: row.web,
            repositories: adoptions_from(row.repositories)?,
            owned: match row.owned {
                Some(owned) => Some(maps::owned_forge_to_domain(owned)),
                None => None,
            },
            admin: row.admin,
            ended_stop: match row.ended_stop {
                Some(stop) => Some(maps::admin_stop_to_domain(stop)),
                None => None,
            },
        })
        .ok()?;
    }
    Some((rows.into_boxed(), record.linked, record.full.into_boxed()))
}

fn installations_from(values: List<w::Installation>) -> Option<Box<[forge::forges::Installation]>> {
    let mut installations = List::with_capacity(values.len());
    for installation in values.into_boxed() {
        installations.push(maps::installation_to_domain(installation)).ok()?;
    }
    Some(installations.into_boxed())
}
fn adoptions_from(values: List<w::Adoption>) -> Option<Box<[forge::forges::Adoption]>> {
    let mut repositories = List::with_capacity(values.len());
    for adopted in values.into_boxed() {
        repositories.push(maps::adoption_to_domain(adopted)).ok()?;
    }
    Some(repositories.into_boxed())
}

fn fetched_from(connection: &w::Connection) -> Option<forge::forges::Fetched> {
    use forge::forges::Fetched;
    let value = match connection {
        w::Connection::SettingUp(value) => value,
        w::Connection::NotConnected
        | w::Connection::NotSetUp
        | w::Connection::SetUpFailed(_)
        | w::Connection::Connecting
        | w::Connection::Connected(_)
        | w::Connection::Stopped(_) => return None,
    };
    match &value.step {
        Some(w::SetUpStep::Fetch) => {}
        Some(w::SetUpStep::Prepare | w::SetUpStep::Run | w::SetUpStep::Source) | None => return None,
    }
    let fetched = value.fetched.as_ref()?;
    if let Some(total) = &fetched.total
        && (*total == 0 || fetched.done > *total)
    {
        return None;
    }
    Some(Fetched {
        binary: maps::set_up_binary_to_domain(fetched.binary.clone()),
        done: fetched.done,
        total: fetched.total,
    })
}

fn connection_from(connection: w::Connection) -> forge::forges::Connection {
    use forge::forges::Connection as C;
    match connection {
        w::Connection::NotConnected => C::NotConnected,
        w::Connection::NotSetUp => C::NotSetUp,
        w::Connection::SettingUp(value) => C::SettingUp {
            step: match value.step {
                Some(step) => Some(maps::set_up_step_to_domain(step)),
                None => None,
            },
        },
        w::Connection::SetUpFailed(value) => C::SetUpFailed {
            why: value.why,
            at: match value.at {
                Some(step) => Some(maps::failed_step_to_domain(step)),
                None => None,
            },
        },
        w::Connection::Connecting => C::Connecting,
        w::Connection::Connected(value) => C::Connected { id: value.id, login: value.login },
        w::Connection::Stopped(value) => C::Stopped { reason: value.reason },
    }
}

/// One fresh picker page; it carries no credentials (client/forge/records.md).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    pub rows: Box<[Reached]>,
    pub more: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reached {
    pub id: u64,
    pub owner: forge::forges::Account,
    pub name: Box<[u8]>,
    pub private: bool,
}
#[must_use]
pub fn page(value: w::Page) -> Page {
    maps::page_to_domain(value)
}

/// Decode the playground's complete record (temper/docs/design/client/forge/family.md, 7).
#[must_use]
pub fn playground(record: &[u8], limits: &w::Limits) -> Option<forge::Playground> {
    let answer = wire::decode_playground_answer(record, limits)?;
    Some(maps::playground_answer_to_domain(answer))
}

/// The forge's keyed follow act (temper/docs/design/domain/forge/follows.md, 5).
#[must_use]
pub fn follow_act(value: w::FollowActRequest, limits: &w::Limits) -> Option<jig_client::Ask> {
    let project = value.project;
    let record = w::Requesting { request: w::Request::FollowAct(value) };
    Some(jig_client::Ask::Family {
        scope: jig_client::contract::Scope::Project(project),
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: record.encode(limits).ok()?,
        },
    })
}

/// A keyed tier hold or release (temper/docs/design/domain/after-landing-runs.md, 2).
#[must_use]
pub fn tier_act(value: w::TierActRequest, limits: &w::Limits) -> Option<jig_client::Ask> {
    let project = value.project;
    let record = w::Requesting { request: w::Request::TierAct(value) };
    Some(jig_client::Ask::Family {
        scope: jig_client::contract::Scope::Project(project),
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: record.encode(limits).ok()?,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dismiss_making_builds_a_bounded_forge_request() {
        let limits = wire::limits(&wire::Bounds {
            segments: 1,
            segment_bytes: 8,
            gates: 1,
            approvals: 1,
            field_bytes: 4,
            checks: 1,
            contexts: 1,
            refusal_bytes: 64,
        })
        .expect("test bounds");
        for into in [None, Some(3)] {
            let family = dismiss_making(7, b"orgx", b"repo", into, &limits).expect("bounded dismissal");
            assert_eq!(family.family, wire::FORGE);
            assert_eq!(family.version, wire::CLIENT_VERSION);
            let record = wire::decode_requesting(&family.bytes, &limits).expect("dismissal decodes");
            let w::Request::DismissMaking(request) = record.request else { panic!("dismissal request") };
            assert_eq!(request.forge, 7);
            assert_eq!(request.org.as_ref(), b"orgx");
            assert_eq!(request.name.as_ref(), b"repo");
            assert_eq!(request.into, into);
        }
        assert!(dismiss_making(7, b"orgxx", b"repo", None, &limits).is_none());
        assert!(dismiss_making(7, b"org", b"repox", None, &limits).is_none());
    }
}
