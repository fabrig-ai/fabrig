//! Application keys preserve the connector's derived order (protocol/store.md, 4.1).
use alloc::boxed::Box;
use jig_test_connector::{self as d, RecordKey};
use skein_kv::{KeyReader, KeyWriter};
use skein_lib::{List, Token};
/// Encode one connector number and record key without the store's owner tag.
#[must_use]
pub fn encode(number: u16, key: &RecordKey) -> Box<[u8]> {
    let mut writer = KeyWriter::new(u32::MAX);
    writer.u64(u64::from(number));
    match key {
        RecordKey::ApplicationObject(value) => {
            writer.tag(0).u64(*value);
        }
        RecordKey::ApplicationAnswer(value) => {
            writer.tag(1).u64(value.raw());
        }
        RecordKey::Procedure(value) => {
            writer.tag(2).u64(*value);
        }
        RecordKey::ProcedureDelegateOutcome(value) => {
            writer.tag(3).u64(*value);
        }
        RecordKey::Result(value) => {
            writer.tag(4).u64(*value);
        }
        RecordKey::Task(value) => {
            writer.tag(5).u64(*value);
        }
        RecordKey::Adoption { project, path } => {
            writer.tag(6).u64(u64::from(*project));
            write_path(&mut writer, path);
        }
        RecordKey::Subscription { topic, task } => {
            writer.tag(7).u64(u64::from(*topic)).u64(*task);
        }
        RecordKey::Pool(path) => {
            writer.tag(8);
            write_path(&mut writer, path);
        }
        RecordKey::Proposal(value) => {
            writer.tag(9).u64(*value);
        }
        RecordKey::Outbox(key) => {
            writer.tag(10);
            write_key(&mut writer, key);
        }
        RecordKey::SettingDefault { kind, scope, number } => {
            writer.tag(12).u64(u64::from(*kind)).u64(u64::from(*scope)).u64(u64::from(*number));
        }
        RecordKey::SettingChange { kind, scope, number } => {
            writer.tag(13).u64(u64::from(*kind)).u64(u64::from(*scope)).u64(u64::from(*number));
        }
        RecordKey::SettingProposal { proposal, kind, scope, number } => {
            writer.tag(14).u64(*proposal).u64(u64::from(*kind)).u64(u64::from(*scope)).u64(u64::from(*number));
        }
        RecordKey::SettingReplaced { kind, scope, number, revision } => {
            writer.tag(15).u64(u64::from(*kind)).u64(u64::from(*scope)).u64(u64::from(*number)).u64(*revision);
        }
        RecordKey::Made(key) => {
            writer.tag(11);
            write_key(&mut writer, key);
        }
    }
    writer.finish().expect("bounded connector key fits u32")
}
/// Decode a whole key, checking narrowed numbers, paths and trailing bytes.
#[must_use]
pub fn decode(bytes: &[u8], limits: &d::Limits) -> Option<(u16, RecordKey)> {
    let mut reader = KeyReader::new(bytes);
    let number = u16::try_from(reader.u64()?).ok()?;
    let key = match reader.tag()? {
        0 => RecordKey::ApplicationObject(reader.u64()?),
        1 => RecordKey::ApplicationAnswer(Token::new(reader.u64()?)),
        2 => RecordKey::Procedure(reader.u64()?),
        3 => RecordKey::ProcedureDelegateOutcome(reader.u64()?),
        4 => RecordKey::Result(reader.u64()?),
        5 => RecordKey::Task(reader.u64()?),
        6 => RecordKey::Adoption { project: u32::try_from(reader.u64()?).ok()?, path: read_path(&mut reader, limits)? },
        7 => RecordKey::Subscription { topic: u16::try_from(reader.u64()?).ok()?, task: reader.u64()? },
        8 => RecordKey::Pool(read_path(&mut reader, limits)?),
        9 => RecordKey::Proposal(reader.u64()?),
        10 => RecordKey::Outbox(read_key(&mut reader)?),
        11 => RecordKey::Made(read_key(&mut reader)?),
        12 => RecordKey::SettingDefault {
            kind: u32::try_from(reader.u64()?).ok()?,
            scope: u32::try_from(reader.u64()?).ok()?,
            number: u32::try_from(reader.u64()?).ok()?,
        },
        13 => RecordKey::SettingChange {
            kind: u32::try_from(reader.u64()?).ok()?,
            scope: u32::try_from(reader.u64()?).ok()?,
            number: u32::try_from(reader.u64()?).ok()?,
        },
        14 => RecordKey::SettingProposal {
            proposal: reader.u64()?,
            kind: u32::try_from(reader.u64()?).ok()?,
            scope: u32::try_from(reader.u64()?).ok()?,
            number: u32::try_from(reader.u64()?).ok()?,
        },
        15 => RecordKey::SettingReplaced {
            kind: u32::try_from(reader.u64()?).ok()?,
            scope: u32::try_from(reader.u64()?).ok()?,
            number: u32::try_from(reader.u64()?).ok()?,
            revision: reader.u64()?,
        },
        _ => return None,
    };
    if !reader.is_empty() {
        return None;
    }
    Some((number, key))
}
fn write_path(writer: &mut KeyWriter, path: &d::Path) {
    for segment in path.segments() {
        writer.tag(1).bytes(segment);
    }
    writer.tag(0);
}
fn read_path(reader: &mut KeyReader<'_>, limits: &d::Limits) -> Option<d::Path> {
    let mut segments = List::with_capacity(limits.path_segments);
    for _ in 0..=limits.path_segments {
        match reader.tag()? {
            0 => return d::Path::new(segments, limits.segment_bytes),
            1 => {
                segments.push(reader.bytes()?).ok()?;
            }
            _ => return None,
        }
    }
    None
}
fn write_key(writer: &mut KeyWriter, key: &d::Key) {
    match key.origin {
        d::Purpose::Procedure => {
            writer.tag(0);
        }
        d::Purpose::Projection => {
            writer.tag(1);
        }
        d::Purpose::Call { attempt, completion, position } => {
            writer.tag(2).u64(attempt).u64(u64::from(completion)).u64(u64::from(position));
        }
    }
    writer.bytes(&key.deployment).u64(key.task).u64(key.purpose);
}
fn read_key(reader: &mut KeyReader<'_>) -> Option<d::Key> {
    let origin = match reader.tag()? {
        0 => d::Purpose::Procedure,
        1 => d::Purpose::Projection,
        2 => d::Purpose::Call {
            attempt: reader.u64()?,
            completion: u32::try_from(reader.u64()?).ok()?,
            position: u32::try_from(reader.u64()?).ok()?,
        },
        _ => return None,
    };
    Some(d::Key {
        origin,
        deployment: reader.bytes()?.as_ref().try_into().ok()?,
        task: reader.u64()?,
        purpose: reader.u64()?,
    })
}
