//! The small store holds the client's own records: a request's 16-byte key, or the stream's.
#[must_use]
pub fn encode(record: jig_client::Record) -> (Box<[u8]>, Box<[u8]>) {
    (record.key, record.bytes)
}

#[must_use]
pub fn decode(record: skein_ui_web::store::Record) -> Option<jig_client::Record> {
    if record.key.len() == 16 || record.key.as_ref() == jig_client::STREAM_KEY {
        return Some(jig_client::Record { key: record.key, bytes: record.value });
    }
    None
}
