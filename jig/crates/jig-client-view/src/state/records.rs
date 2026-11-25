//! The reference UI's own small-store records (client/domain/view-state.md, 8).
use crate::generated::{records as map, records_v1 as w};
use crate::state::address::Address;
use alloc::boxed::Box;
use jig_client::contract::Text;
use skein_codec::Reason;
use skein_lib::{Reader, Writer, bytes};
use w::Record as _;

/// The reference UI's own record, kept beside the client's records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Place { party: u64, address: Address },
    Draft { party: u64, task: u64, words: Text, touched: u64 },
}
/// A key in the reference UI's own namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKey {
    Place,
    Draft { task: u64 },
}
impl RecordKey {
    /// The reference UI's storage key, distinct from the client's 16-byte keys.
    #[must_use]
    pub fn encode(self) -> Box<[u8]> {
        match self {
            Self::Place => Box::new([0]),
            Self::Draft { task } => {
                let mut writer = Writer::new(9);
                writer.put(&[1]).expect("reserved key tag");
                writer.put(&task.to_be_bytes()).expect("reserved task key");
                writer.finish()
            }
        }
    }
}
/// The key and bytes the application keeps.
#[must_use]
pub fn encode(record: &Record) -> (Box<[u8]>, Box<[u8]>) {
    let (key, bounded) = match record {
        Record::Place { .. } => (RecordKey::Place, "bounded place"),
        Record::Draft { task, .. } => (RecordKey::Draft { task: *task }, "bounded draft"),
    };
    let payload = map::view_record_to_record(record).encode(&w::CEILINGS).expect(bounded);
    let record = w::StoredRecord { layout: w::LAYOUT, record: payload };
    (key.encode(), record.encode(&w::CEILINGS).expect("bounded record"))
}
/// Drops unreadable records, other layouts and records under a different key.
#[must_use]
pub fn decode(key: &[u8], bytes: &[u8]) -> Option<Record> {
    let mut reader = Reader::new(bytes);
    let record = w::StoredRecord::decode(&w::CEILINGS, &mut reader).ok()?;
    if reader.remaining() != 0 || record.layout != w::LAYOUT {
        return None;
    }
    let mut payload = Reader::new(&record.record);
    let page = w::ViewRecord::decode(&w::CEILINGS, &mut payload).ok()?;
    if payload.remaining() != 0 {
        return None;
    }
    let record = map::view_record_to_domain(page).ok()?;
    let record_key = match &record {
        Record::Place { .. } => RecordKey::Place,
        Record::Draft { task, .. } => RecordKey::Draft { task: *task },
    };
    if key != record_key.encode().as_ref() {
        return None;
    }
    Some(record)
}

pub(crate) fn words_to_record(value: &Text) -> Box<[u8]> {
    bytes::copy_of(value.bytes())
}
pub(crate) fn words_to_domain(value: Box<[u8]>) -> Result<Text, Reason> {
    match Text::new(value, w::CEILINGS.draft_words) {
        Ok(text) => Ok(text),
        Err(value) if value.len() > skein_lib::length::usize_of(w::CEILINGS.draft_words) => Err(Reason::Bound),
        Err(_) => Err(Reason::Utf8),
    }
}
