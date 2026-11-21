//! Away and decision families (temper/docs/design/protocol/store.md, 2).
use super::w;
use crate::Undecodable;
use skein_codec::Record;
use skein_lib::{List, Reader, Wall, Writer};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_away(row: &d::Away) -> w::Away {
    crate::generated::store::away_to_record(row, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn decode_away(row: w::Away) -> d::Away {
    crate::generated::store::away_to_domain(row, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn encode_decided(row: &d::Decided) -> w::Decided {
    let mut options = List::with_capacity(u32::try_from(row.options.len()).expect("bounded options"));
    for option in &row.options {
        options.push(option.clone()).expect("sized option list");
    }
    w::Decided {
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
    }
}
pub(super) fn decode_decided(row: w::Decided) -> Result<d::Decided, Undecodable> {
    let w::Decided { project, number, party, task, at, question, options, took, why, overturned, reach } = row;
    let options = options.into_boxed();
    match took {
        Some(index) => {
            if !d::DECIDED_OPTIONS.contains(&options.len()) || options.get(usize::from(index)).is_none() {
                return Err(Undecodable::Malformed);
            }
        }
        None => {
            if !options.is_empty() || !why.is_empty() || overturned {
                return Err(Undecodable::Malformed);
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
    let mut writer = Writer::new(
        usize::try_from(
            record.measure(&temper_client_protocol_wire::v1::CEILINGS).expect("admitted client record within ceilings"),
        )
        .expect("reach measure fits"),
    );
    record.write(&temper_client_protocol_wire::v1::CEILINGS, &mut writer).expect("measured reach room");
    w::Reach::decode(&w::CEILINGS, &mut Reader::new(&writer.finish())).expect("matching reach records")
}
pub(super) fn decode_reach(record: w::Reach) -> Result<temper_protocol_reaches::Reach, Undecodable> {
    use temper_client_protocol_wire::v1;
    let bytes = record.encode(&w::CEILINGS).or(Err(Undecodable::Malformed))?;
    let record = v1::Reach::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).or(Err(Undecodable::Malformed))?;
    temper_protocol_reaches::reach_from(&record).ok_or(Undecodable::Malformed)
}

// The codec checks sizes; admission checks the service and subscription key.
pub(super) fn encode_device(row: &d::Device) -> w::Device {
    w::Device {
        party: row.party,
        number: row.number,
        endpoint: row.endpoint.clone(),
        p256dh: row.p256dh,
        auth: row.auth,
        label: row.label.clone(),
        at: row.at.as_nanos(),
    }
}
#[expect(clippy::disallowed_methods, reason = "the stored device label must be UTF-8 (store.md, 2)")]
pub(super) fn decode_device(row: w::Device) -> Result<d::Device, Undecodable> {
    let w::Device { party, number, endpoint, p256dh, auth, label, at } = row;
    if endpoint.is_empty() || label.is_empty() || core::str::from_utf8(&label).is_err() {
        return Err(Undecodable::Malformed);
    }
    Ok(d::Device { party, number, endpoint, p256dh, auth, label, at: Wall::from_nanos(at) })
}

/// Owner-shaped `Reach` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn reach_to_record(
    value: &temper_protocol_reaches::Reach,
    _context: &super::Context,
) -> Result<w::Reach, w::Problem> {
    Ok(encode_reach(*value))
}
pub fn reach_to_domain(
    value: w::Reach,
    _context: &super::Context,
) -> Result<temper_protocol_reaches::Reach, w::Problem> {
    decode_reach(value).or(Err(w::paths::REACH_KIND.problem(skein_codec::Reason::Malformed)))
}
