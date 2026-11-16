//! The milestone totals read (temper/docs/design/client/forge/records.md).
use super::{Limits, wire_limits};
use jig_client::{Family, Readable, Readout, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn read(project: u32, task: u64, limits: &Limits) -> Option<Readable> {
    let bounds = wire_limits(limits)?;
    let read = w::MilestoneRead::new(&bounds, w::MilestoneReadParts { project, task }).ok()?;
    let record = w::Reading::new(&bounds, w::ReadingParts { read: w::Read::Milestone(read) }).ok()?;
    Some(Readable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::reading(&record) },
    })
}

#[must_use]
pub fn answer(readout: &Readout, limits: &Limits) -> Option<w::MilestoneResult> {
    let Readout::Family { family, record: Some(record) } = readout else { return None };
    if *family != wire::FORGE {
        return None;
    }
    Some(wire::decode_milestone_answer(record, &wire_limits(limits)?)?.into_parts().result)
}
