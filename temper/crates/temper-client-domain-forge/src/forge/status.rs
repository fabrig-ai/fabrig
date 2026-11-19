//! A project's status subject and look request (temper/docs/design/client/forge/records.md).
use super::{Limits, wire_limits};
use jig_client::{Ask, Family, Watchable, contract as c};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::StatusSubject { project };
    let watching = w::Watching { watch: w::Watch::Status(subject) };
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: watching.encode(&bounds).ok()? },
    })
}

#[must_use]
pub fn document(record: &Family, limits: Limits) -> Option<w::ProjectStatus> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_project_status(&record.bytes, &wire_limits(&limits)?)
}

#[must_use]
pub fn ask(project: u32, limits: &Limits) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let looked = w::Looked { project };
    let record = w::Requesting { request: w::Request::Looked(looked) };
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: record.encode(&bounds).ok()? },
    })
}

#[derive(Debug)]
pub enum Answer {
    Looked,
    Unknown,
    Refused,
}

#[must_use]
pub fn answer(outcome: &jig_client::Outcome, limits: Limits) -> Answer {
    match outcome {
        jig_client::Outcome::Unknown => Answer::Unknown,
        jig_client::Outcome::Answered(jig_client::Answered::Family(record)) => match answered(record, limits) {
            Some(answer) => answer,
            None => Answer::Refused,
        },
        jig_client::Outcome::Answered(jig_client::Answered::Jig(_))
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Unconfirmed
        | jig_client::Outcome::Held
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost => Answer::Refused,
    }
}

fn answered(record: &Family, limits: Limits) -> Option<Answer> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(&wire_limits(&limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match answer.outcome {
        w::Outcome::Looked => Answer::Looked,
        w::Outcome::Unknown => Answer::Unknown,
        w::Outcome::Dismissed
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
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Removed
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::Reset
        | w::Outcome::DeviceAdded(_)
        | w::Outcome::DeviceRemoved
        | w::Outcome::DeviceRefused(_)
        | w::Outcome::Away
        | w::Outcome::Tried
        | w::Outcome::Filed(_)
        | w::Outcome::Handed
        | w::Outcome::Triaged
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
        | w::Outcome::EndRefused(_)
        | w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_) => Answer::Refused,
    })
}
