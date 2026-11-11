//! Away and decision families (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use crate::Undecodable;
use skein_lib::{List, Reader, Wall, Writer};
use temper_engine_domain_forge as d;

fn wall(value: Option<u64>) -> Option<Wall> {
    match value {
        Some(value) => Some(Wall::from_nanos(value)),
        None => None,
    }
}
fn nanos(value: Option<Wall>) -> Option<u64> {
    match value {
        Some(value) => Some(value.as_nanos()),
        None => None,
    }
}
pub(super) fn encode_away(row: &d::Away) -> w::Away {
    w::Away::new(
        &w::CEILINGS,
        w::AwayParts {
            party: row.party,
            project: row.project,
            since: row.since.as_nanos(),
            until: nanos(row.until),
            landed: row.landed,
            off: nanos(row.off),
        },
    )
    .expect(FITS)
}
pub(super) fn decode_away(row: w::Away) -> d::Away {
    let w::AwayParts { party, project, since, until, landed, off } = row.into_parts();
    d::Away { party, project, since: Wall::from_nanos(since), until: wall(until), landed, off: wall(off) }
}
pub(super) fn encode_decided(row: &d::Decided) -> w::Decided {
    let mut options = List::with_capacity(u32::try_from(row.options.len()).expect("bounded options"));
    for option in &row.options {
        options.push(option.clone()).expect("sized option list");
    }
    w::Decided::new(
        &w::CEILINGS,
        w::DecidedParts {
            project: row.project,
            number: row.number,
            party: row.party,
            task: row.task,
            at: row.at.as_nanos(),
            question: row.question.clone(),
            options,
            took: row.took,
            why: row.why.clone(),
            overturned: row.overturned,
            reach: encode_reach(row.reach),
        },
    )
    .expect(FITS)
}
pub(super) fn decode_decided(row: w::Decided) -> Result<d::Decided, Undecodable> {
    let w::DecidedParts { project, number, party, task, at, question, options, took, why, overturned, reach } =
        row.into_parts();
    let options = options.into_boxed();
    match took {
        Some(index) => {
            if !d::DECIDED_OPTIONS.contains(&options.len()) || options.get(usize::from(index)).is_none() {
                return Err(Undecodable);
            }
        }
        None => {
            if !options.is_empty() || !why.is_empty() || overturned {
                return Err(Undecodable);
            }
        }
    }
    let reach = decode_reach(reach)?;
    Ok(d::Decided {
        project,
        number,
        party,
        task,
        at: Wall::from_nanos(at),
        question,
        options,
        took,
        why,
        overturned,
        reach,
    })
}

pub(super) fn encode_reach(reach: temper_protocol_reaches::Reach) -> w::Reach {
    let record = temper_protocol_reaches::reach_record(&reach);
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("reach measure fits"));
    record.encode(&mut writer).expect("measured reach room");
    w::Reach::decode(&w::CEILINGS, &mut Reader::new(&writer.finish())).expect("matching reach records")
}
pub(super) fn decode_reach(record: w::Reach) -> Result<temper_protocol_reaches::Reach, Undecodable> {
    use temper_client_protocol_wire::v1;
    let mut writer = Writer::new(usize::try_from(record.measure()).or(Err(Undecodable))?);
    record.encode(&mut writer).or(Err(Undecodable))?;
    let record = v1::Reach::decode(&v1::CEILINGS, &mut Reader::new(&writer.finish())).or(Err(Undecodable))?;
    temper_protocol_reaches::reach_from(&record).ok_or(Undecodable)
}

// The codec checks sizes; admission checks the service and subscription key.
pub(super) fn encode_device(row: &d::Device) -> w::Device {
    w::Device::new(&w::CEILINGS, w::DeviceParts { party: row.party, number: row.number, endpoint: row.endpoint.clone(), p256dh: row.p256dh, auth: row.auth, label: row.label.clone(), at: row.at.as_nanos() }).expect(FITS)
}
#[expect(clippy::disallowed_methods, reason = "the stored device label must be UTF-8 (store.md, 2)")]
pub(super) fn decode_device(row: w::Device) -> Result<d::Device, Undecodable> {
    let w::DeviceParts { party, number, endpoint, p256dh, auth, label, at } = row.into_parts();
    if endpoint.is_empty() || label.is_empty() || core::str::from_utf8(&label).is_err() { return Err(Undecodable); }
    Ok(d::Device { party, number, endpoint, p256dh, auth, label, at: Wall::from_nanos(at) })
}
