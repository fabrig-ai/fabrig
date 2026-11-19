//! The reference UI's own small-store records (client/domain/view-state.md, 8).
use crate::state::address::Address;
use alloc::boxed::Box;
use jig_client::contract::Text;
use skein_lib::{Reader, Writer, bytes};
use w::Record as _;
pub(crate) mod address;
use crate::generated::records_v1 as w;
/// The version of the reference UI's records.
pub const RECORDS_VERSION: u16 = 1;
/// The reference UI's own record, kept beside the client's records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Place { version: u16, party: u64, address: Address },
    Draft { version: u16, party: u64, task: u64, words: Text, touched: u64 },
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
    let (version, key, record, bounded) = match record {
        Record::Place { version, party, address: at } => (
            *version,
            RecordKey::Place,
            w::ViewRecord::Place(w::Place { party: *party, address: address::encode(*at) }),
            "bounded place",
        ),
        Record::Draft { version, party, task, words, touched } => (
            *version,
            RecordKey::Draft { task: *task },
            w::ViewRecord::Draft(w::Draft {
                party: *party,
                task: *task,
                words: bytes::copy_of(words.bytes()),
                touched: *touched,
            }),
            "bounded draft",
        ),
    };
    let payload = record.encode(&w::CEILINGS).expect(bounded);
    let record = w::StoredRecord { store_version: version, record: payload };
    (key.encode(), record.encode(&w::CEILINGS).expect("bounded record"))
}
/// Drops unreadable records, other versions and records under a different key.
#[must_use]
pub fn decode(key: &[u8], bytes: &[u8]) -> Option<Record> {
    let mut reader = Reader::new(bytes);
    let record = w::StoredRecord::decode(&w::CEILINGS, &mut reader).ok()?;
    let version = record.store_version;
    if reader.remaining() != 0 || version != RECORDS_VERSION {
        return None;
    }
    let mut payload = Reader::new(&record.record);
    let page = w::ViewRecord::decode(&w::CEILINGS, &mut payload).ok()?;
    if payload.remaining() != 0 {
        return None;
    }
    let (record_key, record) = match page {
        w::ViewRecord::Place(value) => {
            (RecordKey::Place, Record::Place { version, party: value.party, address: address::decode(&value.address) })
        }
        w::ViewRecord::Draft(value) => (
            RecordKey::Draft { task: value.task },
            Record::Draft {
                version,
                party: value.party,
                task: value.task,
                words: Text::new(value.words, w::CEILINGS.draft_words).ok()?,
                touched: value.touched,
            },
        ),
    };
    if key != record_key.encode().as_ref() {
        return None;
    }
    Some(record)
}
