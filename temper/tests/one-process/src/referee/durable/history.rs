//! Decode history only up to the extent in a durable map frame (jig's protocol/store/logs.md, 3).
use super::{Store, in_generation, keys, root, transcripts};
use skein_lib::{List, Reader};
use std::collections::BTreeMap;

pub(super) fn is_payload(path: &[u8]) -> bool {
    in_generation(path).is_some_and(|suffix| suffix.starts_with(b"store/logs/"))
}

impl Store {
    /// Fold each named log independently of the service's typed writes. Reading
    /// its whole extent also covers snapshots and uncertain tails on recovery.
    pub(super) fn histories(&self, ops: &[skein_kv::Op]) -> Result<Vec<skein_kv::Op>, String> {
        let mut found = Vec::new();
        for op in ops {
            let (key, value) = match op {
                skein_kv::Op::Put { key, value } => (key, Some(value)),
                skein_kv::Op::Erase { key } => (key, None),
                skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => unreachable!("map frame operations"),
            };
            if key.len() != 10 || !key.starts_with(&jig_protocol_store::logs::EXTENTS) {
                continue;
            }
            let log = Reader::new(&key[2..]).u64().ok_or("history log number")?;
            if log % 2 != 0 {
                continue;
            }
            let task = log / 2;
            if let Some(value) = value {
                let end = usize::try_from(Reader::new(value).u64().ok_or("history extent")?)
                    .map_err(|why| format!("history length: {why:?}"))?;
                let generation = self.active.as_deref().unwrap_or(b"generations/1");
                let mut path = generation.to_vec();
                path.extend_from_slice(format!("/store/logs/{:016x}/{log:016x}", log >> 12).as_bytes());
                let bytes = self
                    .files
                    .get(&path)
                    .ok_or_else(|| format!("history bytes precede extent: {}", String::from_utf8_lossy(&path)))?;
                let rows = rows(bytes.get(..end).ok_or("history extent is within observed bytes")?, task)?;
                for (key, value) in rows {
                    found.push(match value {
                        Some(value) => skein_kv::Op::Put { key, value },
                        None => skein_kv::Op::Erase { key },
                    });
                }
            } else {
                for row in self.rows.values() {
                    if let root::Record::Core(row) = row {
                        let key = keys::encode(&row.key(), u32::MAX).ok_or("history key encodes")?;
                        if keys::logged(&key) == Some(task) {
                            found.push(skein_kv::Op::Erase { key });
                        }
                    }
                }
            }
        }
        Ok(found)
    }
}

type Rows = BTreeMap<Box<[u8]>, Option<Box<[u8]>>>;
fn rows(mut bytes: &[u8], task: u64) -> Result<Rows, String> {
    let mut rows = BTreeMap::new();
    while !bytes.is_empty() {
        let size = usize::try_from(Reader::new(bytes).u32().ok_or("history frame length")?)
            .map_err(|why| format!("history frame size: {why:?}"))?;
        let frame = bytes.get(..size).ok_or("whole history frame")?;
        let record = record(frame).ok_or("valid history frame")?;
        let page = jig_protocol_store::logs::record::decode(&record).ok_or("valid history record")?;
        for (key, value) in page {
            if keys::logged(&key) != Some(task) {
                return Err("history key belongs to its task".into());
            }
            rows.insert(key, value);
        }
        bytes = &bytes[size..];
    }
    Ok(rows)
}
fn record(bytes: &[u8]) -> Option<Box<[u8]>> {
    let mut reader = Reader::new(bytes);
    let size = reader.u32()?;
    if usize::try_from(size).ok()? != bytes.len() || size < 17 {
        return None;
    }
    let kind = reader.u8()?;
    let length = reader.u32()?;
    let body = reader.bytes(size.checked_sub(17)?)?;
    if reader.u32()? != transcripts::frame::crc32c(bytes.get(..bytes.len().checked_sub(8)?)?) || reader.u32()? != size {
        return None;
    }
    let record = match kind {
        0 => Box::from(body),
        1 => {
            let mut out = List::with_capacity(length);
            skein_deflate::decode(body, &mut out)?;
            out.into_boxed()
        }
        _ => return None,
    };
    if record.len() != usize::try_from(length).ok()? {
        return None;
    }
    Some(record)
}

#[cfg(test)]
mod tests;
